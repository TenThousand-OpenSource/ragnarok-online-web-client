//! What the client's settings of its service say of the aura of the highest level.

use super::*;

/// `{"highest": 99, "homunculus": 275, "plain": [200, 976], "baby": [200, 977], "high": [0, 0]}`, out of
/// `Lua Files\service_korea\ExternalSettings_kr` (the file `CSession::LoadExternalSettings` runs for the
/// Korean service, the one this client is of), as the client asks it through the file's own
/// `GetTableIntValueForC`: the highest base level (`MaxLevelTable.BaseLevel`) and a homunculus's
/// (`BaseLevelHomun`), and the effects of `Level99AuraTable`, each with its `_sub`: for all
/// (`Default99LvAura`), for the baby jobs (`Baby99LvAura`), for the high jobs (`SecondHigh99LvAura`). What
/// the file does not say is 0.
pub fn aura(grf: &Grf) -> serde_json::Value {
    told(&run(grf, &["service_korea\\externalsettings_kr"]))
}

/// The same, of a Lua state the file has run in.
fn told(lua: &Lua) -> serde_json::Value {
    let number = |table: &str, key: &str| -> i64 {
        let value = lua.globals().get::<Table>(table).and_then(|table| table.get::<Value>(key));
        value.ok().as_ref().and_then(whole).unwrap_or(0).max(0)
    };
    let pair = |key: &str| [number("Level99AuraTable", key), number("Level99AuraTable", &format!("{key}_sub"))];
    serde_json::json!({
        "highest": number("MaxLevelTable", "BaseLevel"),
        "homunculus": number("MaxLevelTable", "BaseLevelHomun"),
        "plain": pair("Default99LvAura"),
        "baby": pair("Baby99LvAura"),
        "high": pair("SecondHigh99LvAura"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_levels_and_the_effects_are_read_and_what_is_not_said_is_nothing() {
        let lua = Lua::new();
        lua.load("MaxLevelTable = { BaseLevel = 99, BaseLevelHomun = 275 } Level99AuraTable = { Default99LvAura = 200, Default99LvAura_sub = 976, Baby99LvAura = 200, Baby99LvAura_sub = 977, SecondHigh99LvAura = 0 }").exec().expect("the tables");
        assert_eq!(told(&lua), serde_json::json!({ "highest": 99, "homunculus": 275, "plain": [200, 976], "baby": [200, 977], "high": [0, 0] }));
        assert_eq!(told(&Lua::new())["highest"], 0);
    }
}
