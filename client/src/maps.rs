//! A map by its name, and the client's table of the music of each (`mp3nametable.txt`).

/// A map's name without the ending a file or the server gives it (`prontera.gat`, `prontera.rsw`): what the
/// client's tables know a map by.
pub fn bare(name: &str) -> &str {
    name.trim_end_matches(".gat").trim_end_matches(".rsw")
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
    fn the_music_of_each_map_is_read() {
        let table = "// the towns\nProntera.rsw#bgm\\\\08.mp3#\n moc_fild01.rsw # bgm\\\\37.mp3 #\nno_track.rsw##\nnot_a_map.gat#bgm\\\\01.mp3#\nbroken";
        assert_eq!(music_table(table), [("prontera".to_owned(), "bgm\\08.mp3".to_owned()), ("moc_fild01".to_owned(), "bgm\\37.mp3".to_owned())]);
    }
}
