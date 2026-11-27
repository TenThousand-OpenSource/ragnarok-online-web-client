//! What the client puts at the head of a recording's packets, at the tick 0 (`Replay::RecordCachedPackets`,
//! `Client/Replay/ReplayPacketRecorder.cpp`): the two packets of the map's kind it last had (ZC_MAPPROPERTY_R2
//! 0x099B, ZC_NOTIFY_MAPPROPERTY2 0x01D6, as they came), and the lists of skills written anew as packets, the
//! character's (ZC_SKILLINFO_LIST 0x010F), the homunculus's (ZC_HOSKILLINFO_LIST 0x0235) and the mercenary's
//! (ZC_MER_SKILLINFO_LIST 0x029D), each in the layout of 37 bytes a skill (`BuildSkillInfoListPacket`,
//! `Client/Replay/Replay.cpp`). (A fourth, `BuildStatusReplayPacket`, written only in a mode of the session
//! the decompiled client does not name, is not made.)

/// The numbers of the three lists, and of the two packets of the map's kind.
pub const CHARACTER: u16 = crate::net::inventory::ZC_SKILLINFO_LIST;
pub const HOMUNCULUS: u16 = crate::net::homunculus::ZC_HOSKILLINFO_LIST;
pub const MERCENARY: u16 = crate::net::mercenary::ZC_MER_SKILLINFO_LIST;
pub const MAP_PROPERTY: u16 = crate::net::pvp::ZC_MAPPROPERTY_R2;
pub const MAP_KIND: u16 = crate::net::pvp::ZC_NOTIFY_MAPPROPERTY2;

/// How many bytes a skill takes in a list (`ReplaySkillInfo`).
pub const SKILL_BYTES: usize = 37;

/// A skill as a list tells of it (`ReplaySkillInfo`): its number, how it is used, its level, what it costs,
/// how far it reaches, its name in the client's files (24 bytes, the last a zero), whether it can be raised.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Listed {
    pub id: u16,
    pub target: u32,
    pub level: u16,
    pub sp: u16,
    pub range: u16,
    pub name: Vec<u8>,
    pub raisable: bool,
}

/// A list of skills as the packet of that number: `u16 id, u16 bytes`, then 37 bytes a skill. A list of none
/// is the four bytes alone, as the client writes it.
pub fn list(packet: u16, skills: &[Listed]) -> Vec<u8> {
    // (A packet's length is a `u16`: more skills than fit are left out, 1771 of them.)
    let skills = &skills[..skills.len().min((usize::from(u16::MAX) - 4) / SKILL_BYTES)];
    let mut bytes = Vec::with_capacity(4 + skills.len() * SKILL_BYTES);
    bytes.extend(packet.to_le_bytes());
    bytes.extend(((4 + skills.len() * SKILL_BYTES) as u16).to_le_bytes());
    for skill in skills {
        bytes.extend(skill.id.to_le_bytes());
        bytes.extend(skill.target.to_le_bytes());
        bytes.extend(skill.level.to_le_bytes());
        bytes.extend(skill.sp.to_le_bytes());
        bytes.extend(skill.range.to_le_bytes());
        // (`memcpy` of 24 bytes at most, then `name[23] = 0`.)
        bytes.extend(super::padded(&skill.name[..skill.name.len().min(23)], 24));
        bytes.push(skill.raisable.into());
    }
    bytes
}

/// What stands at the head of a recording's packets, in the client's order, each at the tick 0: the two
/// packets of the map's kind that were last heard (none that was not), then the three lists.
pub fn head(map: [Option<&[u8]>; 2], character: &[Listed], homunculus: &[Listed], mercenary: &[Listed]) -> Vec<(u32, Vec<u8>)> {
    let lists = [(CHARACTER, character), (HOMUNCULUS, homunculus), (MERCENARY, mercenary)];
    let heard = map.into_iter().flatten().map(<[u8]>::to_vec);
    heard.chain(lists.into_iter().map(|(packet, skills)| list(packet, skills))).map(|packet| (0, packet)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::inventory::{skills, Skill};

    fn bash() -> Listed {
        Listed { id: 5, target: 1, level: 10, sp: 15, range: 2, name: b"SM_BASH".to_vec(), raisable: false }
    }

    #[test]
    fn this_clients_reader_reads_the_list_back() {
        let heal = Listed { id: 28, target: 16, level: 3, sp: 19, range: 9, name: b"AL_HEAL".to_vec(), raisable: true };
        let packet = list(CHARACTER, &[bash(), heal]);
        assert_eq!(packet.len(), 4 + 2 * SKILL_BYTES);
        assert_eq!(&packet[..4], &[0x0f, 0x01, 78, 0]);
        // The name stands after the range, the byte that says whether it can be raised last.
        assert_eq!(&packet[16..23], b"SM_BASH");
        assert_eq!(packet[4 + 36], 0);
        let read = skills(&packet);
        assert_eq!(read[0], Skill { id: 5, target: 1, level: 10, sp: 15, range: 2, raisable: false });
        assert_eq!(read[1], Skill { id: 28, target: 16, level: 3, sp: 19, range: 9, raisable: true });
        assert_eq!(read.len(), 2);
    }

    #[test]
    fn a_long_name_is_cut_and_no_skill_is_four_bytes() {
        let long = Listed { name: vec![b'A'; 40], ..bash() };
        let packet = list(HOMUNCULUS, &[long]);
        assert_eq!(&packet[16..39], &[b'A'; 23]);
        assert_eq!(packet[39], 0);
        assert_eq!(list(MERCENARY, &[]), vec![0x9d, 0x02, 4, 0]);
    }

    #[test]
    fn the_head_is_in_the_clients_order_at_tick_nought() {
        let (property, kind) = (vec![0x9b, 0x09, 0, 0, 1, 0, 0, 0], vec![0xd6, 0x01, 0, 0]);
        let head = head([Some(&property), Some(&kind)], &[bash()], &[], &[]);
        let ids: Vec<u16> = head.iter().map(|(_, packet)| u16::from_le_bytes([packet[0], packet[1]])).collect();
        assert_eq!(ids, [0x099b, 0x01d6, 0x010f, 0x0235, 0x029d]);
        assert!(head.iter().all(|(tick, _)| *tick == 0));
        // A map whose kind was never told has the lists alone.
        assert_eq!(super::head([None, None], &[], &[], &[]).len(), 3);
    }
}
