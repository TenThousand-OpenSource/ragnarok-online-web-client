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

/// The list of such a window: `rows` rows of 32 from the title down, from `x` to `width`, scrolled when it
/// has more. The client scrolls it by a bar at the right, 15 wide; here the list itself is scrolled.
pub(super) fn list_of(x: i32, width: i32, rows: i32, lines: &str, marks: &str) -> String {
    format!(r#"<div class="a scroll" style="{}" {marks}><div style="min-height:{}px">{lines}</div></div>"#, at(x, 17, width, rows * 32), rows * 32)
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
    format!(
        r#"<div class="row" tabindex="-1" style="position:relative;height:32px;cursor:default" {}><div class="a cell" style="left:0;top:0;background:url('{}')">{}{}</div><div class="a t" style="left:{}px;top:12px;width:{}px;overflow:hidden;white-space:nowrap;color:inherit">{}</div>{}</div>"#,
        row.marks,
        picture("basic_interface/itemwin_mid.bmp"),
        App::icon(row.resource),
        row.amount.map_or(String::new(), count),
        row.name_at.0,
        row.name_at.1,
        escape(row.name),
        row.aside.as_ref().map_or(String::new(), |Aside { html, inset, colour }| format!(r#"<div class="a t" style="right:{inset}px;top:12px;white-space:nowrap;color:{colour}">{html}</div>"#)),
    )
}

/// The sum under a basket (`UIItemPurchaseWnd::Paint` and the windows like it): "Total : n Zeny", 12 in.
pub(super) fn total_line(y: i32, total: i64) -> String {
    text(12, y, BLACK, &format!("Total : {} Zeny", grouped(total)))
}
