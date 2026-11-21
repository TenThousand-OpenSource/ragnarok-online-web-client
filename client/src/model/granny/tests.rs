use super::*;

/// The client's own models, from a folder they were saved into (they are in the client's archive, not in
/// the repository): `ROWEB_GR2=<folder>`; nothing without it.
fn clients_file(name: &str) -> Option<Vec<u8>> {
    std::fs::read(std::path::Path::new(&std::env::var("ROWEB_GR2").ok()?).join(name)).ok()
}

#[test]
fn the_clients_files_unpack() {
    for name in ["guildflag90_1.gr2", "empelium90_0.gr2", "aguardian90_8.gr2", "kguardian90_7.gr2", "sguardian90_9.gr2", "treasurebox_2.gr2"] {
        let Some(bytes) = clients_file(name) else { return };
        let image = file::Image::read(&bytes).unwrap_or_else(|error| panic!("{name}: {error}"));
        eprintln!("{name}: {} bytes, {} pointers, root {} type {}", image.bytes.len(), image.pointers.len(), image.root, image.root_type);
        if let Ok(folder) = std::env::var("ROWEB_GR2_OUT") {
            let mut text = String::new();
            written(types::Object { image: &image, kind: image.root_type, at: image.root }, 0, &mut text);
            std::fs::write(std::path::Path::new(&folder).join(format!("{name}.txt")), text).unwrap();
        }
    }
}

/// Everything a file holds, written out by its own description (to look at a file: `ROWEB_GR2_OUT`).
fn written(object: types::Object, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    for member in types::members(object.image, object.kind).unwrap() {
        let field = object.read(&member).unwrap_or_else(|error| panic!("{}: {error}", member.name));
        match field {
            types::Field::Numbers(numbers) => out.push_str(&format!("{pad}{} = {numbers:?}\n", member.name)),
            types::Field::Text(text) => out.push_str(&format!("{pad}{} = {text:?}\n", member.name)),
            types::Field::Transform(transform) => out.push_str(&format!("{pad}{} = {transform:?}\n", member.name)),
            types::Field::Nothing => out.push_str(&format!("{pad}{} = nothing\n", member.name)),
            types::Field::Object(inner) if depth < 12 => {
                out.push_str(&format!("{pad}{} (kind {}) @{}:\n", member.name, member.kind, inner.at));
                written(inner, depth + 1, out);
            }
            types::Field::Object(_) => {}
            types::Field::Array(array) => {
                let stride = array.stride().unwrap();
                out.push_str(&format!("{pad}{} [{} x {stride} bytes] @{}:\n", member.name, array.count, array.at));
                match array.numbers() {
                    Ok(numbers) => out.push_str(&format!("{pad}  {:?}\n", &numbers[..numbers.len().min(24)])),
                    Err(_) => {
                        for inner in array.objects().unwrap().into_iter().take(if depth < 4 { 80 } else { 3 }) {
                            out.push_str(&format!("{pad}  -\n"));
                            written(inner, depth + 2, out);
                        }
                    }
                }
            }
            types::Field::Objects(objects) => {
                out.push_str(&format!("{pad}{} [{} objects]:\n", member.name, objects.len()));
                for inner in objects.into_iter().take(80) {
                    out.push_str(&format!("{pad}  - @{}\n", inner.at));
                    if depth < 12 {
                        written(inner, depth + 2, out);
                    }
                }
            }
        }
    }
}

