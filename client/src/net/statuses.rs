//! The states someone is in (buffs and debuffs), as the map server tells of them going on and off.

use super::bytes::{id, int, long, u16_at, u32_at, word};

/// ZC_MSG_STATE_CHANGE: a state goes on or off, with nothing more said.
pub const ZC_MSG_STATE_CHANGE: u16 = 0x0196;
/// ZC_MSG_STATE_CHANGE2: the same with the time left and three values.
pub const ZC_MSG_STATE_CHANGE2: u16 = 0x043f;
/// ZC_MSG_STATE_CHANGE3: the same with the whole time too.
pub const ZC_MSG_STATE_CHANGE3: u16 = 0x0983;
/// ZC_EFST_SET_ENTER2: a state of someone who comes into sight, with its whole time and what is left of it
/// (the client's `CGameMode::ReceiveStatusChangeEntry`, which hands it to `ApplyStatusChange` as one on).
pub const ZC_EFST_SET_ENTER2: u16 = 0x0984;

/// The time the server gives a state that has none (rAthena's `clif_status_change_sub`; the client's
/// `ApplyStatusChange` keeps such a state without a time).
pub const NO_TIME: u32 = 9999;
/// EFST_POSTDELAY: no state to show, but the wait after a skill before any other can be used.
pub const POSTDELAY: u16 = 46;
/// EFST_WEAPONPROPERTY: the element the weapon was given, told by the first value; the client shows it as the
/// state 897 further than the value (`ApplyStatusChange`).
pub const WEAPONPROPERTY: u16 = 64;
const WEAPONPROPERTY_FIRST: u16 = 897;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change {
    pub state: u16,
    pub who: u32,
    pub on: bool,
    /// How long the state lasts in all and how much of it is left, in milliseconds: both nought where the
    /// packet tells no time, the whole nought where it tells only what is left.
    pub total: u32,
    pub remaining: u32,
    pub values: [i32; 3],
}

impl Change {
    /// The state as the client lists it: a weapon's element is a state of its own for each element.
    pub fn shown(&self) -> u16 {
        if self.state == WEAPONPROPERTY { WEAPONPROPERTY_FIRST.wrapping_add(self.values[0] as u16) } else { self.state }
    }

    /// Whether the state runs out by itself: it has a time, and not the one that stands for none.
    pub fn timed(&self) -> bool {
        self.remaining != 0 && self.remaining != NO_TIME
    }
}

/// Reads any of the four:
/// ZC_MSG_STATE_CHANGE (0x0196, 9 bytes): `u16 state`, `u32 who`, `u8 on`;
/// ZC_MSG_STATE_CHANGE2 (0x043f, 25 bytes): then `u32 remaining`, `i32 value[3]`;
/// ZC_MSG_STATE_CHANGE3 (0x0983, 29 bytes): then `u32 total`, `u32 remaining`, `i32 value[3]`;
/// ZC_EFST_SET_ENTER2 (0x0984, 28 bytes): `u32 who`, `u16 state`, `u32 total`, `u32 remaining`,
/// `i32 value[3]`: the state is on.
pub fn changed(packet: &[u8]) -> Option<Change> {
    let values = |at: usize| Some([int(packet, at)?, int(packet, at + 4)?, int(packet, at + 8)?]);
    let id = id(packet)?;
    if id == ZC_EFST_SET_ENTER2 {
        let (total, remaining) = (long(packet, 8)?, long(packet, 12)?);
        return Some(Change { state: word(packet, 6)?, who: long(packet, 2)?, on: true, total, remaining, values: values(16)? });
    }
    let (total, remaining, values) = match id {
        ZC_MSG_STATE_CHANGE => (0, 0, [0; 3]),
        ZC_MSG_STATE_CHANGE2 => (0, long(packet, 9)?, values(13)?),
        ZC_MSG_STATE_CHANGE3 => (long(packet, 9)?, long(packet, 13)?, values(17)?),
        _ => return None,
    };
    Some(Change { state: word(packet, 2)?, who: long(packet, 4)?, on: *packet.get(8)? != 0, total, remaining, values })
}

/// ZC_STATE_CHANGE (0x0229): `u32 who`, `u16 body state`, `u16 health state`, `u32 effect state`, `u8 pk`: whose
/// state it is, the body state and the health state (the client's `ApplyStatusIcons` shows states for both).
/// The effect state of the same packet is read by `net::units::state_changed`, which wants the packet longer.
pub fn body_and_health(packet: &[u8]) -> Option<(u32, u16, u16)> {
    let packet = packet.get(..10)?;
    Some((u32_at(packet, 2), u16_at(packet, 6), u16_at(packet, 8)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_state_with_its_times() {
        let mut packet = vec![0x83, 0x09, 10, 0, 0x40, 0x42, 0x0f, 0, 1];
        packet.extend(240_000u32.to_le_bytes());
        packet.extend(200_000u32.to_le_bytes());
        for value in [10i32, -2, 3] {
            packet.extend(value.to_le_bytes());
        }
        assert_eq!(packet.len(), 29);
        let change = changed(&packet).unwrap();
        assert_eq!(change, Change { state: 10, who: 1_000_000, on: true, total: 240_000, remaining: 200_000, values: [10, -2, 3] });
        assert!(change.timed());
        assert_eq!(changed(&packet[..28]), None);
    }

    #[test]
    fn reads_a_state_of_one_who_comes_into_sight() {
        let mut packet = vec![0x84, 0x09, 0x40, 0x42, 0x0f, 0, 10, 0];
        packet.extend(240_000u32.to_le_bytes());
        packet.extend(200_000u32.to_le_bytes());
        for value in [10i32, -2, 3] {
            packet.extend(value.to_le_bytes());
        }
        assert_eq!(packet.len(), 28);
        assert_eq!(changed(&packet), Some(Change { state: 10, who: 1_000_000, on: true, total: 240_000, remaining: 200_000, values: [10, -2, 3] }));
        assert_eq!(changed(&packet[..27]), None);
    }

    #[test]
    fn reads_the_older_forms() {
        let plain = [0x96, 0x01, 27, 0, 1, 0, 0, 0, 0];
        assert_eq!(changed(&plain), Some(Change { state: 27, who: 1, on: false, total: 0, remaining: 0, values: [0; 3] }));
        let mut second = vec![0x3f, 0x04, 12, 0, 1, 0, 0, 0, 1];
        second.extend(NO_TIME.to_le_bytes());
        second.extend([0; 12]);
        let change = changed(&second).unwrap();
        assert_eq!((change.total, change.remaining), (0, NO_TIME));
        assert!(!change.timed());
    }

    #[test]
    fn a_weapons_element_is_a_state_of_its_own() {
        let change = Change { state: WEAPONPROPERTY, who: 1, on: true, total: 0, remaining: 0, values: [3, 0, 0] };
        assert_eq!(change.shown(), 900);
    }

    #[test]
    fn reads_the_body_and_health_states() {
        let packet = [0x29, 0x02, 7, 0, 0, 0, 2, 0, 1, 0, 0, 0, 0, 0, 0];
        assert_eq!(body_and_health(&packet), Some((7, 2, 1)));
    }
}
