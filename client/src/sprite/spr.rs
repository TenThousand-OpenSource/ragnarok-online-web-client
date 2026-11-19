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
        let raw = reader.take(width * height * 4)?;
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
