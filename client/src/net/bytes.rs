//! Reading the numbers and the texts of a packet, and writing a name into one: written here once, for every
//! file of packets. The numbers are little-endian.
//!
//! Two ways of reading a number, by what the reader knows: `u16_at`, `u32_at` and `u64_at` where the packet's
//! length was checked already (they stop the program past its end), `word` and `long` where it was not (they
//! give `None` there, so a parser can leave with `?`).

/// The `u16` at that place of a packet long enough to have it.
pub(crate) fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

/// The `u32` at that place of a packet long enough to have it.
pub(crate) fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

/// The `u64` at that place of a packet long enough to have it.
pub(crate) fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

/// The `u16` at that place, if the packet reaches so far.
pub(crate) fn word(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

/// The `u32` at that place, if the packet reaches so far.
pub(crate) fn long(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

/// The `u16`s a packet lists from that place to its end (the places in the inventory of some things).
pub(crate) fn words_from(bytes: &[u8], at: usize) -> Vec<u16> {
    bytes.get(at..).unwrap_or_default().chunks_exact(2).map(|word| u16::from_le_bytes([word[0], word[1]])).collect()
}

/// A name or a text as the server sends it in a field: up to its first zero byte, or all of the field.
pub(crate) fn text(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

/// The text in the field of `size` bytes at that place, if the packet reaches so far.
pub(crate) fn text_at(bytes: &[u8], at: usize, size: usize) -> Option<String> {
    bytes.get(at..at + size).map(text)
}

/// A name in a field of `size` bytes at that place (or as much of it as the packet has), for the packets of
/// the way in and of those in sight. Names on this server are ASCII; where one is not UTF-8 its bytes are
/// read as Latin-1, so nothing is lost.
pub(crate) fn name_at(bytes: &[u8], at: usize, size: usize) -> String {
    let field = &bytes[at..(at + size).min(bytes.len())];
    let end = field.iter().position(|&byte| byte == 0).unwrap_or(field.len());
    match std::str::from_utf8(&field[..end]) {
        Ok(name) => name.to_owned(),
        Err(_) => field[..end].iter().map(|&byte| char::from(byte)).collect(),
    }
}

/// A name as the 24 bytes a packet has it in: 23 bytes of it at most, and a zero.
pub(crate) fn name24(name: &str) -> [u8; 24] {
    let mut field = [0u8; 24];
    let length = name.len().min(23);
    field[..length].copy_from_slice(&name.as_bytes()[..length]);
    field
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_numbers_where_the_packet_has_them() {
        let packet = [0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0];
        assert_eq!((u16_at(&packet, 0), u32_at(&packet, 2), u64_at(&packet, 2)), (0x1234, 0x1234_5678, 0x1234_5678));
        assert_eq!((word(&packet, 0), long(&packet, 2)), (Some(0x1234), Some(0x1234_5678)));
        assert_eq!((word(&packet, 9), long(&packet, 7)), (None, None));
        assert_eq!((words_from(&packet, 4), words_from(&packet, 12)), (vec![0x1234, 0, 0], vec![]));
    }

    #[test]
    fn reads_a_text_up_to_its_zero() {
        assert_eq!(text(b"Poring\0junk"), "Poring");
        assert_eq!(text(b"whole"), "whole");
        assert_eq!(text_at(b"..Poring\0.", 2, 7).as_deref(), Some("Poring"));
        assert_eq!(text_at(b"..Por", 2, 7), None);
        // What is no UTF-8 is read byte by byte in a name of the way in, and never past the packet's end.
        assert_eq!(name_at(&[b'A', 0xe9, 0, b'x'], 0, 24), "A\u{e9}");
    }

    #[test]
    fn writes_a_name_with_its_zero() {
        assert_eq!(&name24("Poring")[..8], b"Poring\0\0");
        assert_eq!(name24(&"x".repeat(40))[22..], [b'x', 0]);
    }
}
