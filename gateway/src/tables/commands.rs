//! The chat commands: the help the client prints for them, and what its options say when a command switches one.

use super::*;

/// `{"help": [line, …], "says": {"TT_…": ["MSI_…", "MSI_…"]}}`: the lines of `/help` (`Lua Files\DataInfo\
/// HelpMsgStr`: its table of that name, which the client's `ShowHelpMessages` goes through line by line), and for
/// each option of the Game Settings the names of the messages said when it goes on and off (the `MSGs` of
/// `Lua Files\OptionInfo\GameSettings`, as `CGameSettingsMgr::LoadSettings` reads them: `interface::game_settings`,
/// read already and given here as `settings`). And `"starts": {"/noctrl": true, …}`: how the client's switches
/// stand as it starts, by their commands (`CmdOnOffList` of `System\LuaFiles514\OptionInfo.lub`, which
/// `CSession::LoadOptionInfo` has the script's `SetCmdOnOffList` hand to the client one by one).
pub fn commands(grf: &Grf, settings: &Lua, client_dir: &std::path::Path) -> serde_json::Value {
    let lua = run(grf, &["helpmsgstr"]);
    let help: Vec<String> = lua.globals()
        .get::<Table>("HelpMsgStr")
        .map(|table| lines(&table).into_iter().filter(|line| !line.is_empty()).collect())
        .unwrap_or_default();
    let mut says = serde_json::Map::new();
    for entry in interface::option_entries(settings) {
        let id = text(&entry, "ID").unwrap_or_default();
        let Some(messages) = entry.raw_get::<Option<Table>>("MSGs").ok().flatten() else { continue };
        let said = |at: i64| messages.raw_get::<Option<mlua::String>>(at).ok().flatten().map(|text| korean(&text.as_bytes())).unwrap_or_default();
        let (on, off) = (said(1), said(2));
        if !id.is_empty() && !on.is_empty() && !off.is_empty() {
            says.insert(id, serde_json::json!([on, off]));
        }
    }
    let mut starts = serde_json::Map::new();
    let script = beside(client_dir, &["System/LuaFiles514/OptionInfo.lub", "System/OptionInfo.lub"]);
    let listed = script.as_ref().and_then(|lua| lua.globals().get::<Table>("CmdOnOffList").ok());
    // (The list has a text among its numbers, the skin's name: each value is looked at as what it is.)
    for (command, on) in listed.into_iter().flat_map(|list| list.pairs::<mlua::String, Value>().flatten().collect::<Vec<_>>()) {
        if let Some(on) = whole(&on).filter(|_| command.as_bytes().starts_with(b"/")) {
            starts.insert(korean(&command.as_bytes()), (on != 0).into());
        }
    }
    serde_json::json!({ "help": help, "says": says, "starts": starts })
}
