use super::*;

/// A map of rows of text, the first row the northernmost: `#` a wall, `~` a cliff (shot over, not walked on),
/// anything else open ground.
fn map(rows: &[&str]) -> Gat {
    let (width, height) = (rows[0].len(), rows.len());
    let mut gat = Gat::open(width, height);
    for (row, line) in rows.iter().rev().enumerate() {
        for (column, kind) in line.bytes().enumerate() {
            match kind {
                b'#' => gat.retype(column as i32, row as i32, 1),
                b'~' => gat.retype(column as i32, row as i32, 5),
                _ => {}
            }
        }
    }
    gat
}

#[test]
fn a_wall_between_stops_a_shot_and_a_cliff_does_not() {
    let gat = map(&[".......", "...#...", ".......", "...~...", "......."]);
    // Along the row of the wall, either way, and along the row of the cliff.
    assert!(!gat.sees((0, 3), (6, 3)) && !gat.sees((6, 3), (0, 3)));
    assert!(gat.sees((0, 1), (6, 1)) && gat.sees((6, 1), (0, 1)));
    assert!(gat.sees((0, 2), (6, 2)));
    // Up the column of both: the wall stops it.
    assert!(!gat.sees((3, 0), (3, 4)) && !gat.sees((3, 4), (3, 0)));
    assert!(gat.sees((3, 0), (3, 2)));
    // The two cells themselves are not looked at: out of a wall, into a wall, and onto the same cell.
    assert!(gat.sees((3, 3), (0, 3)) && gat.sees((0, 3), (3, 3)) && gat.sees((2, 2), (2, 2)));
}

/// The cells of the server's line, by its own arithmetic: from (0, 0) to (5, 2) they are (1, 0), (2, 0),
/// (3, 1), (4, 1); the same line asked from the other end is walked from the western cell too.
#[test]
fn the_line_is_the_servers() {
    let line = [(1, 0), (2, 0), (3, 1), (4, 1)];
    for y in 0..3 {
        for x in 0..6 {
            if (x, y) == (0, 0) || (x, y) == (5, 2) {
                continue;
            }
            let mut gat = Gat::open(6, 3);
            gat.block(x as usize, y as usize);
            assert_eq!(gat.sees((0, 0), (5, 2)), !line.contains(&(x, y)), "a wall on {x}, {y}");
            assert_eq!(gat.sees((5, 2), (0, 0)), !line.contains(&(x, y)), "a wall on {x}, {y}, backwards");
        }
    }
    // Falling to the east: from (0, 2) to (2, 0) through (1, 1) alone; a steep line through each row once.
    let mut gat = Gat::open(4, 6);
    gat.block(1, 1);
    assert!(!gat.sees((0, 2), (2, 0)) && !gat.sees((2, 0), (0, 2)));
    assert!(gat.sees((0, 0), (1, 5)));
    gat.block(0, 2);
    assert!(!gat.sees((0, 0), (1, 5)));
}

#[test]
fn the_reach_is_straight_across_less_a_tenth_and_needs_the_line() {
    let mut gat = Gat::open(40, 40);
    // Straight: a cell more than the range, as the server's tenth makes it (10.9 across is 10 cells).
    assert!(gat.reaches((5, 5), (16, 5), 10) && !gat.reaches((5, 5), (17, 5), 10));
    // Slanting: 7 and 7 are 9.9 across, 9 cells; 8 and 8 are 11.3, out of a reach of 10.
    assert!(gat.reaches((5, 5), (12, 12), 9) && !gat.reaches((5, 5), (13, 13), 10));
    // A hand reaches the eight cells beside and the second cell straight on, no further.
    assert!(gat.reaches((5, 5), (6, 6), 1) && gat.reaches((5, 5), (7, 5), 1) && gat.reaches((5, 5), (5, 5), 1));
    assert!(!gat.reaches((5, 5), (8, 5), 1) && !gat.reaches((5, 5), (7, 6), 1));
    // Farther than the server lets anything be aimed at, nothing is reached.
    assert!(gat.reaches((5, 5), (19, 5), 20) && !gat.reaches((5, 5), (20, 5), 20));
    // A wall on the line: in range and not reached, by a bow and by a spear alike.
    gat.block(8, 5);
    assert!(!gat.reaches((5, 5), (11, 5), 10) && !gat.reaches((7, 5), (9, 5), 3) && gat.reaches((5, 6), (11, 6), 10));
}

