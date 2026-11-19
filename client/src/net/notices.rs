//! What the map server tells the player in a line of the chat, with a number or a name to put into it: the
//! packets. The layouts are rAthena's for PACKETVER 20260219 (`packets.hpp`, `clif.cpp`). The client's rules for
//! which line of its message table is said and in which colour are `crate::notices`.
//!
//! Writing what a line asks for into it is `crate::numbers::filled`, and the answer to raising a stat is read
//! with the character's values (`net/packets/zone.rs`): both are still named from here, where they were.

use super::bytes::{i64_at, int, long, word};
use super::packets::number_and_value;

use crate::notices::{always_red, skill_line};
use crate::said::colour::{hex, RED};

/// ZC_ACTION_FAILURE (0x013b, 4 bytes): `u16 kind`: no ammunition, too heavy, or ammunition put on.
pub const ZC_ACTION_FAILURE: u16 = 0x013b;
/// ZC_NOTIFY_MAPINFO (0x0189, 4 bytes): `u16 kind`: what cannot be done on this map.
pub const ZC_NOTIFY_MAPINFO: u16 = 0x0189;
/// ZC_MSG_VALUE (0x07e2, 8 bytes): `u16 line, i32 value`: a line of the message table with a number in it.
pub const ZC_MSG_VALUE: u16 = 0x07e2;
/// ZC_MSG_SKILL (0x07e6, 8 bytes): `u16 skill, i32 line`: a line of the message table about a skill.
pub const ZC_MSG_SKILL: u16 = 0x07e6;
/// ZC_MSG_COLOR (0x09cd, 8 bytes): `u16 line, u32 colour`: a line of the message table in a colour.
pub const ZC_MSG_COLOR: u16 = 0x09cd;
/// ZC_SKILLMSG (0x0215, 6 bytes): `i32 kind`: what Gospel did to the player.
pub const ZC_SKILLMSG: u16 = 0x0215;
/// ZC_NOTIFY_EXP2 (0x0acc, 18 bytes): experience gained or lost.
pub const ZC_NOTIFY_EXP2: u16 = 0x0acc;
/// ZC_RECOVER_PENALTY_OVERWEIGHT (0x0ade, 6 bytes): `i32 percent`: from what share of the weight carried
/// life and SP no longer come back by themselves.
pub const ZC_RECOVER_PENALTY_OVERWEIGHT: u16 = 0x0ade;
/// ZC_PAR_CHANGE_USER (0x01ab, 12 bytes): `u32 who, u16 kind, u32 value`: a value of someone in sight.
pub const ZC_PAR_CHANGE_USER: u16 = 0x01ab;
/// ZC_EQUIPITEM_DAMAGED (0x02bb, 8 bytes): `u16 place, u32 who`: something worn was damaged.
pub const ZC_EQUIPITEM_DAMAGED: u16 = 0x02bb;
/// ZC_CASH_TIME_COUNTER (0x0298, 10 bytes): `u32 item, u32 seconds`: how long a rented item still lasts.
pub const ZC_CASH_TIME_COUNTER: u16 = 0x0298;
/// ZC_CASH_ITEM_DELETE (0x0299, 8 bytes): `u16 index, u32 item`: a rented item is gone.
pub const ZC_CASH_ITEM_DELETE: u16 = 0x0299;
/// ZC_NOTIFY_BIND_ON_EQUIP (0x02d3, 4 bytes): `u16 index`: a thing put on is bound to the character.
pub const ZC_NOTIFY_BIND_ON_EQUIP: u16 = 0x02d3;
/// ZC_MSG (0x0291, 4 bytes): a line of the client's message table is said. That the other does not let their
/// equipment be looked at comes so (`MSI_OPEN_EQUIPEDITEM_REFUSED`; rAthena's `clif_msg`).
pub const ZC_MSG: u16 = 0x0291;
/// ZC_ACK_PVPPOINT (0x0210, 22 bytes): the answer to a request the client no longer makes; its
/// `CGameMode` has no reader for it.
pub const ZC_ACK_PVPPOINT: u16 = 0x0210;

