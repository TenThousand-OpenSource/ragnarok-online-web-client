//! The wavelet of the Bink texture packing, put together again: `granny2.dll` 2.1.0.5's `0x10009700`, which
//! for each of four sizes of a plane first joins every row's two halves (`0x10007d70`, or `0x100094c0` for a
//! row shorter than 12) and then every column's even and odd rows (`0x10008950`, or `0x100095f0` for fewer
//! than 10 rows). No open description exists; read out of the DLL.
//!
//! The long kind is the 9/7 wavelet in whole numbers of 16 bits after the point; its taps and its rounding
//! are the DLL's. How the ends of a line are mirrored was read at the start of a row and where the low half
//! ends, and taken to be the usual whole-sample mirror at the high half's end (not read to its last line).
//! The DLL skips the high half of the rows a plane's flags mark as empty; with the long kind that changes
//! nothing, so the flags are not read here.

/// The taps for the low half: on the spot, beside, and the two farther ones (taken away).
const LOW: [i64; 4] = [0xc9da, 0x6b08, 0xa6b, 0x1086];
/// The taps for the high half, from the middle outward.
const HIGH: [i64; 5] = [0xda4a, 0x609d, 0x1c52, 0x61b, 0x9af];

/// One pass over the part of a plane whose rows lie `step` rows apart: `width` × `height` numbers, each row's
/// low half then its high half, low and high rows by turns.
pub(super) fn join(plane: &mut [i16], full_width: usize, step: usize, width: usize, height: usize) {
    let pitch = full_width * step;
    let mut line = vec![0i16; width.max(height)];
    let mut joined = vec![0i16; width.max(height)];
    for row in 0..height {
        let at = row * pitch;
        line[..width].copy_from_slice(&plane[at..at + width]);
        let (low, high) = line[..width].split_at(width / 2);
        one(low, high, &mut joined[..width], width >= 12);
        plane[at..at + width].copy_from_slice(&joined[..width]);
    }
    let half = height / 2;
    let (mut low, mut high) = (vec![0i16; half], vec![0i16; half]);
    for column in 0..width {
        for row in 0..half {
            low[row] = plane[2 * row * pitch + column];
            high[row] = plane[(2 * row + 1) * pitch + column];
        }
        one(&low, &high, &mut joined[..height], height >= 10);
        for row in 0..height {
            plane[row * pitch + column] = joined[row];
        }
    }
}

/// A line's low and high halves joined: the long way, or for a short line as sums and differences.
pub(super) fn one(low: &[i16], high: &[i16], out: &mut [i16], long: bool) {
    let count = low.len() as isize;
    // Past its ends a line goes on as its mirror picture.
    let mirrored = |half: &[i16], at: isize, shift: isize| -> i64 {
        let at = if at < 0 { -at - shift } else if at >= count { 2 * count - 1 - shift - at } else { at };
        half[at.clamp(0, count - 1) as usize] as i64
    };
    for at in 0..count {
        let (even, odd) = if long {
            let l = |offset: isize| mirrored(low, at + offset, 0);
            let h = |offset: isize| mirrored(high, at + offset, 1);
            let even = LOW[0] * l(0) - LOW[2] * (l(-1) + l(1)) + HIGH[1] * (h(-1) + h(0)) - HIGH[3] * (h(-2) + h(1));
            let odd = LOW[1] * (l(0) + l(1)) - LOW[3] * (l(-1) + l(2)) - HIGH[0] * h(0) + HIGH[2] * (h(-1) + h(1)) - HIGH[4] * (h(-2) + h(2));
            (rounded(even as i32), rounded(odd as i32))
        } else {
            let (l, h) = (2 * low[at as usize] as i32, high[at as usize] as i32);
            (halved(l + h), halved(l - h))
        };
        out[2 * at as usize] = even;
        out[2 * at as usize + 1] = odd;
    }
}

/// A sum of 16 bits after the point as a whole number, the DLL's way: half added away from nothing, then
/// cut toward nothing.
fn rounded(sum: i32) -> i16 {
    (sum.wrapping_add(if sum < 0 { -0x8000 } else { 0x7fff }) / 0x10000) as i16
}

/// Half of a short line's sum, the DLL's way: one added to what is not below nothing, two taken from what
/// is, then cut toward nothing.
fn halved(sum: i32) -> i16 {
    (sum.wrapping_add(if sum < 0 { -2 } else { 1 }) / 2) as i16
}
