//! The hotkey bar as the server keeps it: its slots, and setting one.

use super::*;
use crate::net::bytes::{records, word, Writer};

/// What a slot of the hotkey bar holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hotkey {
    pub skill: bool,
    /// The item's or the skill's ID; 0 for an empty slot.
    pub id: u32,
    /// For a skill, the level it is used at.
    pub count: u16,
}

/// How many slots the bar has: four rows of nine, and two more the server keeps.
pub const HOTKEYS: usize = 38;

/// ZC_SHORTCUT_KEY_LIST (0x0b20): `u8 rotate, u16 tab`, then 7 bytes a slot: `u8 skill, u32 id, u16 count`.
pub fn hotkeys(packet: &[u8]) -> Vec<Hotkey> {
    records(packet, 5, 7).take(HOTKEYS).map(|slot| Hotkey { skill: slot[0] != 0, id: u32_at(slot, 1), count: u16_at(slot, 5) }).collect()
}

/// Which of the two bars a ZC_SHORTCUT_KEY_LIST is of (its `tab`): 0 the first, 1 the one the bar's switch
/// shows in its place (the client's `ReceiveShortcutPage`: `m_shortcutSlots`, `m_secondaryShortcutSlots`).
pub fn hotkeys_bar(packet: &[u8]) -> Option<u16> {
    word(packet, 3)
}

/// CZ_SHORTCUT_KEY_CHANGE (0x0b21): `u16 tab, u16 slot, u8 skill, u32 id, u16 count`, for the first bar.
pub fn set_hotkey(slot: u16, hotkey: Hotkey) -> Vec<u8> {
    set_hotkey_of(0, slot, hotkey)
}

/// The same for a slot of either bar (`tab`: 0 or 1).
pub fn set_hotkey_of(bar: u16, slot: u16, hotkey: Hotkey) -> Vec<u8> {
    Writer::new(0x0b21).u16(bar).u16(slot).u8(u8::from(hotkey.skill)).u32(hotkey.id).u16(hotkey.count).done()
}
