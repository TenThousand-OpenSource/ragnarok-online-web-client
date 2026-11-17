//! What the client knows of the maps: their names and signs, and who stands where in the towns.

use super::*;

/// What the client calls each map (`System\mapInfo_true.lub`, its English one where there is one), by the map's
/// name without `.rsw`: `{"<map>": {"name": …, "title": …, "over": …, "picture": …, "shown": true}}`: the full
/// name, the large and the small line of the sign shown on coming in, the sign's picture (a file of the
/// interface's `display_mapname`), and whether the sign is shown at all. The file is beside the archives, not in
/// them.
pub fn maps(client_dir: &std::path::Path) -> serde_json::Value {
    let Some(lua) = beside(client_dir, &["System/mapInfo_true_EN.lub", "System/mapInfo_true.lub"]) else { return serde_json::json!({}) };
    let Ok(table) = lua.globals().get::<Table>("mapTbl") else { return serde_json::json!({}) };
    let text = |entry: &Table, field: &str| entry.get::<Option<mlua::String>>(field).ok().flatten().map(|text| korean(&text.as_bytes())).unwrap_or_default();
    let found: serde_json::Map<String, serde_json::Value> = table
        .pairs::<mlua::String, Table>()
        .flatten()
        .map(|(map, entry)| {
            let sign = entry.get::<Option<Table>>("signName").ok().flatten();
            let line = |field: &str| sign.as_ref().map(|sign| text(sign, field)).unwrap_or_default();
            let about = serde_json::json!({
                "name": text(&entry, "displayName"),
                "title": line("mainTitle"),
                "over": line("subTitle"),
                "picture": text(&entry, "backgroundBmp"),
                "shown": entry.get::<Option<bool>>("notifyEnter").ok().flatten().unwrap_or(false),
            });
            (korean(&map.as_bytes()).trim_end_matches(".rsw").to_owned(), about)
        })
        .collect();
    found.into()
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
                    let name = one.get::<Option<mlua::String>>("name").ok().flatten().map(|name| korean(&name.as_bytes())).unwrap_or_default();
                    let number = |field: &str| one.get::<Option<i64>>(field).ok().flatten().unwrap_or(-1);
                    serde_json::json!([name, number("X"), number("Y"), number("TYPE")])
                })
                .collect();
            (korean(&map.as_bytes()), people.into())
        })
        .collect();
    found.into()
}