#[test]
fn the_clients_models_are_read_and_stand_as_bound() {
    for name in ["guildflag90_1.gr2", "empelium90_0.gr2", "aguardian90_8.gr2", "kguardian90_7.gr2", "sguardian90_9.gr2", "treasurebox_2.gr2"] {
        let Some(bytes) = clients_file(name) else { return };
        let model = Granny::read(&bytes).unwrap_or_else(|error| panic!("{name}: {error}"));
        let animation = model.animation.as_ref().unwrap();
        eprintln!("{name}: {} textures, {} bones, {} s, meshes:", model.textures.len(), model.bones.len(), animation.duration);
        assert!(model.textures.iter().all(|texture| texture.pixels.len() == texture.width * texture.height * 4));
        // Resting, every bone of the flag is where it was bound: the skin does not move. (A guardian's
        // skeleton rests in another stance than it was bound in.)
        let rest = model.pose(None);
        for mesh in &model.meshes {
            eprintln!("  {} {} corners {} triangles texture {:?} rigid {}", mesh.name, mesh.vertices.len(), mesh.indices.len() / 3, mesh.texture, mesh.rigid);
            for (vertex, (position, _)) in mesh.vertices.iter().zip(model.skin(mesh, &rest)) {
                let off = crate::vector::length(crate::vector::subtract(vertex.position, position));
                assert!(off < if name.starts_with("guildflag") { 0.05 } else { 60.0 }, "{name} {}: {off} off at rest", mesh.name);
            }
        }
        // Moving, it stays near: no corner flies off, and the animation does move something.
        let mut moved = 0.0f32;
        for step in 0..8 {
            let pose = model.pose(Some((animation, animation.duration * step as f32 / 8.0)));
            for mesh in &model.meshes {
                for (vertex, (position, normal)) in mesh.vertices.iter().zip(model.skin(mesh, &pose)) {
                    let off = crate::vector::length(crate::vector::subtract(vertex.position, position));
                    assert!(off < 60.0 && position.iter().chain(&normal).all(|part| part.is_finite()), "{name} {}: {off} off", mesh.name);
                    moved = moved.max(off);
                }
            }
        }
        eprintln!("  moves up to {moved}");
    }
}

/// The smallest packed section of the client's files (192 bytes of `aguardian90_8.gr2`, its third: 672
/// bytes), kept here so that the unpacking is tried without the files.
const SECTION: &[u8] = include_bytes!("section.bin");

#[test]
fn a_packed_section_unpacks() {
    let unpacked = oodle::unpack(SECTION, [672; 3]).unwrap();
    assert_eq!(unpacked.len(), 672);
    // The sum of its bytes and a hash of them (FNV-1a), as the whole file read with them made sense.
    let hash = unpacked.iter().fold(0x811c_9dc5u32, |hash, &byte| (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193));
    assert_eq!((unpacked.iter().map(|&byte| u32::from(byte)).sum::<u32>(), hash), SECTION_SUMS);
    // Cut short or made nonsense it is refused, not read past its end.
    assert!(oodle::unpack(&SECTION[..20], [672; 3]).is_err());
    let _ = oodle::unpack(&SECTION[..100], [672; 3]);
    let nonsense: Vec<u8> = SECTION.iter().map(|byte| byte.wrapping_mul(31)).collect();
    let _ = oodle::unpack(&nonsense, [672; 3]);
}

const SECTION_SUMS: (u32, u32) = (3182, 625_958_629);

#[test]
fn what_is_not_a_granny_file_is_refused() {
    assert!(Granny::read(b"GRSM\x01\x04").is_err());
    assert!(Granny::read(&[]).is_err());
    assert!(is_model("Guildflag90_1.gr2") && is_model("TREASUREBOX_2.GR2") && !is_model("poring"));
}

