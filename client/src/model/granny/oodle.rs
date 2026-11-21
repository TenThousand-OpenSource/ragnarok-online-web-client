//! Oodle0, the packing of a Granny file's sections (compression type 1 of the section table): a
//! Lempel-Ziv coder whose every number goes through an adaptive arithmetic coder.
//!
//! No open description of Oodle0 was found (the open readers — nwn2mdk's `gr2_decompress.cpp`, opengr2's
//! `oodle1.c` — know only its successor Oodle1, type 2, and refuse type 1). The two share their 36 bytes of
//! parameters and the shape of the coder (a literal, or a length and how far back), which those readers
//! showed; the rest was read out of the client's own `granny2.dll` (2.1.0.5), the function behind
//! `GrannyDecompressData` for type 1 and what it calls: the bit-wise arithmetic decoder of 31 bits, the
//! model of counted symbols with its halving (both in `coder.rs`, shared with the textures' packing), and
//! the step of the coder (`0x1001ca70` sets it up, `0x1001cbe0` is one step). Named here by what they do.

use super::coder::{Decoder, Model};

/// A section unpacked: `packed` is the section's bytes in the file, `stops` where each of its three parts
/// ends in the unpacked bytes (the section table's two stops and its unpacked size). Each part has its own
/// parameters and models; the arithmetic decoder runs through all three.
pub(super) fn unpack(packed: &[u8], stops: [usize; 3]) -> Result<Vec<u8>, String> {
    let parameters = packed.get(..36).ok_or("a packed section shorter than its parameters")?;
    let word = |at: usize| u32::from_le_bytes(parameters[at..at + 4].try_into().unwrap());
    let mut decoder = Decoder::new(&packed[36..]);
    let mut out = Vec::with_capacity(stops[2]);
    for (part, &stop) in stops.iter().enumerate() {
        let mut coder = Coder::new([word(part * 12), word(part * 12 + 4), word(part * 12 + 8)]);
        while out.len() < stop {
            coder.step(&mut decoder, &mut out)?;
        }
    }
    if out.len() != stops[2] {
        return Err("a packed section that unpacks past its size".to_owned());
    }
    Ok(out)
}

/// The lengths of a copy for the codes past 60 (the table the DLL's step reads; Oodle1 has the same).
const LONG_COPIES: [usize; 4] = [128, 192, 256, 512];
/// How many codes a length has: nothing copied, 2 to 61 bytes, and the four long ones.
const LENGTH_CODES: u32 = 65;

/// The coder of one part: what its parameters say and its models.
struct Coder {
    /// One more than the largest literal.
    literals: u32,
    /// The farthest a copy reaches back.
    farthest: u32,
    /// How many bytes this part has given, and the length code of the step before, which chooses the model
    /// of the next length.
    given: u32,
    last_length: usize,
    literal: Model,
    lengths: Vec<Model>,
    /// How far back, in two pieces: the two low bits and the rest.
    low: Model,
    rest: Model,
}

impl Coder {
    /// The first word of the parameters holds the literals' range (9 bits) and the farthest copy; the second
    /// how many literals (9 bits) and how many distances the models are to remember; the third, a byte for
    /// each quarter of the lengths' models, how many lengths.
    fn new(parameters: [u32; 3]) -> Self {
        let (literals, farthest) = (parameters[0] & 0x1ff, parameters[0] >> 9);
        let mut lengths: Vec<Model> = (0..64).map(|model| Model::new((parameters[2] >> (24 - 8 * (model / 16))) & 0xff)).collect();
        lengths.push(Model::new(parameters[2] & 0xff));
        Self {
            literals,
            farthest,
            given: 0,
            last_length: 0,
            literal: Model::new(parameters[1] & 0x1ff),
            lengths,
            low: Model::new(farthest.min(4)),
            rest: Model::new(parameters[1] >> 9),
        }
    }

    /// One step: a literal byte, or a copy of bytes given before.
    fn step(&mut self, decoder: &mut Decoder, out: &mut Vec<u8>) -> Result<(), String> {
        let code = self.lengths[self.last_length].decode(decoder, |decoder| decoder.number(LENGTH_CODES))? as usize;
        self.last_length = code;
        let (literals, farthest) = (self.literals, self.farthest);
        if code == 0 {
            out.push(self.literal.decode(decoder, |decoder| decoder.number(literals))? as u8);
            self.given += 1;
            return Ok(());
        }
        let length = if code < 61 { code + 1 } else { *LONG_COPIES.get(code - 61).ok_or("a length that is none")? };
        let low = self.low.decode(decoder, |decoder| decoder.number(farthest.min(4)))? as usize;
        let reach = farthest.min(self.given) / 4 + 1;
        let rest = self.rest.decode(decoder, |decoder| decoder.number(reach))? as usize;
        self.given += length as u32;
        let back = rest * 4 + low + 1;
        let from = out.len().checked_sub(back).ok_or("a copy from before the section")?;
        // Byte by byte: a copy may run into what it writes.
        for at in from..from + length {
            out.push(out[at]);
        }
        Ok(())
    }
}
