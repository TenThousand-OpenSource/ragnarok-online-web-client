//! The wait before a skill may be used again, as the map server tells of it.

use super::bytes::{records, u16_at, u32_at};

/// ZC_SKILL_POSTDELAY: a skill cannot be used again for a time.
pub const ZC_SKILL_POSTDELAY: u16 = 0x043d;
/// ZC_SKILL_POSTDELAY_LIST: the waits still running, each with what is left of it.
pub const ZC_SKILL_POSTDELAY_LIST: u16 = 0x043e;
/// ZC_SKILL_POSTDELAY_LIST2: the same, each with its whole length too; it replaces what was known.
pub const ZC_SKILL_POSTDELAY_LIST2: u16 = 0x0985;

/// A wait as a packet tells it: the skill, how long it is in all and how much of it is left, in milliseconds
/// (the whole is nought where the packet does not tell it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Told {
    pub skill: u16,
    pub duration: u32,
    pub remaining: u32,
}

/// ZC_SKILL_POSTDELAY (0x043d, 8 bytes): `u16 skill`, `u32 time`: the wait starts now (the client's
/// `ApplyCooldownStart`).
pub fn started(packet: &[u8]) -> Option<Told> {
    let packet = packet.get(..8)?;
    Some(Told { skill: u16_at(packet, 2), duration: u32_at(packet, 4), remaining: u32_at(packet, 4) })
}

/// ZC_SKILL_POSTDELAY_LIST (0x043e): `u16 length`, then for each `u16 skill`, `u32 remaining` (the client's
/// `LoadCooldownRecords`).
pub fn running(packet: &[u8]) -> Vec<Told> {
    records(packet, 4, 6).map(|entry| Told { skill: u16_at(entry, 0), duration: 0, remaining: u32_at(entry, 2) }).collect()
}

/// ZC_SKILL_POSTDELAY_LIST2 (0x0985): `u16 length`, then for each `u16 skill`, `u32 duration`, `u32 remaining`
/// (the client's `LoadCooldownSnapshot`).
pub fn snapshot(packet: &[u8]) -> Vec<Told> {
    records(packet, 4, 10).map(|entry| Told { skill: u16_at(entry, 0), duration: u32_at(entry, 2), remaining: u32_at(entry, 6) }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_wait_that_starts() {
        assert_eq!(started(&[0x3d, 0x04, 28, 0, 0xe8, 0x03, 0, 0]), Some(Told { skill: 28, duration: 1000, remaining: 1000 }));
        assert_eq!(started(&[0x3d, 0x04, 28, 0, 0xe8, 0x03, 0]), None);
    }

    #[test]
    fn reads_the_lists() {
        let mut list = vec![0x3e, 0x04, 16, 0];
        list.extend([28, 0, 0xe8, 0x03, 0, 0, 29, 0, 0xd0, 0x07, 0, 0]);
        assert_eq!(running(&list), vec![Told { skill: 28, duration: 0, remaining: 1000 }, Told { skill: 29, duration: 0, remaining: 2000 }]);
        let mut list = vec![0x85, 0x09, 14, 0, 28, 0];
        list.extend(5000u32.to_le_bytes());
        list.extend(1500u32.to_le_bytes());
        assert_eq!(snapshot(&list), vec![Told { skill: 28, duration: 5000, remaining: 1500 }]);
        assert!(snapshot(&[0x85, 0x09, 4, 0]).is_empty());
    }
}