/// An image made by hand: a type of a number, a text, three reals and an array of two bytes, and an object
/// of it.
fn handmade() -> file::Image {
    let mut bytes = Vec::new();
    let mut pointers = std::collections::HashMap::new();
    let member = |bytes: &mut Vec<u8>, kind: u32, width: u32| {
        bytes.extend(kind.to_le_bytes());
        bytes.extend([0; 8]);
        bytes.extend(width.to_le_bytes());
        bytes.extend([0; 16]);
    };
    // The type of a byte, at 0; the object's type, at 64.
    member(&mut bytes, 12, 0);
    member(&mut bytes, 0, 0);
    for (kind, width) in [(19, 0), (8, 0), (10, 3), (3, 0), (0, 0)] {
        member(&mut bytes, kind, width);
    }
    let names = bytes.len();
    bytes.extend(b"Count\0Name\0Place\0Bytes\0Byte\0flag\0");
    for (index, name) in [(2, 0), (3, 6), (4, 11), (5, 17), (0, 23)] {
        pointers.insert(index * 32 + 4, names + name);
    }
    pointers.insert(5 * 32 + 8, 0);
    let object = bytes.len();
    bytes.extend(7i32.to_le_bytes());
    pointers.insert(bytes.len(), names + 28);
    bytes.extend([0; 4]);
    bytes.extend([1.0f32, 2.0, 3.0].iter().flat_map(|real| real.to_le_bytes()));
    bytes.extend(2u32.to_le_bytes());
    pointers.insert(bytes.len(), bytes.len() + 4);
    bytes.extend([0; 4]);
    bytes.extend([200, 100]);
    file::Image { bytes, pointers, root: object, root_type: 64 }
}

#[test]
fn an_object_is_read_by_the_names_of_its_type() {
    let image = handmade();
    let object = types::Object { image: &image, kind: image.root_type, at: image.root };
    assert_eq!(types::size(&image, image.root_type), Ok(28));
    assert_eq!((object.number("Count"), object.text("Name")), (Ok(7.0), Ok("flag".to_owned())));
    assert_eq!((object.numbers("Place"), object.numbers("Bytes")), (Ok(vec![1.0, 2.0, 3.0]), Ok(vec![200.0, 100.0])));
    // A member the type has not is nothing, not an error.
    assert_eq!((object.numbers("Other"), object.objects("Other").map(|objects| objects.len())), (Ok(Vec::new()), Ok(0)));
}

#[test]
fn a_curve_goes_from_control_to_control() {
    let line = Curve { degree: 1, knots: vec![0.0, 1.0, 2.0], controls: vec![0.0, 10.0, 30.0], width: 1 };
    assert_eq!((line.at(0.0), line.at(0.5), line.at(1.5), line.at(9.0)), (Some(vec![0.0]), Some(vec![5.0]), Some(vec![20.0]), Some(vec![30.0])));
    // One value, and nothing.
    let still = Curve { degree: 0, knots: vec![0.0], controls: vec![1.0, 2.0, 3.0], width: 3 };
    assert_eq!((still.at(4.0), Curve::default().at(1.0)), (Some(vec![1.0, 2.0, 3.0]), None));
    // A curve of degree 2 stays between its controls and ends on the last.
    let bent = Curve { degree: 2, knots: vec![0.0, 1.0, 2.0, 3.0], controls: vec![0.0, 4.0, 4.0, 0.0], width: 1 };
    for step in 0..30 {
        let value = bent.at(step as f32 / 10.0).unwrap()[0];
        assert!((0.0..=4.0).contains(&value), "{value}");
    }
    assert!(bent.at(1.5).unwrap()[0] > 2.0);
}

