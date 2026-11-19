//! Marks on the minimap that the server tells of: where the members of the guild are on this map, and where
//! a boss monster is. The layouts are rAthena's for PACKETVER 20260219 (`clif_guild_xy`, `clif_bossmapinfo`).

use super::bytes::{long, name_at, word};

/// ZC_NOTIFY_POSITION_TO_GUILDM (0x01eb, 10 bytes): `u32 account, i16 x, i16 y`: where a member of the guild
/// is on this map; -1, -1 when the member is here no more.
pub const ZC_NOTIFY_POSITION_TO_GUILDM: u16 = 0x01eb;
/// ZC_BOSS_INFO (0x0293, 70 bytes): what is known of the boss monster of this map.
pub const ZC_BOSS_INFO: u16 = 0x0293;

/// EFST_CASH_BOSS_ALARM: the state in which the client marks the boss on the minimap (`CGameMode::DrawMiniMap`
/// draws the mark only while it is on the player).
pub const BOSS_ALARM: u32 = 253;

/// ZC_NOTIFY_POSITION_TO_GUILDM: the member's account, and the cell if there is one.
pub fn guild_place(packet: &[u8]) -> Option<(u32, Option<(u16, u16)>)> {
    let (x, y) = (word(packet, 6)? as i16, word(packet, 8)? as i16);
    Some((long(packet, 2)?, (x >= 0 && y >= 0).then_some((x as u16, y as u16))))
}

/// What ZC_BOSS_INFO tells.
#[derive(Debug, PartialEq)]
pub struct Boss {
    /// 0: there is none here; 1: it lives, and where; 2: the same, to be said; 3: it is dead, and when it
    /// comes again.
    pub kind: u8,
    pub cell: (u32, u32),
    /// The hours and minutes until it may come again.
    pub hours: u16,
    pub minutes: u16,
    pub name: String,
}

/// ZC_BOSS_INFO: `u8 kind, u32 x, u32 y, u16 hours, u16 minutes, u16 latest hours, u16 latest minutes,
/// char name[51]`.
pub fn boss(packet: &[u8]) -> Option<Boss> {
    let packet = packet.get(..70)?;
    Some(Boss { kind: packet[2], cell: (long(packet, 3)?, long(packet, 7)?), hours: word(packet, 11)?, minutes: word(packet, 13)?, name: name_at(packet, 19, 51) })
}

impl Boss {
    /// Where the minimap marks it: only one that lives has a place (`CGameMode::ReceiveBossMonsterInfo` sets
    /// the place to none first, whatever is told).
    pub fn marked(&self) -> Option<(u16, u16)> {
        matches!(self.kind, 1 | 2).then_some((u16::try_from(self.cell.0).ok()?, u16::try_from(self.cell.1).ok()?))
    }

    /// What the chat says of it, in white: the line of the message table and what is written into it. None
    /// here (1223); its place will be shown (1225, said for kind 2 only); it comes within a minute (1261,
    /// when the time told is 0 hours and 1 minute) or in so many hours and minutes (1224).
    pub fn says(&self) -> Option<(usize, Vec<String>)> {
        Some(match self.kind {
            0 => (1223, Vec::new()),
            2 => (1225, vec![self.name.clone()]),
            3 if self.hours == 0 && self.minutes == 1 => (1261, vec![self.name.clone()]),
            3 => (1224, vec![self.name.clone(), self.hours.to_string(), self.minutes.to_string()]),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn told(kind: u8, hours: u16, minutes: u16) -> Vec<u8> {
        let mut packet = vec![0x93, 0x02, kind];
        packet.extend(120u32.to_le_bytes());
        packet.extend(85u32.to_le_bytes());
        packet.extend(hours.to_le_bytes());
        packet.extend(minutes.to_le_bytes());
        packet.extend([0; 4]);
        let mut name = [0u8; 51];
        name[..7].copy_from_slice(b"Baphomet"[..7].as_ref());
        packet.extend(name);
        packet
    }

    #[test]
    fn a_guild_members_place_is_read() {
        assert_eq!(guild_place(&[0xeb, 0x01, 7, 0, 0, 0, 100, 0, 50, 0]), Some((7, Some((100, 50)))));
        assert_eq!(guild_place(&[0xeb, 0x01, 7, 0, 0, 0, 0xff, 0xff, 0xff, 0xff]), Some((7, None)));
        assert_eq!(guild_place(&[0xeb, 0x01, 7, 0, 0, 0, 100, 0]), None);
    }

    #[test]
    fn a_boss_that_lives_is_marked() {
        let packet = told(1, 0, 0);
        assert_eq!(packet.len(), 70);
        let boss = boss(&packet).unwrap();
        assert_eq!((boss.name.as_str(), boss.marked(), boss.says()), ("Baphome", Some((120, 85)), None));
        assert_eq!(super::boss(&told(2, 0, 0)).unwrap().says(), Some((1225, vec!["Baphome".to_owned()])));
    }

    #[test]
    fn a_dead_boss_tells_when_it_comes_again() {
        let dead = boss(&told(3, 2, 30)).unwrap();
        assert_eq!((dead.marked(), dead.says()), (None, Some((1224, vec!["Baphome".to_owned(), "2".to_owned(), "30".to_owned()]))));
        assert_eq!(boss(&told(3, 0, 1)).unwrap().says().map(|said| said.0), Some(1261));
        assert_eq!(boss(&told(0, 0, 0)).unwrap().says(), Some((1223, Vec::new())));
        assert_eq!(boss(&told(3, 0, 1)[..69]), None);
    }
}
