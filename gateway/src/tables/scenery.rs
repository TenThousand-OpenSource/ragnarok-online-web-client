//! What surrounds a map: the sky the client draws behind it.

use super::*;

/// The sky of each map that has one (`MapSkyData\MapSkyData.lub`, the table `MapSkyData` its functions
/// `GetBackgroundColor`, `IsDrawStar` and `GetOldCloudInfo` read for `CMapSkyMgr::Apply`), by the map's name
/// without `.rsw`: `{"<map>": {"colour": [red, green, blue], "stars": true, "clouds": [1]}}`: the colour behind
/// the map (none given: black), whether stars are drawn, and the kinds of the older clouds
/// (`Old_Cloud_Effect`: what `CWeatherEffect::Play` is called with). The newer clouds (`Cloud_Effect`) and the
/// fog of the background (`BG_Fog`), which only later maps have, are left out.
pub fn sky(grf: &Grf) -> serde_json::Value {
    let lua = run(grf, &["\\MapSkyData\\MapSkyData"]);
    let Ok(table) = lua.globals().get::<Table>("MapSkyData") else { return serde_json::json!({}) };
    let found: serde_json::Map<String, serde_json::Value> = table
        .pairs::<mlua::String, Table>()
        .flatten()
        .map(|(map, entry)| {
            let numbers = |field: &str| entry.get::<Option<Table>>(field).ok().flatten().map(|list| list.sequence_values::<i64>().flatten().collect::<Vec<_>>()).unwrap_or_default();
            let about = serde_json::json!({
                "colour": numbers("BG_Color"),
                "stars": entry.get::<Value>("Star_Effect").is_ok_and(|stars| !matches!(stars, Value::Nil | Value::Boolean(false))),
                "clouds": numbers("Old_Cloud_Effect"),
            });
            (korean(&map.as_bytes()).to_ascii_lowercase().trim_end_matches(".rsw").to_owned(), about)
        })
        .collect();
    found.into()
}