#[test]
fn a_skin_goes_with_its_bones() {
    let bone = |name: &str, parent: Option<usize>, position: [f32; 3], from: [f32; 3]| Bone {
        name: name.to_owned(),
        parent,
        rest: Transform { position, ..Transform::IDENTITY },
        from_world: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, from[0], from[1], from[2], 1.0],
    };
    let vertex = |position: [f32; 3], weights: [u8; 4]| Vertex { position, normal: [0.0, 0.0, 1.0], uv: [0.0; 2], weights, bones: [0, 1, 0, 0] };
    // A root and a bone 10 above it; a corner on the root, one on the bone, one half on each.
    let mesh = Mesh {
        name: String::new(),
        vertices: vec![vertex([0.0; 3], [255, 0, 0, 0]), vertex([0.0, 0.0, 10.0], [0, 255, 0, 0]), vertex([0.0, 0.0, 5.0], [128, 127, 0, 0])],
        indices: vec![0, 1, 2],
        texture: None,
        bones: vec![0, 1],
        rigid: false,
    };
    // The bone's track turns it a quarter round the model's x and holds it 10 up.
    let turn = std::f32::consts::FRAC_PI_4;
    let track = Track {
        bone: "tip".to_owned(),
        position: Curve { degree: 0, knots: vec![0.0], controls: vec![0.0, 0.0, 10.0], width: 3 },
        orientation: Curve { degree: 0, knots: vec![0.0], controls: vec![turn.sin(), 0.0, 0.0, turn.cos()], width: 4 },
        scale: Curve::default(),
    };
    let animation = Animation { duration: 2.0, tracks: vec![track] };
    let model = Granny {
        textures: Vec::new(),
        bones: vec![bone("root", None, [0.0; 3], [0.0; 3]), bone("tip", Some(0), [0.0, 0.0, 10.0], [0.0, 0.0, -10.0])],
        meshes: vec![mesh],
        animation: None,
    };
    let near = |a: [f32; 3], b: [f32; 3]| (0..3).all(|axis| (a[axis] - b[axis]).abs() < 1e-3);
    // Resting, nothing moves.
    let rest = model.skin(&model.meshes[0], &model.pose(None));
    assert!(near(rest[0].0, [0.0; 3]) && near(rest[1].0, [0.0, 0.0, 10.0]) && near(rest[2].0, [0.0, 0.0, 5.0]));
    // Turned: the corner on the bone stays at the bone's joint and its normal turns; the one half on
    // each goes half the way (5 under the joint, turned, is 5 beside it).
    let moved = model.skin(&model.meshes[0], &model.pose(Some((&animation, 1.0))));
    assert!(near(moved[0].0, [0.0; 3]) && near(moved[1].0, [0.0, 0.0, 10.0]) && near(moved[1].1, [0.0, -1.0, 0.0]));
    assert!(near(moved[2].0, [0.0, 127.0 / 255.0 * 5.0, 128.0 / 255.0 * 5.0 + 127.0 / 255.0 * 10.0]), "{:?}", moved[2].0);
    // An animation goes round, or ends.
    assert_eq!((animation.time(5.0, true), animation.time(5.0, false)), (1.0, 2.0));
}

#[test]
fn the_clients_other_actions_are_read() {
    for name in ["7_move.gr2", "7_attack.gr2", "8_dead.gr2", "9_damage.gr2", "2_dead.gr2", "1_attack.gr2"] {
        let Some(bytes) = clients_file(&format!("bone/{name}")) else { return };
        let read = Granny::read(&bytes).unwrap_or_else(|error| panic!("{name}: {error}"));
        let animation = read.animation.unwrap_or_else(|| panic!("{name}: no animation"));
        eprintln!("{name}: {} s, {} tracks", animation.duration, animation.tracks.len());
        assert!(animation.duration > 0.0 && !animation.tracks.is_empty());
    }
}

/// Every action of every model of the client's, at every moment: no corner leaves the model's
/// neighbourhood and none is not a number (a corner flung far off is a triangle across the whole screen).
#[test]
fn no_action_flings_a_corner_away() {
    for (name, number) in [("guildflag90_1", 1), ("empelium90_0", 0), ("aguardian90_8", 8), ("kguardian90_7", 7), ("sguardian90_9", 9), ("treasurebox_2", 2)] {
        let Some(bytes) = clients_file(&format!("{name}.gr2")) else { return };
        let model = Granny::read(&bytes).unwrap();
        let mut animations = vec![("stand".to_owned(), model.animation.clone().unwrap())];
        for action in ["move", "attack", "dead", "damage"] {
            if let Some(read) = clients_file(&format!("bone/{number}_{action}.gr2")).and_then(|bytes| Granny::read(&bytes).ok()) {
                animations.extend(read.animation.map(|animation| (action.to_owned(), animation)));
            }
        }
        for (action, animation) in &animations {
            let mut farthest = 0.0f32;
            for step in 0..=200 {
                let time = animation.duration * 1.2 * step as f32 / 200.0;
                let pose = model.pose(Some((animation, time)));
                for mesh in &model.meshes {
                    for (position, normal) in model.skin(mesh, &pose) {
                        assert!(position.iter().chain(&normal).all(|part| part.is_finite()), "{name} {action} at {time}: not a number");
                        farthest = farthest.max(crate::vector::length(position));
                    }
                }
            }
            eprintln!("{name} {action}: {} s, {} tracks, farthest {farthest}", animation.duration, animation.tracks.len());
            assert!(farthest < 150.0, "{name} {action}: a corner {farthest} away");
        }
    }
}

