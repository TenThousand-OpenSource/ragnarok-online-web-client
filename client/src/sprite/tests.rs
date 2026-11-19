use super::*;

#[test]
fn the_frames_whose_sounds_are_due() {
    // The frames passed since the last, the new one among them.
    assert_eq!(frames_sounding(Some((80, 1)), 80, 3, 6), 2..4);
    assert!(frames_sounding(Some((80, 3)), 80, 3, 6).is_empty());
    // An action just begun sounds from its second frame.
    assert_eq!((frames_sounding(None, 80, 2, 6), frames_sounding(Some((0, 2)), 80, 0, 6).is_empty()), (1..3, true));
    // Begun anew: what was left of it.
    assert_eq!(frames_sounding(Some((8, 5)), 8, 1, 8), 6..8);
    assert!(frames_sounding(Some((8, 9)), 8, 1, 8).is_empty());
}

/// A SPR 2.1 with one 3x2 indexed picture (`pixels`, run-length coded here) and a palette where colour N is
/// (N, N+1, N+2).
fn spr(pixels: &[u8; 6]) -> Vec<u8> {
    let mut coded = Vec::new();
    let mut index = 0;
    while index < pixels.len() {
        if pixels[index] == 0 {
            let run = pixels[index..].iter().take_while(|&&pixel| pixel == 0).count();
            coded.extend([0, run as u8]);
            index += run;
        } else {
            coded.push(pixels[index]);
            index += 1;
        }
    }
    let mut file = b"SP".to_vec();
    file.extend(0x201u16.to_le_bytes());
    file.extend(1u16.to_le_bytes());
    file.extend(0u16.to_le_bytes());
    file.extend(3u16.to_le_bytes());
    file.extend(2u16.to_le_bytes());
    file.extend((coded.len() as u16).to_le_bytes());
    file.extend(coded);
    for colour in 0..=255u8 {
        file.extend([colour, colour.wrapping_add(1), colour.wrapping_add(2), 0]);
    }
    file
}

/// An ACT 2.5 with one action of one frame of one layer, and an anchor.
fn act(x: i32, y: i32, mirrored: bool, anchor: (i32, i32)) -> Vec<u8> {
    let mut file = b"AC".to_vec();
    file.extend(0x205u16.to_le_bytes());
    file.extend(1u16.to_le_bytes());
    file.extend([0u8; 10]);
    file.extend(1i32.to_le_bytes());
    file.extend([0u8; 32]);
    file.extend(1i32.to_le_bytes());
    for value in [x, y, 0, i32::from(mirrored)] {
        file.extend(value.to_le_bytes());
    }
    file.extend([255u8; 4]);
    file.extend(1.0f32.to_le_bytes());
    file.extend(1.0f32.to_le_bytes());
    file.extend(0i32.to_le_bytes());
    file.extend(0i32.to_le_bytes());
    file.extend(3i32.to_le_bytes());
    file.extend(2i32.to_le_bytes());
    file.extend((-1i32).to_le_bytes());
    file.extend(1i32.to_le_bytes());
    file.extend([0u8; 4]);
    file.extend(anchor.0.to_le_bytes());
    file.extend(anchor.1.to_le_bytes());
    file.extend(0i32.to_le_bytes());
    file
}

#[test]
fn reads_a_run_length_coded_sprite() {
    let sprite = parse_spr(&spr(&[0, 0, 5, 7, 0, 0])).unwrap();
    assert_eq!(sprite.indexed.len(), 1);
    assert_eq!((sprite.indexed[0].width, sprite.indexed[0].height), (3, 2));
    assert_eq!(sprite.indexed[0].pixels, [0, 0, 5, 7, 0, 0]);
    assert_eq!(sprite.palette[5], [5, 6, 7, 255]);
    assert!(parse_spr(b"BM").is_err());
    assert!(parse_spr(&spr(&[1; 6])[..12]).is_err());
}

