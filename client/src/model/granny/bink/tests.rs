use super::*;

#[test]
fn a_small_texture_is_not_packed() {
    let pixels: Vec<u8> = (0..16).collect();
    assert_eq!(unpack(&pixels, 2, 2, true), Ok(pixels.clone()));
    assert_eq!(unpack(&pixels[..12], 2, 2, false), Ok(vec![0, 1, 2, 255, 3, 4, 5, 255, 6, 7, 8, 255, 9, 10, 11, 255]));
    assert!(unpack(&pixels[..11], 2, 2, false).is_err());
}

#[test]
fn what_is_not_a_texture_is_refused() {
    assert!(unpack(&[], 32, 32, false).is_err());
    assert!(unpack(&[0; 64], 32, 32, true).is_err());
    assert!(unpack(&[0xff; 4096], 256, 256, true).is_err());
    assert!(unpack(&[0; 16], 0, 4, true).is_err());
}

#[test]
fn a_pixel_comes_of_its_planes() {
    assert_eq!(pixel(100, 0, 0, None), [100, 100, 100, 255]);
    // Green is the brightness less a quarter of the differences, cut toward nothing.
    assert_eq!(pixel(100, 40, -20, Some(7)), [135, 95, 75, 7]);
    assert_eq!(pixel(100, -7, 0, Some(300)), [94, 101, 101, 255]);
    assert_eq!(pixel(-5, 0, 300, Some(-1)), [0, 0, 220, 0]);
}

#[test]
fn a_short_line_is_sums_and_differences() {
    let mut out = [0; 4];
    wavelet::one(&[10, -3], &[4, 0], &mut out, false);
    assert_eq!(out, [12, 8, -4, -4]);
}

#[test]
fn a_long_line_keeps_what_is_even_and_shows_a_step() {
    // A low half all alike with nothing in the high half: every number alike, smaller by the root of two.
    let mut out = [0; 16];
    wavelet::one(&[1000; 8], &[0; 8], &mut out, true);
    assert_eq!(out, [707; 16]);
    wavelet::one(&[-1000; 8], &[0; 8], &mut out, true);
    assert_eq!(out, [-707; 16]);
    // One number in the high half shows around its own place, the odd one.
    wavelet::one(&[0; 8], &[0, 0, 0, 1000, 0, 0, 0, 0], &mut out, true);
    assert_eq!(out[2..13], [0, -38, -24, 111, 377, -853, 377, 111, -24, -38, 0]);
}

#[test]
fn a_plane_is_joined_rows_then_columns() {
    // 16 × 16 at full size: the long way both ways; only the smallest picture's number is there.
    let mut plane = vec![0i16; 256];
    for row in (0..16).step_by(2) {
        plane[row * 16..row * 16 + 8].fill(2000);
    }
    wavelet::join(&mut plane, 16, 1, 16, 16);
    assert!(plane.iter().all(|&number| number == 1000), "{plane:?}");
}

/// The client's own models, from a folder they were saved into: `ROWEB_GR2=<folder>`; nothing without it.
/// With `ROWEB_GR2_OUT=<folder>` each texture is written there as a picture (PPM) to be looked at.
#[test]
fn the_clients_textures_unpack() {
    use crate::model::granny::{file, types};
    let Ok(folder) = std::env::var("ROWEB_GR2") else { return };
    for name in ["guildflag90_1", "empelium90_0", "aguardian90_8", "kguardian90_7", "sguardian90_9", "treasurebox_2"] {
        let Ok(bytes) = std::fs::read(std::path::Path::new(&folder).join(format!("{name}.gr2"))) else { return };
        let image = file::Image::read(&bytes).unwrap();
        let root = types::Object { image: &image, kind: image.root_type, at: image.root };
        for (index, texture) in root.objects("Textures").unwrap().into_iter().enumerate() {
            let (width, height) = (texture.number("Width").unwrap() as usize, texture.number("Height").unwrap() as usize);
            assert_eq!(texture.number("Encoding"), Ok(3.0));
            let alpha = texture.object("Layout").unwrap().unwrap().numbers("BitsForComponent").unwrap()[3] != 0.0;
            let level = texture.objects("Images").unwrap()[0].objects("MIPLevels").unwrap()[0];
            let packed: Vec<u8> = level.numbers("Pixels").unwrap().into_iter().map(|byte| byte as u8).collect();
            let pixels = unpack(&packed, width, height, alpha).unwrap_or_else(|error| panic!("{name} {index}: {error}"));
            assert_eq!(pixels.len(), width * height * 4);
            // A picture, not noise: neighbours differ little on the whole.
            let rough: usize = pixels.chunks(4).zip(pixels.chunks(4).skip(1)).map(|(a, b)| a[1].abs_diff(b[1]) as usize).sum();
            assert!(width * height <= 0x100 || rough / (width * height) < 24, "{name} {index}: rough {}", rough / (width * height));
            if let Ok(out) = std::env::var("ROWEB_GR2_OUT") {
                let mut picture = format!("P6\n{width} {height}\n255\n").into_bytes();
                picture.extend(pixels.chunks(4).flat_map(|pixel| [pixel[0], pixel[1], pixel[2]]));
                std::fs::write(std::path::Path::new(&out).join(format!("{name}_{index}.ppm")), picture).unwrap();
                let mut mask = format!("P5\n{width} {height}\n255\n").into_bytes();
                mask.extend(pixels.chunks(4).map(|pixel| pixel[3]));
                std::fs::write(std::path::Path::new(&out).join(format!("{name}_{index}_alpha.pgm")), mask).unwrap();
            }
        }
    }
}
