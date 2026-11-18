use super::*;

/// A GAT of `width` by `height` cells, each with the four corner heights and type `cell(x, y)` gives.
fn gat_bytes(width: u32, height: u32, cell: impl Fn(u32, u32) -> ([f32; 4], i32)) -> Vec<u8> {
    let mut file = b"GRAT\x01\x02".to_vec();
    file.extend(width.to_le_bytes());
    file.extend(height.to_le_bytes());
    for y in 0..height {
        for x in 0..width {
            let (heights, kind) = cell(x, y);
            heights.iter().for_each(|height| file.extend(height.to_le_bytes()));
            file.extend(kind.to_le_bytes());
        }
    }
    file
}

#[test]
fn reads_a_gat() {
    // Heights point down in the file. The cell at (1, 0) is a wall, (2, 0) a cliff.
    let gat = Gat::parse(&gat_bytes(3, 2, |x, y| ([-10.0, -20.0, -30.0, -40.0], if y == 0 { [0, 1, 5][x as usize] } else { 0 }))).unwrap();
    assert_eq!((gat.width, gat.height), (3, 2));
    assert!(gat.walkable(0, 0) && !gat.walkable(1, 0) && !gat.walkable(2, 0) && gat.walkable(1, 1));
    // Off the map is not walkable.
    assert!(!gat.walkable(-1, 0) && !gat.walkable(3, 0) && !gat.walkable(0, 2));
    // Corners, edges and the middle of a cell.
    assert_eq!(gat.height_at(0.0, 0.0), 10.0);
    assert_eq!(gat.height_at(1.0, 0.0), 10.0);
    assert_eq!(gat.height_at(0.5, 0.0), 15.0);
    assert_eq!(gat.height_at(0.5, 0.5), 25.0);
    assert!(Gat::parse(b"GRGN").is_err());
    assert!(Gat::parse(&gat_bytes(3, 2, |_, _| ([0.0; 4], 0))[..40]).is_err());
}

#[test]
fn walks_straight_and_diagonally() {
    let gat = Gat::open(10, 10);
    assert_eq!(gat.path((1, 1), (4, 1)), [(2, 1), (3, 1), (4, 1)]);
    assert_eq!(gat.path((1, 1), (3, 3)), [(2, 2), (3, 3)]);
    // A knight's move: one diagonal and one straight step.
    assert_eq!(gat.path((1, 1), (2, 3)).len(), 2);
    assert!(gat.path((1, 1), (1, 1)).is_empty());
}

#[test]
fn walks_around_walls_and_never_cuts_corners() {
    let mut gat = Gat::open(5, 5);
    // A wall across the middle, open at its east end.
    for x in 0..4 {
        gat.block(x, 2);
    }
    let path = gat.path((0, 0), (0, 4));
    assert_eq!(path.last(), Some(&(0, 4)));
    assert!(path.contains(&(4, 2)), "{path:?}");
    // Every step is to a neighbour, and no diagonal squeezes past a wall's corner.
    let mut last = (0, 0);
    for &cell in &path {
        assert!(gat.walkable(cell.0, cell.1));
        assert!((cell.0 - last.0).abs() <= 1 && (cell.1 - last.1).abs() <= 1);
        if cell.0 != last.0 && cell.1 != last.1 {
            assert!(gat.walkable(cell.0, last.1) && gat.walkable(last.0, cell.1), "{last:?} to {cell:?} cuts a corner");
        }
        last = cell;
    }
}

#[test]
fn finds_no_path_to_walls_closed_rooms_or_too_far() {
    let mut gat = Gat::open(60, 6);
    gat.block(3, 3);
    assert!(gat.path((0, 0), (3, 3)).is_empty());
    // A cell walled in on all sides.
    for (x, y) in [(9, 0), (9, 1), (9, 2), (10, 2), (11, 2), (11, 1), (11, 0)] {
        gat.block(x, y);
    }
    assert!(gat.path((0, 0), (10, 0)).is_empty());
    // Farther than the server walks in one go.
    assert!(gat.path((0, 5), (40, 5)).is_empty());
    assert_eq!(gat.path((0, 5), (32, 5)).len(), 32);
}

#[test]
fn a_click_out_of_reach_goes_as_far_as_it_can() {
    let mut gat = Gat::open(60, 6);
    // Too far: the cell as many steps along the line as a player may ask the server for (its `max_walk_path`).
    assert_eq!(gat.goal_towards((0, 5), (50, 5)), Some((17, 5)));
    // And on from there, a stretch at a time, to the place itself.
    assert_eq!(gat.goal_towards((17, 5), (50, 5)), Some((34, 5)));
    assert_eq!(gat.goal_towards((34, 5), (50, 5)), Some((50, 5)));
    // Just within reach: the place itself; a step past it: the last cell within.
    assert_eq!(gat.goal_towards((0, 5), (17, 5)), Some((17, 5)));
    assert_eq!(gat.goal_towards((0, 5), (18, 5)), Some((17, 5)));
    // On a wall: the cell before it.
    gat.block(5, 0);
    assert_eq!(gat.goal_towards((0, 0), (5, 0)), Some((4, 0)));
    assert_eq!(gat.goal_towards((0, 0), (3, 0)), Some((3, 0)));
    // Where it stands already: nowhere.
    assert_eq!(gat.goal_towards((0, 0), (0, 0)), None);
}

