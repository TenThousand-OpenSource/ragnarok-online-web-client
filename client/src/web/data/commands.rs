//! The chat commands: the help the client prints for them, and what an option says when it is switched.

use super::*;

thread_local! {
    /// The lines of `/help` (the client's `HelpMsgStr`).
    static HELP: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// By option of the Game Settings: the names of the messages said when it goes on and when it goes off.
    static SAYS: RefCell<HashMap<String, (String, String)>> = RefCell::new(HashMap::new());
    /// By option: whether its row starts ticked, for those the client's `CmdOnOffList` has a command of.
    static STARTS: RefCell<HashMap<&'static str, bool>> = RefCell::new(HashMap::new());
}

/// How an option's row starts by the client's list of its switches (`System\OptionInfo.lub`,
/// `crate::commands::starts`); none for an option the list has no command of, and with a gateway that does
/// not read the list.
pub fn option_listed(option: &str) -> Option<bool> {
    STARTS.with_borrow(|starts| starts.get(option).copied())
}

/// The lines the client prints for `/help`.
pub fn command_help() -> Vec<String> {
    HELP.with_borrow(Clone::clone)
}

/// What the client says when an option was switched on or off: the line of its message table (`GameSettings.lub`
/// names the message; the client's own table of names says which line it is). None for an option that says
/// nothing.
pub fn option_says(option: &str, on: bool) -> Option<String> {
    let name = SAYS.with_borrow(|says| says.get(option).map(|(said_on, said_off)| if on { said_on.clone() } else { said_off.clone() }))?;
    message(crate::commands::message(&name)?)
}

/// Whether the Game Settings have an option of that name that is switched on and off (not one that does
/// something at a press).
pub fn option_switched(option: &str) -> bool {
    [1, 2, 4].into_iter().flat_map(options).any(|known| known.id == option && !known.run)
}

/// Takes its tables out of what the gateway gave (`tables/commands.rs`).
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let commands = field(tables, "commands");
    // A gateway older than this table gives none: then there is no help and the options say nothing.
    HELP.set(texts(&field(&commands, "help")));
    SAYS.set(
        entries::<String>(&field(&commands, "says"))
            .into_iter()
            .filter_map(|(option, messages)| {
                let messages: js_sys::Array = messages.dyn_into().ok()?;
                Some((option, (messages.get(0).as_string()?, messages.get(1).as_string()?)))
            })
            .collect(),
    );
    STARTS.set(
        entries::<String>(&field(&commands, "starts"))
            .into_iter()
            .filter_map(|(command, on)| crate::commands::starts(&command, on.as_bool()?))
            .collect(),
    );
}
