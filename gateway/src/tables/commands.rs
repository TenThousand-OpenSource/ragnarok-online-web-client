//! The chat commands: the help the client prints for them, and what its options say when a command switches one.

use super::*;

/// `{"help": [line, …], "says": {"TT_…": ["MSI_…", "MSI_…"]}}`: the lines of `/help` (`Lua Files\DataInfo\
/// HelpMsgStr`: its table of that name, which the client's `ShowHelpMessages` goes through line by line), and for
/// each option of the Game Settings the names of the messages said when it goes on and off (the `MSGs` of
/// `Lua Files\OptionInfo\GameSettings`, as `CGameSettingsMgr::LoadSettings` reads them).
pub fn commands(grf: &Grf) -> serde_json::Value {
    let lua = run(grf, &["helpmsgstr"]);
    let globals = lua.globals();
    let help: Vec<String> = globals
        .get::<Table>("HelpMsgStr")
        .map(|lines| lines.sequence_values::<mlua::String>().flatten().map(|line| korean(&line.as_bytes())).filter(|line| !line.is_empty()).collect())
        .unwrap_or_default();
    let options = Lua::new_with(mlua::StdLib::TABLE | mlua::StdLib::STRING | mlua::StdLib::MATH, mlua::LuaOptions::default()).expect("a Lua state");
    // Every name the file uses (the options' own, the pages, the kinds) stands for itself.
    if let Err(error) = options.load("setmetatable(_G, { __index = function(_, name) return name end })").exec() {
        eprintln!("tables: commands: {error}");
    }
    run_in(&options, grf, &["optioninfo\\gamesettings"]);
    let mut says = serde_json::Map::new();
    if let Ok(table) = options.globals().raw_get::<Table>("OptionTbl") {
        for entry in table.sequence_values::<Table>().flatten() {
            let text = |value: Option<mlua::String>| value.map(|text| korean(&text.as_bytes())).unwrap_or_default();
            let id = text(entry.raw_get::<Option<mlua::String>>("ID").ok().flatten());
            let Some(messages) = entry.raw_get::<Option<Table>>("MSGs").ok().flatten() else { continue };
            let on = text(messages.raw_get::<Option<mlua::String>>(1).ok().flatten());
            let off = text(messages.raw_get::<Option<mlua::String>>(2).ok().flatten());
            if !id.is_empty() && !on.is_empty() && !off.is_empty() {
                says.insert(id, serde_json::json!([on, off]));
            }
        }
    }
    serde_json::json!({ "help": help, "says": says })
}
