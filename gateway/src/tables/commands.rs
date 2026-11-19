//! The chat commands: the help the client prints for them, and what its options say when a command switches one.

use super::*;

/// `{"help": [line, …], "says": {"TT_…": ["MSI_…", "MSI_…"]}}`: the lines of `/help` (`Lua Files\DataInfo\
/// HelpMsgStr`: its table of that name, which the client's `ShowHelpMessages` goes through line by line), and for
/// each option of the Game Settings the names of the messages said when it goes on and off (the `MSGs` of
/// `Lua Files\OptionInfo\GameSettings`, as `CGameSettingsMgr::LoadSettings` reads them: `interface::game_settings`).
pub fn commands(grf: &Grf) -> serde_json::Value {
    let lua = run(grf, &["helpmsgstr"]);
    let help: Vec<String> = lua.globals().get::<Table>("HelpMsgStr").map(|table| lines(&table).into_iter().filter(|line| !line.is_empty()).collect()).unwrap_or_default();
    let options = interface::game_settings(grf);
    let mut says = serde_json::Map::new();
    for entry in interface::option_entries(&options) {
        let id = text(&entry, "ID").unwrap_or_default();
        let Some(messages) = entry.raw_get::<Option<Table>>("MSGs").ok().flatten() else { continue };
        let said = |at: i64| messages.raw_get::<Option<mlua::String>>(at).ok().flatten().map(|text| korean(&text.as_bytes())).unwrap_or_default();
        let (on, off) = (said(1), said(2));
        if !id.is_empty() && !on.is_empty() && !off.is_empty() {
            says.insert(id, serde_json::json!([on, off]));
        }
    }
    serde_json::json!({ "help": help, "says": says })
}
