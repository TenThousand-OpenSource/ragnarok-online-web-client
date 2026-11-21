//! The numbers of one plane of a Bink-packed texture, as the wavelet left them: `granny2.dll` 2.1.0.5's
//! `0x10006650`, which reads the smallest picture (`0x10006a30`) and then, for each of four sizes, three
//! bands of differences (`0x10006e50`, its models started by `0x10007540`). No open description exists;
//! read out of the DLL.
//!
//! A plane's packed bytes are two sizes, then so many bytes read through the arithmetic decoder, then so
//! many read as plain bits (sizes of things, signs, a run not met before). In the plane a band's rows lie
//! apart: the low rows of a size on its even rows, the high ones on its odd rows; within a row the low half
//! comes first.

use super::coder::{Bits, Decoder, Model};

/// The lengths of a run of numbers for the codes from 60, and of a run of nothing for the codes from 252.
const LONG_NUMBERS: [usize; 4] = [128, 256, 512, 1024];
const LONG_NOTHING: [usize; 4] = [512, 1024, 2048, 3072];

struct Streams<'a> {
    coded: Decoder<'a>,
    plain: Bits<'a>,
}

/// One plane of `width` × `height` (both multiples of 16) read from the start of `packed`; gives the plane
/// and how many bytes it took.
pub(super) fn plane(packed: &[u8], width: usize, height: usize) -> Result<(Vec<i16>, usize), String> {
    let size = |at: usize| packed.get(at..at + 4).map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) as usize);
    let (coded, plain) = size(0).zip(size(4)).ok_or("a texture's plane ends too early")?;
    let end = coded.checked_add(plain).and_then(|sizes| sizes.checked_add(8)).filter(|&end| end <= packed.len());
    let end = end.ok_or("a texture's plane larger than the texture")?;
    let mut streams = Streams { coded: Decoder::new(&packed[8..]), plain: Bits::new(&packed[8 + coded..end]) };
    let mut plane = vec![0i16; width * height];
    smallest(&mut streams, &mut Band { plane: &mut plane, at: 0, pitch: 16 * width, width: width / 16, height: height / 16 })?;
    for step in [16, 8, 4, 2] {
        let (pitch, wide, high) = (step * width, width / step, height / step);
        for at in [wide, pitch / 2, pitch / 2 + wide] {
            band(&mut streams, &mut Band { plane: &mut plane, at, pitch, width: wide, height: high })?;
        }
    }
    Ok((plane, end))
}

/// A band's place in its plane: where it starts, how far its rows lie apart, and its size.
struct Band<'a> {
    plane: &'a mut [i16],
    at: usize,
    pitch: usize,
    width: usize,
    height: usize,
}

impl Band<'_> {
    fn get(&self, at: usize) -> Result<i32, String> {
        self.plane.get(at).map(|&number| number as i32).ok_or_else(|| "a texture's band past its plane".to_owned())
    }

    fn set(&mut self, at: usize, number: i32) -> Result<(), String> {
        *self.plane.get_mut(at).ok_or("a texture's band past its plane")? = number as i16;
        Ok(())
    }

    fn fill(&mut self, number: i32) -> Result<(), String> {
        for row in 0..self.height {
            for column in 0..self.width {
                self.set(self.at + row * self.pitch + column, number)?;
            }
        }
        Ok(())
    }
}

/// The smallest picture of a plane: one number for all of it, or each as its difference from its
/// neighbours (the one to the left in the first row, the one above in the first column, half their sum
/// elsewhere).
fn smallest(streams: &mut Streams, band: &mut Band) -> Result<(), String> {
    let Streams { coded, plain } = streams;
    if plain.take(1)? == 1 {
        return band.fill(plain.take(16)? as i32);
    }
    let range = plain.take(16)? + 1;
    let mut model = Model::new(range);
    let mut left = plain.take(16)? as i32;
    for row in 0..band.height {
        let start = band.at + row * band.pitch;
        for column in 0..band.width {
            if row == 0 && column == 0 {
                band.set(start, left)?;
                continue;
            }
            let difference = plain.signed(model.decode(coded, |coded| coded.number(range))?)?;
            left = match (row, column) {
                (0, _) => left,
                (_, 0) => band.get(start - band.pitch)?,
                _ => (band.get(start - band.pitch + column)? + left) / 2,
            } + difference;
            band.set(start + column, left)?;
        }
    }
    Ok(())
}

/// How large the numbers around are, as one of sixteen kinds: how many bits their size takes.
fn kind(size: u32) -> usize {
    (32 - size.leading_zeros()).min(15) as usize
}

