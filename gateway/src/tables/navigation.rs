//! The navigation: the client's tables of the maps it guides across, who and what is on each, the ways out of
//! each and what walking between them costs.

use std::collections::HashMap;

use super::*;

/// The client's files of the navigation, as `CSession::LoadNavigationScripts` loads them for the Korean main
/// service (`Navi_Map_krpri` and the rest; this client's folder has no other).
const FILES: [&str; 8] = [
    "navigation\\navi_map_krpri",
    "navigation\\navi_npc_krpri",
    "navigation\\navi_mob_krpri",
    "navigation\\navi_link_krpri",
    "navigation\\navi_linkdistance_krpri",
    "navigation\\navi_npcdistance_krpri",
    "navigation\\navi_scroll_krpri",
    "navigation\\navi_picknpc_krpri",
];

/// The names the translated client gives in place of the tables' Korean ones: `SystemEN\Navi_Data.lub`, which
/// its `navi_f_krpri.lub` runs (`dofile`) and whose four tables its `queryNavi_…` functions look a name up in.
#[derive(Default)]
struct Names {
    maps: HashMap<String, String>,
    npcs: HashMap<String, String>,
    mobs: HashMap<String, String>,
    links: HashMap<String, String>,
}

impl Names {
    /// Reads them from beside the archives; none where the file is not there (the tables' own names stay).
    fn read(client_dir: &std::path::Path) -> Self {
        let Some(lua) = beside(client_dir, &["SystemEN/Navi_Data.lub"]) else { return Self::default() };
        let table = |name: &str| -> HashMap<String, String> {
            let Ok(table) = lua.globals().get::<Table>(name) else { return HashMap::new() };
            table.pairs::<mlua::String, mlua::String>().flatten().map(|(key, value)| (korean(&key.as_bytes()), latin(&value.as_bytes()))).collect()
        };
        Self { maps: table("Navi_Data_Map"), npcs: table("Navi_Data_NPC"), mobs: table("Navi_Data_Mob"), links: table("Navi_Data_Link") }
    }
}

/// A row with the text at `place` given its other name, where the names have one.
fn renamed(mut row: serde_json::Value, place: usize, names: &HashMap<String, String>) -> serde_json::Value {
    if let Some(other) = row.get(place).and_then(|name| name.as_str()).and_then(|name| names.get(name)) {
        row[place] = other.clone().into();
    }
    row
}

/// The rows of a table, each as `named` leaves it.
fn rows(table: serde_json::Value, named: impl Fn(serde_json::Value) -> serde_json::Value) -> Vec<serde_json::Value> {
    match table {
        serde_json::Value::Array(rows) => rows.into_iter().map(named).collect(),
        _ => Vec::new(),
    }
}

/// What walking from each way out of a map, or to each NPC of it, costs (`Navi_Distance`, `Navi_NpcDistance`):
/// the client's tables are a map's name, how many entries it has and the entries, one after another; an
/// entry is the number of the way out (or of the NPC) and then its passes, the last of them a mark of the
/// end (`"NULL"`, or a pass to nothing at no cost). Given as `[[map, number, [pass, …]], …]`, the passes as
/// the tables have them and the mark of the end left out.
fn ways(table: &serde_json::Value) -> Vec<serde_json::Value> {
    let Some(table) = table.as_array() else { return Vec::new() };
    let mut ways = Vec::new();
    for group in table.chunks_exact(3) {
        let (Some(map), Some(entries)) = (group[0].as_str(), group[2].as_array()) else { continue };
        for entry in entries {
            let Some(entry) = entry.as_array().filter(|entry| !entry.is_empty()) else { continue };
            let ended = |pass: &&serde_json::Value| {
                let first = pass.get(0).and_then(|kind| kind.as_str()).unwrap_or_default();
                first != "NULL" && !(pass.get(1).and_then(|id| id.as_i64()) == Some(0) && pass.get(2).and_then(|cost| cost.as_i64()) == Some(0))
            };
            let passes: Vec<&serde_json::Value> = entry[1..].iter().take_while(ended).collect();
            ways.push(serde_json::json!([map, entry[0], passes]));
        }
    }
    ways
}

/// The server's own tables, where it has them: the same files under the same names, as rAthena's
/// `map-server-generator --generate-navi` writes them from the server's scripts (`src/map/navi.cpp`; made here
/// by `server/navigation.sh`), in `folder`. None where the folder has no table of the maps. The generator
/// writes no scrolls and no dressed NPCs (`Navi_Scroll`, `navi_picknpc`): those are then empty.
fn of_server(folder: &std::path::Path) -> Option<Lua> {
    let file = |name: &str| folder.join(format!("{}.lub", name.rsplit('\\').next().unwrap_or(name)));
    if !file(FILES[0]).is_file() {
        return None;
    }
    let lua = state();
    for name in FILES {
        let Ok(bytes) = std::fs::read(file(name)) else { continue };
        if let Err(error) = exec(&lua, &bytes, None) {
            eprintln!("tables: {}: {error}", file(name).display());
        }
    }
    Some(lua)
}

/// What the client's own table calls each map, by the map (`Navi_Map` of the archives, in the translated
/// client's words): the generator calls a map by its file's name, and this is put in its place.
fn titles(grf: &Grf, names: &Names) -> HashMap<String, serde_json::Value> {
    let table = run(grf, &FILES[..1]).globals().get::<Value>("Navi_Map").map(|table| json_of(&table)).unwrap_or(serde_json::Value::Null);
    let titled = |row: serde_json::Value| Some((row.get(0)?.as_str()?.to_owned(), renamed(row.clone(), 1, &names.maps).get(1)?.clone()));
    rows(table, |row| row).into_iter().filter_map(titled).collect()
}

