//! What the client knows of the maps: their names and signs, and who stands where in the towns.

use super::*;

/// What the client calls each map, by the map's name without `.rsw`:
/// `{"<map>": {"name": …, "title": …, "over": …, "picture": …, "shown": true}}`: the full name, the large and the
/// small line of the sign shown on coming in, the sign's picture (a file of the interface's `display_mapname`),
/// and whether the sign is shown at all. The client has two sources and so has this: `System\mapInfo_true.lub`
/// (its English one where there is one; the file is beside the archives, not in them), run as `CMapInfoMgr` is
/// made, and then `data\mapNameTable.txt` of the archives (`named`), whose names are written over the first's.
pub fn maps(client_dir: &std::path::Path, grf: &Grf) -> serde_json::Value {
    let mut found = signs(client_dir);
    match grf.read("data\\mapnametable.txt") {
        Ok(bytes) => renamed(&mut found, &korean(&bytes)),
        Err(error) => eprintln!("tables: mapNameTable.txt: {error}"),
    }
    found.into()
}

/// The names of `mapNameTable.txt` (lines of `prontera.rsw#Prontera#`) written into the table of the maps, as
/// the client's `CSession::LoadMapNameTable` does after `mapInfo_true.lub` was run: each line is
/// `CMapInfoMgr::AddMapDisplayName(map, name, false)`, which makes the map if the first source had none of it
/// and replaces its name if it had; of two lines for one map the later stays.
///
/// Decided here against the client: that call also sets "the sign is shown" to false, so by the decompiled code
/// a map this file names shows no sign on coming in, whatever `mapInfo_true.lub` said. This is not done (the
/// sign stays as the first source has it) until it was looked at in the Windows client: `docs/DEBTS.md`.
fn renamed(maps: &mut serde_json::Map<String, serde_json::Value>, table: &str) {
    for fields in hash_fields(table) {
        let [map, name, ..] = fields[..] else { continue };
        let about = maps
            .entry(map.trim_end_matches(".rsw"))
            .or_insert_with(|| serde_json::json!({"name": "", "title": "", "over": "", "picture": "", "shown": false}));
        about["name"] = name.into();
    }
}

/// The first source, `System\mapInfo_true.lub`: its `mapTbl`, each map's `displayName`, `signName`,
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
    fn the_table_of_names_is_written_over_the_first_source() {
        let mut maps = serde_json::Map::new();
        maps.insert("geffen".to_owned(), serde_json::json!({"name": "Geffen", "title": "Geffen", "over": "", "picture": "village", "shown": true}));
        let table = "// towns\r\ngeffen.rsw#Geffen, the Magic City#\r\njob_hunter.rsw#Hunter Guild#\r\n\r\nmoc_fild04.rsw#Sograt#\r\nmoc_fild04.rsw#Morroc Field#\r\nbroken\r\n";
        renamed(&mut maps, table);
        assert_eq!(maps["geffen"], serde_json::json!({"name": "Geffen, the Magic City", "title": "Geffen", "over": "", "picture": "village", "shown": true}));
        assert_eq!(maps["job_hunter"], serde_json::json!({"name": "Hunter Guild", "title": "", "over": "", "picture": "", "shown": false}));
        assert_eq!(maps["moc_fild04"]["name"], "Morroc Field");
        assert_eq!(maps.len(), 3);
    }
}
