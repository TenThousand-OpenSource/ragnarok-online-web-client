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

/// How the client writes a ranking of each kind (`RankNoticeStyles` and `RankingBoardStyles`, the same table
/// twice): its heading, the colour of the heading and of the rules, the colour of the lines between (the
/// client's `0xBBGGRR` turned). A kind it has no row for gets a rule as its heading, and black.
fn ranking_style(kind: u16) -> (&'static str, &'static str, &'static str) {
    match kind {
        0 => ("========== BLACKSMITH RANK ============", "#9b9b9b", "#cdcdcd"),
        1 => ("=========== ALCHEMIST RANK =============", "#9bcd9b", "#cdffcd"),
        2 => ("============ TAEKWON RANK =============", "#9b9bcd", "#cdcdff"),
        3 => ("=========== KILLER RANK ==============", "#9b9b9b", "#cdcdcd"),
        _ => (RANKING_RULE, "#000000", "#000000"),
    }
}

/// The rule under the ten and around the player's own points.
const RANKING_RULE: &str = "===========================================";

/// The line of the client's message table a ranking asked by number says while the names are not all known
/// (`MSI_RANK_LOADING`), how long it waits for them, and how often it looks (`CSession::SetShowRankingSchedule`:
/// a schedule of 1000 ms with a delay of 3000).
pub const RANKING_LOADING: usize = 3481;
pub const RANKING_WAITS: f64 = 3000.0;
pub const RANKING_LOOKS_EVERY: i32 = 1000;

/// A ranking as the client writes it into the chat, each line with its colour: the heading of its kind, a
/// line to each of the ten, a rule, the player's own points and a rule again. `padded` is the ranking told
/// with its names (`CGameMode::ReceiveRankingList`: "[ %2d ] %-18s  :  %4d POINT"); the one told by numbers
/// writes a name as long as it is (`CShowRankingScheduler::Show`: "[ %2d ] %s  :  %4d POINT").
pub fn ranking_lines(kind: u16, ranked: &[(String, u32)], own: u32, padded: bool) -> Vec<(String, &'static str)> {
    let (heading, ruled, lined) = ranking_style(kind);
    let width = if padded { 18 } else { 0 };
    let mut lines = vec![(heading.to_owned(), ruled)];
    lines.extend(
        ranked
            .iter()
            .enumerate()
            .map(|(place, (name, points))| (format!("[ {:2} ] {name:<width$}  :  {points:4} POINT", place + 1), lined)),
    );
    lines.extend([(RANKING_RULE.to_owned(), ruled), (format!("MY POINT    :  {own:4} POINT "), lined), (RANKING_RULE.to_owned(), ruled)]);
    lines
}

/// The ten of a ranking told by numbers, each with its name, and whether none of the names is still waited
/// for (`CShowRankingScheduler::Process` and `Show`): a place nobody has (number 0) and a character whose
/// name is not known are written with `nameless` (`MSI_NAMELESS_USER`), and only the second is waited for.
pub fn ranking_named(ranked: &[(u32, u32)], name: impl Fn(u32) -> Option<String>, nameless: &str) -> (Vec<(String, u32)>, bool) {
    let names: Vec<Option<String>> = ranked.iter().map(|(character, _)| (*character != 0).then(|| name(*character)).flatten()).collect();
    let known = ranked.iter().zip(&names).all(|((character, _), name)| *character == 0 || name.is_some());
    let named = ranked.iter().zip(names).map(|((_, points), name)| (name.unwrap_or_else(|| nameless.to_owned()), *points)).collect();
    (named, known)
}

/// The line of the client's message table that tells of points of fame gained, and its colour, by the kind
/// of ranking (`CGameMode::ReceiveTypedNotice`, its name in the decompilation for the reader of
/// ZC_UPDATE_RANKING_POINT: `MSI_BLACKSMITH_POINT`, `MSI_ALCHEMIST_POINT`, `MSI_TAEKWON_POINT`,
/// `MSI_KILLER_POINT`); none for a kind it does not know.
pub fn fame_line(kind: u16) -> Option<(usize, &'static str)> {
    Some(match kind {
        0 => (901, "#cdcdcd"),
        1 => (902, "#cdffcd"),
        2 => (926, "#cdcdff"),
        3 => (953, "#cdcdcd"),
        _ => return None,
    })
}

