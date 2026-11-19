use super::*;

fn key(frame: i32, kind: u32, x: f32) -> Key {
    Key { frame, kind, transform: Transform { x, ..Transform::default() } }
}

fn file(layers: &[(&[&str], &[Key])]) -> Vec<u8> {
    let mut bytes = b"STRM".to_vec();
    for value in [0x94u32, 60, 10, layers.len() as u32] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend([0; 16]);
    for (textures, keys) in layers {
        bytes.extend((textures.len() as u32).to_le_bytes());
        for texture in *textures {
            let mut name = texture.as_bytes().to_vec();
            name.resize(128, 0);
            bytes.extend(name);
        }
        bytes.extend((keys.len() as u32).to_le_bytes());
        for key in *keys {
            bytes.extend(key.frame.to_le_bytes());
            bytes.extend(key.kind.to_le_bytes());
            let transform = &key.transform;
            for float in [transform.x, transform.y, transform.u, transform.v, transform.us, transform.vs, 0.0, 0.0, 0.0, 0.0] {
                bytes.extend(float.to_le_bytes());
            }
            for float in transform.ax.iter().chain(&transform.ay) {
                bytes.extend(float.to_le_bytes());
            }
            bytes.extend(transform.aniframe.to_le_bytes());
            bytes.extend(transform.anitype.to_le_bytes());
            bytes.extend(transform.anidelta.to_le_bytes());
            bytes.extend(transform.rz.to_le_bytes());
            for float in transform.colour {
                bytes.extend(float.to_le_bytes());
            }
            for value in [transform.source_blend, transform.destination_blend, 0] {
                bytes.extend(value.to_le_bytes());
            }
        }
    }
    bytes
}

#[test]
fn reads_layers_textures_and_keys() {
    let keys = [key(0, 0, 320.0), key(0, 1, 2.0), key(5, 0, 400.0)];
    let effect = parse(&file(&[(&["a.bmp", "b.bmp"], &keys)]), |raw| String::from_utf8_lossy(raw).into_owned()).unwrap();
    assert_eq!((effect.frames_per_second, effect.last_frame, effect.layers.len()), (60, 10, 1));
    assert_eq!(effect.layers[0].textures, ["a.bmp", "b.bmp"]);
    assert_eq!(effect.layers[0].keys, keys);
    assert!(parse(b"nope", |_| String::new()).is_err());
}

/// A key that sets, a key that adds from the same frame on, and the key that ends the layer.
#[test]
fn a_layer_is_set_then_added_to_frame_by_frame_then_gone() {
    let layer = Layer { textures: vec![], keys: vec![key(2, 0, 100.0), key(2, 1, 5.0), key(6, 0, 0.0)] };
    assert_eq!(layer.at(1), None);
    assert_eq!(layer.at(2).map(|transform| transform.x), Some(100.0));
    assert_eq!(layer.at(3).map(|transform| transform.x), Some(105.0));
    assert_eq!(layer.at(5).map(|transform| transform.x), Some(115.0));
    // On the last key's frame the layer still shows what it had; after it, nothing.
    assert!(layer.at(6).is_some());
    assert_eq!(layer.at(7), None);
}

#[test]
fn a_picture_is_in_the_folder_of_its_file() {
    assert_eq!(texture_path("lightning.str", "lightning.bmp"), "lightning.bmp");
    assert_eq!(texture_path("stormcannon/stormcannon/stormcannon.str", "a.tga"), "stormcannon/stormcannon/a.tga");
    assert_eq!(texture_path("new_cart_cannon\\new_cart_cannon.str", "a.tga"), "new_cart_cannon\\a.tga");
}

#[test]
fn corners_are_around_the_middle_and_turn_with_it() {
    let flat = Transform { x: 320.0, y: 240.0, us: 1.0, vs: 1.0, ax: [-10.0, 10.0, 10.0, -10.0], ay: [-5.0, -5.0, 5.0, 5.0], ..Transform::default() };
    assert_eq!(flat.corners()[0], ([-10.0, -5.0], [0.0, 0.0]));
    assert_eq!(flat.corners()[2], ([10.0, 5.0], [1.0, 1.0]));
    // A quarter turn (256 of 1024) sends the top left corner to the top right.
    let turned = Transform { rz: 256.0, ..flat };
    let [x, y] = turned.corners()[0].0;
    assert!((x - 5.0).abs() < 1e-3 && (y + 10.0).abs() < 1e-3, "{x} {y}");
}
