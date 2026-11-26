//! The achievements and the titles they give: the list the map server sends on entering, one achievement as
//! it changes, the reward asked for and the answer, and the title worn. The layouts are rAthena's
//! (`clif_achievement_list_all`, `clif_achievement_update`, `clif_achievement_reward_ack`,
//! `clif_change_title_ack` in `src/map/clif.cpp`); the client's readers are `CGameMode::ReceiveAchievementList`,
//! `ReceiveAchievementUpdate`, `ReceiveAchievementResult` and `ReceiveJobTitle`.

use super::bytes::{Writer, long, records, word};

/// ZC_ALL_ACH_LIST (0x0a23, its length in it): `u32 count, u32 score, u16 rank, u32 points, u32 next`, then a
/// record of 50 bytes for each achievement the character has begun or completed. Not sent to one who has none.
pub const ZC_ALL_ACH_LIST: u16 = 0x0a23;
/// ZC_ACH_UPDATE (0x0a24, 66 bytes): `u32 score, u16 rank, u32 points, u32 next`, then one record: an
/// achievement moved on, or was completed, or its reward was taken.
pub const ZC_ACH_UPDATE: u16 = 0x0a24;
/// ZC_REQ_ACH_REWARD_ACK (0x0a26, 7 bytes): `u8 failed, u32 achievement`: the answer to a reward asked for.
pub const ZC_REQ_ACH_REWARD_ACK: u16 = 0x0a26;
/// ZC_ACK_CHANGE_TITLE (0x0a2f, 7 bytes): `u8 failed, u32 title`: the answer to a title asked for.
pub const ZC_ACK_CHANGE_TITLE: u16 = 0x0a2f;

/// How many goals an achievement has at most (rAthena's `MAX_ACHIEVEMENT_OBJECTIVES`, the client's ten
/// `conditions` of `UIAchViewWnd`).
pub const GOALS: usize = 10;

/// One achievement as the server tells of it (50 bytes): `u32 id, u8 completed, u32 goals[10], u32
/// completed_at, u8 rewarded`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Record {
    pub id: u32,
    pub completed: bool,
    /// How far each goal has come: a count, or more than nought for one that is only done or not.
    pub goals: [u32; GOALS],
    /// When it was completed, in seconds since 1970; 0 for one that is not.
    pub completed_at: u32,
    pub rewarded: bool,
}

/// What the server tells with every list and every change: the score, the rank, and how far the points have
/// come towards the next rank.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Standing {
    pub score: u32,
    pub rank: u16,
    pub points: u32,
    pub next: u32,
}

fn record(bytes: &[u8]) -> Option<Record> {
    let mut goals = [0; GOALS];
    for (place, goal) in goals.iter_mut().enumerate() {
        *goal = long(bytes, 5 + place * 4)?;
    }
    Some(Record { id: long(bytes, 0)?, completed: *bytes.get(4)? != 0, goals, completed_at: long(bytes, 45)?, rewarded: *bytes.get(49)? != 0 })
}

/// Reads ZC_ALL_ACH_LIST.
pub fn list(packet: &[u8]) -> Option<(Standing, Vec<Record>)> {
    let standing = Standing { score: long(packet, 8)?, rank: word(packet, 12)?, points: long(packet, 14)?, next: long(packet, 18)? };
    Some((standing, records(packet, 22, 50).filter_map(record).collect()))
}

/// Reads ZC_ACH_UPDATE.
pub fn update(packet: &[u8]) -> Option<(Standing, Record)> {
    let standing = Standing { score: long(packet, 2)?, rank: word(packet, 6)?, points: long(packet, 8)?, next: long(packet, 12)? };
    Some((standing, record(packet.get(16..66)?)?))
}

/// Reads ZC_REQ_ACH_REWARD_ACK and ZC_ACK_CHANGE_TITLE, which are laid out alike: whether it failed, and the
/// achievement or the title.
pub fn answered(packet: &[u8]) -> Option<(bool, u32)> {
    Some((*packet.get(2)? != 0, long(packet, 3)?))
}