#[test]
fn directions_are_numbered_from_south_clockwise() {
    assert_eq!([direction(0, -1), direction(-1, -1), direction(-1, 0), direction(-1, 1)], [0, 1, 2, 3]);
    assert_eq!([direction(0, 1), direction(1, 1), direction(1, 0), direction(1, -1)], [4, 5, 6, 7]);
}

#[test]
fn a_walk_takes_longer_on_diagonals() {
    let walk = Walk::new((0, 0), &[(1, 0), (2, 1), (2, 2)], 150.0);
    assert_eq!(walk.duration(), 150.0 + 210.0 + 150.0);
    assert_eq!(walk.goal(), (2, 2));
    assert_eq!(walk.at(0.0), ((0.0, 0.0), 6));
    assert_eq!(walk.at(75.0), ((0.5, 0.0), 6));
    // Half-way along the diagonal, facing north-east.
    assert_eq!(walk.at(150.0 + 105.0), ((1.5, 0.5), 5));
    // At the end, and after it, the walker stands on the goal, still facing the way it came.
    assert_eq!(walk.at(510.0), ((2.0, 2.0), 4));
    assert_eq!(walk.at(9999.0), ((2.0, 2.0), 4));
}

/// A GND of one texture and one patch of baked light, 2 by 1 cubes: the west one at height 0, the east one
/// raised by 10, with a wall between them.
fn gnd_bytes() -> Vec<u8> {
    let mut file = b"GRGN\x01\x07".to_vec();
    for value in [2i32, 1] {
        file.extend(value.to_le_bytes());
    }
    file.extend(10.0f32.to_le_bytes());
    file.extend(1i32.to_le_bytes());
    file.extend(8i32.to_le_bytes());
    file.extend(b"wall.bmp");
    for value in [1i32, 8, 8, 1] {
        file.extend(value.to_le_bytes());
    }
    // Brightness 200 everywhere, colour (1, 2, 3).
    file.extend([200u8; 64]);
    file.extend([1u8, 2, 3].repeat(64));
    file.extend(1i32.to_le_bytes());
    for value in [0.0f32, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0] {
        file.extend(value.to_le_bytes());
    }
    file.extend(0u16.to_le_bytes());
    file.extend(0u16.to_le_bytes());
    // Blue, green, red, alpha.
    file.extend([30u8, 20, 10, 255]);
    for (height, east) in [(0.0f32, 0i32), (-10.0, -1)] {
        for _ in 0..4 {
            file.extend(height.to_le_bytes());
        }
        for surface in [0i32, -1, east] {
            file.extend(surface.to_le_bytes());
        }
    }
    file
}

#[test]
fn builds_the_ground_mesh() {
    let gnd = Gnd::parse(&gnd_bytes(), |name| String::from_utf8_lossy(name).into_owned()).unwrap();
    assert_eq!((gnd.width, gnd.height, gnd.zoom), (2, 1, 10.0));
    assert_eq!(gnd.textures, ["wall.bmp"]);

    let mesh = gnd.mesh();
    // Two tops and one wall: three faces of four corners and two triangles each.
    assert_eq!(mesh.vertices.len(), 12);
    assert_eq!(mesh.by_texture, [vec![0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7, 8, 9, 10, 10, 9, 11]]);
    // The west cube's top, on the ground; the wall, from the west cube's east edge up to the east cube.
    assert_eq!(mesh.vertices[0].position, [0.0, 0.0, 0.0]);
    assert_eq!(mesh.vertices[3].position, [10.0, 0.0, 10.0]);
    assert_eq!(mesh.vertices[4].position, [10.0, 0.0, 10.0]);
    assert_eq!(mesh.vertices[6].position, [10.0, 10.0, 10.0]);
    // The east cube's top, raised.
    assert_eq!(mesh.vertices[8].position, [10.0, 10.0, 0.0]);
    // The tint is stored blue first.
    assert_eq!(mesh.vertices[0].colour, [10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0]);
    // The light: colour to add, brightness in alpha; a face uses the patch inside its one-pixel border.
    assert_eq!(mesh.light_atlas_size, (8, 8));
    assert_eq!(mesh.light_atlas[..4], [1, 2, 3, 200]);
    assert_eq!(mesh.vertices[0].light_uv, [1.0 / 8.0, 1.0 / 8.0]);
    assert_eq!(mesh.vertices[3].light_uv, [7.0 / 8.0, 7.0 / 8.0]);
}

#[test]
fn refuses_files_that_are_not_ground() {
    let name = |name: &[u8]| String::from_utf8_lossy(name).into_owned();
    assert!(Gnd::parse(b"GRAT\x01\x07", name).is_err());
    assert!(Gnd::parse(&gnd_bytes()[..100], name).is_err());
    let mut odd = gnd_bytes();
    odd[5] = 3;
    assert!(Gnd::parse(&odd, name).is_err());
}
