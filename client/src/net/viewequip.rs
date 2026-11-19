//! Looking at what another player wears: asking the server, its answer with the look and the things worn, and
//! the player's own switch that lets others look. The layouts are rAthena's for PACKETVER 20260219
//! (`clif_viewequip_ack`, `clif_parse_ViewPlayerEquip`, `clif_parse_configuration`, `clif_configuration`,
//! `clif_equipcheckbox` and `clif_msg` in `clif.cpp`, `PACKET_ZC_EQUIPWIN_MICROSCOPE` in `packets_struct.hpp`).

use super::bytes::{text, u16_at, u32_at};
use super::items::{self, Item};

/// ZC_EQUIPWIN_MICROSCOPE (0x0b37): what another player wears (the client's `ID_ZC_EQUIPWIN_MICROSCOPE_V8`).
pub const ZC_EQUIPWIN_MICROSCOPE: u16 = 0x0b37;
/// ZC_CONFIG (0x02d9, 10 bytes): a setting of the player's was changed.
pub const ZC_CONFIG: u16 = 0x02d9;
/// ZC_CONFIG_NOTIFY (0x02da, 3 bytes): whether others may look at the player's equipment, told on entering.
pub const ZC_CONFIG_NOTIFY: u16 = 0x02da;
/// ZC_MSG (0x0291, 4 bytes): a line of the client's message table is said. That the other does not let their
/// equipment be looked at comes so (`MSI_OPEN_EQUIPEDITEM_REFUSED`).
pub const ZC_MSG: u16 = 0x0291;

/// The setting of CZ_CONFIG and ZC_CONFIG that lets others look at the equipment (rAthena's
/// `CONFIG_OPEN_EQUIPMENT_WINDOW`).
pub const OPEN_EQUIPMENT: u32 = 0;

/// What the server shows of another player: the name, the look, and the things worn.
#[derive(Debug, Clone, PartialEq)]
pub struct Seen {
    pub name: String,
    pub job: u16,
    /// 1 for male, 0 for female.
    pub sex: u8,
    pub hair_style: u16,
    pub hair_colour: u16,
    /// The headgear as view IDs: bottom, top, middle (the order `web::figure::Figure` takes them in).
    pub headgear: [u16; 3],
    pub robe: u16,
    pub body_palette: u16,
    pub items: Vec<Item>,
}

/// ZC_EQUIPWIN_MICROSCOPE (0x0b37): `u16 length, char name[24], i16 job, i16 hair, i16 bottom, i16 middle,
/// i16 top, i16 robe, i16 hair colour, i16 clothes colour, i16 body2, u8 sex`, 47 bytes, then the things worn,
/// each as in the inventory's list of equipment (`items::worn_list`).
pub fn seen(packet: &[u8]) -> Option<Seen> {
    let head = packet.get(..47)?;
    let name = &head[4..28];
    Some(Seen {
        name: text(name),
        job: u16_at(head, 28),
        hair_style: u16_at(head, 30),
        headgear: [u16_at(head, 32), u16_at(head, 36), u16_at(head, 34)],
        robe: u16_at(head, 38),
        hair_colour: u16_at(head, 40),
        body_palette: u16_at(head, 42),
        sex: head[46],
        items: items::worn_list(&packet[47..]),
    })
}

/// ZC_CONFIG (0x02d9, 10 bytes): `u32 setting, u32 on`.
pub fn config(packet: &[u8]) -> Option<(u32, bool)> {
    let packet = packet.get(..10)?;
    Some((u32_at(packet, 2), u32_at(packet, 6) != 0))
}

/// ZC_CONFIG_NOTIFY (0x02da, 3 bytes): `u8 open`: others may look at the player's equipment.
pub fn open(packet: &[u8]) -> Option<bool> {
    packet.get(2).map(|open| *open != 0)
}

/// ZC_MSG (0x0291, 4 bytes): `u16 line`: which line of the message table.
pub fn said(packet: &[u8]) -> Option<usize> {
    Some(usize::from(u16_at(packet.get(..4)?, 2)))
}

