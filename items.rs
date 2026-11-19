//! What the windows show of an item or a skill, written once: its picture by its number, and the grid of
//! cells the windows made of the client's `UIItemBaseWnd` keep their things in (the cart, one's own shop).
//!
//! (A row of a list is `hud/lists.rs`; `App::icon`, `item_name` and `carried` are still in
//! `hud/inventory.rs`, and `item_marks` in `hud/lists.rs`.)

// Not taken up by the windows yet (they are being moved onto these).
#![allow(dead_code)]

use super::*;

/// A thing in a cell of a grid (`UIItemBaseWnd::Paint`): its picture with how many there are, in the cell at
/// that `place` of a grid `columns` wide; `marks` is what the cell's tag says more (what it does when
/// dragged), and `tip` what it says when pointed at.
pub(in crate::web) fn grid_cell(place: usize, columns: i32, item: &Item, amount: u16, marks: &str, tip: &str) -> String {
    format!(
        r#"<div class="a cell" style="left:{}px;top:{}px" {marks} {} title="{}">{}{}</div>"#,
        place as i32 % columns * 32,
        place as i32 / columns * 32,
        item_marks(item.id, item.identified),
        escape(tip),
        App::icon_of(item),
        count(amount)
    )
}

/// The grid of such a window: `columns` cells of 32 across from 40 in, `rows` of them showing under the
/// title, scrolled when it has more (the client scrolls it by a bar at the right).
pub(in crate::web) fn grid_of(columns: i32, rows: i32, cells: &[String]) -> String {
    let filled = (cells.len().div_ceil(columns.max(1) as usize) as i32).max(rows);
    format!(
        r#"<div class="a scroll" style="{};background:url('{}')"><div style="position:relative;height:{}px">{}</div></div>"#,
        at(40, 17, 32 * columns, 32 * rows),
        picture("basic_interface/itemwin_mid.bmp"),
        filled * 32,
        cells.concat()
    )
}

impl App {
    /// The picture of an item by its number: the one it has while it is not identified, if it is not; nothing
    /// while the tables do not know the item.
    pub(in crate::web) fn item_icon(id: u32, identified: bool) -> String {
        Self::icon(data::item_as(id, identified).map(|(_, resource)| resource).as_deref())
    }

    /// The picture of an item carried, kept or offered.
    pub(in crate::web) fn icon_of(item: &Item) -> String {
        Self::item_icon(item.id, item.identified)
    }

    /// The picture of a skill by its number (its file is named in small letters); nothing while the tables do
    /// not know the skill.
    pub(in crate::web) fn skill_icon(id: u16) -> String {
        Self::icon(data::skill(id).as_ref().map(|(resource, ..)| resource.to_lowercase()).as_deref())
    }
}
