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
//!
//! The bar beside them (`UIScrollBar`, upright) is `Bar`: how long its handle is and where it stands, which
//! part of it a press is on, and how far a drag of the handle moves the list. The bar counts in the list's
//! own steps: rows for a list of rows, lines for a list of lines, pixels for a text.

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

/// The client's upright scroll bar as it stands at a moment (`UIScrollBar`): `length` long with an arrow of
/// `ARROW` at each end, beside a list of `total` steps of which `visible` show, the first shown being
/// `position` of `maximum` (`UIScrollBar::SetRange`, `SetPosition`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bar {
    pub length: i32,
    pub maximum: i32,
    pub visible: i32,
    pub total: i32,
    pub position: i32,
}

/// How wide the bar is and how long each of its arrows (`UIScrollBar::UIScrollBar`: `thickness` and
/// `arrowLength`, 13 both, the size of `scroll0up.bmp`).
pub const ARROW: i32 = 13;

/// What of the bar a press is on (`UIScrollBar::Part`, the client's `SB_UPBTN`…`SB_DNBTN`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Up,
    PageUp,
    Thumb,
    PageDown,
    Down,
}

impl Bar {
    /// The bar of a list `height` high and `whole` high with all in it, scrolled `top` down, each step of
    /// the list `unit` pixels: what the client's windows hand `SetRange` (the steps past the last shown, the
    /// steps shown, all of them), in the measures a page has of a list.
    pub fn of(length: i32, unit: i32, height: i32, whole: i32, top: i32) -> Self {
        let unit = unit.max(1);
        let steps = |pixels: i32| (pixels.max(0) + unit - 1) / unit;
        let maximum = steps(whole - height);
        Self { length, maximum, visible: (height / unit).max(1), total: steps(whole), position: ((top + unit / 2) / unit).clamp(0, maximum) }
    }

    /// Whether the bar is drawn and takes presses: not when there is nowhere to scroll (`UIScrollBar::Paint`
    /// and `OnLBtnDown` with `maximum` 0).
    pub fn shown(&self) -> bool {
        self.maximum > 0
    }

    /// How long the handle is (`UIScrollBar::ThumbLength`): the part of the way between the arrows that the
    /// steps shown are of all, 8 at the least.
    pub fn thumb(&self) -> i32 {
        let way = self.length - 2 * ARROW;
        let length = if self.total != 0 { way * self.visible / self.total } else { way };
        length.max(8)
    }

    /// How far the handle travels from end to end (`UIScrollBar::Travel`).
    pub fn travel(&self) -> i32 {
        self.length - 2 * ARROW - self.thumb()
    }

    /// Where the handle's top is, counted from the bar's (`UIScrollBar::Paint`: the arrow, then
    /// `ThumbOffset`).
    pub fn thumb_at(&self) -> i32 {
        ARROW + if self.maximum > 0 { self.travel() * self.position / self.maximum } else { 0 }
    }

    /// What a press `y` down the bar is on (`UIScrollBar::Part`); nothing on a bar not shown or off its ends.
    pub fn part(&self, y: i32) -> Option<Part> {
        if !self.shown() || y < 0 || y >= self.length {
            return None;
        }
        Some(match y {
            y if y < ARROW => Part::Up,
            y if y < self.thumb_at() => Part::PageUp,
            y if y < self.thumb_at() + self.thumb() => Part::Thumb,
            y if y < self.length - ARROW => Part::PageDown,
            _ => Part::Down,
        })
    }

    /// How many steps a press on that part moves the list, down for more than none, a page being `page`
    /// steps (`UIScrollBar::OnLBtnDown`: an arrow is one step, `verticalStep`); none for the handle, which
    /// is dragged.
    pub fn pressed(part: Part, page: i32) -> i32 {
        match part {
            Part::Up => -1,
            Part::PageUp => -page,
            Part::Thumb => 0,
            Part::PageDown => page,
            Part::Down => 1,
        }
    }