#[test]
fn the_way_into_reach_goes_round_to_the_nearest_cell_that_sees() {
    // An archer south of a wall, the target right behind it: in range, not seen.
    let gat = map(&[
        "...........",
        ".....T.....",
        "..#######..",
        "...........",
        ".....A.....",
        "...........",
    ]);
    let (archer, target) = ((5, 1), (5, 4));
    assert!(!gat.reaches(archer, target, 10));
    let way = gat.way_into_reach(archer, target, |cell| gat.reaches(cell, target, 10), |_| false).unwrap();
    let stand = *way.last().unwrap();
    assert!(gat.reaches(stand, target, 10));
    // Every step is one cell on open ground, and no cell nearer by walking reaches the target.
    let mut at = archer;
    for step in &way {
        assert!(gat.walkable(step.0, step.1) && (step.0 - at.0).abs().max((step.1 - at.1).abs()) == 1);
        at = *step;
    }
    assert!(way[..way.len() - 1].iter().all(|cell| !gat.reaches(*cell, target, 10)));
    // The archer goes round the end of the wall until the line clears it, not up to the target.
    assert_eq!(way.len(), 6);
    // A hand must go all the way round, to the target's side of the wall.
    let near = gat.way_into_reach(archer, target, |cell| gat.reaches(cell, target, 1), |_| false).unwrap();
    let beside = *near.last().unwrap();
    assert!(gat.reaches(beside, target, 1) && beside.1 >= 4 && near.len() > way.len());
    // From where the target is reached already, the way is empty.
    assert_eq!(gat.way_into_reach((5, 5), target, |cell| gat.reaches(cell, target, 1), |_| false), Some(Vec::new()));
}

#[test]
fn a_cell_somebody_stands_on_is_not_stood_on_and_a_shut_place_is_given_up() {
    let gat = map(&[
        "#######",
        "#..T..#",
        "#######",
        ".......",
        "...A...",
    ]);
    let (own, target) = ((3, 0), (3, 3));
    // Walled in: within range of a bow and of nothing that can be walked to. No way, and the search ends.
    assert_eq!(gat.way_into_reach(own, target, |cell| gat.reaches(cell, target, 10), |_| false), None);
    assert_eq!(gat.way_into_reach(own, target, |cell| gat.reaches(cell, target, 1), |_| false), None);
    // Over a cliff the arrow flies, and the hand still finds no way.
    let gat = map(&["~~~~~~~", "~..T..~", "~~~~~~~", "~~~~~~~", "...A..."]);
    assert_eq!(gat.way_into_reach(own, target, |cell| gat.reaches(cell, target, 10), |_| false), Some(Vec::new()));
    assert_eq!(gat.way_into_reach(own, target, |cell| gat.reaches(cell, target, 1), |_| false), None);
    // Open ground: the cell nearest is taken by somebody, so another is stood on (a reach of 3 is 4 cells straight on).
    let gat = Gat::open(9, 3);
    let free = gat.way_into_reach((0, 1), (6, 1), |cell| gat.reaches(cell, (6, 1), 3), |_| false).unwrap();
    let busy = gat.way_into_reach((0, 1), (6, 1), |cell| gat.reaches(cell, (6, 1), 3), |cell| cell == (2, 1)).unwrap();
    assert_eq!(free.last(), Some(&(2, 1)));
    assert!(busy.last() != Some(&(2, 1)) && gat.reaches(*busy.last().unwrap(), (6, 1), 3));
}

#[test]
fn a_long_way_is_asked_for_in_stretches() {
    let way: Vec<(i32, i32)> = (1..=20).map(|x| (x, 0)).collect();
    assert_eq!(stretch(&way), Some((MAX_ASKED_ROUND as i32, 0)));
    assert_eq!(stretch(&way[..5]), Some((5, 0)));
    assert_eq!(stretch(&[]), None);
}

#[test]
fn an_order_takes_the_place_of_a_walk_at_once() {
    // Standing: asked for when in reach, walked to when not.
    assert_eq!((order(true, false, false), order(false, false, false)), (Order::Ask, Order::Go));
    // Walking elsewhere (a place pressed far away): in reach now, the walk is ended and the order asked for
    // in the same breath; out of reach, the walk into reach replaces the walk under way.
    assert_eq!((order(true, true, false), order(false, true, false)), (Order::Halt, Order::Go));
    // Walking to where the target is in reach (the walk the order itself asked for): asked for as soon as it
    // comes into reach, and nothing asked anew before (a press again restarts nothing).
    assert_eq!((order(true, true, true), order(false, true, true)), (Order::Ask, Order::Wait));
}
