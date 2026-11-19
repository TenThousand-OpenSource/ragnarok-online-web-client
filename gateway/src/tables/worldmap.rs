//! The world map: the client's worlds, the picture of each, and where each map lies on it.

use super::*;

/// The worlds of the client's World Map window (`Lua Files\worldviewdata\worldviewdata_list` and `_table`, with the
/// names of `_language`; read as its `CWorldViewMgr::Load` does through `queryWorld_List`, `querymaptable_load`
/// and `queryfieldtable_load`): `[{"name": "Midgard", "picture": "worldmap.jpg", "size": [w, h], "fields":
/// [[id, "map", left, top, right, bottom, "name", "level"], …], "groups": [[id, left, top, right, bottom,
/// "name", "levels"], …]}, …]`. A field is one map's rectangle on the picture, in the picture's own points; the
/// fields with an ID under 100 are the floors of a dungeon, whose mark and name are the group of that ID. The
/// picture's size is read from the picture itself (the client scales the table's points by it).
pub fn worldmap(grf: &Grf) -> serde_json::Value {
    let lua = run(grf, &["worldviewdata\\worldviewdata_language", "worldviewdata\\worldviewdata_list", "worldviewdata\\worldviewdata_table"]);
    let Ok(worlds) = lua.globals().get::<Table>("World_List") else { return serde_json::json!([]) };
    worlds
        .sequence_values::<Table>()
        .flatten()
        .filter_map(|world| {
            let text = |at: usize| world.get::<Option<mlua::String>>(at).ok().flatten().map(|text| korean(&text.as_bytes()));
            let (name, fields, groups, picture) = (text(1)?, text(2)?, text(3)?, text(4)?);
            let rows = |table: &str| lua
                .globals()
                .get::<Option<Table>>(table)
                .ok()
                .flatten()
                .map(|table| json_of(&Value::Table(table)))
                .unwrap_or_else(|| serde_json::json!([]));
            let fields = match rows(&fields) {
                serde_json::Value::Array(rows) => rows.into_iter().map(without_rsw).collect(),
                _ => Vec::new(),
            };
            Some(serde_json::json!({ "name": name, "picture": picture, "size": picture_size(grf, &picture), "fields": fields, "groups": rows(&groups) }))
        })
        .collect()
}

/// A row of a world's table with its map named without `.rsw`.
fn without_rsw(mut row: serde_json::Value) -> serde_json::Value {
    if let Some(map) = row.get_mut(1) {
        if let Some(name) = map.as_str().map(|name| name.trim_end_matches(".rsw").to_owned()) {
            *map = name.into();
        }
    }
    row
}

/// How wide and tall a picture of the interface is, read from it as the browser is given it; nothing for one
/// that is not there.
fn picture_size(grf: &Grf, picture: &str) -> [u32; 2] {
    let path = format!("data\\texture\\유저인터페이스\\{picture}");
    let size = grf.read(&path).ok().and_then(|bytes| crate::images::to_png(&path, &bytes).ok()).and_then(|png| png_size(&png));
    size.unwrap_or([0, 0])
}

/// The size a PNG says it has: two numbers of four bytes after its first sixteen.
fn png_size(png: &[u8]) -> Option<[u32; 2]> {
    let number = |at: usize| Some(u32::from_be_bytes(png.get(at..at + 4)?.try_into().ok()?));
    Some([number(16)?, number(20)?])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_png_tells_its_size() {
        let mut png = vec![0x89, b'P', b'N', b'G', 13, 10, 26, 10, 0, 0, 0, 13, b'I', b'H', b'D', b'R'];
        png.extend([0, 0, 5, 0, 0, 0, 3, 0x20]);
        assert_eq!(png_size(&png), Some([1280, 800]));
        assert_eq!(png_size(&png[..18]), None);
    }

    #[test]
    fn a_map_is_named_without_its_ending() {
        assert_eq!(without_rsw(serde_json::json!([112, "malangdo.rsw", 1, 2, 3, 4, "Malangdo", ""])), serde_json::json!([
            112, "malangdo", 1, 2, 3, 4, "Malangdo", ""
        ]));
    }
}