/// CZ_EQUIPWIN_MICROSCOPE (0x02d6, 6 bytes): `u32 account`: what does that player wear? (The client's
/// `MENUID_VIEW_OTHERUSER_EQUIPWND`.)
pub fn ask(account: u32) -> Vec<u8> {
    [&0x02d6u16.to_le_bytes()[..], &account.to_le_bytes()].concat()
}

/// CZ_CONFIG (0x02d8, 10 bytes): `u32 setting, u32 on` (the client's `MM_OPEN_EQUIPEDITEM_CHANGE` sends the
/// setting 0).
pub fn set(setting: u32, on: bool) -> Vec<u8> {
    [&0x02d8u16.to_le_bytes()[..], &setting.to_le_bytes(), &u32::from(on).to_le_bytes()].concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A thing worn as the server lists it: 68 bytes.
    fn worn(index: u16, id: u32, place: u32, refine: u8) -> Vec<u8> {
        let mut item = vec![0u8; 68];
        item[..2].copy_from_slice(&index.to_le_bytes());
        item[2..6].copy_from_slice(&id.to_le_bytes());
        item[6] = 5;
        item[7..11].copy_from_slice(&place.to_le_bytes());
        item[11..15].copy_from_slice(&place.to_le_bytes());
        item[15..19].copy_from_slice(&4001u32.to_le_bytes());
        item[65] = refine;
        item[67] = 1;
        item
    }

    #[test]
    fn reads_what_another_wears() {
        let mut packet = vec![0x37, 0x0b, 0, 0];
        let mut name = [0u8; 24];
        name[..5].copy_from_slice(b"Other");
        packet.extend(name);
        for value in [4008u16, 7, 11, 22, 33, 44, 3, 2, 0] {
            packet.extend(value.to_le_bytes());
        }
        packet.push(1);
        assert_eq!(packet.len(), 47);
        packet.extend(worn(3, 1101, 2, 7));
        packet.extend(worn(9, 2301, 16, 0));
        let length = packet.len() as u16;
        packet[2..4].copy_from_slice(&length.to_le_bytes());

        let seen = seen(&packet).unwrap();
        assert_eq!((seen.name.as_str(), seen.job, seen.sex), ("Other", 4008, 1));
        assert_eq!((seen.hair_style, seen.hair_colour, seen.body_palette, seen.robe), (7, 3, 2, 44));
        // Bottom, top, middle.
        assert_eq!(seen.headgear, [11, 33, 22]);
        assert_eq!(seen.items.len(), 2);
        assert_eq!((seen.items[0].id, seen.items[0].worn, seen.items[0].refine, seen.items[0].cards[0], seen.items[0].identified), (1101, 2, 7, 4001, true));
        assert_eq!(seen.items[1].worn, 16);
        // One who wears nothing is the 47 bytes alone; less is no answer.
        assert!(super::seen(&packet[..47]).is_some_and(|seen| seen.items.is_empty()));
        assert_eq!(super::seen(&packet[..46]), None);
    }

    #[test]
    fn reads_the_settings() {
        assert_eq!(config(&[0xd9, 0x02, 0, 0, 0, 0, 1, 0, 0, 0]), Some((OPEN_EQUIPMENT, true)));
        assert_eq!(config(&[0xd9, 0x02, 2, 0, 0, 0, 0, 0, 0, 0]), Some((2, false)));
        assert_eq!(config(&[0xd9, 0x02, 0]), None);
        assert_eq!(open(&[0xda, 0x02, 1]), Some(true));
        assert_eq!(open(&[0xda, 0x02, 0]), Some(false));
        assert_eq!(said(&[0x91, 0x02, 0x4d, 0x05]), Some(1357));
    }

    #[test]
    fn writes_what_the_client_sends() {
        assert_eq!(ask(2_000_001), vec![0xd6, 0x02, 0x81, 0x84, 0x1e, 0x00]);
        assert_eq!(set(OPEN_EQUIPMENT, true), vec![0xd8, 0x02, 0, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(set(OPEN_EQUIPMENT, false)[6], 0);
    }
}
