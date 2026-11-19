//! Looking at what another player wears: asking the server, its answer with the look and the things worn, and
//! the player's own switch that lets others look. The layouts are rAthena's for PACKETVER 20260219
//! (`clif_viewequip_ack`, `clif_parse_ViewPlayerEquip`, `clif_equipcheckbox` in `clif.cpp`,
//! `PACKET_ZC_EQUIPWIN_MICROSCOPE` in `packets_struct.hpp`).
//!
//! The switch is one of those the server keeps (`net/config.rs`), and that the other does not let their
//! equipment be looked at is a line of the message table said (ZC_MSG, `net/notices.rs`): both are still named
//! from here, where they were.

use super::bytes::{text, u16_at, Writer};
use super::items::{self, Item};

pub use super::config::{config, set, ZC_CONFIG};
pub use super::notices::{said, ZC_MSG};

/// ZC_EQUIPWIN_MICROSCOPE (0x0b37): what another player wears (the client's `ID_ZC_EQUIPWIN_MICROSCOPE_V8`).
pub const ZC_EQUIPWIN_MICROSCOPE: u16 = 0x0b37;
/// ZC_CONFIG_NOTIFY (0x02da, 3 bytes): whether others may look at the player's equipment, told on entering.
pub const ZC_CONFIG_NOTIFY: u16 = 0x02da;

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

/// ZC_CONFIG_NOTIFY (0x02da, 3 bytes): `u8 open`: others may look at the player's equipment.
pub fn open(packet: &[u8]) -> Option<bool> {
    packet.get(2).map(|open| *open != 0)
}

/// CZ_EQUIPWIN_MICROSCOPE (0x02d6, 6 bytes): `u32 account`: what does that player wear? (The client's
/// `MENUID_VIEW_OTHERUSER_EQUIPWND`.)
pub fn ask(account: u32) -> Vec<u8> {
    Writer::new(0x02d6).u32(account).done()
}

#[cfg(test)]
mod tests;
