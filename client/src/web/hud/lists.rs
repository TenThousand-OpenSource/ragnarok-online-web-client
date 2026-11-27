//! The client's windows that list things a row to a thing (`UIItemBaseWnd` and those made of it: the list and
//! the basket of a shop, the storage and the windows that show a part of it): the frame they share and a row
//! of their lists, each written once.

use super::*;

/// What every window of this kind has under its list (`UIItemBaseWnd::Paint`): the title bar, the strips at
/// its sides (`itemwin_left.bmp`, `itemwin_right.bmp`, 20 wide, down to the bar at the bottom), white between
/// them, and the bar its buttons stand on, 21 high.
pub(super) fn list_frame(width: i32, height: i32, title: &str) -> String {
    let side = |x: i32, name: &str| format!(r#"<div class="a" style="{};background:url('{}') 0 0 repeat-y"></div>"#, at(x, 17, 20, height - 38), picture(name));
    format!(
        "{}{}{}{}{}",
        title_bar(width, title),
        fill(0, 17, width, height - 38, WHITE),
        side(0, "basic_interface/itemwin_left.bmp"),
        side(width - 20, "basic_interface/itemwin_right.bmp"),
        button_bar(width, height),
    )
}

/// What marks a list of rows, each `row` high and `shown` of them showing, as one the wheel moves by whole
/// rows, as the client moves all of its own (`crate::scrolling::rows_a_notch`): a row is never cut at the
/// list's top. (Which lists of the windows do not have it yet: `docs/DEBTS.md`.)
pub(in crate::web) fn rows_stepped(row: i32, shown: i32) -> String {
    stepped(row, crate::scrolling::rows_a_notch(shown))
}

/// What marks a list of lines of the client's `UIListBox`, each `line` high: the wheel moves it three lines.
pub(in crate::web) fn lines_stepped(line: i32) -> String {
    stepped(line, crate::scrolling::LINES_A_NOTCH)
}

/// What marks a text of the client's `UITagTextViewer` (what an NPC says, a page of a tip): the wheel moves it
/// 30 pixels a notch.
pub(in crate::web) fn text_stepped() -> String {
    stepped(crate::scrolling::TEXT_STEP, 1)
}

/// The client's scroll bar (`UIScrollBar`, upright: 13 wide, `scroll0mid.bmp` down it between an arrow at
/// each end, the handle of `scroll0bar_up`, `_mid` and `_down.bmp`) for the list written RIGHT BEFORE it, each
/// step of which is `unit` pixels: at `x`, `y` of the window, `length` long. It is the one bar of every
/// list and text that has one; the page stands its handle and takes its presses (`app/scrollbars.rs`, by the
/// rules of `crate::scrolling::Bar`), and it shows only while its list has more than fits.
pub(in crate::web) fn scroll_bar(x: i32, y: i32, length: i32, unit: i32) -> String {
    bar_of(x, y, length, unit, "")
}

/// The same for a text of the client's `UITagTextViewer`, which its bar counts in pixels: an arrow moves it
/// one, a press above or below the handle 30 (`UITagTextViewer::SendMsg`).
pub(in crate::web) fn text_bar(x: i32, y: i32, length: i32) -> String {
    bar_of(x, y, length, 1, &format!(r#" data-page="{}""#, crate::scrolling::TEXT_STEP))
}

/// The same beside a list that begins at its end and is scrolled back from there, as the chat's lines are.
pub(in crate::web) fn end_bar(x: i32, y: i32, length: i32, unit: i32) -> String {
    bar_of(x, y, length, unit, " data-reversed")
}

fn bar_of(x: i32, y: i32, length: i32, unit: i32, more: &str) -> String {
    // The arrow up, the way, the arrow down, and last the handle.
    format!(
        r#"<div class="a sbar" style="{};visibility:hidden" data-bar="{unit}"{more}>{}</div>"#,
        at(x, y, crate::scrolling::ARROW, length),
        r#"<i class="u"></i><i class="m"></i><i class="d"></i><i class="h"></i>"#
    )
}

/// The list of such a window: `rows` rows of 32 from the title down, from `x` to `width`, scrolled when it
/// has more, by whole rows (`rows_stepped`), with the client's bar at its right: 15 from the window's right
/// edge, which is where the list ends, from 14 down and as long as the list (the windows made of
/// `UIItemBaseWnd`: `Configure(width - 15, 14, 1, height - 38)`).
pub(super) fn list_of(x: i32, width: i32, rows: i32, lines: &str, marks: &str) -> String {
    format!(
        r#"<div class="a scroll" style="{}" {} {marks}><div style="min-height:{}px">{lines}</div></div>{}"#,
        at(x, 17, width, rows * 32),
        rows_stepped(32, rows),
        rows * 32,
        scroll_bar(x + width, 14, rows * 32, 32)
    )
}

/// What a row of a list shows of its thing.
pub(super) struct Row<'a> {
    /// The thing's picture, by its resource name; how many there are, for a list that says so.
    pub resource: Option<&'a str>,
    pub amount: Option<u32>,
    pub name: &'a str,
    /// Where the name begins, counted from the cell's left edge, and how wide it may be.
    pub name_at: (i32, i32),
    /// What is written at the right end of the row: a price.
    pub aside: Option<Aside<'a>>,
    /// What the row's tag says more: what it does when dragged, double-clicked, pressed with the right button.
    pub marks: &'a str,
}

/// What stands at the right end of a row: written already as the page takes it (a price may have a colour and
/// a rim of its own), ending `inset` from the list's right edge, in that `colour` (`inherit` for the row's).
pub(super) struct Aside<'a> {
    pub html: &'a str,
    pub inset: i32,
    pub colour: &'a str,
}

/// A row of a list (`UIItemShopWnd::Paint` and the windows like it): the cell of the inventory at its left
/// with the thing's picture 4 in and 4 down and how many there are, the name 12 down from the row's top, and
/// the price ending at the right. The row under the pointer's press is marked as the client marks the one it
/// hovers over.
pub(super) fn item_row(row: &Row) -> String {
    [
        format!(r#"<div class="row" tabindex="-1" style="position:relative;height:32px;cursor:default" {}>"#, row.marks),
        format!(
            r#"<div class="a cell" style="left:0;top:0;background:url('{}')">{}{}</div>"#,
            picture("basic_interface/itemwin_mid.bmp"),
            App::icon(row.resource),
            row.amount.map_or(String::new(), count)
        ),
        format!(r#"<div class="a t" style="left:{}px;top:12px;width:{}px;overflow:hidden;white-space:nowrap;"#, row.name_at.0, row.name_at.1),
        format!(
            r#"color:inherit">{}</div>{}</div>"#,
            escape(row.name),
            row.aside.as_ref().map_or(String::new(), |Aside { html, inset, colour }| format!(
                r#"<div class="a t" style="right:{inset}px;top:12px;white-space:nowrap;color:{colour}">{html}</div>"#
            ))
        )
    ].concat()
}

/// The sum under a basket (`UIItemPurchaseWnd::Paint` and the windows like it): "Total : n Zeny", 12 in.
pub(super) fn total_line(y: i32, total: i64) -> String {
    text(12, y, BLACK, &format!("Total : {} Zeny", grouped(total)))
}
