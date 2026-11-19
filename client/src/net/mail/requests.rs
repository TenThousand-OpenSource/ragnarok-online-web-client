//! What the client sends about the mail: opening and closing the mailbox, reading, deleting and taking, giving a letter back, and writing one (the receiver checked, things put in and taken out, sending).

use super::*;

/// A packet of an ID alone.
pub(super) fn bare(id: u16) -> Vec<u8> {
    id.to_le_bytes().to_vec()
}

/// A packet of an ID and a name of 24 bytes, cut to 23 so that it ends in a zero.
pub(super) fn named(id: u16, name: &str, more: usize) -> Vec<u8> {
    let mut packet = vec![0u8; 26 + more];
    packet[..2].copy_from_slice(&id.to_le_bytes());
    packet[2..26].copy_from_slice(&crate::net::bytes::name24(name));
    packet
}

/// CZ_OPEN_MAILBOX2 (0x0ac0) and CZ_REQ_REFRESH_MAIL_LIST2 (0x0ac1), 26 bytes: the last number the client has
/// of the letters of the first kind, of those returned and of the account's (`Store::last`, in that order).
/// The first asks for the mailbox as the window opens, the second anew.
pub fn open(anew: bool, last: [i64; 3]) -> Vec<u8> {
    let mut packet = bare(if anew { 0x0ac1 } else { 0x0ac0 });
    for id in last {
        packet.extend_from_slice(&id.to_le_bytes());
    }
    packet
}

/// CZ_CLOSE_MAILBOX (0x09e9): the mailbox's window was closed.
pub fn close() -> Vec<u8> {
    bare(0x09e9)
}

/// CZ_REQ_READ_MAIL (0x09ea) and CZ_REQ_DELETE_MAIL (0x09f5), 11 bytes: `u8 tab`, `i64 id`.
pub(super) fn about(id: u16, tab: u8, letter: i64) -> Vec<u8> {
    [&id.to_le_bytes()[..], &[tab], &letter.to_le_bytes()].concat()
}

pub fn ask(tab: u8, letter: i64) -> Vec<u8> {
    about(0x09ea, tab, letter)
}

pub fn delete(tab: u8, letter: i64) -> Vec<u8> {
    about(0x09f5, tab, letter)
}

/// CZ_REQ_ZENY_FROM_MAIL (0x09f1) and CZ_REQ_ITEM_FROM_MAIL (0x09f3), 11 bytes: `i64 id`, `u8 tab`.
pub fn take(items: bool, tab: u8, letter: i64) -> Vec<u8> {
    [&(if items { 0x09f3u16 } else { 0x09f1 }).to_le_bytes()[..], &letter.to_le_bytes(), &[tab]].concat()
}

/// CZ_RODEX_RETURN (0x0b98, 6 bytes): `u32 id`: the letter is to go back to its sender. rAthena reads it and
/// does nothing ("not supported for now").
pub fn give_back(letter: i64) -> Vec<u8> {
    [&0x0b98u16.to_le_bytes()[..], &(letter as u32).to_le_bytes()].concat()
}

/// CZ_REQ_OPEN_WRITE_MAIL (0x0a08, 26 bytes): `char receiver[24]`: a letter is to be written, to that one if
/// a name is given.
pub fn write(receiver: &str) -> Vec<u8> {
    named(0x0a08, receiver, 0)
}

/// CZ_REQ_CANCEL_WRITE_MAIL (0x0a03): the window of the letter being written was closed.
pub fn cancel_write() -> Vec<u8> {
    bare(0x0a03)
}

/// CZ_CHECKNAME2 (0x0b97, 27 bytes): `char name[24]`, `u8 known`: who is that, and whether the client knows
/// the name as one of the player's own characters, a friend or of the guild.
pub fn check_receiver(name: &str, known: bool) -> Vec<u8> {
    let mut packet = named(0x0b97, name, 1);
    packet[26] = u8::from(known);
    packet
}

/// CZ_REQ_ADD_ITEM_TO_MAIL (0x0a04) and CZ_REQ_REMOVE_ITEM_MAIL (0x0a06), 6 bytes: `u16 index`, `u16 amount`.
pub fn attach(index: u16, amount: u16) -> Vec<u8> {
    [0x0a04u16.to_le_bytes(), index.to_le_bytes(), amount.to_le_bytes()].concat()
}

pub fn detach(index: u16, amount: u16) -> Vec<u8> {
    [0x0a06u16.to_le_bytes(), index.to_le_bytes(), amount.to_le_bytes()].concat()
}

/// CZ_REQ_WRITE_MAIL2 (0x0a6e): `u16 length`, `char receiver[24]`, `char sender[24]`, `i64 zeny`,
/// `u16 title_length`, `u16 text_length`, `u32 receiver_character`, then the title and the text, each with
/// the zero that ends it counted in its length (the client's `MM_SEND_MAIL`).
pub fn send(receiver: &Receiver, sender: &str, zeny: i64, title: &str, body: &str) -> Vec<u8> {
    let mut packet = named(0x0a6e, "", 42);
    for (at, name) in [(4, receiver.name.as_str()), (28, sender)] {
        packet[at..at + 24].copy_from_slice(&crate::net::bytes::name24(name));
    }
    packet[52..60].copy_from_slice(&zeny.to_le_bytes());
    packet[60..62].copy_from_slice(&(title.len() as u16 + 1).to_le_bytes());
    packet[62..64].copy_from_slice(&(body.len() as u16 + 1).to_le_bytes());
    packet[64..68].copy_from_slice(&receiver.character.to_le_bytes());
    for words in [title, body] {
        packet.extend_from_slice(words.as_bytes());
        packet.push(0);
    }
    let length = packet.len() as u16;
    packet[2..4].copy_from_slice(&length.to_le_bytes());
    packet
}
