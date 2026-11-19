//! Reading the numbers and the texts of a packet, and writing a name into one: written here once, for every
//! file of packets. The numbers are little-endian.
//!
//! Two ways of reading a number, by what the reader knows: `u16_at`, `u32_at` and `u64_at` where the packet's
//! length was checked already (they stop the program past its end), `word` and `long` where it was not (they
//! give `None` there, so a parser can leave with `?`). The signed ones follow the same two ways: `i16_at`,
//! `i32_at`, `i64_at`, and `short`, `int`, `quad`.
//!
//! Writing a packet is `Writer`: its number, then each field in turn. A text goes into a field of a fixed
//! size in one of two ways, and a builder keeps the one its packet had: cut wherever the field ends (`field`,
//! `name24`, `Writer::text`), or cut at the end of a whole character (`field_whole_chars`).
//!
//! What the tests of the packets share is in `testing`.

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

/// The `i16` at that place of a packet long enough to have it.
pub(crate) fn i16_at(bytes: &[u8], at: usize) -> i16 {
    u16_at(bytes, at) as i16
}

/// The `i32` at that place of a packet long enough to have it.
pub(crate) fn i32_at(bytes: &[u8], at: usize) -> i32 {
    u32_at(bytes, at) as i32
}

/// The `i64` at that place of a packet long enough to have it.
pub(crate) fn i64_at(bytes: &[u8], at: usize) -> i64 {
    u64_at(bytes, at) as i64
}

/// The `i16` at that place, if the packet reaches so far.
pub(crate) fn short(bytes: &[u8], at: usize) -> Option<i16> {
    word(bytes, at).map(|word| word as i16)
}

/// The `i32` at that place, if the packet reaches so far.
pub(crate) fn int(bytes: &[u8], at: usize) -> Option<i32> {
    long(bytes, at).map(|long| long as i32)
}

/// The `i64` at that place, if the packet reaches so far.
pub(crate) fn quad(bytes: &[u8], at: usize) -> Option<i64> {
    Some(i64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?))
}

/// A packet's number: its first two bytes, if it has them.
pub(crate) fn id(packet: &[u8]) -> Option<u16> {
    word(packet, 0)
}

/// The records of `size` bytes each that follow the first `from` bytes of a packet; what is left over at the
/// end and is no whole record is passed by, and a packet shorter than `from` has none.
pub(crate) fn records(packet: &[u8], from: usize, size: usize) -> impl Iterator<Item = &[u8]> {
    packet.get(from..).unwrap_or_default().chunks_exact(size)
}

