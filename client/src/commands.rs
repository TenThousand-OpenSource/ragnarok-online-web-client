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

/// A ranking as the client writes it into the chat: a heading with the ranking's name, a line to each of the
/// ten ("[ %2d ] %-18s  :  %4d POINT"), a rule, the player's own points and a rule again.
pub fn ranking_lines(kind: u16, ranked: &[(String, u32)], own: u32) -> Vec<String> {
    let named = ["BLACKSMITH", "ALCHEMIST", "TAEKWON"].get(usize::from(kind)).copied().unwrap_or("KILLER");
    let rule = "=".repeat(39);
    let mut lines = vec![format!("========== {named} RANK ============")];
    lines.extend(
        ranked
            .iter()
            .enumerate()
            .map(|(place, (name, points))| format!("[ {:2} ] {name:<18}  :  {points:4} POINT", place + 1)),
    );
    lines.extend([rule.clone(), format!("MY POINT    :  {own:4} POINT "), rule]);
    lines
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

    #[test]
    fn a_ranking_is_asked_for_read_and_written() {
        let lines = ranking_lines(2, &[("Kick".to_owned(), 1200)], 7);
        assert_eq!(lines[0], "========== TAEKWON RANK ============");
        assert_eq!(lines[1], "[  1 ] Kick                :  1200 POINT");
        assert_eq!(lines[3], "MY POINT    :     7 POINT ");
        assert_eq!((lines.len(), lines[2].as_str(), lines[4].as_str()), (5, "=".repeat(39).as_str(), "=".repeat(39).as_str()));
        // A kind the client has no name for is the killers'.
        assert_eq!(ranking_lines(3, &[], 0)[0], "========== KILLER RANK ============");
    }
}