/// What the reading of a band is at.
enum Next {
    Runs,
    Numbers,
    Nothing,
    RowEnd,
}

/// A band being read: where the next number goes, the row above it, and the numbers around.
struct Walk {
    at: usize,
    above: usize,
    left_in_row: usize,
    rows: usize,
    north: i32,
    north_west: i32,
    west: i32,
}

impl Walk {
    /// On to the next row: nothing lies to the left yet, so the number above stands in for it.
    fn row_end(&mut self, band: &Band) -> Result<bool, String> {
        self.rows -= 1;
        if self.rows == 0 {
            return Ok(true);
        }
        self.at += band.pitch - band.width;
        self.above = self.at.checked_sub(band.pitch).ok_or("a texture's band before its plane")?;
        self.north = band.get(self.above)?;
        (self.north_west, self.west) = (self.north, self.north);
        self.above += 1;
        self.left_in_row = band.width;
        Ok(false)
    }
}

/// A band of differences: all one number, or runs of numbers and of nothing. A number's model is chosen by
/// the sizes of the numbers already read around it; each is a size times the band's step, and a sign.
fn band(streams: &mut Streams, band: &mut Band) -> Result<(), String> {
    let Streams { coded, plain } = streams;
    if band.width == 0 || band.height == 0 {
        return Ok(());
    }
    let step = plain.take(16)? as i32;
    if plain.take(1)? == 1 {
        return band.fill((plain.take(16)? as i32).wrapping_mul(step));
    }
    let most = plain.take(16)?;
    let range = most + 1;
    let mut sizes: Vec<Model> = (0..=kind(most.wrapping_mul(step as u32))).map(|_| Model::new(range)).collect();
    let (mut number_runs, mut nothing_runs) = (Model::new(0x40), Model::new(0x100));
    let first = plain.signed(coded.number(range)?)?.wrapping_mul(step);
    band.set(band.at, first)?;
    let mut walk =
        Walk { at: band.at + 1, above: band.at, left_in_row: band.width - 1, rows: band.height, north: first, north_west: first, west: first };
    let (mut numbers, mut nothing) = (0usize, 0usize);
    let mut next = if band.width == 1 { Next::RowEnd } else { Next::Runs };
    loop {
        next = match next {
            Next::Runs => {
                numbers = number_runs.decode(coded, |_| plain.take(6))? as usize;
                if numbers >= 60 {
                    numbers = LONG_NUMBERS[numbers - 60];
                }
                nothing = match nothing_runs.decode(coded, |_| plain.take(8))? as usize {
                    0 => 0,
                    long @ 252.. => *LONG_NOTHING.get(long - 252).ok_or("a run of no length")? + 2,
                    short => short + 2,
                };
                Next::Numbers
            }
            Next::Numbers if numbers == 0 => Next::Nothing,
            Next::Numbers if walk.left_in_row == 0 => Next::RowEnd,
            Next::Numbers => {
                let last = walk.left_in_row == 1;
                let north_east = if last { 0 } else { band.get(walk.above)? };
                let around = if last { (2 * walk.west).unsigned_abs() } else { north_east.unsigned_abs() + walk.west.unsigned_abs() };
                let around = (around + walk.north_west.unsigned_abs() + walk.north.unsigned_abs()) >> 2;
                let model = sizes.get_mut(kind(around)).ok_or("a texture's number larger than its band allows")?;
                let number = plain.signed(model.decode(coded, |coded| coded.number(range))?)?.wrapping_mul(step);
                band.set(walk.at, number)?;
                walk.at += 1;
                numbers -= 1;
                if last {
                    Next::RowEnd
                } else {
                    walk.above += 1;
                    walk.left_in_row -= 1;
                    (walk.west, walk.north_west, walk.north) = (number, walk.north, north_east);
                    Next::Numbers
                }
            }
            Next::RowEnd => {
                if walk.row_end(band)? {
                    return Ok(());
                }
                Next::Numbers
            }
            Next::Nothing if nothing == 0 => Next::Runs,
            Next::Nothing => {
                let run = nothing.min(walk.left_in_row);
                for at in walk.at..walk.at + run {
                    band.set(at, 0)?;
                }
                walk.at += run;
                if nothing >= walk.left_in_row {
                    nothing -= walk.left_in_row;
                    if walk.row_end(band)? {
                        return Ok(());
                    }
                    Next::Nothing
                } else {
                    walk.left_in_row -= run;
                    walk.above += run;
                    nothing = 0;
                    (walk.north_west, walk.north, walk.west) = (band.get(walk.above.wrapping_sub(2))?, band.get(walk.above.wrapping_sub(1))?, 0);
                    Next::Runs
                }
            }
        };
    }
}
