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
    // Too far, as the client does it: the place itself is asked for, whatever the server makes of it.
    assert_eq!(gat.goal_towards((0, 5), (50, 5), |_| false, false), Some((50, 5)));
    // With the project's far walk: the cell as many steps along the way as a player may ask the server for
    // (its `max_walk_path`), and on from there, a stretch at a time, to the place itself.
    assert_eq!(gat.goal_towards((0, 5), (50, 5), |_| false, true), Some((14, 5)));
    assert_eq!(gat.goal_towards((14, 5), (50, 5), |_| false, true), Some((28, 5)));
    assert_eq!(gat.goal_towards((42, 5), (50, 5), |_| false, true), Some((50, 5)));
    // Just within reach: the place itself; a step past it: the last cell within.
    assert_eq!(gat.goal_towards((0, 5), (14, 5), |_| false, true), Some((14, 5)));
    assert_eq!(gat.goal_towards((0, 5), (18, 5), |_| false, true), Some((14, 5)));
    // On a wall: the first walkable cell beside it in the client's order (east of it first), a way to it found.
    gat.block(5, 0);
    assert_eq!(gat.goal_towards((0, 0), (5, 0), |_| false, false), Some((6, 0)));
    assert_eq!(gat.goal_towards((0, 0), (3, 0), |_| false, false), Some((3, 0)));
    // Where it stands already: nowhere.
    assert_eq!(gat.goal_towards((0, 0), (0, 0), |_| false, false), None);
}

#[test]
fn the_far_walk_goes_round_what_is_near_however_long_the_way() {
    // A ridge thirty cells long between the two cells, its end ten cells to the north: more cells than the
    // client's search looks at, and near.
    let mut gat = Gat::open(80, 80);
    for y in 10..40 {
        gat.block(30, y);
    }
    let (from, to) = ((33, 30), (27, 20));
    assert!(gat.client_path(from, to).is_empty());
    assert_eq!(gat.goal_towards(from, to, |_| false, false), None);
    // Stretch by stretch the character comes to the place, each stretch one the server walks.
    let (mut at, mut stretches) = (from, 0);
    while at != to {
        let next = gat.goal_towards(at, to, |_| false, true).expect("a way round");
        assert!((1..=MAX_ASKED_ROUND).contains(&gat.path(at, next).len()));
        (at, stretches) = (next, stretches + 1);
        assert!(stretches < 10);
    }
    assert!(stretches >= 2);
    // A wall whose only gap is far away is not gone round.
    for x in 0..75 {
        gat.block(x, 50);
    }
    assert_eq!(gat.goal_towards((10, 45), (10, 55), |_| false, true), None);
}

#[test]
fn a_press_on_a_cell_where_somebody_stands_goes_beside_it() {
    let gat = Gat::open(20, 20);
    // Somebody on the cell pressed: the next cell in the client's order, east of it.
    assert_eq!(gat.goal_towards((2, 10), (10, 10), |cell| cell == (10, 10), false), Some((11, 10)));
    // That one taken as well: north-east, then south-east.
    let crowd = [(10, 10), (11, 10), (11, 11)];
    assert_eq!(gat.goal_towards((2, 10), (10, 10), |cell| crowd.contains(&cell), false), Some((11, 9)));
    // Everything around taken: the next cell on the line back to the character.
    let ring = |cell: (i32, i32)| (cell.0 - 10).abs() <= 1 && (cell.1 - 10).abs() <= 1;
    assert_eq!(gat.goal_towards((2, 10), (10, 10), ring, false), Some((8, 10)));
}

#[test]
fn directions_are_numbered_from_south_clockwise() {
    assert_eq!([direction(0, -1), direction(-1, -1), direction(-1, 0), direction(-1, 1)], [0, 1, 2, 3]);
    assert_eq!([direction(0, 1), direction(1, 1), direction(1, 0), direction(1, -1)], [4, 5, 6, 7]);
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

#[test]
fn a_press_behind_something_goes_round_what_is_near_and_never_across_the_map() {
    // A short wall between: the client's search finds the way round it.
    let mut gat = Gat::open(60, 60);
    for x in 8..13 {
        gat.block(x, 30);
    }
    let way = gat.client_path((10, 27), (10, 33));
    assert_eq!(way.last(), Some(&(10, 33)));
    assert!(way.iter().all(|cell| cell.1 != 30 || !(8..13).contains(&cell.0)));
    assert_eq!(gat.goal_towards((10, 27), (10, 33), |_| false, false), Some((10, 33)));
    // A wall across the map with one gap far to the side: the search gives up after its 149 cells, and the
    // press does nothing, as in the client (the place itself is the first walkable cell on the line back).
    for x in 0..55 {
        gat.block(x, 30);
    }
    assert!(gat.client_path((10, 20), (10, 40)).is_empty());
    assert_eq!(gat.goal_towards((10, 20), (10, 40), |_| false, false), None);
    // A press on the wall itself: the client takes the first walkable cell beside it in its fixed order — here
    // one behind the wall, to which it finds no way — and looks no further.
    assert_eq!(gat.goal_towards((10, 20), (10, 30), |_| false, false), None);
    // On a wall two cells thick the cells beside the press are wall too, and the next cell on the line back is
    // taken, with the cells beside it in the same order: one before the wall.
    for x in 0..55 {
        gat.block(x, 31);
        gat.block(x, 29);
    }
    assert_eq!(gat.goal_towards((10, 20), (10, 30), |_| false, false), Some((11, 28)));
}
