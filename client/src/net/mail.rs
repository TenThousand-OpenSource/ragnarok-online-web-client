//! The mail (the client's RODEX): the letters the map server lists, one read, its zeny and things taken, a
//! letter deleted, and one written with zeny and things put into it.
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`src/map/packets_struct.hpp`, `clif.cpp` from
//! `clif_Mail_setattachment` on); what is kept of the letters and how they are ordered is the client's
//! (`RodexMailStore`, `UIRodexInboxWnd`). At this version the server sends the whole mailbox at once
//! (ZC_ACK_MAIL_LIST3, 0x0ac2); the pages of six are cut by the client.

use super::bytes::{long, quad, text, word};
use super::inventory::{Amount, Identified, Item, Layout};

// What is kept of the letters and the rules of the mailbox are no packets: they are in `crate::mail`.
use crate::mail::*;

/// ZC_NOTIFY_UNREADMAIL: whether there is mail not read.
pub const ZC_NOTIFY_UNREADMAIL: u16 = 0x09e7;
/// ZC_ACK_WRITE_MAIL: what became of the letter sent.
pub const ZC_ACK_WRITE_MAIL: u16 = 0x09ed;
/// ZC_ACK_ZENY_FROM_MAIL and ZC_ACK_ITEM_FROM_MAIL: what became of taking a letter's zeny, and its things.
pub const ZC_ACK_ZENY_FROM_MAIL: u16 = 0x09f2;
pub const ZC_ACK_ITEM_FROM_MAIL: u16 = 0x09f4;
/// ZC_ACK_DELETE_MAIL: a letter was deleted.
pub const ZC_ACK_DELETE_MAIL: u16 = 0x09f6;
/// ZC_ACK_REMOVE_ITEM_MAIL: a thing was taken back out of the letter being written.
pub const ZC_ACK_REMOVE_ITEM_MAIL: u16 = 0x0a07;
/// ZC_ACK_OPEN_WRITE_MAIL: whether a letter may be written now.
pub const ZC_ACK_OPEN_WRITE_MAIL: u16 = 0x0a12;
/// ZC_CHECKNAME: who the receiver typed is.
pub const ZC_CHECKNAME: u16 = 0x0a51;
/// ZC_ACK_MAIL_LIST3: the mailbox.
pub const ZC_MAIL_LIST: u16 = 0x0ac2;
/// ZC_ACK_ADD_ITEM_RODEX: a thing was put into the letter being written, or why not.
pub const ZC_ACK_ADD_ITEM_RODEX: u16 = 0x0b3f;
/// ZC_ACK_READ_RODEX: a letter's text, zeny and things.
pub const ZC_ACK_READ_RODEX: u16 = 0x0b63;

/// ZC_ACK_MAIL_LIST3 (0x0ac2): `u16 length`, `u8 end`, then each letter: `u8 tab`, `i64 id`, `u8 read`,
/// `u8 has`, `char sender[24]`, `u32 expires_in`, `u16 title_length`, `char title[title_length]`. Gives back
/// whether the list is whole, and the letters.
pub fn list(packet: &[u8]) -> Option<(bool, Vec<Listed>)> {
    let length = usize::from(word(packet, 2)?).min(packet.len());
    let whole = *packet.get(4)? == 1;
    let mut letters = Vec::new();
    let mut at = 5;
    while at + 41 <= length {
        let title = usize::from(word(packet, at + 39)?);
        letters.push(Listed {
            tab: packet[at],
            id: quad(packet, at + 1)?,
            read: packet[at + 9] != 0,
            has: packet[at + 10],
            sender: text(&packet[at + 11..at + 35]),
            expires_in: long(packet, at + 35)?,
            title: text(packet.get(at + 41..at + 41 + title)?),
        });
        at += 41 + title;
    }
    Some((whole, letters))
}

/// A letter read: its kind and number, its text, zeny and things.
#[derive(Debug, Clone, PartialEq)]
pub struct Read {
    pub tab: u8,
    pub id: i64,
    pub body: String,
    pub zeny: i64,
    pub items: Vec<Item>,
}

/// ZC_ACK_READ_RODEX (0x0b63): `u16 length`, `u8 tab`, `i64 id`, `u16 text_length`, `i64 zeny`, `u8 count`,
/// `char text[text_length]`, then `count` things of 60 bytes: `u16 amount`, `u32 item`, `u8 identified`,
/// `u8 damaged`, `u32 card[4]`, `u32 location`, `u8 type`, `u16 view`, `u16 bound`, five options of 5 bytes,
/// `u8 refine`, `u8 grade`.
pub fn read(packet: &[u8]) -> Option<Read> {
    /// A thing of the letter, counted from its amount.
    const IN_LETTER: Layout =
        Layout {
            index: None, id: 2, kind: 28, amount: Amount::Word(0), wearable: Some(24), worn: None,
            refine: Some(58), identified: Identified::Byte(6), cards: 8,
        };

    let length = usize::from(word(packet, 13)?);
    let things = packet.get(24 + length..)?;
    let items = things.chunks_exact(60).take(usize::from(*packet.get(23)?)).filter_map(|thing| Item::read(thing, &IN_LETTER)).collect();
    Some(Read { tab: *packet.get(4)?, id: quad(packet, 5)?, body: text(&packet[24..24 + length]), zeny: quad(packet, 15)?, items })
}

