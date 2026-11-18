//! The game's mouse pointer: which of its pictures shows at a moment, as the client's `CMode::DrawCursor` has it.
//!
//! The pointer is a sprite (`cursors.spr`) with an action for each thing it can say (0 plain, 1 talking, 2 a
//! finger, 4 turning the camera, 5 fighting, 7 a door, 9 a hand, 10 and 11 a skill aimed, 13 no way). The client
//! does not go by the sprite's own timing: a tick is 24 milliseconds and each action shows a picture for its own
//! number of ticks.

/// How many of the client's ticks one picture of an action shows.
fn ticks(action: usize) -> f64 {
    match action {
        5 => 20.0,
        7 => 10.0,
        9 => 1.0,
        10 => 6.0,
        11 => 3.0,
        13 => 2.0,
        _ => 4.0,
    }
}

/// Which of an action's `pictures` shows, `since` milliseconds after the pointer became that action. `held` is
/// for how long the left button has been down, if it is.
///
/// Most actions run round and round. The one for talking (1) runs once and stays on its last picture. The hand
/// (9) is open while the button is up; when the button goes down it closes, and stays closed.
pub fn frame(action: usize, pictures: usize, since: f64, held: Option<f64>) -> usize {
    let (count, last) = (pictures.max(1), pictures.saturating_sub(1));
    let after = |milliseconds: f64| (milliseconds / (24.0 * ticks(action))).max(0.0) as usize;
    match (action, held) {
        (9, None) => 0,
        (9, Some(held)) => after(held).min(last),
        (1, _) => after(since).min(last),
        _ => after(since) % count,
    }
}

#[cfg(test)]
mod tests {
    use super::frame;

    #[test]
    fn the_plain_pointer_runs_round() {
        // Four ticks a picture: 96 milliseconds each, eleven pictures.
        assert_eq!(frame(0, 11, 0.0, None), 0);
        assert_eq!(frame(0, 11, 96.0, None), 1);
        assert_eq!(frame(0, 11, 96.0 * 11.0, None), 0);
    }

    #[test]
    fn the_talking_pointer_runs_once() {
        assert_eq!(frame(1, 9, 96.0 * 3.0, None), 3);
        assert_eq!(frame(1, 9, 96.0 * 100.0, None), 8);
    }

    #[test]
    fn the_hand_closes_when_the_button_goes_down() {
        // Open however long the pointer has been a hand, while the button is up.
        assert_eq!(frame(9, 3, 5000.0, None), 0);
        // A picture every tick once it is down, and then it stays closed.
        assert_eq!(frame(9, 3, 5000.0, Some(0.0)), 0);
        assert_eq!(frame(9, 3, 5000.0, Some(24.0)), 1);
        assert_eq!(frame(9, 3, 5000.0, Some(48.0)), 2);
        assert_eq!(frame(9, 3, 5000.0, Some(4000.0)), 2);
    }

    #[test]
    fn a_pointer_without_pictures_shows_its_first() {
        assert_eq!(frame(5, 0, 1000.0, None), 0);
    }
}
