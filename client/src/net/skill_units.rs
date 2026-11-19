//! Skill units, what skills leave standing on the ground, as the map server tells of them (rAthena's
//! `clif_getareachar_skillunit` and `clif_skill_delunit`, at PACKETVER 20260219).
//!
//! A trap comes in the same ZC_SKILL_ENTRY5 as every unit, under its own kind (143 to 153); the server makes
//! another kind of it with ZC_SPRITE_CHANGE of its base (`net/actors.rs`) when it springs, and a trap that was
//! not to be seen is shown with ZC_SKILL_UPDATE (0x01ac, `seen`). A unit that carries words, Graffiti, comes
//! in a packet of its own (ZC_SKILL_ENTRY2, 0x01c9, `written`), and a talkie box says its words in another
//! (ZC_TALKBOX_CHATCONTENTS, 0x0191, `talkie`). A unit of Ice Wall comes with a ZC_UPDATE_MAPINFO (0x0192)
//! that closes its cell to walking.

use super::bytes::{long, name_at, word};

/// ZC_SKILL_ENTRY5 (0x09ca, 23 bytes): `u16 length, u32 id, u32 creator, u16 x, u16 y, u32 unit, u8 range,
/// u8 visible, u8 level`. A unit stands on a cell: one that comes into sight, or a new one.
pub const ZC_SKILL_ENTRY5: u16 = 0x09ca;
/// ZC_SKILL_DISAPPEAR (0x0120, 6 bytes): `u32 id`. A unit is gone.
pub const ZC_SKILL_DISAPPEAR: u16 = 0x0120;
/// ZC_UPDATE_MAPINFO (0x0192, 24 bytes): `u16 x, u16 y, u16 type, char map[16]`: a cell of the map is of another
/// type from now on (rAthena's `clif_changemapcell`: an Ice Wall makes its cells type 5 and gives them their own
/// type back when it is gone).
pub const ZC_UPDATE_MAPINFO: u16 = 0x0192;
/// ZC_SKILL_UPDATE (0x01ac): `u32 unit`: a unit on the ground changes (a trap sprung is seen).
pub const ZC_SKILL_UPDATE: u16 = 0x01ac;
/// ZC_TALKBOX_CHATCONTENTS (0x0191): `u32 unit, char words[80]`: a talkie box says its words.
pub const ZC_TALKBOX_CHATCONTENTS: u16 = 0x0191;
/// ZC_SKILL_ENTRY2 (0x01c9): a unit on the ground that carries words: Graffiti (`written`).
pub const ZC_SKILL_ENTRY2: u16 = 0x01c9;

/// A unit on the ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub id: u32,
    /// Who made it.
    pub creator: u32,
    pub cell: (u16, u16),
    /// What kind of unit it is (the client's `UNT_` numbers: 126 a Safety Wall, 127 a cell of Fire Wall…).
    pub unit: u32,
    /// How many cells around its own it covers.
    pub range: u8,
    /// Whether everyone sees it (a trap is hidden from those it is laid for).
    pub visible: bool,
    /// The level of the skill that made it.
    pub level: u8,
}

/// Reads ZC_SKILL_ENTRY5.
pub fn entry(packet: &[u8]) -> Option<Entry> {
    let packet = packet.get(..23)?;
    Some(Entry { id: long(packet, 4)?, creator: long(packet, 8)?, cell: (word(packet, 12)?, word(packet, 14)?), unit: long(packet, 16)?, range: packet[20], visible: packet[21] != 0, level: packet[22] })
}

/// Reads ZC_UPDATE_MAPINFO: the cell and its new type.
pub fn cell_changed(packet: &[u8]) -> Option<((u16, u16), u16)> {
    Some(((word(packet, 2)?, word(packet, 4)?), word(packet, 6)?))
}

/// Reads ZC_SKILL_UPDATE (0x01ac, 6 bytes: `u32 id`; rAthena's `clif_skillunit_update`): which unit is seen
/// from now on.
pub fn seen(packet: &[u8]) -> Option<u32> {
    long(packet.get(..6)?, 2)
}

/// Reads ZC_SKILL_DISAPPEAR: which unit is gone.
pub fn gone(packet: &[u8]) -> Option<u32> {
    long(packet.get(..6)?, 2)
}

/// What a talkie box says: `(unit, words)`.
pub fn talkie(packet: &[u8]) -> Option<(u32, String)> {
    Some((long(packet, 2)?, name_at(packet.get(..86)?, 6, 80)))
}

/// A unit on the ground with words (ZC_SKILL_ENTRY2: `u32 id, u32 maker, i16 x, i16 y, u8 kind, u8 seen,
/// u8 has words, char words[80]`).
#[derive(Debug, Clone, PartialEq)]
pub struct Written {
    pub id: u32,
    pub maker: u32,
    pub cell: (u16, u16),
    pub kind: u8,
    pub seen: bool,
    /// What is written; nothing for a unit that carries no words.
    pub words: String,
}

