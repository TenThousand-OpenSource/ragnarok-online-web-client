//! The battlegrounds: what the list of them is made of, when applying for one is refused by the client itself,
//! what the server's answers mean, and where the score of a battle is drawn. The client's `CEntryQueueMgr`
//! (`Source/Contents/EntryQueue/EntryQueueMgr.cpp`), `UIEntryQueueWnd` and its three companions
//! (`Source/UI/EntryQueue/UIEntryQueueWnd.cpp`), and `CRenderer::DrawBattleFieldIcon`.

/// A battleground of the client's list (`EntryQueueInfo`, filled by `AddEntryQueue` out of
/// `EntryQueueList.lub`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Entry {
    pub id: u32,
    /// The name the server knows it by.
    pub name: String,
    /// The name shown.
    pub shown: String,
    /// How many fight on each side.
    pub allies: u32,
    pub enemies: u32,
    /// The kind of the level's bounds: 1 up to `level`, 2 from it, 3 from it to `level_end`; anything else, none.
    pub level_kind: u8,
    pub level: u32,
    pub level_end: u32,
    /// What is won, drawn and lost, and the goal, as the list words them.
    pub win: String,
    pub draw: String,
    pub loss: String,
    pub goal: String,
    /// The picture of the map, in the folder `battle_field`.
    pub picture: String,
}

impl Entry {
    /// The line of the message table that words the level's bounds, with the numbers its blanks take
    /// (`UIEntryQueueWnd::SelectQueue`).
    pub fn level_line(&self) -> (usize, Vec<u32>) {
        match self.level_kind {
            1 => (2151, vec![self.level]),
            2 => (2152, vec![self.level]),
            3 => (2153, vec![self.level, self.level_end]),
            _ => (2154, Vec::new()),
        }
    }

    /// How many the queue waits for: both sides (`CEntryQueueMgr::OnStatus` hands the sum to the window).
    pub fn needed(&self) -> u32 {
        self.allies + self.enemies
    }
}

use crate::press::battleground::Way;

/// What CZ_REQ_ENTRY_QUEUE_APPLY calls a way to apply.
pub fn code(way: Way) -> u16 {
    match way {
        Way::Alone => 1,
        Way::Party => 2,
        Way::Guild => 4,
    }
}

/// The line of the message table the client asks with before it applies that way (`UIEntryQueueWnd::SendMsg`).
pub fn question(way: Way) -> usize {
    match way {
        Way::Alone => 2146,
        Way::Party => 2147,
        Way::Guild => 2148,
    }
}

/// Whether someone of that team is fought by one of `own` team on a battleground: anyone of another team, and
/// one the server has named no team of is of team 0 (`CGameMode::HandleActorPick`: `playerTeam !=
/// battleTeam`).
pub fn foe(own: u8, theirs: u8) -> bool {
    own != theirs
}

/// What the client looks at before it lets one apply.
#[derive(Clone, Copy, Debug, Default)]
pub struct Standing {
    /// The map is a battleground itself (the map's property 0x13).
    pub battlefield: bool,
    /// The state `EFST_ENTRY_QUEUE_APPLY_DELAY` (668) is on: a queue was left less than a minute ago.
    pub delayed: bool,
    /// How many the party has (none: no party), whether the player leads it, how many of it are online.
    pub party: usize,
    pub leader: bool,
    pub party_online: usize,
    /// Whether there is a guild, whether the player is its master, how many of it are online.
    pub guild: bool,
    pub master: bool,
    pub guild_online: usize,
}

/// The state that forbids applying for a minute after a queue was left (`EFST_ENTRY_QUEUE_APPLY_DELAY`).
pub const APPLY_DELAY: u32 = 668;

/// Why the client itself does not apply, as the line of the message table it says in a box; `None` when it
/// asks whether the player means it and then applies (`UIEntryQueueWnd::SendMsg`: a side of `allies` is the
/// most a party or a guild may have online).
pub fn refused(way: Way, standing: &Standing, allies: u32) -> Option<usize> {
    if standing.battlefield {
        return Some(2155);
    }
    if standing.delayed {
        return Some(2156);
    }
    match way {
        Way::Alone => None,
        Way::Party if standing.party == 0 => Some(2157),
        Way::Party if !standing.leader => Some(2158),
        Way::Party if (allies as usize) < standing.party_online => Some(2159),
        Way::Party => None,
        Way::Guild if !standing.guild => Some(2160),
        Way::Guild if !standing.master => Some(2161),
        Way::Guild if (allies as usize) < standing.guild_online => Some(2162),
        Way::Guild => None,
    }
}

