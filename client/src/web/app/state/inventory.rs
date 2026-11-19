//! What the app keeps of the inventory: what is carried, and the windows of it.

use super::*;

/// What the character carries, and the windows of it.
#[derive(Default)]
pub(in crate::web) struct Bag {
    pub items: Vec<Item>,
    /// The Inventory window is open, and which of its tabs.
    pub open: bool,
    pub tab: Tab,
    /// The places in the inventory of the things kept under the tab "Fav", and whether that tab's button
    /// "Align items" is on (the client's `m_sortPrivateTab`).
    pub favourites: Vec<u16>,
    pub aligned: bool,
    pub equipment_open: bool,
    /// The Equipment window shows its Costume page.
    pub costumes: bool,
    /// The item whose description is open, and whether it is identified.
    pub info: Option<(u32, bool)>,
    /// The card whose illustration is open (the client's `UICardItemIllustWnd`).
    pub illust: Option<u32>,
    /// The items a magnifier can identify (their places in the inventory) while its list is open, and the row
    /// picked in it (none at first, as in the client's `UIItemIdentifyWnd`).
    pub identify: Option<Vec<u16>>,
    pub identify_picked: Option<usize>,
    /// The thing being dropped while its count is asked for: where it is carried, and how many there are.
    pub throwing: Option<(u16, u16)>,
    /// What was picked up last, and whether the chat has told of it (its name may arrive a moment later).
    pub obtained: Option<(u32, u16, bool)>,
    /// When it was picked up (milliseconds since 1970): the note of it shows for five seconds from then.
    pub obtained_at: f64,
    pub told_obtained: bool,
}

impl Bag {
    /// The first of an item (by its number) that is carried without being worn.
    pub fn first_unworn(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id && item.worn == 0)
    }

    /// The thing carried at a place of the inventory.
    pub fn at(&self, index: u16) -> Option<&Item> {
        self.items.iter().find(|item| item.index == index)
    }

    /// The same, to be changed.
    pub fn at_mut(&mut self, index: u16) -> Option<&mut Item> {
        self.items.iter_mut().find(|item| item.index == index)
    }
}
