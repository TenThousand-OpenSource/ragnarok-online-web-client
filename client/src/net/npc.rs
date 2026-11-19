//! Talking to NPCs: what the player sends, and what the script answers (rAthena, `clif_scriptmes` and its
//! neighbours in `src/map/clif.cpp`, at PACKETVER 20260219); and the line in a colour of its own that an NPC,
//! or the server, says into the chat (ZC_NPC_CHAT).
//!
//! Whispers are `net/whispers.rs` and CZ_RESTART is with the basics of the map server (`net/packets/zone.rs`):
//! both are still named from here, where they were.

use super::bytes::{long, text, Writer};

pub use super::packets::restart;
pub use super::whispers::{whisper, whisper_answered, whispered, ZC_ACK_WHISPER, ZC_WHISPER};

/// ZC_SAY_DIALOG (0x00b4): `u16 length, u32 npc, char[] text`; ZC_SAY_DIALOG2 (0x0972) has a `u8 type` before
/// the text.
pub const ZC_SAY_DIALOG: u16 = 0x00b4;
pub const ZC_SAY_DIALOG2: u16 = 0x0972;
/// ZC_WAIT_DIALOG (0x00b5; 0x0973 with a `u8 type` more): the script waits for "next".
pub const ZC_WAIT_DIALOG: u16 = 0x00b5;
pub const ZC_WAIT_DIALOG2: u16 = 0x0973;
/// ZC_CLOSE_DIALOG (0x00b6): the script is over, and the window stays until "close" is pressed.
pub const ZC_CLOSE_DIALOG: u16 = 0x00b6;
/// ZC_MENU_LIST (0x00b7): `u16 length, u32 npc, char[] choices`, the choices divided by colons.
pub const ZC_MENU_LIST: u16 = 0x00b7;
/// ZC_CLEAR_DIALOG (0x08d6): the text so far goes away.
pub const ZC_CLEAR_DIALOG: u16 = 0x08d6;
/// ZC_OPEN_EDITDLG (0x0142) asks for a number, ZC_OPEN_EDITDLGSTR (0x01d4) for words.
pub const ZC_OPEN_EDITDLG: u16 = 0x0142;
pub const ZC_OPEN_EDITDLGSTR: u16 = 0x01d4;

/// ZC_NPC_CHAT (0x02c1): `u16 length, u32 who, u32 colour, char[] text`: a line for the chat in a colour of its
/// own (blue first), said by someone in sight or by the server itself (rAthena's `clif_messagecolor`).
pub const ZC_NPC_CHAT: u16 = 0x02c1;

/// ZC_NPC_CHAT: who says it, the colour as red, green and blue, and the words.
pub fn coloured_line(packet: &[u8]) -> Option<(u32, [u8; 3], String)> {
    let who = long(packet, 4)?;
    let colour = packet.get(8..11)?;
    Some((who, [colour[0], colour[1], colour[2]], text(packet.get(12..)?)))
}

/// The NPC a packet is about: at offset 4 where a length comes first, else at 2.
pub fn npc(id: u16, packet: &[u8]) -> Option<u32> {
    let at = if matches!(id, ZC_SAY_DIALOG | ZC_SAY_DIALOG2 | ZC_MENU_LIST) { 4 } else { 2 };
    long(packet, at)
}

/// What an NPC says: one line of its window.
pub fn said(id: u16, packet: &[u8]) -> Option<String> {
    Some(text(packet.get(if id == ZC_SAY_DIALOG2 { 9 } else { 8 }..)?))
}

/// The choices of a menu. An empty one is still a choice: the server counts it.
pub fn menu(packet: &[u8]) -> Option<Vec<String>> {
    let mut choices: Vec<String> = text(packet.get(8..)?).split(':').map(str::to_owned).collect();
    // The list ends with a colon.
    if choices.last().is_some_and(String::is_empty) {
        choices.pop();
    }
    Some(choices)
}

/// CZ_CONTACTNPC (0x0090): `u32 npc, u8 type` (1): the player presses an NPC.
pub fn contact(npc: u32) -> Vec<u8> {
    Writer::new(0x0090).u32(npc).u8(1).done()
}

/// CZ_REQ_NEXT_SCRIPT (0x00b9): "next" was pressed.
pub fn next(npc: u32) -> Vec<u8> {
    Writer::new(0x00b9).u32(npc).done()
}

/// CZ_CLOSE_DIALOG (0x0146): "close" was pressed.
pub fn close(npc: u32) -> Vec<u8> {
    Writer::new(0x0146).u32(npc).done()
}

/// CZ_CHOOSE_MENU (0x00b8): `u32 npc, u8 choice`, the first being 1; 255 turns the menu down.
pub fn choose(npc: u32, choice: u8) -> Vec<u8> {
    Writer::new(0x00b8).u32(npc).u8(choice).done()
}

/// CZ_INPUT_EDITDLG (0x0143): `u32 npc, i32 value`.
pub fn number(npc: u32, value: i32) -> Vec<u8> {
    Writer::new(0x0143).u32(npc).i32(value).done()
}

/// CZ_INPUT_EDITDLGSTR (0x01d5): `u16 length, u32 npc, char[] words` ending in a zero.
pub fn words(npc: u32, words: &str) -> Vec<u8> {
    Writer::sized(0x01d5).u32(npc).bytes(words.as_bytes()).u8(0).done()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_line_in_a_colour_of_its_own() {
        let mut packet = vec![0xc1, 0x02, 16, 0, 7, 0, 0, 0, 0, 0, 255, 0];
        packet.extend(b"No!\0");
        assert_eq!(coloured_line(&packet), Some((7, [0, 0, 255], "No!".to_owned())));
    }

    #[test]
    fn reads_what_an_npc_says_in_both_forms() {
        let mut old = vec![0xb4, 0, 14, 0, 7, 0, 0, 0];
        old.extend(b"Hello\0");
        assert_eq!((npc(ZC_SAY_DIALOG, &old), said(ZC_SAY_DIALOG, &old)), (Some(7), Some("Hello".to_owned())));
        let mut new = vec![0x72, 0x09, 15, 0, 7, 0, 0, 0, 1];
        new.extend(b"Hello\0");
        assert_eq!((npc(ZC_SAY_DIALOG2, &new), said(ZC_SAY_DIALOG2, &new)), (Some(7), Some("Hello".to_owned())));
    }

    #[test]
    fn reads_a_menu_and_keeps_empty_choices() {
        let mut packet = vec![0xb7, 0, 0, 0, 9, 0, 0, 0];
        packet.extend(b"Save::Quit:\0");
        assert_eq!(menu(&packet), Some(vec!["Save".to_owned(), String::new(), "Quit".to_owned()]));
    }

    #[test]
    fn builds_the_answers() {
        assert_eq!(contact(0x01020304), [0x90, 0, 4, 3, 2, 1, 1]);
        assert_eq!(choose(5, 2), [0xb8, 0, 5, 0, 0, 0, 2]);
        assert_eq!(next(5), [0xb9, 0, 5, 0, 0, 0]);
        assert_eq!(words(5, "ab"), [0xd5, 0x01, 11, 0, 5, 0, 0, 0, b'a', b'b', 0]);
        crate::net::bytes::testing::assert_lengths(&[contact(1), choose(1, 1), next(1), close(1), number(1, 1)]);
    }
}
