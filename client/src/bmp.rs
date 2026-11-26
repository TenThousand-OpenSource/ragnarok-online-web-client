//! Windows BMP decoder for the client's textures: uncompressed 8-bit paletted, 24-bit and 32-bit images.
//! Magenta (255, 0, 255) is the client's transparent colour.

pub struct Image {
    pub width: u32,
    pub height: u32,
    /// RGBA, top row first.
    pub rgba: Vec<u8>,
}

pub fn decode(bytes: &[u8]) -> Result<Image, String> {
    if bytes.len() < 54 || &bytes[..2] != b"BM" {
        return Err("not a BMP file".into());
    }
    // The file header (14 bytes) and the start of the picture's: all of it is there, the file being 54 bytes at least.
    let mut reader = crate::bytes::Reader::new(bytes);
    reader.take(10)?;
    let pixels_at = reader.u32()? as usize;
    let header_size = reader.u32()? as usize;
    let width = reader.i32()?;
    let raw_height = reader.i32()?;
    // The count of planes.
    reader.take(2)?;
    let bits = reader.u16()?;
    let compression = reader.u32()?;
    if compression != 0 {
        return Err(format!("compressed BMP (type {compression}) is not supported"));
    }
    if width <= 0 || raw_height == 0 {
        return Err("empty BMP".into());
    }
    // A positive height means the rows are stored bottom-up.
    let (height, bottom_up) = (raw_height.unsigned_abs() as usize, raw_height > 0);
    let width = width as usize;

    let bytes_per_pixel = match bits {
        8 => 1,
        24 => 3,
        32 => 4,
        _ => return Err(format!("{bits}-bit BMP is not supported")),
    };
    // The sizes are the file's own word: every count made of them is checked, a file that says more than
    // a `usize` holds (32 bits in the browser) being as broken as one cut short.
    let too_large = || "BMP too large".to_owned();
    let row = width.checked_mul(bytes_per_pixel).ok_or_else(too_large)?;
    let stride = row.checked_add(3).ok_or_else(too_large)? / 4 * 4;
    let needed = stride.checked_mul(height - 1).and_then(|rows| rows.checked_add(row)).and_then(|size| size.checked_add(pixels_at)).ok_or_else(too_large)?;
    if bytes.len() < needed {
        return Err("truncated BMP".into());
    }
    let palette_at = 14usize.checked_add(header_size).ok_or_else(too_large)?;
    if bits == 8 && pixels_at < palette_at.saturating_add(4) {
        return Err("BMP without a palette".into());
    }

    let mut rgba = vec![0u8; width.checked_mul(height).and_then(|pixels| pixels.checked_mul(4)).ok_or_else(too_large)?];
    for row in 0..height {
        let source = pixels_at + stride * if bottom_up { height - 1 - row } else { row };
        for column in 0..width {
            let [r, g, b] = match bits {
                8 => {
                    // Palette entries are BGRX. An index past the palette's end reads as black.
                    let entry = palette_at + usize::from(bytes[source + column]) * 4;
                    if entry + 2 < pixels_at { [bytes[entry + 2], bytes[entry + 1], bytes[entry]] } else { [0, 0, 0] }
                }
                _ => {
                    let at = source + column * bytes_per_pixel;
                    [bytes[at + 2], bytes[at + 1], bytes[at]]
                }
            };
            let alpha = if (r, g, b) == (255, 0, 255) { 0 } else { 255 };
            rgba[(row * width + column) * 4..][..4].copy_from_slice(&[r, g, b, alpha]);
        }
    }
    Ok(Image { width: width as u32, height: height as u32, rgba })
}