pub fn written(packet: &[u8]) -> Option<Written> {
    let has_words = *packet.get(16)? != 0;
    Some(Written {
        id: long(packet, 2)?,
        maker: long(packet, 6)?,
        cell: (word(packet, 10)?, word(packet, 12)?),
        kind: *packet.get(14)?,
        seen: *packet.get(15)? != 0,
        words: if has_words { name_at(packet.get(..97)?, 17, 80) } else { String::new() },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::packet;

    #[test]
    fn reads_a_unit_on_the_ground() {
        // As `clif_skill_unit_test` lays it out.
        let mut packet = vec![0xca, 0x09, 23, 0];
        packet.extend(1000u32.to_le_bytes());
        packet.extend(2000u32.to_le_bytes());
        packet.extend(150u16.to_le_bytes());
        packet.extend(160u16.to_le_bytes());
        packet.extend(127u32.to_le_bytes());
        packet.extend([1, 1, 10]);
        assert_eq!(packet.len(), 23);
        assert_eq!(crate::net::packet_length(ZC_SKILL_ENTRY5), Some(23));
        assert_eq!(entry(&packet), Some(Entry { id: 1000, creator: 2000, cell: (150, 160), unit: 127, range: 1, visible: true, level: 10 }));
        assert_eq!(entry(&packet[..22]), None);
    }

    #[test]
    fn reads_a_trap_on_the_ground() {
        // An Ankle Snare (`UNT_ANKLESNARE`, 145) as rAthena's `clif_getareachar_skillunit` writes it at
        // PACKETVER 20260219: the header 0x09ca, its length, and the trap's own kind, for everyone to see.
        let mut packet = vec![0xca, 0x09, 23, 0];
        packet.extend(0x0500_0001u32.to_le_bytes());
        packet.extend(2_000_001u32.to_le_bytes());
        packet.extend(101u16.to_le_bytes());
        packet.extend(215u16.to_le_bytes());
        packet.extend(145u32.to_le_bytes());
        packet.extend([0, 1, 5]);
        let trap = entry(&packet).unwrap();
        assert_eq!((trap.unit, trap.cell, trap.creator, trap.visible, trap.level), (145, (101, 215), 2_000_001, true, 5));
        // Every trap of the Hunter's comes under a kind the client has a model for.
        for unit in [143u32, 144, 145, 147, 148, 149, 150, 151, 152, 153] {
            packet[16..20].copy_from_slice(&unit.to_le_bytes());
            assert!(crate::skill_units::traps::model(entry(&packet).unwrap().unit).is_some());
        }
        // One hidden from the player comes as the kind 134 and not to be seen: nothing shows.
        packet[16..20].copy_from_slice(&134u32.to_le_bytes());
        packet[21] = 0;
        let hidden = entry(&packet).unwrap();
        assert!(!hidden.visible && crate::skill_units::traps::model(hidden.unit).is_none());
    }

    #[test]
    fn reads_a_unit_being_seen() {
        assert_eq!(crate::net::packet_length(crate::net::skill_units::ZC_SKILL_UPDATE), Some(6));
        assert_eq!(seen(&[0xac, 0x01, 0xe8, 0x03, 0, 0]), Some(1000));
        assert_eq!(seen(&[0xac, 0x01, 0xe8]), None);
    }

    #[test]
    fn reads_a_cell_changing() {
        let mut packet = vec![0x92, 0x01, 150, 0, 160, 0, 5, 0];
        packet.extend([0u8; 16]);
        assert_eq!(crate::net::packet_length(ZC_UPDATE_MAPINFO), Some(packet.len() as i32));
        assert_eq!(cell_changed(&packet), Some(((150, 160), 5)));
        assert_eq!(cell_changed(&packet[..7]), None);
    }

    #[test]
    fn reads_a_unit_going() {
        assert_eq!(crate::net::packet_length(ZC_SKILL_DISAPPEAR), Some(6));
        assert_eq!(gone(&[0x20, 0x01, 0xe8, 0x03, 0, 0]), Some(1000));
        assert_eq!(gone(&[0x20, 0x01, 0xe8]), None);
    }

    #[test]
    fn the_words_of_a_talkie_box_and_of_graffiti() {
        let mut said = packet(ZC_TALKBOX_CHATCONTENTS, &[&77u32.to_le_bytes(), b"beware"]);
        said.resize(86, 0);
        assert_eq!(talkie(&said), Some((77, "beware".to_owned())));
        let mut wall = packet(ZC_SKILL_ENTRY2, &[&77u32.to_le_bytes(), &5u32.to_le_bytes(), &150u16.to_le_bytes(), &160u16.to_le_bytes(), &[0xb0, 1, 1], b"hello"]);
        wall.resize(97, 0);
        assert_eq!(written(&wall), Some(Written { id: 77, maker: 5, cell: (150, 160), kind: 0xb0, seen: true, words: "hello".to_owned() }));
    }
}
