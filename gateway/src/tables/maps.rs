//! What the client knows of the maps: their names and signs, and who stands where in the towns.

use super::*;

/// What the client calls each map, by the map's name without `.rsw`:
/// `{"<map>": {"name": …, "title": …, "over": …, "picture": …, "shown": true}}`: the full name, the large and the
/// small line of the sign shown on coming in, the sign's picture (a file of the interface's `display_mapname`),
/// and whether the sign is shown at all. The client has two sources and so has this, in the client's order
/// (`ClientMessenger.cpp:6346`): first `data\mapNameTable.txt` of the archives (`CSession::LoadSessionTables`,
/// `LoadMapNameTable`; `named`), then `System\mapInfo_true.lub` (its English one where there is one; the file is
/// beside the archives, not in them: `g_mapInfoMgr->LoadScript()`), written over the first (`signed`).
pub fn maps(client_dir: &std::path::Path, grf: &Grf) -> serde_json::Value {
    let mut found = serde_json::Map::new();
    match grf.read("data\\mapnametable.txt") {
        Ok(bytes) => named(&mut found, &korean(&bytes)),
        Err(error) => eprintln!("tables: mapNameTable.txt: {error}"),
    }
    signed(&mut found, signs(client_dir));
    found.into()
}

/// The names of `mapNameTable.txt` (lines of `prontera.rsw#Prontera#`), as the client's
/// `CSession::LoadMapNameTable` reads them (`Source/Contents/Map/MapNameTable.cpp:21`): each line is
/// `CMapInfoMgr::AddMapDisplayName(map, name, false)`, a map with that name and no sign; of two lines for one
/// map the later stays.
fn named(maps: &mut serde_json::Map<String, serde_json::Value>, table: &str) {
    for fields in hash_fields(table) {
        let [map, name, ..] = fields[..] else { continue };
        maps.insert(
            map.trim_end_matches(".rsw").to_owned(),
            serde_json::json!({"name": name, "title": "", "over": "", "picture": "", "shown": false}),
        );
    }
}

/// What `mapInfo_true.lub` says of its maps written over the names of the text table, as the client's
/// `CMapInfoMgr::LoadScript` does when it runs after `LoadMapNameTable`: the file's
/// `AddMapDisplayName(map, displayName, notifyEnter)` replaces the name and says whether the sign shows, so a
/// map of both files has the Lua's name and the Lua's sign (Geffen shows its sign, as the owner saw in the
/// Windows client), and a map only the text table names has none. A map the Lua gives no name keeps the text
/// table's (not looked at in the client's Lua, which is compiled: what its `main` does with a map without a
/// `displayName`). Only `CSession::ReloadLanguageTables`, on a change of language at the login screen, reads
/// the text table again over the Lua and would take the signs off; this client has no such change.
fn signed(maps: &mut serde_json::Map<String, serde_json::Value>, signs: serde_json::Map<String, serde_json::Value>) {
    for (map, mut about) in signs {
        if about["name"].as_str().is_none_or(str::is_empty) {
            about["name"] = maps.get(&map).map_or_else(|| "".into(), |named| named["name"].clone());
        }
        maps.insert(map, about);
    }
}

/// The second source, `System\mapInfo_true.lub`: its `mapTbl`, each map's `displayName`, `signName`,
/// `backgroundBmp` and `notifyEnter`.
fn signs(client_dir: &std::path::Path) -> serde_json::Map<String, serde_json::Value> {
    let Some(lua) = beside(client_dir, &["System/mapInfo_true_EN.lub", "System/mapInfo_true.lub"]) else { return Default::default() };
    let Ok(table) = lua.globals().get::<Table>("mapTbl") else { return Default::default() };
    let words = |entry: &Table, field: &str| text(entry, field).unwrap_or_default();
    table
        .pairs::<mlua::String, Table>()
        .flatten()
        .map(|(map, entry)| {
            let sign = entry.get::<Option<Table>>("signName").ok().flatten();
            let line = |field: &str| sign.as_ref().map(|sign| words(sign, field)).unwrap_or_default();
            let about = serde_json::json!({
                "name": words(&entry, "displayName"),
                "title": line("mainTitle"),
                "over": line("subTitle"),
                "picture": words(&entry, "backgroundBmp"),
                "shown": entry.get::<Option<bool>>("notifyEnter").ok().flatten().unwrap_or(false),
            });
            (korean(&map.as_bytes()).trim_end_matches(".rsw").to_owned(), about)
        })
        .collect()
}

/// Who of use stands where in each town (`System\\Towninfo.lub`, its English one where there is one): by the
/// map's name, `[[name, x, y, type], …]` in the table's order. The types are the client's: 0 tool dealer, 1 weapon
/// dealer, 2 armour dealer, 3 smith, 4 guide, 5 inn, 6 Kafra, 7 styling shop, and further.
pub fn towns(client_dir: &std::path::Path) -> serde_json::Value {
    let Some(lua) = beside(client_dir, &["System/Towninfo_EN.lub", "System/Towninfo.lub"]) else { return serde_json::json!({}) };
    let Ok(table) = lua.globals().get::<Table>("mapNPCInfoTable") else { return serde_json::json!({}) };
    let found: serde_json::Map<String, serde_json::Value> = table
        .pairs::<mlua::String, Table>()
        .flatten()
        .map(|(map, people)| {
            let people: Vec<serde_json::Value> = people
                .sequence_values::<Table>()
                .flatten()
                .map(|one| {
                    let number = |field: &str| one.get::<Option<i64>>(field).ok().flatten().unwrap_or(-1);
                    serde_json::json!([text(&one, "name").unwrap_or_default(), number("X"), number("Y"), number("TYPE")])
                })
                .collect();
            (korean(&map.as_bytes()), people.into())
        })
        .collect();
    found.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lua_is_written_over_the_table_of_names() {
        let mut maps = serde_json::Map::new();
        let table = "// towns\r\ngeffen.rsw#Geffen, the Magic City#\r\njob_hunter.rsw#Hunter Guild#\r\n\r\nmoc_fild04.rsw#Sograt#\r\n\
                     moc_fild04.rsw#Morroc Field#\r\nbroken\r\n";
        named(&mut maps, table);
        assert_eq!(maps["job_hunter"], serde_json::json!({"name": "Hunter Guild", "title": "", "over": "", "picture": "", "shown": false}));
        assert_eq!((maps["moc_fild04"]["name"].as_str(), maps.len()), (Some("Morroc Field"), 3));
        let mut signs = serde_json::Map::new();
        signs.insert("geffen".to_owned(), serde_json::json!({"name": "Geffen", "title": "Geffen", "over": "", "picture": "village", "shown": true}));
        signs.insert("moc_fild04".to_owned(), serde_json::json!({"name": "", "title": "", "over": "", "picture": "field", "shown": true}));
        signs.insert("prontera".to_owned(), serde_json::json!({"name": "Prontera", "title": "Prontera", "over": "", "picture": "", "shown": true}));
        signed(&mut maps, signs);
        // A map of both has the Lua's name and its sign; one only the text table names has no sign.
        assert_eq!(maps["geffen"], serde_json::json!({"name": "Geffen", "title": "Geffen", "over": "", "picture": "village", "shown": true}));
        assert_eq!(maps["job_hunter"]["shown"], false);
        // One the Lua gives no name keeps the text table's.
        assert_eq!(maps["moc_fild04"], serde_json::json!({"name": "Morroc Field", "title": "", "over": "", "picture": "field", "shown": true}));
        assert_eq!((maps["prontera"]["name"].as_str(), maps.len()), (Some("Prontera"), 4));
    }
}
