use super::*;

fn close(a: Vector, b: Vector) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() < 1e-4)
}

#[test]
fn matrices_move_turn_and_stretch() {
    let moved = Matrix::translation([1.0, 2.0, 3.0]);
    assert!(close(moved.point([1.0, 1.0, 1.0]), [2.0, 3.0, 4.0]));
    let stretched = Matrix::scale([2.0, 3.0, 4.0]);
    assert!(close(stretched.point([1.0, 1.0, 1.0]), [2.0, 3.0, 4.0]));
    // A quarter turn around y takes x to -z.
    let turned = Matrix::rotation([0.0, 1.0, 0.0], std::f32::consts::FRAC_PI_2);
    assert!(close(turned.point([1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]));
    // The same turn as a quaternion.
    let half = std::f32::consts::FRAC_PI_4;
    assert!(close(Matrix::quaternion([0.0, half.sin(), 0.0, half.cos()]).point([1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]));
    // Stretched first, then moved.
    assert!(close(moved.after(&stretched).point([1.0, 1.0, 1.0]), [3.0, 5.0, 7.0]));
    assert!(close(stretched.after(&moved).point([1.0, 1.0, 1.0]), [4.0, 9.0, 16.0]));
    // No axis, no turn.
    assert_eq!(Matrix::rotation([0.0; 3], 1.0), Matrix::IDENTITY);
}

fn padded(name: &str, size: usize) -> Vec<u8> {
    let mut field = name.as_bytes().to_vec();
    field.resize(size, 0);
    field
}

fn floats(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

/// A RSW 2.1 with water, a sun, a light, a sound and one model.
fn rsw_bytes() -> Vec<u8> {
    let mut file = b"GRSW\x02\x01".to_vec();
    file.extend(padded("", 160));
    file.extend(floats(&[-12.5]));
    file.extend(4i32.to_le_bytes());
    file.extend(floats(&[1.0, 2.0, 50.0]));
    file.extend(5i32.to_le_bytes());
    file.extend(45i32.to_le_bytes());
    file.extend(60i32.to_le_bytes());
    file.extend(floats(&[1.0, 0.9, 0.8, 0.3, 0.3, 0.4, 1.0]));
    file.extend([0u8; 16]);
    file.extend(3i32.to_le_bytes());
    file.extend(2i32.to_le_bytes());
    file.extend(vec![0u8; 108]);
    file.extend(3i32.to_le_bytes());
    file.extend(vec![0u8; 192]);
    file.extend(1i32.to_le_bytes());
    file.extend(vec![0u8; 52]);
    file.extend(padded("town\\house.rsm", 80));
    file.extend(padded("", 80));
    file.extend(floats(&[10.0, -5.0, 20.0, 0.0, 90.0, 0.0, 1.0, 2.0, 1.0]));
    file
}

fn text(name: &[u8]) -> String {
    String::from_utf8_lossy(name).into_owned()
}

#[test]
fn reads_a_map_description() {
    let rsw = Rsw::parse(&rsw_bytes(), text).unwrap();
    assert_eq!(rsw.water, Water { level: 12.5, kind: 4, frame_ticks: 5 });
    assert_eq!((rsw.light.diffuse, rsw.light.ambient), ([1.0, 0.9, 0.8], [0.3, 0.3, 0.4]));
    // The sun is above the ground.
    assert!(rsw.light.direction[1] > 0.0);
    assert_eq!(rsw.placements, [Placement { model: "town\\house.rsm".into(), position: [10.0, -5.0, 20.0], rotation: [0.0, 90.0, 0.0], scale: [1.0, 2.0, 1.0] }]);

    assert!(Rsw::parse(b"GRSM\x02\x01", text).is_err());
    assert!(Rsw::parse(&rsw_bytes()[..300], text).is_err());
    let mut newer = rsw_bytes();
    newer[5] = 6;
    assert!(Rsw::parse(&newer, text).is_err());
}

/// A RSM 1.4 of one node: a flat square of two triangles, 10 wide, standing from y = -20 to y = 0 … moved up
/// (y down) by its node, so its base is not at the origin until the model is assembled.
fn rsm_bytes() -> Vec<u8> {
    let mut file = b"GRSM\x01\x04".to_vec();
    file.extend([0u8; 8]);
    file.push(255);
    file.extend([0u8; 16]);
    file.extend(1i32.to_le_bytes());
    file.extend(padded("wall.bmp", 40));
    file.extend(padded("root", 40));
    file.extend(1i32.to_le_bytes());

    file.extend(padded("root", 40));
    file.extend(padded("", 40));
    file.extend(1i32.to_le_bytes());
    file.extend(0i32.to_le_bytes());
    // No turn in the mesh, no offset; the node moved to (100, -7, 50), not turned, twice as wide.
    file.extend(floats(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]));
    file.extend(floats(&[0.0, 0.0, 0.0]));
    file.extend(floats(&[100.0, -7.0, 50.0]));
    file.extend(floats(&[0.0, 0.0, 1.0, 0.0]));
    file.extend(floats(&[2.0, 1.0, 1.0]));
    file.extend(4i32.to_le_bytes());
    file.extend(floats(&[0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 0.0, -20.0, 0.0, 10.0, -20.0, 0.0]));
    file.extend(4i32.to_le_bytes());
    for (u, v) in [(0.0f32, 1.0f32), (1.0, 1.0), (0.0, 0.0), (1.0, 0.0)] {
        file.extend(0u32.to_le_bytes());
        file.extend(floats(&[u, v]));
    }
    file.extend(2i32.to_le_bytes());
    for corners in [[0u16, 1, 2], [2, 1, 3]] {
        for corner in corners.iter().chain(&corners) {
            file.extend(corner.to_le_bytes());
        }
        file.extend(0u16.to_le_bytes());
        file.extend(0u16.to_le_bytes());
        file.extend(0i32.to_le_bytes());
        file.extend(0i32.to_le_bytes());
    }
    file.extend(0i32.to_le_bytes());
    file
}

#[test]
fn puts_a_model_on_its_base() {
    let model = Rsm::parse(&rsm_bytes(), text).unwrap();
    assert_eq!(model.textures, ["wall.bmp"]);
    assert_eq!(model.triangles.len(), 2);
    // 20 wide after the node's stretch, centred on x; its lowest point (the largest y) at 0.
    let first = &model.triangles[0];
    assert!(close(first.positions[0], [-10.0, 0.0, 0.0]), "{:?}", first.positions);
    assert!(close(first.positions[1], [10.0, 0.0, 0.0]));
    assert!(close(first.positions[2], [-10.0, -20.0, 0.0]));
    assert_eq!(first.uvs, [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]]);
    assert_eq!(first.texture, 0);

    assert!(Rsm::parse(b"GRSW\x01\x04", text).is_err());
    assert!(Rsm::parse(&rsm_bytes()[..200], text).is_err());
    let mut newer = rsm_bytes();
    newer[4] = 2;
    newer[5] = 3;
    assert!(Rsm::parse(&newer, text).unwrap_err().contains("2.3"));
}

#[test]
fn stands_a_model_on_the_map() {
    let model = Rsm::parse(&rsm_bytes(), text).unwrap();
    let light = Light { direction: [0.0, 0.0, 1.0], diffuse: [0.5; 3], ambient: [0.25; 3] };
    // On a map 400 by 600 units: 10 east and 20 north of its middle, 5 above the zero height.
    let straight = Placement { model: String::new(), position: [10.0, -5.0, 20.0], rotation: [0.0; 3], scale: [1.0; 3] };
    let placed = place(&model, &straight, (400.0, 600.0), &light);
    assert_eq!(placed.len(), 2);
    let (texture, corners) = &placed[0];
    assert_eq!(*texture, 0);
    // The base at height 5, the top 20 above it: y points up in the world.
    assert!(close(corners[0].position, [200.0, 5.0, 320.0]), "{:?}", corners[0].position);
    assert!(close(corners[1].position, [220.0, 5.0, 320.0]));
    assert!(close(corners[2].position, [200.0, 25.0, 320.0]));
    // A wall facing north and south, lit by a sun in the north: ambient and all of the sunlight.
    assert!(close(corners[0].colour, [0.75; 3]));

    // Turned a quarter around the upright axis, the wall runs north-south and the same sun only grazes it.
    let turned = Placement { rotation: [0.0, 90.0, 0.0], ..straight.clone() };
    let (_, corners) = &place(&model, &turned, (400.0, 600.0), &light)[0];
    assert!((corners[0].position[0] - corners[1].position[0]).abs() < 1e-3, "{:?}", corners);
    assert!((corners[0].position[2] - corners[1].position[2]).abs() > 19.9);
    assert!(close(corners[0].colour, [0.25; 3]));

    // Twice as tall.
    let tall = Placement { scale: [1.0, 2.0, 1.0], ..straight };
    let (_, corners) = &place(&model, &tall, (400.0, 600.0), &light)[0];
    assert!(close(corners[2].position, [200.0, 45.0, 320.0]));
}
