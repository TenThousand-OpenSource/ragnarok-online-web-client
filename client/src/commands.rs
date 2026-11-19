//! The client's chat commands: the words typed after a slash, and what each asks for.
//!
//! `commands.txt` is the client's own table, written out by `tools/client-tables/commands.py`: every command
//! with the name the client has for what it asks (`TT_…`, which for the options of the Game Settings is the
//! option's own name), and where the message table has what is said when something goes on or off.

/// The lines of `commands.txt` of one sort, each as its two fields after the first.
fn lines(sort: &'static str) -> impl Iterator<Item = (&'static str, &'static str)> {
    crate::tables::sorted(include_str!("commands.txt"), sort).filter_map(|fields| Some((*fields.first()?, *fields.get(1)?)))
}

/// What a command asks for, by the client's name for it (`CSession::GetTalkType`): commands are told apart
/// whatever the size of their letters.
pub fn asks(command: &str) -> Option<&'static str> {
    let command = command.to_lowercase();
    lines("command").find(|(named, _)| *named == command).map(|(_, asks)| asks)
}

/// Every command of the client's table, with what it asks for.
pub fn all() -> impl Iterator<Item = (&'static str, &'static str)> {
    lines("command")
}

/// The line of the client's message table that a message's name stands for (`MSI_BGM_ON`).
pub fn message(name: &str) -> Option<usize> {
    lines("message").find(|(named, _)| *named == name).and_then(|(_, line)| line.parse().ok())
}

/// A line typed into the chat as a command: its first word, and the rest with the spaces around it taken off.
/// Nothing for a line that does not begin with a slash.
pub fn split(words: &str) -> Option<(&str, &str)> {
    let words = words.trim();
    words.starts_with('/').then(|| words.split_once(char::is_whitespace).map_or((words, ""), |(first, rest)| (first, rest.trim())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_ask_for_what_the_client_says() {
        assert_eq!(asks("/who"), Some("TT_REQ_NUM_USER"));
        assert_eq!(asks("/W"), Some("TT_REQ_NUM_USER"));
        assert_eq!(asks("/nc"), Some("TT_NOCTRL_ON_OFF"));
        assert_eq!(asks("/snap"), Some("TT_ATTACK_SNAP_ON_OFF"));
        assert_eq!(asks("/exall"), Some("TT_EX_WHISPER_ALL_BLOCK"));
        // `/help` is in the table twice: the first holds.
        assert_eq!(asks("/help"), Some("TT_EX_HELP"));
        assert_eq!(asks("/nothing"), None);
        assert!(all().count() > 200);
    }

    #[test]
    fn messages_are_found_by_name() {
        assert_eq!(message("MSI_BGM_ON"), Some(31));
        assert_eq!(message("MSI_SOUND_OFF"), Some(28));
        assert_eq!(message("MSI_NONE"), None);
        // Past the three names the client misspells (`MIS_…`, lines 2077 to 2079) a line is where the client's
        // own list of names has it (`messageNames`), which is where `msgstringtable.txt` has it too: "Aura
        // effect is ON" and "…OFF" are its lines 2994 and 2995.
        assert_eq!((message("MSI_AURA_EFFECT_ON"), message("MSI_AURA_EFFECT_OFF")), (Some(2994), Some(2995)));
        assert_eq!((message("MSI_CREATE_GUILD_ON"), message("MSI_MONSTER_HP_ON")), (None, Some(2180)));
    }

    #[test]
    fn a_line_is_split_into_command_and_rest() {
        assert_eq!(split("/ex  Some Name "), Some(("/ex", "Some Name")));
        assert_eq!(split("/who"), Some(("/who", "")));
        assert_eq!(split("hello /who"), None);
    }
}