/// A row of the server's table of maps with the client's name for the map, where the client knows the map.
fn titled(mut row: serde_json::Value, titles: &HashMap<String, serde_json::Value>) -> serde_json::Value {
    if let Some(title) = row.get(0).and_then(|map| map.as_str()).and_then(|map| titles.get(map)) {
        row[1] = title.clone();
    }
    row
}

/// The navigation's tables for the browser (`GET /tables/navigation.json`), as `CNavigationManager::LoadScript`
/// reads them through the files' own `queryNavi_…` functions: `{"maps": [[map, name, kind, width, height], …],
/// "npcs": [[map, number, kind, job, name, name in the script, x, y], …], "mobs": [[map, number, kind, job and
/// how many, name, name in the database, level, what it is], …], "links": [[map, number, kind, job, name, name
/// in the script, x, y, map it leads to, x, y there], …], "ways": …, "npc_ways": … (`ways`), "picks": [[map, x,
/// y, job, sex, …], …]}`. The names are the translated client's where it has them (`Names`): a map's, an NPC's
/// unless its job is 99999, a monster's and a way's.
///
/// The tables are the server's own where `server_dir` holds them (`of_server`: who stands where, the ways out
/// and the monsters of the world this server runs), else the client's, which are the Korean service's.
pub fn navigation(grf: &Grf, client_dir: &std::path::Path, server_dir: &std::path::Path) -> serde_json::Value {
    let names = Names::read(client_dir);
    let own = of_server(server_dir);
    let titles = if own.is_some() { titles(grf, &names) } else { HashMap::new() };
    let lua = own.unwrap_or_else(|| run(grf, &FILES));
    let table = |name: &str| lua.globals().get::<Value>(name).map(|table| json_of(&table)).unwrap_or(serde_json::Value::Null);
    let npc = |row: serde_json::Value| if row.get(3).and_then(|job| job.as_i64()) == Some(99999) { row } else { renamed(row, 4, &names.npcs) };
    serde_json::json!({
        "maps": rows(table("Navi_Map"), |row| titled(renamed(row, 1, &names.maps), &titles)),
        "npcs": rows(table("Navi_Npc"), npc),
        "mobs": rows(table("Navi_Mob"), |row| renamed(row, 4, &names.mobs)),
        "links": rows(table("Navi_Link"), |row| renamed(row, 4, &names.links)),
        "ways": ways(&table("Navi_Distance")),
        "npc_ways": ways(&table("Navi_NpcDistance")),
        "picks": rows(table("navi_picknpc"), |row| row),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_replaced_where_the_translation_has_one() {
        let names = HashMap::from([("프론테라".to_owned(), "Prontera".to_owned())]);
        assert_eq!(renamed(serde_json::json!(["prontera", "프론테라", 5001]), 1, &names), serde_json::json!(["prontera", "Prontera", 5001]));
        assert_eq!(renamed(serde_json::json!(["moc", "모로크", 5001]), 1, &names), serde_json::json!(["moc", "모로크", 5001]));
    }

    #[test]
    fn the_costs_are_listed_way_by_way_without_the_mark_of_the_end() {
        let table = serde_json::json!([
            "abbey01", 2, [[23098, ["E", 23099, 678], ["P", 23107, 6], ["NULL", 0, 0]], [23099, ["NULL", 0, 0]]],
            "abyss_03", 1, [[30330, ["abyss_02", 22338, 69], ["", 0, 0]]]
        ]);
        assert_eq!(ways(&table), [
            serde_json::json!(["abbey01", 23098, [["E", 23099, 678], ["P", 23107, 6]]]),
            serde_json::json!(["abbey01", 23099, []]),
            serde_json::json!(["abyss_03", 30330, [["abyss_02", 22338, 69]]])
        ]);
        assert!(ways(&serde_json::Value::Null).is_empty());
    }

    #[test]
    fn the_servers_own_tables_are_read_from_their_folder_and_none_from_an_empty_one() {
        let folder = std::env::temp_dir().join(format!("roweb-navigation-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        assert!(of_server(&folder).is_none());
        // As rAthena's generator writes them: plain Lua, a map called by its file's name.
        std::fs::write(folder.join("navi_map_krpri.lub"), "Navi_Map = {\n\t{\"prontera\", \"prontera\", 5001, 312, 392},\n}\n").unwrap();
        let costs = "Navi_Distance = {\n\t\"prontera\", 1,\n\t{\n\t\t{ 13350, -- ( prontera, 1, 2)\n\t\t\t{ \"E\", 13351, 40},\n\t\t\t{\"NULL\", 0, 0}\n\t\t},\n\t},\n}\n";
        std::fs::write(folder.join("navi_linkdistance_krpri.lub"), costs).unwrap();
        let lua = of_server(&folder).expect("the tables");
        let table = |name: &str| json_of(&lua.globals().get::<Value>(name).unwrap());
        let titles = HashMap::from([("prontera".to_owned(), serde_json::json!("Prontera"))]);
        assert_eq!(rows(table("Navi_Map"), |row| titled(row, &titles)), [serde_json::json!(["prontera", "Prontera", 5001, 312, 392])]);
        assert_eq!(ways(&table("Navi_Distance")), [serde_json::json!(["prontera", 13350, [["E", 13351, 40]]])]);
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