/// What a packet that carries its own length (a `u16` at 2) has from `from` up to that length, never past
/// what arrived of it; none when it has no length, or the length ends before `from`.
pub(crate) fn body(packet: &[u8], from: usize) -> Option<&[u8]> {
    let length = usize::from(word(packet, 2)?).min(packet.len());
    packet.get(from..length)
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

/// A text in a field of `N` bytes, the last of them a zero: `N - 1` bytes of it at most, cut where the field
/// ends, be it in the middle of a character (`name24` is this for 24).
pub(crate) fn field<const N: usize>(words: &str) -> [u8; N] {
    let mut field = [0; N];
    let size = words.len().min(N - 1);
    field[..size].copy_from_slice(&words.as_bytes()[..size]);
    field
}

/// A text in a field of `N` bytes, the last of them a zero, cut at the end of a whole character.
pub(crate) fn field_whole_chars<const N: usize>(words: &str) -> [u8; N] {
    let mut field = [0; N];
    let mut size = words.len().min(N - 1);
    while !words.is_char_boundary(size) {
        size -= 1;
    }
    field[..size].copy_from_slice(&words.as_bytes()[..size]);
    field
}

/// A packet being written: its number first, then each field as it is named, little-endian; `done` gives the
/// bytes. One begun with `sized` carries its own length after its number, written by `done`.
pub(crate) struct Writer {
    bytes: Vec<u8>,
    sized: bool,
}

impl Writer {
    /// A packet of that number.
    pub(crate) fn new(id: u16) -> Self {
        Self { bytes: id.to_le_bytes().to_vec(), sized: false }
    }

    /// A packet of that number with its length, a `u16`, after it: `done` writes how long it came to be.
    pub(crate) fn sized(id: u16) -> Self {
        Self { bytes: [id.to_le_bytes(), [0, 0]].concat(), sized: true }
    }

    pub(crate) fn u8(mut self, value: u8) -> Self {
        self.bytes.push(value);
        self
    }

    pub(crate) fn u16(mut self, value: u16) -> Self {
        self.bytes.extend(value.to_le_bytes());
        self
    }

    pub(crate) fn u32(mut self, value: u32) -> Self {
        self.bytes.extend(value.to_le_bytes());
        self
    }

    pub(crate) fn i32(mut self, value: i32) -> Self {
        self.bytes.extend(value.to_le_bytes());
        self
    }

    pub(crate) fn i64(mut self, value: i64) -> Self {
        self.bytes.extend(value.to_le_bytes());
        self
    }

    /// Bytes as they are: a text of the packet's own length, a list, a record made elsewhere.
    pub(crate) fn bytes(mut self, bytes: &[u8]) -> Self {
        self.bytes.extend_from_slice(bytes);
        self
    }

    /// A name in its 24 bytes (`name24`).
    pub(crate) fn name(self, name: &str) -> Self {
        self.bytes(&name24(name))
    }

    /// A text in a field of `size` bytes, zero-padded, cut where the field ends; the last byte always stays
    /// zero, as the server expects a terminator (`field`, for a size known only as the packet is written).
    pub(crate) fn text(mut self, text: &str, size: usize) -> Self {
        let bytes = &text.as_bytes()[..text.len().min(size - 1)];
        self.bytes.extend(bytes);
        self.bytes.resize(self.bytes.len() + size - bytes.len(), 0);
        self
    }

    /// A text in a field of `N` bytes, cut where the field ends (`field`).
    pub(crate) fn field<const N: usize>(self, words: &str) -> Self {
        self.bytes(&field::<N>(words))
    }

    /// A text in a field of `N` bytes, cut at the end of a whole character (`field_whole_chars`).
    pub(crate) fn field_whole_chars<const N: usize>(self, words: &str) -> Self {
        self.bytes(&field_whole_chars::<N>(words))
    }

    /// The packet's bytes, its length written into it if it carries one.
    pub(crate) fn done(mut self) -> Vec<u8> {
        if self.sized {
            let length = (self.bytes.len() as u16).to_le_bytes();
            self.bytes[2..4].copy_from_slice(&length);
        }
        self.bytes
    }
}

/// What the tests of the packets build their packets with, and the check every file of requests makes.
#[cfg(test)]
pub(crate) mod testing {
    /// A text in a field of `size` bytes, zero-padded, as the server writes one.
    pub(crate) fn field(text: &str, size: usize) -> Vec<u8> {
        let mut field = text.as_bytes().to_vec();
        field.resize(size, 0);
        field
    }

    /// A packet: its number, then its parts one after another.
    pub(crate) fn packet(id: u16, parts: &[&[u8]]) -> Vec<u8> {
        let mut packet = id.to_le_bytes().to_vec();
        for part in parts {
            packet.extend_from_slice(part);
        }
        packet
    }

    /// A packet with its length after its number.
    pub(crate) fn sized(id: u16, body: &[u8]) -> Vec<u8> {
        [&id.to_le_bytes()[..], &((body.len() + 4) as u16).to_le_bytes(), body].concat()
    }

    /// `u32`s one after another, as a packet has them.
    pub(crate) fn longs(values: &[u32]) -> Vec<u8> {
        values.iter().flat_map(|value| value.to_le_bytes()).collect()
    }

    /// Each of these requests is as long as the server's table says a packet of its number is.
    pub(crate) fn assert_lengths(packets: &[Vec<u8>]) {
        for packet in packets {
            let id = u16::from_le_bytes([packet[0], packet[1]]);
            assert_eq!(crate::net::packet_length(id), Some(packet.len() as i32), "{id:#06x}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_signed_numbers_both_ways() {
        let packet = [0xfe, 0xff, 0xfd, 0xff, 0xff, 0xff, 0xfc, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        assert_eq!((i16_at(&packet, 0), i32_at(&packet, 2), i64_at(&packet, 6)), (-2, -3, -4));
        assert_eq!((short(&packet, 0), int(&packet, 2), quad(&packet, 6)), (Some(-2), Some(-3), Some(-4)));
        assert_eq!((short(&packet, 13), int(&packet, 11), quad(&packet, 7)), (None, None, None));
        assert_eq!((id(&packet), id(&packet[..1])), (Some(0xfffe), None));
    }

    #[test]
    fn cuts_a_packet_into_its_records_and_its_body() {
        let packet = [0x01, 0x02, 9, 0, 1, 2, 3, 4, 5, 6, 7];
        assert_eq!(records(&packet, 4, 2).collect::<Vec<_>>(), [[1, 2], [3, 4], [5, 6]]);
        assert_eq!(records(&packet, 20, 2).count(), 0);
        // Up to the length the packet names, and never past what arrived of it.
        assert_eq!(body(&packet, 4), Some(&[1, 2, 3, 4, 5][..]));
        assert_eq!(body(&[0x01, 0x02, 99, 0, 1, 2], 4), Some(&[1, 2][..]));
        assert_eq!((body(&[0x01, 0x02, 4, 0, 1], 5), body(&[0x01, 0x02, 9], 4)), (None, None));
    }

    #[test]
    fn writes_a_text_into_a_field_in_two_ways() {
        assert_eq!(field::<4>("ab"), [b'a', b'b', 0, 0]);
        assert_eq!(field::<4>("abcdef"), [b'a', b'b', b'c', 0]);
        assert_eq!(field::<24>("Poring"), name24("Poring"));
        // "é" is two bytes: one way cuts it in half, the other leaves it out whole.
        assert_eq!(field::<4>("ab\u{e9}"), [b'a', b'b', 0xc3, 0]);
        assert_eq!(field_whole_chars::<4>("ab\u{e9}"), [b'a', b'b', 0, 0]);
        assert_eq!(field_whole_chars::<5>("ab\u{e9}"), [b'a', b'b', 0xc3, 0xa9, 0]);
    }

    #[test]
    fn writes_a_packet_field_by_field() {
        let packet = Writer::new(0x0064).u8(1).u16(0x0203).u32(0x0405_0607).i32(-2).bytes(&[9]).done();
        assert_eq!(packet, [0x64, 0, 1, 3, 2, 7, 6, 5, 4, 0xfe, 0xff, 0xff, 0xff, 9]);
        assert_eq!(Writer::new(1).i64(-1).done()[2..], [255, 255, 255, 255, 255, 255, 255, 255]);
        assert_eq!(Writer::new(1).text("abcdef", 4).text("a", 3).done()[2..], [b'a', b'b', b'c', 0, b'a', 0, 0]);
        assert_eq!(Writer::new(1).name("Poring").done()[2..], name24("Poring"));
        assert_eq!(Writer::new(1).field::<3>("ab\u{e9}").field_whole_chars::<4>("ab\u{e9}").done()[2..], [b'a', b'b', 0, b'a', b'b', 0, 0]);
        // One with a length says how long it came to be.
        assert_eq!(Writer::sized(0x00d5).u32(5).bytes(b"ab\0").done(), [0xd5, 0, 11, 0, 5, 0, 0, 0, b'a', b'b', 0]);
    }

    #[test]
    fn the_tests_build_packets_as_the_server_does() {
        assert_eq!(testing::field("ab", 4), [b'a', b'b', 0, 0]);
        assert_eq!(testing::packet(0x0102, &[&[1], &[2, 3]]), [2, 1, 1, 2, 3]);
        assert_eq!(testing::sized(0x0102, &[7]), [2, 1, 5, 0, 7]);
        assert_eq!(testing::longs(&[1, 0x0102_0304]), [1, 0, 0, 0, 4, 3, 2, 1]);
        // CZ_REQUEST_TIME (0x0360) is 6 bytes in the server's table.
        testing::assert_lengths(&[Writer::new(0x0360).u32(0).done()]);
    }

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
