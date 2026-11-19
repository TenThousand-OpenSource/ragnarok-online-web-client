//! The lists a skill or a card opens to pick one thing of, and the answers to each: the places of Teleport
//! and Warp Portal (`warps`), the arrows to craft, the spells of Auto Spell, what can be forged, brewed and
//! cooked (`making`), what can be repaired and the weapons a smith can refine (`mending`), the equipment a
//! card goes into (`cards`); and the name of a character asked by its number (who made a thing). Each child
//! is named here whole, so everything is `net::choosing::…`. What is a rule and no packet (what goes into a
//! forging, the effect of an outcome, a card set into its slot) is in `crate::choosing`.
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`src/map/packets.hpp`, `packets_struct.hpp`).

use super::bytes::{Writer, long, records, text, u16_at, u32_at, word, words_from};

/// ZC_WARPLIST (0x0abe; the older 0x011c of four fixed names is not sent at this version).
pub const ZC_WARPLIST: u16 = 0x0abe;
pub const ZC_ACK_REMEMBER_WARPPOINT: u16 = 0x011e;
pub const ZC_MAKINGARROW_LIST: u16 = 0x01ad;
/// ZC_AUTOSPELLLIST (0x0afb; the older 0x01cd of seven fixed skills is not sent at this version).
pub const ZC_AUTOSPELLLIST: u16 = 0x0afb;
pub const ZC_MAKABLEITEMLIST: u16 = 0x018d;
pub const ZC_ACK_REQMAKINGITEM: u16 = 0x018f;
pub const ZC_MAKINGITEM_LIST: u16 = 0x025a;
/// ZC_REPAIRITEMLIST (0x0b65; the older 0x01fc is not sent at this version).
pub const ZC_REPAIRITEMLIST: u16 = 0x0b65;
pub const ZC_ACK_ITEMREPAIR: u16 = 0x01fe;
pub const ZC_NOTIFY_WEAPONITEMLIST: u16 = 0x0221;
pub const ZC_ACK_WEAPONREFINE: u16 = 0x0223;
pub const ZC_ACK_ITEMREFINING: u16 = 0x0188;
pub const ZC_ITEMCOMPOSITION_LIST: u16 = 0x017b;
pub const ZC_ACK_ITEMCOMPOSITION: u16 = 0x017d;

mod cards;
mod making;
mod mending;
mod warps;
pub use {cards::*, making::*, mending::*, warps::*};

// Asking what a character is called by its number is with its answer, in `net/actors.rs`.

#[cfg(test)]
mod tests;
