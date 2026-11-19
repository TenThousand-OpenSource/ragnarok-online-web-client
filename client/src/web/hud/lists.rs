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
        r#"{}<div class="a" style="{};background:#fff"></div>{}{}{}{}{}"#,
        title_bar(width, title),
        at(0, 17, width, height - 38),
        side(0, "basic_interface/itemwin_left.bmp"),
        side(width - 20, "basic_interface/itemwin_right.bmp"),
        art("basic_interface/btnbar_left.bmp", 0, height - 21, 21, 21),
        art("basic_interface/btnbar_mid.bmp", 21, height - 21, width - 42, 21),
        art("basic_interface/btnbar_right.bmp", width - 21, height - 21, 21, 21),
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
    /// What is written at the right end of the row, `inset` from the list's right edge: a price.
    pub price: Option<(&'a str, i32)>,
    /// What the row's tag says more: what it does when dragged, double-clicked, pressed with the right button.
    pub marks: &'a str,
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
        row.price.map_or(String::new(), |(price, inset)| format!(r#"<div class="a t" style="right:{inset}px;top:12px;white-space:nowrap;color:inherit">{}</div>"#, escape(price))),
    )
}

/// What a cell or a row says of the item in it for the right button and for the picture that follows the
/// pointer when it is dragged (`data-info`: the item, with a `?` while it is not identified).
pub(super) fn item_marks(id: u32, identified: bool) -> String {
    format!(r#"data-info="{id}{}""#, if identified { "" } else { "?" })
}
