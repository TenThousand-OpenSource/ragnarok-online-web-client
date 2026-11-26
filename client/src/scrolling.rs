//! How the client's lists and texts move under the wheel: by whole steps, never by pixels as a page does.
//!
//! A list of rows moves by rows, and a notch of the wheel is three of them, or one less than the list shows
//! when it shows fewer than four (`UIItemBaseWnd::OnWheel`, `UISkillListWnd::OnWheel`,
//! `UIItemIdentifyWnd::OnWheel`, `UIGuildMemberManageWnd::OnWheel`, `UIMessengerGroupWnd::OnWheel`: `min(rows
//! shown - 1, 3)` sent to the bar as `UM_SCROLLVERT`). A list of lines (`UIListBox::OnWheel`: a menu, the
//! words of a message box) moves three lines. What an NPC says and a tip (`UITagTextViewer`) move 30 pixels,
//! by the wheel and by the arrows up and down alike (`UITagTextViewer::SendMsg`, `UM_SCROLLVERTPAGEUP` and
//! `UM_SCROLLVERTPAGEDOWN`: the 10 of `TagTextViewer::UIScroll::SendMsg` under it is never reached, for the
//! call is the viewer's own).

/// How many steps a notch of the wheel moves a list that shows so many rows (`min(shown - 1, 3)`): none for
/// a list of one row, as in the client.
pub fn rows_a_notch(shown: i32) -> i32 {
    (shown - 1).clamp(0, 3)
}

/// How many lines a notch moves a list of lines (`UIListBox::OnWheel`).
pub const LINES_A_NOTCH: i32 = 3;

/// How far a notch, or an arrow, moves a text of `UITagTextViewer`, in pixels (what an NPC said by the arrows:
/// `UISayDialogWnd::SendMsg` hands `UM_UPPUSH` and `UM_DOWNPUSH` to its viewer).
pub const TEXT_STEP: i32 = 30;

/// Where a list stands after it is moved `steps` steps of `step` pixels, down for more than none, from `top`:
/// on a whole step (a list found between two is taken as on the nearer one first), and never past its ends,
/// `most` being the farthest it goes. The far end is kept as it is, for a text's end is not a whole step.
pub fn moved(top: i32, step: i32, steps: i32, most: i32) -> i32 {
    let most = most.max(0);
    if step <= 0 {
        return top.clamp(0, most);
    }
    let at = if top >= most && steps > 0 { return most } else { (top + step / 2) / step };
    ((at + steps) * step).clamp(0, most)
}

/// How far the wheel must be turned for a notch, in the pixels a page counts it in: a notch of a mouse's
/// wheel is about a hundred.
pub const NOTCH: f64 = 100.0;

/// The notches in what the wheel was turned, added to what was left of the turns before (`kept`): how many
/// whole ones, down for more than none, and what is left for the next. A mouse's wheel gives one at once; a
/// touchpad, which sends many small turns, one for every hundred pixels. A turn the other way forgets what
/// was left.
pub fn notches(kept: f64, turned: f64) -> (i32, f64) {
    let kept = if kept * turned < 0.0 { 0.0 } else { kept };
    let all = kept + turned;
    let whole = (all / NOTCH).trunc();
    (whole as i32, all - whole * NOTCH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_notch_is_three_rows_or_one_less_than_shown() {
        assert_eq!(rows_a_notch(8), 3);
        assert_eq!(rows_a_notch(4), 3);
        assert_eq!(rows_a_notch(3), 2);
        assert_eq!(rows_a_notch(2), 1);
        // `min(1 - 1, 3)`: the wheel does nothing to a list of one row.
        assert_eq!(rows_a_notch(1), 0);
    }

    #[test]
    fn a_list_moves_by_whole_rows_and_stops_at_its_ends() {
        // Rows of 32, twelve of them in a list of five: 224 is the farthest.
        assert_eq!(moved(0, 32, 3, 224), 96);
        assert_eq!(moved(96, 32, 3, 224), 192);
        assert_eq!(moved(192, 32, 3, 224), 224);
        assert_eq!(moved(224, 32, -3, 224), 128);
        assert_eq!(moved(32, 32, -3, 224), 0);
        // Found between two rows (a page scrolled it so), it goes on from the nearer one.
        assert_eq!(moved(40, 32, 1, 224), 64);
        assert_eq!(moved(50, 32, 1, 224), 96);
        assert_eq!(moved(50, 32, -1, 224), 32);
    }

    #[test]
    fn a_text_keeps_its_own_end() {
        // What a smith says is 148 high in a window of 140: one notch shows its end, 8 down.
        assert_eq!(moved(0, TEXT_STEP, 1, 8), 8);
        assert_eq!(moved(8, TEXT_STEP, 1, 8), 8);
        assert_eq!(moved(8, TEXT_STEP, -1, 8), 0);
        // Nothing to move.
        assert_eq!(moved(0, TEXT_STEP, 1, 0), 0);
        assert_eq!(moved(0, TEXT_STEP, 1, -5), 0);
    }

    #[test]
    fn the_wheel_is_counted_in_notches() {
        // A mouse.
        assert_eq!(notches(0.0, 100.0), (1, 0.0));
        assert_eq!(notches(0.0, -120.0), (-1, -20.0));
        // A touchpad: nothing until a hundred pixels are turned.
        assert_eq!(notches(0.0, 40.0), (0, 40.0));
        assert_eq!(notches(40.0, 40.0), (0, 80.0));
        assert_eq!(notches(80.0, 40.0), (1, 20.0));
        // Turned back, what was left is forgotten.
        assert_eq!(notches(80.0, -40.0), (0, -40.0));
    }
}