impl Image {
    /// The picture as the client keeps one read from a BMP on the GPU: 5 bits to a colour and one of alpha
    /// (`CTextureDX9::UploadSurface` for `D3DFMT_A1R5G5B5`, `Render/DX9/TextureDX9.cpp`, which
    /// `CTextureCache::Load` asks for whenever the picture has no alpha of its own). Each colour loses its
    /// three low bits; a pixel that is magenta then (31, 0, 31) is nothing at all, black and not seen, and so
    /// is one that is black then, if the picture is read with black as transparent (`black_unseen`: the
    /// pictures of an effect's `.str` file are, `g_texMgr.Load(path, 1, …)` in `EZeffectResource.cpp`).
    /// So the dim rim of a picture of light, under 8 of 255, adds nothing to what is behind it, and the rim of
    /// a keyed picture is dark, not pink, where the pixels are blended.
    pub fn as_the_client_keeps(&mut self, black_unseen: bool) {
        for pixel in self.rgba.chunks_exact_mut(4) {
            let five = [pixel[0] >> 3, pixel[1] >> 3, pixel[2] >> 3];
            if five == [31, 0, 31] || (black_unseen && five == [0, 0, 0]) {
                pixel.copy_from_slice(&[0, 0, 0, 0]);
            } else {
                // (Five bits shown as eight: the high bits again below, as a GPU widens them.)
                let wide = five.map(|colour| colour << 3 | colour >> 2);
                pixel.copy_from_slice(&[wide[0], wide[1], wide[2], 255]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_picture_is_kept_in_five_bits_a_colour() {
        let mut image = super::Image { width: 5, height: 1, rgba: vec![255, 255, 255, 255, 7, 7, 7, 255, 100, 8, 0, 255, 255, 0, 255, 0, 250, 5, 249, 255] };
        let mut plain = super::Image { width: 5, height: 1, rgba: image.rgba.clone() };
        image.as_the_client_keeps(true);
        // White stays white; what is under 8 is black and unseen; a colour loses its low bits; magenta, and what
        // is magenta in five bits, is nothing.
        assert_eq!(image.rgba, [255, 255, 255, 255, 0, 0, 0, 0, 99, 8, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0]);
        // Read without black as transparent, the black stays to be seen.
        plain.as_the_client_keeps(false);
        assert_eq!(plain.rgba[4..8], [0, 0, 0, 255]);
    }

    /// A 2x2 24-bit image, stored bottom-up with row padding: red, magenta / green, blue.
    #[test]
    fn decodes_24_bit_bottom_up() {
        let mut file = vec![0u8; 54];
        file[..2].copy_from_slice(b"BM");
        file[10..14].copy_from_slice(&54u32.to_le_bytes());
        file[14..18].copy_from_slice(&40u32.to_le_bytes());
        file[18..22].copy_from_slice(&2i32.to_le_bytes());
        file[22..26].copy_from_slice(&2i32.to_le_bytes());
        file[28..30].copy_from_slice(&24u16.to_le_bytes());
        // Bottom row first: green, blue (BGR), 2 padding bytes; then red, magenta.
        file.extend([0, 255, 0, 255, 0, 0, 0, 0]);
        file.extend([0, 0, 255, 255, 0, 255, 0, 0]);

        let image = super::decode(&file).unwrap();
        assert_eq!((image.width, image.height), (2, 2));
        assert_eq!(image.rgba, [255, 0, 0, 255, 255, 0, 255, 0, 0, 255, 0, 255, 0, 0, 255, 255]);
    }

    /// Sizes whose product passes a `usize` are an error, not an overflow.
    #[test]
    fn rejects_sizes_past_counting() {
        let mut file = vec![0u8; 54];
        file[..2].copy_from_slice(b"BM");
        file[10..14].copy_from_slice(&54u32.to_le_bytes());
        file[14..18].copy_from_slice(&40u32.to_le_bytes());
        file[18..22].copy_from_slice(&i32::MAX.to_le_bytes());
        file[22..26].copy_from_slice(&i32::MIN.to_le_bytes());
        file[28..30].copy_from_slice(&32u16.to_le_bytes());
        assert!(super::decode(&file).is_err());
        file[10..14].copy_from_slice(&u32::MAX.to_le_bytes());
        file[14..18].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(super::decode(&file).is_err());
    }

    #[test]
    fn rejects_other_files() {
        assert!(super::decode(b"not a bitmap").is_err());
    }
}
