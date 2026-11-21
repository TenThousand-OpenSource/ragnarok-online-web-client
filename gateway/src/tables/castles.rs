//! The castles guilds hold, as the client's own table has them.

use super::*;

/// `{"<number>": {"name": "Kriemhild", "cost": 100, "siege": 100000}}`, out of `Lua Files\Agit\AgitConfig`
/// (the table `tbl_AgitInfo`, by the castle's map) and `Lua Files\Agit\AgitConfig_F`, the two files
/// `CSession` runs one after the other: every castle by its `Number` (the number the map server names a
/// castle by), with what the client asks the second file's own functions of it: `get_agit_name(number)` for
/// the guild window's list (`UIGuildInfoManageWnd::RefreshAgitList`), and `get_agit_costzeny(number, false)`
/// and `(number, true)` for the zeny going there costs, and costs during a siege (`UIListBoxAgitMenu::SendMsg`).
pub fn castles(grf: &Grf) -> serde_json::Value {
    told(&run(grf, &["\\Agit\\AgitConfig", "\\Agit\\AgitConfig_F"]))
}

/// The same, of a Lua state the two files have run in. A castle the functions say nothing of is left out.
fn told(lua: &Lua) -> serde_json::Value {
    let globals = lua.globals();
    let Ok(table) = globals.get::<Table>("tbl_AgitInfo") else { return serde_json::json!({}) };
    let (Ok(name), Ok(cost)) = (globals.get::<mlua::Function>("get_agit_name"), globals.get::<mlua::Function>("get_agit_costzeny")) else {
        return serde_json::json!({});
    };
    let mut numbers: Vec<i64> =
        table.pairs::<Value, Table>().flatten().filter_map(|(_, castle)| whole(&castle.get::<Value>("Number").ok()?)).collect();
    numbers.sort_unstable();
    let costs = |number: i64, siege: bool| cost.call::<Value>((number, siege)).ok().as_ref().and_then(whole).unwrap_or(0).max(0);
    let found: serde_json::Map<String, serde_json::Value> = numbers
        .into_iter()
        .filter_map(|number| {
            let name = name.call::<Value>(number).ok().as_ref().and_then(text_only).filter(|name| !name.is_empty())?;
            Some((number.to_string(), serde_json::json!({ "name": name, "cost": costs(number, false), "siege": costs(number, true) })))
        })
        .collect();
    found.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_castle_is_listed_by_its_number_with_what_the_client_s_functions_say_of_it() {
        let lua = Lua::new();
        lua.load(
            "tbl_AgitInfo = { ['prtg_cas01.gat'] = { Number = 1, Name = 'Kriemhild', Warp = { CostZeny = 100, CostZenySeigeTime = 100000 } }, \
            ['payg_cas01.gat'] = { Number = 16, Name = 'Bright Arbor', Warp = { CostZeny = 200, CostZenySeigeTime = 5 } }, \
            ['nameless.gat'] = { Number = 40, Name = '', Warp = { CostZeny = 1, CostZenySeigeTime = 1 } } } \
            local function of(number) for _, castle in pairs(tbl_AgitInfo) do if castle.Number == number then return castle end end end \
            function get_agit_name(number) local castle = of(number) return castle and castle.Name or '' end \
            function get_agit_costzeny(number, siege) local castle = of(number) if not castle then return 0 end \
            if siege then return castle.Warp.CostZenySeigeTime end return castle.Warp.CostZeny end"
        ).exec().expect("the tables");
        assert_eq!(told(&lua), serde_json::json!({
            "1": { "name": "Kriemhild", "cost": 100, "siege": 100000 },
            "16": { "name": "Bright Arbor", "cost": 200, "siege": 5 },
        }));
        // A client without the files, or with the table and no functions, has no castles.
        assert_eq!(told(&Lua::new()), serde_json::json!({}));
        let bare = Lua::new();
        bare.load("tbl_AgitInfo = { a = { Number = 1 } }").exec().expect("the table");
        assert_eq!(told(&bare), serde_json::json!({}));
    }
}
