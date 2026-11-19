//! SPR: a sprite's pictures, palette-indexed and truecolor, and PAL, a palette to recolour them with.
//!
//! Format facts follow the layouts documented by korangar's `ragnarok-formats` (MIT) and rustnarok-client's
//! `src/sprite.rs` (MIT OR Apache-2.0).

use crate::bytes::Reader;

pub type Palette = [[u8; 4]; 256];

pub struct Picture {
    pub width: usize,
    pub height: usize,
    /// Palette indices for `Sprite::indexed`, RGBA for `Sprite::truecolor`; top row first.
    pub pixels: Vec<u8>,
}

pub struct Sprite {
    pub indexed: Vec<Picture>,
    pub truecolor: Vec<Picture>,
    pub palette: Palette,
}

/// A PAL file, or the palette at the end of a SPR: 256 colours of 4 bytes (red, green, blue, unused).
pub fn parse_palette(bytes: &[u8]) -> Option<Palette> {
    let bytes = bytes.get(..1024)?;
    let mut palette = [[0u8; 4]; 256];
    for (colour, raw) in palette.iter_mut().zip(bytes.chunks_exact(4)) {
        *colour = [raw[0], raw[1], raw[2], 255];
    }
    Some(palette)
}

/// How many bytes a picture of this size takes, `each` to a pixel; an error when the count does not fit a
/// `usize` (two `u16` multiplied by 4 pass 32 bits, which is all the browser's build has).
fn area(width: usize, height: usize, each: usize) -> Result<usize, String> {
    width.checked_mul(height).and_then(|pixels| pixels.checked_mul(each)).ok_or_else(|| format!("SPR picture of {width}x{height} is too large"))
}

pub fn parse_spr(bytes: &[u8]) -> Result<Sprite, String> {
    let mut reader = Reader::new(bytes);
    if reader.take(2)? != b"SP" {
        return Err("not a SPR file".to_owned());
    }
    let version = reader.u16()?;
    let indexed_count = usize::from(reader.u16()?);
    let truecolor_count = if version >= 0x200 { usize::from(reader.u16()?) } else { 0 };

    let mut indexed = Vec::with_capacity(indexed_count.min(4096));
    for _ in 0..indexed_count {
        let (width, height) = (usize::from(reader.u16()?), usize::from(reader.u16()?));
        let pixels = if version >= 0x201 {
            // Run-length coded: a zero byte is followed by how many zeros it stands for.
            let coded = { let size = usize::from(reader.u16()?); reader.take(size)? };
            // A byte of the code stands for 255 pixels at most: a size the code cannot fill is a broken
            // file, refused before memory is asked for it (65535 x 65535 would be 4 GB).
            if area(width, height, 1)? > coded.len() * 255 {
                return Err(format!("SPR picture of {width}x{height} from {} coded bytes", coded.len()));
            }
            let mut pixels = Vec::with_capacity(width * height);
            let mut bytes = coded.iter();
            while let Some(&byte) = bytes.next() {
                if byte == 0 {
                    let run = usize::from(*bytes.next().unwrap_or(&1)).max(1);
                    pixels.resize(pixels.len() + run, 0);
                } else {
                    pixels.push(byte);
                }
            }
            pixels.resize(width * height, 0);
            pixels
        } else {
            reader.take(width * height)?.to_vec()
        };
        indexed.push(Picture { width, height, pixels });
    }

    let mut truecolor = Vec::with_capacity(truecolor_count.min(4096));
    for _ in 0..truecolor_count {
        let (width, height) = (usize::from(reader.u16()?), usize::from(reader.u16()?));
        // Stored bottom row first, each pixel as alpha, blue, green, red.
        let raw = reader.take(area(width, height, 4)?)?;
        let mut pixels = vec![0u8; raw.len()];
        for row in 0..height {
            for column in 0..width {
                let from = ((height - 1 - row) * width + column) * 4;
                let to = (row * width + column) * 4;
                pixels[to..to + 4].copy_from_slice(&[raw[from + 3], raw[from + 2], raw[from + 1], raw[from]]);
            }
        }
        truecolor.push(Picture { width, height, pixels });
    }

    let palette = if version >= 0x101 { bytes.len().checked_sub(1024).and_then(|at| parse_palette(&bytes[at..])) } else { None };
    Ok(Sprite { indexed, truecolor, palette: palette.unwrap_or([[0, 0, 0, 255]; 256]) })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run-length coded picture that says 65535 x 65535 over an empty code: refused, not 4 GB of zeros.
    #[test]
    fn a_size_the_code_cannot_fill_is_refused() {
        let mut file = b"SP".to_vec();
        file.extend(0x201u16.to_le_bytes());
        file.extend([1, 0, 0, 0]);
        file.extend([0xff, 0xff, 0xff, 0xff, 0, 0]);
        assert!(parse_spr(&file).is_err());
    }

    #[test]
    fn a_coded_picture_is_read_and_filled() {
        let mut file = b"SP".to_vec();
        file.extend(0x201u16.to_le_bytes());
        file.extend([1, 0, 0, 0]);
        // 3 x 2 from: 7, three zeros, 9; the sixth pixel is filled in.
        file.extend([3, 0, 2, 0, 4, 0, 7, 0, 3, 9]);
        let sprite = parse_spr(&file).unwrap();
        assert_eq!(sprite.indexed[0].pixels, [7, 0, 0, 0, 9, 0]);
    }

    /// What the browser's 32 bits cannot hold is an error; here it is asked with sizes no `u16` has.
    #[test]
    fn an_area_past_usize_is_an_error() {
        assert_eq!(area(3, 2, 4), Ok(24));
        assert!(area(usize::MAX / 2, 3, 1).is_err());
        assert!(area(usize::MAX / 4, 2, 4).is_err());
    }

    #[test]
    fn a_truecolor_picture_larger_than_the_file_is_refused() {
        let mut file = b"SP".to_vec();
        file.extend(0x200u16.to_le_bytes());
        file.extend([0, 0, 1, 0]);
        file.extend([0xff, 0xff, 0xff, 0xff]);
        assert!(parse_spr(&file).is_err());
    }
}
