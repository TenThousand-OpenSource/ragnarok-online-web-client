//! The arithmetic decoder and the model of counted symbols of the client's `granny2.dll` (2.1.0.5), which
//! both of its packings read their numbers with: Oodle0 for a file's sections (`oodle.rs`) and the Bink
//! packing of a texture (`bink.rs`).
//!
//! No open description of them was found; they were read out of the DLL: `0x1000ddc0` narrows the interval,
//! `0x1000e020` reads an even number, `0x1000e0c0` starts a model, `0x1000e6f0` reads a symbol (finding it
//! through `0x1000e7f0`), `0x1000ec30` halves the counts, `0x1000e130` chooses the stretches of its sums.

/// Symbols as they were met, each with how often: the first place is "one not met yet" (the symbol itself
/// follows, read by whoever asks: now from the coded bits, now from plain ones), the others the symbols
/// remembered.
pub(super) struct Model {
    weights: Vec<u16>,
    values: Vec<u16>,
    total: u32,
    /// How many symbols are remembered, and how many may be.
    used: usize,
    most: usize,
}

impl Model {
    pub(super) fn new(most: u32) -> Self {
        let most = most as usize;
        let mut weights = vec![0; most + 2];
        weights[0] = 3;
        Self { weights, values: vec![0; most + 2], total: 3, used: 0, most }
    }

    /// The next symbol; one not met before is read by `fresh` and remembered.
    pub(super) fn decode(&mut self, decoder: &mut Decoder, fresh: impl FnOnce(&mut Decoder) -> Result<u32, String>) -> Result<u32, String> {
        if self.total >= 0x4000 {
            self.halve();
        }
        let target = decoder.target(self.total);
        let (mut at, mut below) = (0, 0);
        while below + self.weights[at] as u32 <= target {
            below += self.weights[at] as u32;
            at += 1;
            if at > self.used {
                return Err("a symbol past the model".to_owned());
            }
        }
        decoder.remove(below, self.weights[at] as u32, self.total)?;
        self.weights[at] += 1;
        self.total += 1;
        if at != 0 {
            return Ok(self.values[at] as u32);
        }
        self.used += 1;
        if self.used >= self.weights.len() {
            return Err("more symbols than the model holds".to_owned());
        }
        self.weights[self.used] += 2;
        self.total += 2;
        if self.used == self.most {
            self.total -= self.weights[0] as u32;
            self.weights[0] = 0;
        }
        let value = fresh(decoder)?;
        self.values[self.used] = value as u16;
        Ok(value)
    }

    /// Every count halved once they add up to too much: a symbol whose count comes to nothing is forgotten
    /// (the last one takes its place), and the most frequent is put where the DLL's table of sums finds it
    /// soonest. The order of the symbols is part of the code, so this follows the DLL step by step.
    fn halve(&mut self) {
        let (shift, limit) = buckets(self.used + 1);
        self.weights[0] >>= 1;
        let (mut best, mut best_at) = (0, 0);
        let mut at = 1;
        'symbols: while at <= self.used {
            while self.weights[at] <= 1 {
                let last = self.used;
                self.used -= 1;
                if at >= last {
                    self.weights[at] = 0;
                    break 'symbols;
                }
                self.weights[at] = self.weights[last];
                self.values[at] = self.values[last];
                self.weights[last] = 0;
            }
            self.weights[at] >>= 1;
            if self.weights[at] > best {
                (best, best_at) = (self.weights[at], at);
            }
            at += 1;
        }
        if best != 0 {
            let place = if self.used < limit { (self.used >> shift) << shift } else { limit }.max(1);
            self.weights.swap(place, best_at);
            self.values.swap(place, best_at);
        }
        if self.used != self.most && self.weights[0] == 0 {
            self.weights[0] = 2;
        }
        self.total = self.weights[..=self.used].iter().map(|&weight| weight as u32).sum();
    }
}

/// The DLL keeps sums of the counts over sixteen stretches of symbols to find one faster: the stretch's
/// length as a shift, and the first symbol of the last stretch. Only the halving's choice of a place for
/// the most frequent symbol depends on them. Fewer than six symbols have no stretches.
fn buckets(symbols: usize) -> (u32, usize) {
    if symbols < 6 {
        return (15, 0);
    }
    let (mut shift, mut least) = (0, usize::MAX);
    for candidate in 0..16 {
        let size = 1usize << candidate;
        let stretches = symbols.div_ceil(size).min(16);
        let last = (symbols - (stretches - 1) * size).max(size);
        if last < least {
            (shift, least) = (candidate, last);
        }
        if size > symbols {
            break;
        }
    }
    (shift, 15 << shift)
}

/// The arithmetic decoder: an interval of 31 bits narrowed by each symbol, and the code read into it bit by
/// bit (the file's bits from the lowest of each byte; past the end the DLL reads what lies there, here
/// nothing).
pub(super) struct Decoder<'a> {
    bytes: &'a [u8],
    bit: usize,
    low: u32,
    high: u32,
    code: u32,
}

const ALL: u32 = 0x7fff_ffff;
const TOP: u32 = 0x4000_0000;
const SECOND: u32 = 0x2000_0000;

impl<'a> Decoder<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        let mut decoder = Self { bytes, bit: 0, low: 0, high: ALL, code: 0 };
        for _ in 0..31 {
            decoder.code = decoder.code << 1 | decoder.next_bit();
        }
        decoder
    }

    fn next_bit(&mut self) -> u32 {
        let bit = self.bytes.get(self.bit / 8).map_or(0, |byte| (byte >> (self.bit % 8)) & 1);
        self.bit += 1;
        bit as u32
    }

    fn target(&self, total: u32) -> u32 {
        let width = (self.high - self.low) as u64 + 1;
        let place = (self.code.wrapping_sub(self.low) as u64 + 1) * total as u64;
        (place.saturating_sub(1) / width) as u32
    }

    fn remove(&mut self, below: u32, count: u32, total: u32) -> Result<(), String> {
        if self.bit > self.bytes.len() * 8 + 64 || total == 0 {
            return Err("coded bits that end too early".to_owned());
        }
        let width = (self.high - self.low) as u64 + 1;
        self.high = (self.low as u64 + width * (below + count) as u64 / total as u64).wrapping_sub(1) as u32;
        self.low += (width * below as u64 / total as u64) as u32;
        if self.high < self.low || self.high > ALL {
            return Err("coded bits that cannot be".to_owned());
        }
        loop {
            if (self.low ^ self.high) & TOP == 0 {
                self.low = (self.low << 1) & ALL;
                self.high = (self.high << 1 | 1) & ALL;
                self.code = (self.code << 1 | self.next_bit()) & ALL;
            } else if self.low & SECOND != 0 && self.high & SECOND == 0 {
                self.low = (self.low & (SECOND - 1)) << 1;
                self.high = (self.high << 1 | TOP | 1) & ALL;
                self.code = ((self.code ^ SECOND) << 1 | self.next_bit()) & ALL;
            } else {
                return Ok(());
            }
        }
    }

    /// A number under `range`, each as likely as another.
    pub(super) fn number(&mut self, range: u32) -> Result<u32, String> {
        if range == 0 {
            return Err("a number of no range".to_owned());
        }
        let number = self.target(range);
        self.remove(number, 1, range)?;
        Ok(number)
    }
}
