//! `GET /image/<client path>`: a picture out of the GRF as a PNG, for the parts of the client drawn by the browser
//! itself (the windows). Browsers show neither TGA nor the client's transparent colour.

use std::io::Cursor;

use image::{ImageFormat, RgbaImage};

/// Decodes a BMP, TGA or JPEG of the client and encodes it as PNG. Magenta (255, 0, 255) is the client's
/// transparent colour in pictures that carry no alpha of their own.
pub fn to_png(path: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
    let lower = path.to_ascii_lowercase();
    let format = if lower.ends_with(".bmp") {
        ImageFormat::Bmp
    } else if lower.ends_with(".tga") {
        ImageFormat::Tga
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        ImageFormat::Jpeg
    } else {
        return Err("not a picture".to_owned());
    };
    let decoded = image::load_from_memory_with_format(bytes, format).map_err(|error| error.to_string())?;
    let has_alpha = decoded.color().has_alpha();
    let mut picture: RgbaImage = decoded.into_rgba8();
    if !has_alpha && format != ImageFormat::Jpeg {
        for pixel in picture.pixels_mut() {
            if pixel.0[..3] == [255, 0, 255] {
                pixel.0 = [0, 0, 0, 0];
            }
        }
    }
    let mut png = Vec::new();
    picture.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).map_err(|error| error.to_string())?;
    Ok(png)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2x1 24-bit BMP: magenta, then red.
    fn bmp() -> Vec<u8> {
        let mut file = vec![0u8; 54];
        file[..2].copy_from_slice(b"BM");
        file[10..14].copy_from_slice(&54u32.to_le_bytes());
        file[14..18].copy_from_slice(&40u32.to_le_bytes());
        file[18..22].copy_from_slice(&2i32.to_le_bytes());
        file[22..26].copy_from_slice(&1i32.to_le_bytes());
        file[26..28].copy_from_slice(&1u16.to_le_bytes());
        file[28..30].copy_from_slice(&24u16.to_le_bytes());
        file.extend([255, 0, 255, 0, 0, 255, 0, 0]);
        file
    }

    #[test]
    fn magenta_becomes_transparent() {
        let png = to_png("data\\texture\\x.BMP", &bmp()).unwrap();
        let picture = image::load_from_memory_with_format(&png, ImageFormat::Png).unwrap().into_rgba8();
        assert_eq!(picture.dimensions(), (2, 1));
        assert_eq!(picture.get_pixel(0, 0).0[3], 0);
        assert_eq!(picture.get_pixel(1, 0).0, [255, 0, 0, 255]);
    }

    #[test]
    fn refuses_what_is_not_a_picture() {
        assert!(to_png("data\\prontera.gat", b"GRAT").is_err());
        assert!(to_png("data\\x.bmp", b"not a bitmap").is_err());
    }
}
