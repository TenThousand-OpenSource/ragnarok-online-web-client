//! What the map server tells the player in a line of the chat, as the client says it: which line of its message
//! table is said and in which colour. The rules are the client's handlers, each named where it is written out;
//! the packets are `net/notices.rs`.

use crate::net::notices::Experience;
use crate::said::colour::{PALE_CYAN, RED, YELLOW};

/// The colours only these lines have, as the page takes them (the client's `0xBBGGRR` turned): what Gospel
/// did (`0x9B9BFF`), a skill's message (`0xFFCDCD`), a thing bound (`0x6EFFFF`), experience of a quest
/// (`0xBE58BE`).
pub const GOSPEL: &str = "#ff9b9b";
pub const SKILL: &str = "#cdcdff";
pub const BOUND: &str = "#ffff6e";
pub const QUEST_EXPERIENCE: &str = "#be58be";

/// What ZC_ACTION_FAILURE says (`CGameMode::ReceiveEquipAck`): the line of the message table and its colour,
/// by the kind and the player's job. 0: no ammunition (`MSI_ERR_ATTACK_ARROW`, red); 1 and 2: too heavy to
/// attack, to use skills (`0xFFFF64`); 3: ammunition was put on: throwing daggers for the assassins, bullets
/// for a gunslinger, nothing for a ninja, arrows for everyone else.
pub fn action_failure(kind: u16, job: u16) -> Option<(usize, &'static str)> {
    Some(match kind {
        0 => (242, RED),
        1 => (243, PALE_CYAN),
        2 => (244, PALE_CYAN),
        3 => match job {
            // Assassin, Assassin Cross, their baby, Guillotine Cross (with its high and baby forms), Shadow Cross.
            12 | 4013 | 4035 | 4059 | 4065 | 4101 | 4254 => (1040, PALE_CYAN),
            24 => (1175, PALE_CYAN),
            25 | 4222 => return None,
            _ => (245, PALE_CYAN),
        },
        _ => return None,
    })
}

/// What ZC_NOTIFY_MAPINFO says (`CGameMode`, `case ID_ZC_NOTIFY_MAPINFO`): no teleporting here (red), this
/// place cannot be kept for Warp Portal, no skills here, no items here (the three in `0xFFFF00`).
pub fn map_info(kind: u16) -> Option<(usize, &'static str)> {
    Some(match kind {
        0 => (500, RED),
        1 => (501, crate::said::colour::CYAN),
        2 => (1334, crate::said::colour::CYAN),
        3 => (1335, crate::said::colour::CYAN),
        _ => return None,
    })
}

/// What ZC_ACK_GIVE_MANNER_POINT says (`case ID_ZC_ACK_GIVE_MANNER_POINT`, the client's `MP_SUCCESS`,
/// `MP_SUCCESS_BY_ADMIN`, `MP_BYSELF`): a manner point was given (`MSI_SEND_MANNER_POINT`, in `0xFFFF00`), a
/// game master took the player's chat away (`MSI_GM_BAD_MANNER_START`), the server did by itself
/// (`MSI_AUTO_BAD_MANNER_START`; the two in `0xFFFF`). The client writes the two last as the table has them,
/// their `%d` not filled. It says nothing for any other result.
pub fn manner_answer(result: u32) -> Option<(usize, &'static str)> {
    Some(match result {
        0 => (854, crate::said::colour::CYAN),
        3 => (830, crate::said::colour::YELLOW),
        4 => (833, crate::said::colour::YELLOW),
        _ => return None,
    })
}

/// The line ZC_NOTIFY_MANNER_POINT_GIVEN says with the giver's name in it (`MSI_RECEIVE_PLUS_MANNER_POINT`, in
/// `0xFFFF00`): only a point for the better (kind 0) is told (`case ID_ZC_NOTIFY_MANNER_POINT_GIVEN`).
pub fn manner_given(kind: u8) -> Option<(usize, &'static str)> {
    (kind == 0).then_some((387, crate::said::colour::CYAN))
}

/// The line of ZC_MSG_VALUE that takes an item's name where the others take a number
/// (`CGameMode::ReceiveFormattedMessage`).
pub const NAMES_AN_ITEM: usize = 3278;

/// The lines of ZC_MSG_COLOR that are red whatever colour the server names (`CGameMode::Zc_Msg_Color`: the
/// party's call that failed, a job or a sex that does not fit, `MSI_QUESTUI_TAB_HOLD`).
pub fn always_red(line: usize) -> bool {
    matches!(line, 1221 | 1222 | 1879 | 1880 | 2182)
}

/// What ZC_SKILLMSG says, by its kind (`CGameMode::ReceiveStatusNotice`): the ten lines of Gospel
/// (`MSI_GOSPEL1`…) and that Full Strip found its target shielded (`MSI_FULLSTRIP`); nothing for any other
/// number.
pub fn skill_line(kind: u32) -> Option<usize> {
    Some(match kind {
        kind @ 21..=25 => 889 + kind as usize - 21,
        kind @ 28..=32 => 894 + kind as usize - 28,
        40 => 915,
        _ => return None,
    })
}