/// The five lines the client writes of ZC_ACK_STATUS_GM, each with its colour (`CGameMode::ReceiveStatDump`:
/// yellow `0xFFFF` and green `0xFF37` in turn). The values come in the packet's order: six stats each followed by
/// its standard value, then the fourteen numbers after them.
pub fn stat_dump(stats: [u8; 12], more: [i16; 14]) -> [(String, &'static str); 5] {
    let [strength, standard_str, agi, standard_agi, vit, standard_vit, int, standard_int, dex, standard_dex, luk, standard_luk] = stats;
    let [att, refining, max_matk, min_matk, item_def, plus_def, mdef, plus_mdef, hit, avoid, plus_avoid, _critical, aspd, plus_aspd] = more;
    const GREEN: &str = "#37ff00";
    const YELLOW: &str = "#ffff00";
    [
        (
            format!("             STR={strength:3}      AGI={agi:3}      VIT={vit:3}      INT={int:3}      DEX={dex:3}      LUK={luk}"),
            YELLOW,
        ),
        (
            [
                format!("standard STR={standard_str:3}      AGI={standard_agi:3}      VIT={standard_vit:3}"),
                format!("      INT={standard_int:3}      DEX={standard_dex:3}      LUK={standard_luk:2}"),
            ]
            .concat(),
            GREEN,
        ),
        (
            format!("    attPower={att:3}    refiningPow={refining:3}    MAXmatPow={max_matk:3}    MINmatPower={min_matk:3}       ASPD={aspd:3}"),
            YELLOW,
        ),
        (
            format!("itemdefPow={item_def:3}     plusdefPow={plus_def:3}    mdefPower={mdef:3}    plusmdefPow={plus_mdef:3}    plusASPD={plus_aspd:3}"),
            GREEN,
        ),
        (format!("hitSuccessVal={hit:3}    avoidSuccessVal={avoid:3}    plusAvoidSuccessValue={plus_avoid:3}"), YELLOW),
    ]
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
    fn a_ranking_is_written_as_the_client_writes_it() {
        let lines = ranking_lines(2, &[("Kick".to_owned(), 1200)], 7, true);
        assert_eq!(lines[0], ("============ TAEKWON RANK =============".to_owned(), "#9b9bcd"));
        assert_eq!(lines[1], ("[  1 ] Kick                :  1200 POINT".to_owned(), "#cdcdff"));
        assert_eq!(lines[3], ("MY POINT    :     7 POINT ".to_owned(), "#cdcdff"));
        assert_eq!((lines.len(), &lines[2], &lines[4]), (5, &("=".repeat(43), "#9b9bcd"), &("=".repeat(43), "#9b9bcd")));
        // The ranking told by numbers writes a name as long as it is.
        assert_eq!(ranking_lines(0, &[("Kick".to_owned(), 5)], 0, false)[1], ("[  1 ] Kick  :     5 POINT".to_owned(), "#cdcdcd"));
        assert_eq!(ranking_lines(1, &[], 0, true)[0], ("=========== ALCHEMIST RANK =============".to_owned(), "#9bcd9b"));
        assert_eq!(ranking_lines(3, &[], 0, true)[0].0, "=========== KILLER RANK ==============");
        // A kind the client has no row for: a rule for a heading, in black.
        assert_eq!(ranking_lines(9, &[], 0, true)[0], ("=".repeat(43), "#000000"));
    }

    #[test]
    fn a_ranking_by_numbers_waits_for_the_names_it_does_not_know() {
        let name = |character: u32| (character == 7).then(|| "Kick".to_owned());
        let (named, known) = ranking_named(&[(7, 30), (0, 0)], name, "Nameless");
        assert_eq!((named, known), (vec![("Kick".to_owned(), 30), ("Nameless".to_owned(), 0)], true));
        let (named, known) = ranking_named(&[(7, 30), (8, 20)], name, "Nameless");
        assert_eq!((named[1].clone(), known), (("Nameless".to_owned(), 20), false));
    }

    #[test]
    fn fame_gained_and_a_dump_of_stats_have_the_clients_lines() {
        assert_eq!((fame_line(0), fame_line(1), fame_line(2)), (Some((901, "#cdcdcd")), Some((902, "#cdffcd")), Some((926, "#cdcdff"))));
        assert_eq!((fame_line(3), fame_line(4)), (Some((953, "#cdcdcd")), None));
        let lines = stat_dump([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], [100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113]);
        assert_eq!(lines[0], ("             STR=  1      AGI=  3      VIT=  5      INT=  7      DEX=  9      LUK=11".to_owned(), "#ffff00"));
        assert_eq!(lines[1], ("standard STR=  2      AGI=  4      VIT=  6      INT=  8      DEX= 10      LUK=12".to_owned(), "#37ff00"));
        assert_eq!(lines[2].0, "    attPower=100    refiningPow=101    MAXmatPow=102    MINmatPower=103       ASPD=112");
        assert_eq!(lines[3].0, "itemdefPow=104     plusdefPow=105    mdefPower=106    plusmdefPow=107    plusASPD=113");
        assert_eq!(lines[4], ("hitSuccessVal=108    avoidSuccessVal=109    plusAvoidSuccessValue=110".to_owned(), "#ffff00"));
    }
}
