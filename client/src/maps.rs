//! A map by its name, and the client's table of the music of each (`mp3nametable.txt`).

/// A map's name without the ending a file or the server gives it (`prontera.gat`, `prontera.rsw`): what the
/// client's tables know a map by.
pub fn bare(name: &str) -> &str {
    name.trim_end_matches(".gat").trim_end_matches(".rsw")
}

/// A map's name without the number of its copy: the server names a copy of a map made for one party
/// `<three digits of the copy><the map's name>` (`0011@tower` is the copy 1 of `1@tower`), and the client cuts
/// those three letters out before it looks the map up anywhere: its files and its minimap
/// (`ResourceMgr::Resolve`, which then finds the map by the line `resnametable.txt` has for it), its music
/// (`CSession::GetBgmFileName`), its name and its sign (`CMapInfoMgr::FindMapInfo`), whether it is indoors
/// (`IsIndoorMapName`), its fog and its view (`GetFogParameters`, `GetMapViewParameters`), its sky and its
/// effects (`ScriptMapName`, `CurrentMapScriptName`). The rule is the same line in each: where the last `@` of
/// the name is at its fifth letter or farther, the three letters that end one before it are taken out
/// (`name.erase(at - 4, 3)`), whatever they are; a name may come with a folder before it and an ending after.
pub fn original(name: &str) -> std::borrow::Cow<'_, str> {
    match name.rfind('@') {
        Some(at) if at >= 4 && name.is_char_boundary(at - 4) && name.is_char_boundary(at - 1) => {
            std::borrow::Cow::Owned([&name[..at - 4], &name[at - 1..]].concat())
        }
        _ => std::borrow::Cow::Borrowed(name),
    }
}

/// The client's table of which music plays on which map, read: lines of `map.rsw#bgm\\08.mp3#`, a line that
/// begins with `//` passed by. Each map in small letters without its `.rsw`, with its track; a line whose
/// map is no `.rsw`, or that names no track, is none.
pub fn music_table(table: &str) -> Vec<(String, String)> {
    table
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .filter_map(|line| {
            let mut parts = line.split('#');
            let (map, track) = (parts.next()?.trim(), parts.next()?.trim());
            Some((map.to_ascii_lowercase().strip_suffix(".rsw")?.to_owned(), track.replace("\\\\", "\\")))
        })
        .filter(|(_, track)| !track.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_map_is_known_by_its_name_alone() {
        assert_eq!(bare("prontera.gat"), "prontera");
        assert_eq!(bare("prontera.rsw"), "prontera");
        assert_eq!(bare("prontera"), "prontera");
        assert_eq!(bare("1@tower.gat"), "1@tower");
    }

    #[test]
    fn a_copy_of_a_map_is_looked_up_as_the_map() {
        assert_eq!(original("0011@tower"), "1@tower");
        assert_eq!(original("0422@orcs.gat"), "2@orcs.gat");
        assert_eq!(original("data\\1231@cata.rsw"), "data\\1@cata.rsw");
        // A map that is no copy has its `@` nearer the beginning than the fifth letter, or none.
        assert_eq!(original("1@tower"), "1@tower");
        assert_eq!(original("12@tower.gat"), "12@tower.gat");
        assert_eq!(original("prontera"), "prontera");
        // The client asks nothing of the three letters: whatever stands there is cut.
        assert_eq!(original("abcd@x"), "d@x");
        // The last `@` counts.
        assert_eq!(original("1@ab0011@tower"), "1@ab1@tower");
    }

    #[test]
    fn the_music_of_each_map_is_read() {
        let table = "// the towns\nProntera.rsw#bgm\\\\08.mp3#\n moc_fild01.rsw # bgm\\\\37.mp3 #\nno_track.rsw##\nnot_a_map.gat#bgm\\\\01.mp3#\nbroken";
        assert_eq!(music_table(table), [("prontera".to_owned(), "bgm\\08.mp3".to_owned()), ("moc_fild01".to_owned(), "bgm\\37.mp3".to_owned())]);
    }
}
