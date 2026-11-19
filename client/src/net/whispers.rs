//! Whispers: one heard, one sent and the answer to it. (Whose whispers are let in is `net/commands.rs`; the
//! rules of a talk in a window of its own are `crate::whispers`.)

use super::bytes::{text, Writer};

/// ZC_WHISPER (0x09de): `u16 length, u32 sender, char[24] name, u8 admin, char[] words`: someone whispers.
pub const ZC_WHISPER: u16 = 0x09de;
/// ZC_ACK_WHISPER (0x09df): `u8 result, u32 character`: 0 if the whisper arrived.
pub const ZC_ACK_WHISPER: u16 = 0x09df;

/// Who whispers, and what.
pub fn whispered(packet: &[u8]) -> Option<(String, String)> {
    Some((text(packet.get(8..32)?), text(packet.get(33..)?)))
}

/// ZC_ACK_WHISPER (0x09df, 7 bytes): how the whisper went (`crate::whispers::answer`).
pub fn whisper_answered(packet: &[u8]) -> Option<u8> {
    packet.get(..7).map(|packet| packet[2])
}

/// CZ_WHISPER (0x0096): `u16 length, char[24] name, char[] words` ending in a zero.
pub fn whisper(name: &str, words: &str) -> Vec<u8> {
    Writer::sized(0x0096).name(name).bytes(words.as_bytes()).u8(0).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::{field, sized};

    #[test]
    fn reads_how_a_whisper_went() {
        assert_eq!(whisper_answered(&[0xdf, 0x09, 1, 0, 0, 0, 0]), Some(crate::whispers::answer::ABSENT));
        assert_eq!(whisper_answered(&[0xdf, 0x09, 0]), None);
    }

    #[test]
    fn a_whisper_is_heard_and_sent() {
        let heard = sized(ZC_WHISPER, &[&[7, 0, 0, 0][..], &field("Somebody", 24), &[0], b"psst\0"].concat());
        assert_eq!(whispered(&heard), Some(("Somebody".to_owned(), "psst".to_owned())));
        assert_eq!(whispered(&heard[..32]), None);
        let sent = whisper("Somebody", "psst");
        assert_eq!((&sent[..4], &sent[4..13], &sent[28..]), (&[0x96, 0x00, 33, 0][..], &b"Somebody\0"[..], &b"psst\0"[..]));
    }
}