/// What ZC_ACK_ENTRY_QUEUE_APPLY means (`CEntryQueueMgr::OnApplyResult`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    /// One is in the queue now.
    Queued,
    /// Refused: a line of the message table, said in a box.
    Refused(usize),
    /// A result the client does nothing at.
    Nothing,
}

/// The answer to applying by its number (rAthena's `e_bg_queue_apply_ack`; the client knows a 15 beside them).
pub fn applied(result: u8) -> Applied {
    match result {
        1 => Applied::Queued,
        2..=8 => Applied::Refused(2098 + result as usize),
        // "Only the leader may apply" and "your class cannot": the two lines stand the other way round in
        // the table than the results do in the server's list.
        9 => Applied::Refused(2108),
        10 => Applied::Refused(2107),
        15 => Applied::Refused(2109),
        _ => Applied::Nothing,
    }
}

/// What ZC_ACK_ENTRY_QUEUE_CANCEL says in a box, and whether the window of the queue closes with it
/// (`CEntryQueueMgr::OnCancelResult`). Whatever the result, the chat is told that the queue was left.
pub fn cancelled(result: u8) -> Option<(usize, bool)> {
    match result {
        1 => Some((2110, true)),
        3 => Some((2111, false)),
        11 => Some((2112, false)),
        14 => Some((2113, true)),
        _ => None,
    }
}

/// What ZC_REPLY_ACK_ENTRY_QUEUE_ADMISSION says in a box (`CEntryQueueMgr::OnResultNotice`).
pub fn admission_refused(result: u8) -> Option<usize> {
    match result {
        3 => Some(2111),
        11 => Some(2112),
        _ => None,
    }
}

/// What of a line typed goes to the battleground's own chat while that is switched on (`CGameMode`'s
/// `TT_NORMAL`, which takes the mark of a line for the party, the guild or the clan off its front before it
/// looks whether the battle chat is on): the words without such a mark; none when nothing is left.
pub fn chat_words(typed: &str) -> Option<&str> {
    let words = typed.strip_prefix(['%', '$', '#']).unwrap_or(typed);
    (!words.is_empty()).then_some(words)
}

/// How long the window that asks whether to enter waits, in seconds (`UIEntryQueueRequestWnd::Paint`).
pub const ENTRY_WAIT: i32 = 20;

/// The seconds the window that asks whether to enter still shows, so many thousandths after it opened; at
/// nought or less it answers no by itself (`UIEntryQueueRequestWnd::Paint`: `20 - elapsed / 1000 % 60`, the
/// time counted in whole seconds).
pub fn entry_seconds(elapsed: u32) -> i32 {
    ENTRY_WAIT - (elapsed / 1000 % 60) as i32
}

/// The two emblems of the score: the picture and its top, each 48 square, 280 left of the screen's right edge
/// (`CRenderer::DrawBattleFieldIcon`; drawn with the alpha 0xCD).
pub const EMBLEMS: [(&str, i32); 2] = [("effect/사자(A).bmp", 26), ("effect/독수리(B).bmp", 84)];
/// Where the emblems begin, counted from the screen's right edge, and their side.
pub const EMBLEM_LEFT: i32 = -280;
pub const EMBLEM_SIDE: i32 = 48;

