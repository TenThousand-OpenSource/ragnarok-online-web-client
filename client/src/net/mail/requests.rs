//! What the client sends about the mail: opening and closing the mailbox, reading, deleting and taking, giving
//! a letter back, and writing one (the receiver checked, things put in and taken out, sending).

use super::*;
use crate::net::bytes::Writer;

/// CZ_OPEN_MAILBOX2 (0x0ac0) and CZ_REQ_REFRESH_MAIL_LIST2 (0x0ac1), 26 bytes: the last number the client has
/// of the letters of the first kind, of those returned and of the account's (`Store::last`, in that order).
/// The first asks for the mailbox as the window opens, the second anew.
pub fn open(anew: bool, last: [i64; 3]) -> Vec<u8> {
    last.into_iter().fold(Writer::new(if anew { 0x0ac1 } else { 0x0ac0 }), Writer::i64).done()
}

/// CZ_CLOSE_MAILBOX (0x09e9): the mailbox's window was closed.
pub fn close() -> Vec<u8> {
    Writer::new(0x09e9).done()
}

/// CZ_REQ_READ_MAIL (0x09ea) and CZ_REQ_DELETE_MAIL (0x09f5), 11 bytes: `u8 tab`, `i64 id`.
pub fn ask(tab: u8, letter: i64) -> Vec<u8> {
    Writer::new(0x09ea).u8(tab).i64(letter).done()
}

pub fn delete(tab: u8, letter: i64) -> Vec<u8> {
    Writer::new(0x09f5).u8(tab).i64(letter).done()
}

/// CZ_REQ_ZENY_FROM_MAIL (0x09f1) and CZ_REQ_ITEM_FROM_MAIL (0x09f3), 11 bytes: `i64 id`, `u8 tab`.
pub fn take(items: bool, tab: u8, letter: i64) -> Vec<u8> {
    Writer::new(if items { 0x09f3 } else { 0x09f1 }).i64(letter).u8(tab).done()
}

/// CZ_RODEX_RETURN (0x0b98, 6 bytes): `u32 id`: the letter is to go back to its sender. rAthena reads it and
/// does nothing ("not supported for now").
pub fn give_back(letter: i64) -> Vec<u8> {
    Writer::new(0x0b98).u32(letter as u32).done()
}

/// CZ_REQ_OPEN_WRITE_MAIL (0x0a08, 26 bytes): `char receiver[24]`: a letter is to be written, to that one if
/// a name is given. A name is cut to 23 bytes so that it ends in a zero, here and in the two below.
pub fn write(receiver: &str) -> Vec<u8> {
    Writer::new(0x0a08).name(receiver).done()
}

/// CZ_REQ_CANCEL_WRITE_MAIL (0x0a03): the window of the letter being written was closed.
pub fn cancel_write() -> Vec<u8> {
    Writer::new(0x0a03).done()
}

/// CZ_CHECKNAME2 (0x0b97, 27 bytes): `char name[24]`, `u8 known`: who is that, and whether the client knows
/// the name as one of the player's own characters, a friend or of the guild.
pub fn check_receiver(name: &str, known: bool) -> Vec<u8> {
    Writer::new(0x0b97).name(name).u8(u8::from(known)).done()
}

/// CZ_REQ_ADD_ITEM_TO_MAIL (0x0a04) and CZ_REQ_REMOVE_ITEM_MAIL (0x0a06), 6 bytes: `u16 index`, `u16 amount`.
pub fn attach(index: u16, amount: u16) -> Vec<u8> {
    Writer::new(0x0a04).u16(index).u16(amount).done()
}

pub fn detach(index: u16, amount: u16) -> Vec<u8> {
    Writer::new(0x0a06).u16(index).u16(amount).done()
}

/// CZ_REQ_WRITE_MAIL2 (0x0a6e): `u16 length`, `char receiver[24]`, `char sender[24]`, `i64 zeny`,
/// `u16 title_length`, `u16 text_length`, `u32 receiver_character`, then the title and the text, each with
/// the zero that ends it counted in its length (the client's `MM_SEND_MAIL`).
pub fn send(receiver: &Receiver, sender: &str, zeny: i64, title: &str, body: &str) -> Vec<u8> {
    Writer::sized(0x0a6e)
        .name(&receiver.name)
        .name(sender)
        .i64(zeny)
        .u16(title.len() as u16 + 1)
        .u16(body.len() as u16 + 1)
        .u32(receiver.character)
        .bytes(title.as_bytes())
        .u8(0)
        .bytes(body.as_bytes())
        .u8(0)
        .done()
}
