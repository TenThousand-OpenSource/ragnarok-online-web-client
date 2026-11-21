//! The packing of a Granny texture's pixels (texture encoding 3, "Bink"): each of three or four planes — a
//! brightness, two colour differences and, if the texture has one, an alpha — is taken apart four times by a
//! wavelet and its numbers are packed with an arithmetic coder.
//!
//! No open description of it was found; it was read out of the client's own `granny2.dll` (2.1.0.5): the
//! function behind `GrannyCopyTextureImage` for encoding 3 (`0x10009e50`), which rounds the sizes up to
//! sixteens (`0x10009c30`), reads the planes (`0x100045b0`: `bink/bands.rs`, then `bink/wavelet.rs`) and
//! turns them into pixels (`0x10009a30`). A texture of no more than 256 pixels is not packed at all.

mod bands;
mod coder;
mod wavelet;

#[cfg(test)]
mod tests;

/// The largest side read here; the client's textures are 256.
const LARGEST: usize = 4096;

/// A texture's pixels unpacked: `width` × `height` of red, green, blue, alpha, a byte each, rows from the top.
/// `alpha` is whether the texture's layout has bits for an alpha (the DLL's `GrannyTextureHasAlpha`): with
/// it a fourth plane is packed, without it every pixel is opaque.
pub(super) fn unpack(packed: &[u8], width: usize, height: usize, alpha: bool) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 || width > LARGEST || height > LARGEST {
        return Err("a texture of a size that cannot be".to_owned());
    }
    if width * height <= 0x100 {
        return plain(packed, width * height, alpha);
    }
    // The planes are whole sixteens wide and high; the picture is their top left corner.
    let (wide, high) = (width.div_ceil(16) * 16, height.div_ceil(16) * 16);
    // The first word is how much room the DLL's reader wants for itself.
    let mut rest = packed.get(4..).ok_or("a packed texture with nothing in it")?;
    let mut planes = Vec::new();
    for _ in 0..if alpha { 4 } else { 3 } {
        let (mut plane, taken) = bands::plane(rest, wide, high)?;
        for step in [8, 4, 2, 1] {
            wavelet::join(&mut plane, wide, step, wide / step, high / step);
        }
        planes.push(plane);
        rest = &rest[taken..];
    }
    let mut pixels = Vec::with_capacity(width * height * 4);
    for row in 0..height {
        for at in row * wide..row * wide + width {
            pixels.extend(pixel(planes[0][at], planes[1][at], planes[2][at], planes.get(3).map(|plane| plane[at])));
        }
    }
    Ok(pixels)
}

/// A small texture's pixels as they lie: red, green, blue and, with an alpha, that.
fn plain(pixels: &[u8], count: usize, alpha: bool) -> Result<Vec<u8>, String> {
    let each = if alpha { 4 } else { 3 };
    let pixels = pixels.get(..count * each).ok_or("a small texture with too few pixels")?;
    Ok(pixels.chunks(each).flat_map(|pixel| [pixel[0], pixel[1], pixel[2], if alpha { pixel[3] } else { 255 }]).collect())
}

/// A pixel of its planes: green is the brightness less a quarter of the two differences, red and blue are
/// their differences added to green; each kept within a byte.
fn pixel(brightness: i16, red: i16, blue: i16, alpha: Option<i16>) -> [u8; 4] {
    let (red, blue) = (red as i32, blue as i32);
    let green = brightness as i32 - (red + blue) / 4;
    let byte = |number: i32| number.clamp(0, 255) as u8;
    [byte(red + green), byte(green), byte(blue + green), alpha.map_or(255, |alpha| byte(alpha as i32))]
}
