//! The client's tooltip as rules (`CMode::ShowTooltip`, a `UITransBalloonText`): how its words are cut into
//! lines and where on the screen its box is put. The box itself is the page's (`web/app/tooltip.rs`).

/// How many letters a line of a tooltip has at the most (`CMode::ShowTooltip`: `SetText(text, 35)`).
pub const LINE: usize = 35;

/// The colour of the words of a thing not identified yet (`CSession::GetItemNameColor`: `0xFFAAAAAA`); every
/// other tooltip is white. (A thing with cards past its slots has a colour of its own there,
/// `ItemCardColor`: no item of the classic game has such cards.)
pub const UNKNOWN: &str = "#aaaaaa";

/// A tooltip's words as its lines (`UIBalloonText::SetText`, `WrapPlainText`).
pub fn lines(words: &str) -> Vec<String> {
    crate::broadcast::wrapped(words, LINE)
}

/// Where a tooltip of that size is put when it is asked for at `at`, on a screen of `view`
/// (`CMode::ShowTooltip`): there, but no more than 3 off the screen's left and top, and no more than 3 past
/// its right and bottom; the left and the top win on a screen too small.
pub fn placed(at: (i32, i32), size: (i32, i32), view: (i32, i32)) -> (i32, i32) {
    let along = |at: i32, size: i32, view: i32| at.max(-3).min(view - size + 3);
    (along(at.0, size.0, view.0), along(at.1, size.1, view.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tooltip_is_cut_at_35_letters() {
        assert_eq!(lines("Red Potion: 3 ea"), ["Red Potion: 3 ea"]);
        assert_eq!(lines("+7 Quadruple Hurricane Saber [4]: 1 ea"), ["+7 Quadruple Hurricane Saber [4]: 1", "ea"]);
        assert!(lines("").is_empty());
    }

    #[test]
    fn a_tooltip_stays_on_the_screen() {
        assert_eq!(placed((100, 50), (80, 16), (1024, 768)), (100, 50));
        assert_eq!(placed((-20, -30), (80, 16), (1024, 768)), (-3, -3));
        assert_eq!(placed((1000, 760), (80, 16), (1024, 768)), (947, 755));
    }
}