/// The flag's second mesh, which carries the guild's emblem, against its cloth: a board a little before
/// the cloth that reaches from left of its middle to its right edge. The emblem itself lies in the board's
/// upper left three quarters (`shown::emblem`: 24 pixels in a texture of 32), and that part is in the
/// middle of the cloth, under its top.
#[test]
fn the_flags_emblem_lies_in_the_middle_of_its_cloth() {
    let Some(bytes) = clients_file("guildflag90_1.gr2") else { return };
    let model = Granny::read(&bytes).unwrap();
    let span = |points: &mut dyn Iterator<Item = [f32; 3]>, axis: usize| {
        points.fold((f32::MAX, f32::MIN), |(low, high), point| (low.min(point[axis]), high.max(point[axis])))
    };
    let (cloth, board) = (&model.meshes[0], &model.meshes[1]);
    // The cloth is what hangs on the bones of the "tail".
    let hangs = |vertex: &&Vertex| model.bones[cloth.bones[vertex.bones[0] as usize]].name.contains("Tail");
    let hanging = || cloth.vertices.iter().filter(hangs).map(|vertex| vertex.position);
    let boarded = || board.vertices.iter().map(|vertex| vertex.position);
    let (cloth_x, cloth_y, cloth_z) = (span(&mut hanging(), 0), span(&mut hanging(), 1), span(&mut hanging(), 2));
    let (board_x, board_y, board_z) = (span(&mut boarded(), 0), span(&mut boarded(), 1), span(&mut boarded(), 2));
    // Before the cloth (the model's front is its -y), by less than one unit, and flat.
    assert!(board_y.1 < cloth_y.0 && cloth_y.0 - board_y.0 < 1.0 && board_y.1 - board_y.0 < 0.2, "{board_y:?} {cloth_y:?}");
    // The whole board is not in the middle: it ends at the cloth's right edge.
    assert!((board_x.1 - cloth_x.1).abs() < 0.1 && board_x.0 > cloth_x.0 + 2.5, "{board_x:?} {cloth_x:?}");
    // Its picture runs from 0 to 1 across and down, the top of the picture at the top.
    let top_left = board.vertices.iter().find(|vertex| vertex.uv[0] < 0.01 && vertex.uv[1] < 0.01).unwrap();
    assert!((top_left.position[0] - board_x.0).abs() < 0.01 && (top_left.position[2] - board_z.1).abs() < 0.1);
    // The emblem's three quarters: in the cloth's middle, to a third of a unit, and within the cloth.
    let share = 24.0 / 32.0;
    let (emblem_right, emblem_low) = (board_x.0 + (board_x.1 - board_x.0) * share, board_z.1 - (board_z.1 - board_z.0) * share);
    assert!(((board_x.0 + emblem_right) / 2.0 - (cloth_x.0 + cloth_x.1) / 2.0).abs() < 0.35, "{board_x:?} {emblem_right} {cloth_x:?}");
    assert!(board_x.0 > cloth_x.0 && emblem_right < cloth_x.1 && emblem_low > cloth_z.0 && board_z.1 < cloth_z.1 + 0.1);
}
