//! What the windows show of an item or a skill, written once: its picture by its number, and the grid of
//! cells the windows made of the client's `UIItemBaseWnd` keep their things in (the cart, one's own shop).
//!
//! (A row of a list is `hud/lists.rs`.)

use super::*;
use crate::item_names::Named;

/// A thing in a cell of a grid (`UIItemBaseWnd::Paint`): its picture with how many there are, in the cell at
/// that `place` of a grid `columns` wide; `marks` is what the cell's tag says more (what it does when
/// dragged), and `says` what it says when pointed at: in the client's tooltip, 16 above the cell
/// (`UIItemWnd::OnMouseHover`: the cell's left, its top less 16), white, or grey for a thing not identified
/// (`CSession::GetItemNameColor`).
pub(in crate::web) fn grid_cell(place: usize, columns: i32, item: &Item, amount: u16, marks: &str, says: &str) -> String {
    format!(
        r#"<div class="a cell" style="left:{}px;top:{}px" {marks} {} {}>{}{}</div>"#,
        place as i32 % columns * 32,
        place as i32 / columns * 32,
        item_marks(item.id, item.identified),
        tip_in(says, if item.identified { "#fff" } else { crate::tooltip::UNKNOWN }),
        App::icon_of(item),
        count(amount)
    )
}

/// The grid of such a window: `columns` cells of 32 across from 40 in, `rows` of them showing under the
/// title, scrolled when it has more, with the client's bar a pixel right of it from `bar` down (17 in the
/// inventory, `UIItemWnd::OnCreate`: `Configure(width - 15, 17, …)`; 14 in the windows of `UIItemBaseWnd`).
pub(in crate::web) fn grid_of(columns: i32, rows: i32, bar: i32, cells: &[String]) -> String {
    let filled = (cells.len().div_ceil(columns.max(1) as usize) as i32).max(rows);
    format!(
        r#"<div class="a scroll" style="{};background:url('{}')" {}><div style="position:relative;height:{}px">{}</div></div>{}"#,
        at(40, 17, 32 * columns, 32 * rows),
        picture("basic_interface/itemwin_mid.bmp"),
        rows_stepped(32, rows),
        filled * 32,
        cells.concat(),
        scroll_bar(41 + 32 * columns, bar, 32 * rows, 32)
    )
}

/// What a cell or a row says of the item in it for the right button and for the picture that follows the
/// pointer when it is dragged (`data-info`: the item, with a `?` while it is not identified).
pub(in crate::web) fn item_marks(id: u32, identified: bool) -> String {
    format!(r#"data-info="{id}{}""#, if identified { "" } else { "?" })
}

/// What a thing says when pointed at where there may be several of it: its name and how many there are.
pub(in crate::web) fn amount_tip(name: &str, amount: impl std::fmt::Display) -> String {
    format!("{name} : {amount} {}", tr("ea"))
}

/// What an item is called by its number alone, as the client names the rows of these lists
/// (`item.SetItemId(id)`, identified, `DisplayName(0)`).
pub(in crate::web) fn called(id: u32) -> String {
    data::item_title(&Named { id, identified: true, ..Named::default() }, false)
}

impl App {
    /// The picture of an item or a skill, or nothing while its name is not known.
    pub(in crate::web) fn icon(resource: Option<&str>) -> String {
        match resource {
            // An item this client's tables do not have is shown as the original shows it: an apple.
            Some(resource) if resource.is_empty() => Self::icon(Some("사과")),
            Some(resource) => format!(r#"<img src="{}" alt="" draggable="false" onerror="this.remove()">"#, picture(&format!("item/{resource}.bmp"))),
            None => String::new(),
        }
    }

    /// How many of an item the character carries without wearing them (arrows readied are carried still).
    pub(in crate::web) fn carried(&self, id: u32) -> u32 {
        self.bag.items.iter().filter(|item| item.id == id && item.in_bag()).map(|item| u32::from(item.amount)).sum()
    }

    /// An item's name as the client writes it everywhere (`ITEM_INFO::DisplayName`: `data::item_title`,
    /// `crate::item_names`): its refinement, its cards' words, its slots, who made it.
    pub(in crate::web) fn item_name(item: &Item) -> String {
        data::item_title(&item.into(), false)
    }

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
