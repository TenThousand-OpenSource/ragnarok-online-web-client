//! Skill units, what skills leave standing on the ground, as the map server tells of them (rAthena's
//! `clif_getareachar_skillunit` and `clif_skill_delunit`, at PACKETVER 20260219).
//!
//! Not read here: ZC_SKILL_UPDATE (0x01ac: a trap that was hidden is seen), which matters only to the traps'
//! models, not drawn yet; and the packet Graffiti's unit comes in (0x01c9), not drawn either. A unit of Ice Wall
//! comes with a ZC_UPDATE_MAPINFO (0x0192) that closes its cell to walking, which is not read either.

use super::bytes::{long, word};

/// ZC_SKILL_ENTRY5 (0x09ca, 23 bytes): `u16 length, u32 id, u32 creator, u16 x, u16 y, u32 unit, u8 range,
/// u8 visible, u8 level`. A unit stands on a cell: one that comes into sight, or a new one.
pub const ZC_SKILL_ENTRY5: u16 = 0x09ca;
/// ZC_SKILL_DISAPPEAR (0x0120, 6 bytes): `u32 id`. A unit is gone.
pub const ZC_SKILL_DISAPPEAR: u16 = 0x0120;

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

/// ZC_UPDATE_MAPINFO (0x0192, 24 bytes): `u16 x, u16 y, u16 type, char map[16]`: a cell of the map is of another
/// type from now on (rAthena's `clif_changemapcell`: an Ice Wall makes its cells type 5 and gives them their own
/// type back when it is gone).
pub const ZC_UPDATE_MAPINFO: u16 = 0x0192;

/// The cell and its new type.
pub fn cell_changed(packet: &[u8]) -> Option<((u16, u16), u16)> {
    Some(((word(packet, 2)?, word(packet, 4)?), word(packet, 6)?))
}

/// Reads ZC_SKILL_DISAPPEAR: which unit is gone.
pub fn gone(packet: &[u8]) -> Option<u32> {
    long(packet.get(..6)?, 2)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