#[test]
fn reads_an_action_file() {
    let act = parse_act(&act(4, -9, true, (1, -30))).unwrap();
    assert_eq!(act.actions.len(), 1);
    let frame = &act.actions[0][0];
    assert_eq!(frame.anchor, Some((1, -30)));
    assert_eq!(frame.layers, [Layer { x: 4, y: -9, picture: 0, mirrored: true, colour: [255; 4], scale: (1.0, 1.0), rotation: 0, truecolor: false, size: Some((3, 2)) }]);
    // A layer's turn follows its two scales, 84 bytes into this file.
    let mut turned = self::act(4, -9, true, (1, -30));
    turned[84..88].copy_from_slice(&(-30i32).to_le_bytes());
    assert_eq!(parse_act(&turned).unwrap().actions[0][0].layers[0].rotation, -30);
    assert!(parse_act(b"SP").is_err());
}

/// A count that the file cannot hold is an error, not an allocation.
#[test]
fn refuses_impossible_counts() {
    let mut file = act(0, 0, false, (0, 0));
    file[16..20].copy_from_slice(&i32::MAX.to_le_bytes());
    assert!(parse_act(&file).is_err());
}

fn pixel(canvas: &Canvas, x: usize, y: usize) -> [u8; 4] {
    canvas.rgba[(y * canvas.width + x) * 4..][..4].try_into().unwrap()
}

#[test]
fn draws_a_layer_centred_on_its_point() {
    let sprite = parse_spr(&spr(&[1, 2, 3, 4, 0, 6])).unwrap();
    let frame = &parse_act(&act(0, 0, false, (0, 0))).unwrap().actions[0][0];
    let mut canvas = Canvas::new(8, 8);
    canvas.draw((4, 4), &sprite, None, frame);
    // 3x2 around (4, 4): columns 2-4, rows 3-4.
    assert_eq!(pixel(&canvas, 2, 3), [1, 2, 3, 255]);
    assert_eq!(pixel(&canvas, 4, 3), [3, 4, 5, 255]);
    assert_eq!(pixel(&canvas, 2, 4), [4, 5, 6, 255]);
    // Index 0 is transparent, and nothing is drawn outside the picture.
    assert_eq!(pixel(&canvas, 3, 4), [0, 0, 0, 0]);
    assert_eq!(pixel(&canvas, 5, 3), [0, 0, 0, 0]);
}

#[test]
fn mirrors_and_recolours() {
    let sprite = parse_spr(&spr(&[1, 2, 3, 4, 0, 6])).unwrap();
    let frame = &parse_act(&act(0, 0, true, (0, 0))).unwrap().actions[0][0];
    let mut other = [[0u8; 4]; 256];
    other[3] = [200, 100, 50, 255];
    other[1] = [9, 9, 9, 255];
    let mut canvas = Canvas::new(8, 8);
    canvas.draw((4, 4), &sprite, Some(&other), frame);
    assert_eq!(pixel(&canvas, 2, 3), [200, 100, 50, 255]);
    assert_eq!(pixel(&canvas, 4, 3), [9, 9, 9, 255]);
}

/// Parts of a picture that fall outside the canvas are cut off, on every side.
#[test]
fn clips_at_the_edges() {
    let sprite = parse_spr(&spr(&[1, 2, 3, 4, 5, 6])).unwrap();
    let frame = &parse_act(&act(0, 0, false, (0, 0))).unwrap().actions[0][0];
    for origin in [(0, 0), (7, 7), (-50, 3), (3, 500)] {
        let mut canvas = Canvas::new(8, 8);
        canvas.draw(origin, &sprite, None, frame);
    }
    let mut canvas = Canvas::new(8, 8);
    canvas.draw((0, 0), &sprite, None, frame);
    assert_eq!(pixel(&canvas, 0, 0), [6, 7, 8, 255]);
}

#[test]
fn a_head_follows_the_bodys_anchor() {
    let body = Frame { layers: Vec::new(), sound: None, anchor: Some((2, -40)) };
    let head = Frame { layers: Vec::new(), sound: None, anchor: Some((-1, -10)) };
    assert_eq!(head_offset(&body, &head), (3, -30));
    assert_eq!(head_offset(&body, &Frame::default()), (0, 0));
}
