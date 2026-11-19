//! What the client's Lua says of how characters, monsters and NPCs are drawn: how large the shadow under each
//! is, and in which order what is worn on the head is laid over the rest.

use super::*;

/// `{"shadows": {"<job>": factor}, "layers": {"usual": [low, top, middle], "views": {"<view>": [[8 numbers] × 3]}}}`.
///
/// - `shadows`: how many times its size the shadow under a job (a player's, or the number a monster or an NPC
///   is shown under) is drawn, as `CSession::GetNpcLocationRadius` gives it to the actors: twice the radius
///   `GetNpcLocationRadius` names and one more, for the few it names one, else what `ReqshadowFactor` says
///   (`ShadowTable.lub`). A job neither names is drawn with a shadow of its own size and is left out.
/// - `layers`: the priority of what is worn on the head, by where it is worn (the client's layers 2, 3 and 4:
///   low, top, middle) and the direction it is seen from (0 to 7), as `GetLayerPriority(direction, layer,
///   view)` gives it (`TB_Layer_Priority.lub`): the lower is drawn first. `usual` is what a view the table
///   does not name gets; `views` has those that differ from it anywhere, each with its three rows.
pub fn figures(grf: &Grf) -> serde_json::Value {
    let lua = run(grf, &["jobidentity", "npcidentity", "accessoryid", "spriterobeid", "shadowtable", "shadowtable_f", "npclocationradius", "npclocationradius_f", "tb_layer_priority"]);
    let globals = lua.globals();
    let numbers = |table: &str| -> Vec<i64> {
        let Ok(table) = globals.get::<Table>(table) else { return Vec::new() };
        table.pairs::<Value, Value>().flatten().filter_map(|(key, _)| whole(&key)).collect()
    };
    let mut shadows = serde_json::Map::new();
    if let Ok(factor) = globals.get::<mlua::Function>("ReqshadowFactor") {
        for job in numbers("ShadowFactorTable") {
            if let Ok(factor) = factor.call::<f64>(job) {
                shadows.insert(job.to_string(), factor.into());
            }
        }
    }
    if let Ok(radius) = globals.get::<mlua::Function>("GetNpcLocationRadius") {
        for job in numbers("NpcLocationRadius") {
            if let Some(radius) = radius.call::<i64>(job).ok().filter(|radius| *radius != 0) {
                shadows.insert(job.to_string(), (radius * 2 + 1).into());
            }
        }
    }
    let mut views = serde_json::Map::new();
    let mut usual = [-1i64; 3];
    if let Ok(priority) = globals.get::<mlua::Function>("GetLayerPriority") {
        let rows = |view: i64| -> [[i64; 8]; 3] { [2, 3, 4].map(|layer| std::array::from_fn(|direction| priority.call::<i64>((direction as i64, layer, view)).unwrap_or(-1))) };
        // A view no table names: what every other gets.
        let plain = rows(i64::from(i32::MAX));
        usual = plain.map(|row| row[0]);
        let listed = globals.get::<Table>("TB_Layer_Priority").and_then(|table| table.get::<Table>("Items_List"));
        for (view, _) in listed.into_iter().flat_map(|list| list.pairs::<i64, Value>().flatten().collect::<Vec<_>>()) {
            let own = rows(view);
            if own != plain {
                views.insert(view.to_string(), serde_json::json!(own));
            }
        }
    }
    serde_json::json!({ "shadows": shadows, "layers": { "usual": usual, "views": views } })
}
