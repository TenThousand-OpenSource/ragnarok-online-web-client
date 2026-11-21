//! A plain reader of bits for the Bink texture packing, beside the arithmetic decoder and the model it
//! shares with Oodle0 (`granny/coder.rs`).

pub(super) use super::super::coder::{Decoder, Model};

/// Bits as they lie, from the lowest of each byte (the DLL reads 32-bit words and shifts them down).
pub(super) struct Bits<'a> {
    bytes: &'a [u8],
    bit: usize,
}

impl<'a> Bits<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, bit: 0 }
    }

    /// The next `count` bits as a number, the first bit lowest.
    pub(super) fn take(&mut self, count: u32) -> Result<u32, String> {
        let mut number = 0;
        for place in 0..count {
            let byte = self.bytes.get(self.bit / 8).ok_or("a texture's plain bits end too early")?;
            number |= ((byte >> (self.bit % 8)) as u32 & 1) << place;
            self.bit += 1;
        }
        Ok(number)
    }

    /// A number with its sign in the next bit, when it is not nothing.
    pub(super) fn signed(&mut self, size: u32) -> Result<i32, String> {
        Ok(if size != 0 && self.take(1)? == 1 { -(size as i32) } else { size as i32 })
    }
}

