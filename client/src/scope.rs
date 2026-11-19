//! The circle on the ground under a place a skill is being cast on, as the client's `CGroundScope` has it.
//!
//! While a skill aimed at a place is cast (`LaunchSkillCastEffect`, for a cast with no one aimed at), the client
//! lays a picture (`effect\magic_target.tga`) over a square of cells around that place, turning, until the cast
//! is over. The square is the skill's scope and two more cells across (`CGroundScope::SendMsg`, AM_SET_SKILL);
//! the scope of each skill is the client's own table (`CGroundScope::SkillScopeType`, written out by
//! `tools/client-tables/ground_scopes.py`). The picture is not turned as a thing: each cell of the square shows
//! the part of the picture that a turned picture would have over it (`CGroundScope::Render`).

/// How many cells across the circle of a skill is: its scope and two. A skill the client's table does not name
/// is asked of its own action there (`CSkillActionMgr::GetSkillScopeType`), 0 for one without: two cells.
pub fn side(skill: u16) -> usize {
    let scope = crate::effect_tables::ground_scope(skill).unwrap_or(0);
    (scope + 2).max(0) as usize
}

/// How far the circle turns each tick of the client, in degrees: half a degree for each cell across, shared out.
pub fn turn(side: usize) -> f32 {
    if side == 0 { 0.0 } else { 0.5 / side as f32 }
}

/// How much of the circle shows, 0 to 1, turned by `angle` degrees: half and a little, the little swelling with
/// the sine of the turn (the client's `(sine * 40 << 24) + 0x80FFFFFF`).
pub fn alpha(angle: f32) -> f32 {
    ((angle.to_radians().sin() * 40.0) as i32 + 0x80).clamp(0, 255) as f32 / 255.0
}

/// The cells of a circle `side` across around a cell, turned by `angle` degrees: each cell, and where of the
/// picture its four corners show — the corner at the cell's own corner, the next cell along, the next row, and
/// both (`CGroundScope::Render`).
pub fn cells(cell: (i32, i32), side: usize, angle: f32) -> Vec<((i32, i32), [[f32; 2]; 4])> {
    let (sine, cosine) = angle.to_radians().sin_cos();
    let (centre, size) = (side as f32 * 0.5, side as f32);
    let (left, top) = (cell.0 - side as i32 / 2, cell.1 - side as i32 / 2);
    let shown = |row: f32, column: f32| [(row * sine + column * cosine + centre) / size, (row * cosine - column * sine + centre) / size];
    (0..side)
        .flat_map(|row| (0..side).map(move |column| (row, column)))
        .map(|(row, column)| {
            let (row0, column0) = (row as f32 - centre, column as f32 - centre);
            ((left + column as i32, top + row as i32), [shown(row0, column0), shown(row0, column0 + 1.0), shown(row0 + 1.0, column0), shown(row0 + 1.0, column0 + 1.0)])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_circle_is_the_skills_scope_and_two_cells_more() {
        // Thunder Storm's scope is 5, Storm Gust's 9, Napalm Beat's 3 (the client's table).
        assert_eq!((side(21), side(89), side(11)), (7, 11, 5));
        // A skill the table does not name: two cells.
        assert_eq!(side(18), 2);
    }

    #[test]
    fn an_unturned_circle_shows_the_whole_picture_once() {
        let cells = cells((100, 50), 7, 0.0);
        assert_eq!(cells.len(), 49);
        // It is laid from three cells before the place to three after.
        assert_eq!((cells[0].0, cells[48].0), ((97, 47), (103, 53)));
        // The first cell's own corner is the picture's corner, the last cell's far corner the opposite one.
        assert_eq!(cells[0].1[0], [0.0, 0.0]);
        assert_eq!(cells[48].1[3], [1.0, 1.0]);
        // Neighbours share the corners between them.
        assert_eq!(cells[0].1[1], cells[1].1[0]);
    }

    #[test]
    fn it_turns_about_its_middle() {
        // The middle of the picture stays in the middle of the square whatever the turn.
        let middle = |angle: f32| cells((0, 0), 2, angle)[0].1[3];
        assert_eq!(middle(0.0), [0.5, 0.5]);
        assert!((middle(37.0)[0] - 0.5).abs() < 1e-6 && (middle(37.0)[1] - 0.5).abs() < 1e-6);
        // A quarter turn carries a corner of the picture to the next corner of the square.
        let turned = cells((0, 0), 2, 90.0)[0].1[0];
        assert!((turned[0] - 0.0).abs() < 1e-6 && (turned[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn it_turns_slower_the_larger_it_is_and_swells_as_it_turns() {
        assert_eq!(turn(2), 0.25);
        assert!(turn(11) < turn(7));
        assert_eq!(alpha(0.0), 128.0 / 255.0);
        assert_eq!(alpha(90.0), 168.0 / 255.0);
    }
}