/// The one number of a packet that is nothing else: the kind of ZC_ACTION_FAILURE and ZC_NOTIFY_MAPINFO, the
/// index of ZC_NOTIFY_BIND_ON_EQUIP.
pub fn kind(packet: &[u8]) -> Option<u16> {
    word(packet, 2)
}

/// ZC_MSG: `u16 line`: which line of the message table.
pub fn said(packet: &[u8]) -> Option<usize> {
    word(packet, 2).map(usize::from)
}

/// ZC_MSG_VALUE: the line and the number to write into it.
pub fn value(packet: &[u8]) -> Option<(usize, i32)> {
    number_and_value(packet).map(|(line, value)| (usize::from(line), value))
}

/// ZC_MSG_SKILL: the skill and the line said of it.
pub fn skill(packet: &[u8]) -> Option<(u16, usize)> {
    let (skill, line) = number_and_value(packet)?;
    Some((skill, usize::try_from(line).ok()?))
}

/// ZC_MSG_COLOR: the line and the colour it is said in, as the page takes it. Five lines are red whatever
/// the server names (`crate::notices::always_red`).
pub fn coloured(packet: &[u8]) -> Option<(usize, String)> {
    let line = usize::from(word(packet, 2)?);
    let colour = packet.get(4..8)?;
    let named = if always_red(line) { RED.to_owned() } else { hex([colour[0], colour[1], colour[2]]) };
    Some((line, named))
}

/// ZC_SKILLMSG: `i32 kind`: the line of the message table it says (`crate::notices::skill_line`); nothing for
/// a kind that has none.
pub fn skill_message(packet: &[u8]) -> Option<usize> {
    skill_line(long(packet, 2)?)
}

/// Experience gained or lost, as ZC_NOTIFY_EXP2 tells it.
#[derive(Debug, PartialEq)]
pub struct Experience {
    pub who: u32,
    /// How much: less than nought for what was lost.
    pub amount: i64,
    /// 1 for base experience, 2 for job experience.
    pub kind: u16,
    /// It was a quest that gave it.
    pub quest: bool,
}

/// ZC_NOTIFY_EXP2 (0x0acc, 18 bytes): `u32 who, i64 amount, u16 kind, u16 quest`.
pub fn experience(packet: &[u8]) -> Option<Experience> {
    let packet = packet.get(..18)?;
    Some(Experience { who: long(packet, 2)?, amount: i64_at(packet, 6), kind: word(packet, 14)?, quest: word(packet, 16)? == 1 })
}

/// ZC_RECOVER_PENALTY_OVERWEIGHT: the share of the weight, if it is one (`ReceiveOverweightThreshold` keeps
/// only a number over nought).
pub fn overweight(packet: &[u8]) -> Option<u32> {
    long(packet, 2).filter(|percent| (*percent as i32) > 0)
}

/// The kind of ZC_PAR_CHANGE_USER the client reads (`case ID_ZC_PAR_CHANGE_USER`): the manner points.
pub const MANNER: u16 = 4;

/// ZC_PAR_CHANGE_USER: who, which value, and the value.
pub fn user_value(packet: &[u8]) -> Option<(u32, u16, i32)> {
    Some((long(packet, 2)?, word(packet, 6)?, int(packet, 8)?))
}

/// ZC_EQUIPITEM_DAMAGED: the place of what was damaged (the bits of where a thing is worn) and whose it is;
/// nothing for no place.
pub fn damaged(packet: &[u8]) -> Option<(u16, u32)> {
    Some((word(packet, 2).filter(|place| *place != 0)?, long(packet, 4)?))
}

/// ZC_CASH_TIME_COUNTER: the item and the seconds it still lasts.
pub fn rental_time(packet: &[u8]) -> Option<(u32, u32)> {
    Some((long(packet, 2)?, long(packet, 6)?))
}

/// ZC_CASH_ITEM_DELETE: where the item was carried, and which it is.
pub fn rental_over(packet: &[u8]) -> Option<(u16, u32)> {
    Some((word(packet, 2)?, long(packet, 4)?))
}

#[cfg(test)]
mod tests;
