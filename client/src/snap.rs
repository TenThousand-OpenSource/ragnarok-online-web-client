//! A window being moved docks to what is near it, as the client's do with `/window` on: to the edges of the
//! screen and to the outer edges of the other windows, from 10 points away (`SnapRectangle::Snap`).

/// How near an edge has to be to another for the window to go to it.
pub const NEAR: f64 = 10.0;

/// A rectangle: left, top, right, bottom.
pub type Rect = [f64; 4];

/// The smallest of the shifts that is no more than `near`, if any.
fn nearest(shifts: impl Iterator<Item = f64>, near: f64) -> Option<f64> {
    shifts.filter(|shift| shift.abs() <= near).min_by(|one, other| one.abs().total_cmp(&other.abs()))
}

/// Whether two spans overlap, or lie no more than `near` apart.
fn beside(low: f64, high: f64, other_low: f64, other_high: f64, near: f64) -> bool {
    low <= other_high + near && other_low <= high + near
}

/// How far a window being moved is to be shifted, across and down, to dock: its sides to the screen's and to
/// the facing sides of the windows it is beside; and, docked to the side of one, its top to that one's top or
/// its bottom to that one's bottom (and so for the sides of one it is docked over or under). `near` is how
/// near counts (`NEAR`, in the measure the rectangles are in).
pub fn shift(moved: Rect, others: &[Rect], screen: (f64, f64), near: f64) -> (f64, f64) {
    let [left, top, right, bottom] = moved;
    let level = |other: &&Rect| beside(top, bottom, other[1], other[3], near);
    let upright = |other: &&Rect| beside(left, right, other[0], other[2], near);
    let across = nearest([-left, screen.0 - right].into_iter().chain(others.iter().filter(level).flat_map(|other| [other[2] - left, other[0] - right])), near);
    let down = nearest([-top, screen.1 - bottom].into_iter().chain(others.iter().filter(upright).flat_map(|other| [other[3] - top, other[1] - bottom])), near);
    // Docked one way only, the window lines up with the one it is docked to the other way.
    let touches_side = |other: &&Rect, by: f64| (other[2] - (left + by)).abs() < 0.5 || (other[0] - (right + by)).abs() < 0.5;
    let touches_level = |other: &&Rect, by: f64| (other[3] - (top + by)).abs() < 0.5 || (other[1] - (bottom + by)).abs() < 0.5;
    let down = down.or_else(|| across.and_then(|by| nearest(
        others.iter().filter(level).filter(|other| touches_side(other, by)).flat_map(|other| [other[1] - top, other[3] - bottom]),
        near
    )));
    let across = across.or_else(|| down.and_then(|by| nearest(
        others.iter().filter(upright).filter(|other| touches_level(other, by)).flat_map(|other| [other[0] - left, other[2] - right]),
        near
    )));
    (across.unwrap_or(0.0), down.unwrap_or(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: (f64, f64) = (1024.0, 768.0);

    #[test]
    fn a_window_docks_to_the_screen_from_ten_points() {
        assert_eq!(shift([7.0, 300.0, 207.0, 400.0], &[], SCREEN, NEAR), (-7.0, 0.0));
        assert_eq!(shift([11.0, 300.0, 211.0, 400.0], &[], SCREEN, NEAR), (0.0, 0.0));
        assert_eq!(shift([820.0, 660.0, 1020.0, 760.0], &[], SCREEN, NEAR), (4.0, 8.0));
    }

    #[test]
    fn a_window_docks_to_the_side_of_another_and_lines_up_with_it() {
        let other = [300.0, 200.0, 500.0, 400.0];
        // Its left side 6 from the other's right, its top 4 under the other's top.
        assert_eq!(shift([506.0, 204.0, 606.0, 304.0], &[other], SCREEN, NEAR), (-6.0, -4.0));
        // Under the other, its left side 3 off the other's.
        assert_eq!(shift([303.0, 408.0, 403.0, 508.0], &[other], SCREEN, NEAR), (-3.0, -8.0));
        // One far above or below the other does not dock to its side.
        assert_eq!(shift([506.0, 450.0, 606.0, 550.0], &[other], SCREEN, NEAR), (0.0, 0.0));
    }

    #[test]
    fn the_nearest_edge_wins() {
        let others = [[300.0, 200.0, 500.0, 400.0], [100.0, 200.0, 191.0, 400.0]];
        assert_eq!(shift([195.0, 250.0, 292.0, 300.0], &others, SCREEN, NEAR), (-4.0, 0.0));
    }
}