/// The digits of a battle's score (`CRenderer::DrawBattleFieldIcon`): each an action of the sprite
/// `이팩트\RankFont` with where its origin stands, across from the screen's right edge and down from its top.
/// Nothing while either score is below nought, which is how the client keeps "no score". A score is written
/// to two digits; both lines stand 32 further right when either has two.
pub fn score(first: i32, second: i32) -> Vec<(usize, i32, i32)> {
    if first < 0 || second < 0 {
        return Vec::new();
    }
    let right = if first > 9 || second > 9 { -175 } else { -207 };
    let mut digits = Vec::new();
    for (points, y) in [(first, 50), (second, 108)] {
        digits.push(((points % 10) as usize, right, y));
        if points > 9 {
            digits.push(((points / 10 % 10) as usize, right - 32, y));
        }
    }
    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_level_is_worded_by_the_kind_of_its_bounds() {
        let entry = |level_kind, level, level_end| Entry { level_kind, level, level_end, ..Entry::default() };
        assert_eq!(entry(1, 59, 0).level_line(), (2151, vec![59]));
        assert_eq!(entry(2, 80, 0).level_line(), (2152, vec![80]));
        assert_eq!(entry(3, 60, 79).level_line(), (2153, vec![60, 79]));
        assert_eq!(entry(0, 0, 0).level_line(), (2154, vec![]));
        assert_eq!(Entry { allies: 6, enemies: 5, ..Entry::default() }.needed(), 11);
    }

    #[test]
    fn the_ways_to_apply_are_the_servers_one_two_and_four() {
        assert_eq!(Way::ALL.map(code), [1, 2, 4]);
        assert_eq!(Way::ALL.map(question), [2146, 2147, 2148]);
        assert!(foe(1, 2) && foe(1, 0) && !foe(2, 2) && !foe(0, 0));
    }

    #[test]
    fn the_client_refuses_before_the_server_is_asked() {
        let free = Standing::default();
        assert_eq!(refused(Way::Alone, &free, 6), None);
        assert_eq!(refused(Way::Alone, &Standing { battlefield: true, delayed: true, ..free }, 6), Some(2155));
        assert_eq!(refused(Way::Guild, &Standing { delayed: true, ..free }, 6), Some(2156));
        assert_eq!(refused(Way::Party, &free, 6), Some(2157));
        let member = Standing { party: 3, party_online: 3, ..free };
        assert_eq!(refused(Way::Party, &member, 6), Some(2158));
        assert_eq!(refused(Way::Party, &Standing { leader: true, ..member }, 6), None);
        assert_eq!(refused(Way::Party, &Standing { leader: true, party: 9, party_online: 7, ..free }, 6), Some(2159));
        assert_eq!(refused(Way::Party, &Standing { leader: true, party: 9, party_online: 6, ..free }, 6), None);
        assert_eq!(refused(Way::Guild, &free, 5), Some(2160));
        let guild = Standing { guild: true, guild_online: 5, ..free };
        assert_eq!(refused(Way::Guild, &guild, 5), Some(2161));
        assert_eq!(refused(Way::Guild, &Standing { master: true, ..guild }, 5), None);
        assert_eq!(refused(Way::Guild, &Standing { master: true, guild_online: 6, ..guild }, 5), Some(2162));
    }

    #[test]
    fn the_answers_to_applying_are_the_lines_the_client_says() {
        assert_eq!(applied(1), Applied::Queued);
        assert_eq!(applied(2), Applied::Refused(2100));
        assert_eq!(applied(6), Applied::Refused(2104));
        assert_eq!(applied(7), Applied::Refused(2105));
        assert_eq!(applied(8), Applied::Refused(2106));
        assert_eq!(applied(9), Applied::Refused(2108));
        assert_eq!(applied(10), Applied::Refused(2107));
        assert_eq!(applied(15), Applied::Refused(2109));
        assert_eq!([applied(0), applied(11), applied(14), applied(16)], [Applied::Nothing; 4]);
    }

    #[test]
    fn leaving_a_queue_says_why_and_closes_its_window_for_two_of_the_reasons() {
        assert_eq!(cancelled(1), Some((2110, true)));
        assert_eq!(cancelled(3), Some((2111, false)));
        assert_eq!(cancelled(11), Some((2112, false)));
        assert_eq!(cancelled(14), Some((2113, true)));
        assert_eq!(cancelled(0), None);
        assert_eq!([admission_refused(3), admission_refused(11), admission_refused(1)], [Some(2111), Some(2112), None]);
    }

    #[test]
    fn the_entry_is_asked_for_twenty_seconds() {
        assert_eq!(entry_seconds(0), 20);
        assert_eq!(entry_seconds(999), 20);
        assert_eq!(entry_seconds(1000), 19);
        assert_eq!(entry_seconds(19_999), 1);
        assert_eq!(entry_seconds(20_000), 0);
    }

    #[test]
    fn a_line_for_the_battle_chat_loses_the_mark_of_another_chat() {
        assert_eq!([chat_words("go left"), chat_words("%go"), chat_words("$go"), chat_words("#go")], [Some("go left"), Some("go"), Some("go"), Some("go")]);
        assert_eq!([chat_words("%"), chat_words(""), chat_words("%%go")], [None, None, Some("%go")]);
    }

    #[test]
    fn a_score_is_written_in_one_or_two_digits_from_the_right() {
        assert!(score(-1, 0).is_empty());
        assert!(score(3, -1).is_empty());
        assert_eq!(score(3, 0), vec![(3, -207, 50), (0, -207, 108)]);
        assert_eq!(score(12, 7), vec![(2, -175, 50), (1, -207, 50), (7, -175, 108)]);
        assert_eq!(score(99, 40), vec![(9, -175, 50), (9, -207, 50), (0, -175, 108), (4, -207, 108)]);
    }
}