    /// How many steps more the list is to move when the handle, taken at a place, is found `moved` pixels
    /// down from there, `dragged` steps having been moved since it was taken (`UIScrollBar::OnMouseMove`:
    /// `moved * (maximum + 1) / Travel() - dragAmount`). Nothing for a handle with nowhere to go.
    pub fn dragged(&self, moved: i32, dragged: i32) -> i32 {
        if self.travel() <= 0 { 0 } else { moved * (self.maximum + 1) / self.travel() - dragged }
    }
}

/// How many steps a press on the bar's way above or below the handle moves a list that shows so many: one
/// less than it shows (`UIListBox::SendMsg`, `UINewSkillListWnd::SendMsg`, `UITreeView::SendMsg`,
/// `UM_SCROLLVERTPAGEUP` and `UM_SCROLLVERTPAGEDOWN`). A text moves `TEXT_STEP` pixels (`UITagTextViewer`).
pub fn page(shown: i32) -> i32 {
    (shown - 1).max(1)
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

    #[test]
    fn the_bar_is_the_clients() {
        // Twelve rows of 32 in a list of five, a bar of 160: 134 between the arrows, of which 5/12 is handle.
        let bar = Bar::of(160, 32, 160, 384, 0);
        assert_eq!(bar, Bar { length: 160, maximum: 7, visible: 5, total: 12, position: 0 });
        assert_eq!((bar.thumb(), bar.travel(), bar.thumb_at()), (55, 79, 13));
        let low = Bar { position: 7, ..bar };
        assert_eq!(low.thumb_at(), 13 + 79);
        assert_eq!(Bar { position: 3, ..bar }.thumb_at(), 13 + 79 * 3 / 7);
        // The parts of it, from the top.
        assert_eq!(bar.part(-1), None);
        assert_eq!(bar.part(0), Some(Part::Up));
        assert_eq!(bar.part(13), Some(Part::Thumb));
        assert_eq!(bar.part(67), Some(Part::Thumb));
        assert_eq!(bar.part(68), Some(Part::PageDown));
        assert_eq!(bar.part(146), Some(Part::PageDown));
        assert_eq!(bar.part(147), Some(Part::Down));
        assert_eq!(bar.part(160), None);
        assert_eq!(low.part(20), Some(Part::PageUp));
        // A list that shows all it has: no bar, no presses.
        let still = Bar::of(160, 32, 160, 160, 0);
        assert!(!still.shown());
        assert_eq!(still.part(5), None);
        // A long list: the handle is never under 8.
        assert_eq!(Bar::of(160, 32, 160, 32000, 0).thumb(), 8);
    }

    #[test]
    fn a_press_on_the_bar_and_a_drag_of_its_handle() {
        assert_eq!(page(5), 4);
        assert_eq!(page(1), 1);
        assert_eq!(Bar::pressed(Part::Up, 4), -1);
        assert_eq!(Bar::pressed(Part::PageUp, 4), -4);
        assert_eq!(Bar::pressed(Part::PageDown, 4), 4);
        assert_eq!(Bar::pressed(Part::Down, 4), 1);
        assert_eq!(Bar::pressed(Part::Thumb, 4), 0);
        // The handle of the bar above, 79 of travel for 8 places: 10 pixels down is a row, and once that
        // row is moved the same place asks for no more.
        let bar = Bar::of(160, 32, 160, 384, 0);
        assert_eq!(bar.dragged(9, 0), 0);
        assert_eq!(bar.dragged(10, 0), 1);
        assert_eq!(bar.dragged(10, 1), 0);
        assert_eq!(bar.dragged(79, 1), 7);
        assert_eq!(bar.dragged(-20, 0), -2);
        // A text counts in pixels: 148 high in a viewer of 140.
        let text = Bar::of(140, 1, 140, 148, 8);
        assert_eq!((text.maximum, text.visible, text.total, text.position), (8, 140, 148, 8));
        assert_eq!(text.thumb(), 114 * 140 / 148);
    }
}