/// ZC_NOTIFY_UNREADMAIL (0x09e7, 3 bytes): `u8 unread`.
pub fn unread(packet: &[u8]) -> Option<bool> {
    Some(*packet.get(2)? == 1)
}

/// ZC_ACK_ZENY_FROM_MAIL (0x09f2) and ZC_ACK_ITEM_FROM_MAIL (0x09f4), 12 bytes: `i64 id`, `u8 tab`,
/// `u8 result` (0 taken, 1 failed, 2 more than the character can hold). Gives back the kind, the number and
/// the result.
pub fn taken(packet: &[u8]) -> Option<(u8, i64, u8)> {
    Some((*packet.get(10)?, quad(packet, 2)?, *packet.get(11)?))
}

/// ZC_ACK_DELETE_MAIL (0x09f6, 11 bytes): `u8 tab`, `i64 id`. It comes only for a letter that was deleted.
pub fn deleted(packet: &[u8]) -> Option<(u8, i64)> {
    Some((*packet.get(2)?, quad(packet, 3)?))
}

/// ZC_ACK_OPEN_WRITE_MAIL (0x0a12, 27 bytes): `char receiver[24]`, `u8 allowed`.
pub fn write_opened(packet: &[u8]) -> Option<(String, bool)> {
    Some((text(packet.get(2..26)?), *packet.get(26)? == 1))
}

/// Who a letter goes to, as the server knows them.
#[derive(Debug, Clone, PartialEq)]
pub struct Receiver {
    pub character: u32,
    pub job: u16,
    pub level: u16,
    pub name: String,
}

/// ZC_CHECKNAME (0x0a51, 34 bytes): `u32 character`, `u16 job`, `u16 level`, `char name[24]`. A character of
/// 0 says there is nobody of the name asked about.
pub fn receiver(packet: &[u8]) -> Option<Receiver> {
    Some(Receiver { character: long(packet, 2)?, job: word(packet, 6)?, level: word(packet, 8)?, name: text(packet.get(10..34)?) })
}

/// What ZC_ACK_ADD_ITEM_RODEX answers with.
pub mod added {
    pub const DONE: u8 = 0;
    pub const OVERWEIGHT: u8 = 1;
    pub const FAILED: u8 = 2;
    pub const TOO_MANY: u8 = 3;
    pub const NOT_ALLOWED: u8 = 4;
}

/// ZC_ACK_ADD_ITEM_RODEX (0x0b3f, 64 bytes): `u8 result`, `u16 index`, `u16 amount`, `u32 item`, `u8 type`,
/// `u8 identified`, `u8 damaged`, `u32 card[4]`, five options of 5 bytes, `u16 weight`, `u8 favorite`,
/// `u32 location`, `u8 refine`, `u8 grade`. Gives back the result, the thing as it lies in the letter (where it
/// is carried, and how many of it went in) and the weight of everything in the letter.
pub fn add_answered(packet: &[u8]) -> Option<(u8, Item, u16)> {
    let packet = packet.get(..64)?;
    /// The thing put in, counted from the packet's start.
    const PUT_IN: Layout =
        Layout {
            index: Some(3), id: 7, kind: 11, amount: Amount::Word(5), wearable: Some(58), worn: None,
            refine: Some(62), identified: Identified::Byte(12), cards: 14,
        };
    let item = Item::read(packet, &PUT_IN)?;
    Some((packet[2], item, word(packet, 55)?))
}

/// ZC_ACK_REMOVE_ITEM_MAIL (0x0a07, 9 bytes): `u8 done`, `u16 index`, `u16 amount`, `u16 weight`: so many of
/// the thing carried there came back out of the letter, and what is left in it weighs that.
pub fn remove_answered(packet: &[u8]) -> Option<(bool, u16, u16, u16)> {
    Some((*packet.get(2)? == 1, word(packet, 3)?, word(packet, 5)?, word(packet, 7)?))
}

/// ZC_ACK_WRITE_MAIL (0x09ed, 3 bytes): `u8 result`: what became of the letter sent: 0 sent, 1 not delivered,
/// 2 too many sent, 3 things that may not be sent, 4 nobody of that name (the lines the client says of each
/// are 2587, 2597, 2603, 2598 and 2599 of its message table).
pub fn sent(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}

mod requests;
pub use requests::*;

#[cfg(test)]
mod tests;