/// The title a player wears, as ZC_ACK_REQNAMEALL (0x0a30, `net::guild`) tells it after the four names: `u32
/// title` at 102 (`PACKET_ZC_ACK_REQNAMEALL::title_id`); 0 for none.
pub fn title_named(packet: &[u8]) -> Option<u32> {
    long(packet, 102)
}

/// CZ_REQ_ACH_REWARD (0x0a25, 6 bytes): `u32 achievement`: the reward of a completed achievement is asked
/// for (`UIAchViewWnd::SendMsg`, the button 474).
pub fn ask_reward(achievement: u32) -> Vec<u8> {
    Writer::new(0x0a25).u32(achievement).done()
}

/// CZ_REQ_CHANGE_TITLE (0x0a2e, 6 bytes): `u32 title`: a title is put on, or with 0 taken off
/// (`UIEquipWnd::SendMsg`, the list 480).
pub fn wear_title(title: u32) -> Vec<u8> {
    Writer::new(0x0a2e).u32(title).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::{assert_lengths, longs, packet, sized};

    fn a_record(id: u32, completed: u8, at: u32, rewarded: u8) -> Vec<u8> {
        [&longs(&[id])[..], &[completed], &longs(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]), &longs(&[at]), &[rewarded]].concat()
    }

    #[test]
    fn reads_the_list_with_the_standing() {
        let body = [&longs(&[2, 30])[..], &7u16.to_le_bytes(), &longs(&[12, 18]), &a_record(127001, 0, 0, 0), &a_record(200001, 1, 1_700_000_000, 1)].concat();
        let packet = sized(ZC_ALL_ACH_LIST, &body);
        assert_eq!(packet.len(), 22 + 100);
        let (standing, records) = list(&packet).unwrap();
        assert_eq!(standing, Standing { score: 30, rank: 7, points: 12, next: 18 });
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], Record { id: 127001, completed: false, goals: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10], completed_at: 0, rewarded: false });
        assert!(records[1].completed && records[1].rewarded && records[1].completed_at == 1_700_000_000);
    }

    #[test]
    fn reads_one_achievement_changed() {
        let packet = packet(ZC_ACH_UPDATE, &[&longs(&[40]), &2u16.to_le_bytes(), &longs(&[3, 9]), &a_record(200005, 1, 99, 0)]);
        assert_eq!(crate::net::packet_length(ZC_ACH_UPDATE), Some(packet.len() as i32));
        let (standing, record) = update(&packet).unwrap();
        assert_eq!(standing, Standing { score: 40, rank: 2, points: 3, next: 9 });
        assert_eq!((record.id, record.completed, record.completed_at, record.rewarded), (200005, true, 99, false));
        assert_eq!(update(&packet[..60]), None);
    }

    #[test]
    fn reads_the_answers() {
        let reward = packet(ZC_REQ_ACH_REWARD_ACK, &[&[0], &longs(&[200005])]);
        assert_eq!(crate::net::packet_length(ZC_REQ_ACH_REWARD_ACK), Some(reward.len() as i32));
        assert_eq!(answered(&reward), Some((false, 200005)));
        let title = packet(ZC_ACK_CHANGE_TITLE, &[&[1], &longs(&[1013])]);
        assert_eq!(crate::net::packet_length(ZC_ACK_CHANGE_TITLE), Some(title.len() as i32));
        assert_eq!(answered(&title), Some((true, 1013)));
    }

    #[test]
    fn reads_the_title_told_with_a_players_names() {
        let named = packet(0x0a30, &[&longs(&[2_000_001]), &[0; 96], &longs(&[1013])]);
        assert_eq!(crate::net::packet_length(0x0a30), Some(named.len() as i32));
        assert_eq!(title_named(&named), Some(1013));
        assert_eq!(title_named(&named[..104]), None);
    }

    #[test]
    fn the_requests_are_as_long_as_the_server_expects() {
        assert_lengths(&[ask_reward(200005), wear_title(1013)]);
    }
}