impl Experience {
    /// The lines of the message table it is said with (`CGameMode::ReceiveExpGainNotice`): the one that names
    /// the experience with its amount (3220 for base, 3227 for job; none for any other kind), and the one that
    /// follows it, "acquired" or "has lost" (`MSI_GET`, `MSI_LOSS`). Nothing is said of none.
    pub fn lines(&self) -> Option<(Option<usize>, usize)> {
        let named = match self.kind {
            1 => Some(3220),
            2 => Some(3227),
            _ => None,
        };
        (self.amount != 0).then_some((named, if self.amount >= 0 { 1615 } else { 1616 }))
    }

    /// Its colour: yellow, or the colour of a quest's.
    pub fn colour(&self) -> &'static str {
        if self.quest { QUEST_EXPERIENCE } else { YELLOW }
    }
}

/// The line of the message table that names what a party member had damaged, by its place
/// (`CGameMode::ReceiveEquipItemDamaged`): the left hand, the right hand, the body, the top of the head. Any
/// other place is not told of.
pub fn damaged_part(place: u16) -> Option<usize> {
    Some(match place {
        0x20 => 0x109a,
        0x02 => 0x109b,
        0x10 => 1303,
        0x100 => 0x10cb,
        _ => return None,
    })
}

/// What ZC_CASH_TIME_COUNTER says (`CGameMode::ReceiveRentalItemTime`): the lines of the message table and
/// the number that goes into the first after the item's name: the minutes left (1256), or the seconds when
/// there is less than a minute (1270); with one minute left a second line says so (1257).
pub fn rental_lines(seconds: u32) -> (usize, u32, Option<usize>) {
    match seconds / 60 {
        0 => (1270, seconds, None),
        minutes => (1256, minutes, (minutes == 1).then_some(1257)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_action_that_failed_says_its_line() {
        assert_eq!(action_failure(0, 0), Some((242, RED)));
        assert_eq!(action_failure(1, 0), Some((243, PALE_CYAN)));
        assert_eq!(action_failure(2, 0), Some((244, PALE_CYAN)));
        // Ammunition put on: arrows, an assassin's daggers, a gunslinger's bullets, nothing for a ninja.
        assert_eq!(action_failure(3, 3), Some((245, PALE_CYAN)));
        assert_eq!(action_failure(3, 12), Some((1040, PALE_CYAN)));
        assert_eq!(action_failure(3, 24), Some((1175, PALE_CYAN)));
        assert_eq!(action_failure(3, 25), None);
        assert_eq!(action_failure(4, 0), None);
    }

    #[test]
    fn what_a_map_forbids_is_said() {
        assert_eq!(map_info(0), Some((500, RED)));
        assert_eq!(map_info(1).map(|said| said.0), Some(501));
        assert_eq!(map_info(2).map(|said| said.0), Some(1334));
        assert_eq!(map_info(3).map(|said| said.0), Some(1335));
        assert_eq!(map_info(4), None);
        assert_eq!((manner_answer(0), manner_answer(3)), (Some((854, crate::said::colour::CYAN)), Some((830, crate::said::colour::YELLOW))));
        assert_eq!((manner_answer(4), manner_answer(5), manner_answer(1)), (Some((833, crate::said::colour::YELLOW)), None, None));
        assert_eq!((manner_given(0), manner_given(1)), (Some((387, crate::said::colour::CYAN)), None));
    }

    #[test]
    fn five_lines_are_red_whatever_the_server_names() {
        assert_eq!([1221, 1222, 1879, 1880, 2182].map(always_red), [true; 5]);
        assert_eq!((always_red(16), always_red(1223), always_red(0)), (false, false, false));
    }

    #[test]
    fn gospel_says_its_ten_lines() {
        assert_eq!((skill_line(21), skill_line(25), skill_line(28), skill_line(32)), (Some(889), Some(893), Some(894), Some(898)));
        assert_eq!(skill_line(40), Some(915));
        assert_eq!((skill_line(26), skill_line(0), skill_line(u32::MAX)), (None, None, None));
    }

    #[test]
    fn experience_is_read_and_said() {
        let gained = Experience { who: 1_000_000, amount: 1234, kind: 1, quest: true };
        assert_eq!((gained.lines(), gained.colour()), (Some((Some(3220), 1615)), QUEST_EXPERIENCE));
        let lost = Experience { who: 1, amount: -5, kind: 2, quest: false };
        assert_eq!((lost.lines(), lost.colour()), (Some((Some(3227), 1616)), YELLOW));
        assert_eq!(Experience { who: 1, amount: 0, kind: 1, quest: false }.lines(), None);
        assert_eq!(Experience { who: 1, amount: 7, kind: 3, quest: false }.lines(), Some((None, 1615)));
    }

    #[test]
    fn a_damaged_part_has_its_line() {
        assert_eq!((damaged_part(0x10), damaged_part(0x20), damaged_part(0x40)), (Some(1303), Some(0x109a), None));
        assert_eq!((damaged_part(0x02), damaged_part(0x100), damaged_part(0)), (Some(0x109b), Some(0x10cb), None));
    }

    #[test]
    fn a_rented_item_tells_its_time_in_minutes_then_seconds() {
        assert_eq!(rental_lines(3600), (1256, 60, None));
        assert_eq!(rental_lines(90), (1256, 1, Some(1257)));
        assert_eq!(rental_lines(59), (1270, 59, None));
    }
}
