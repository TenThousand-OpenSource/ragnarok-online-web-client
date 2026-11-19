//! What the app keeps of the storage: the Kafra's, or a guild's, which is the same window under another name.

use super::*;
use crate::storage::Sorted;

/// A storage while it is open.
#[derive(Default)]
pub(in crate::web) struct Stored {
    /// What the server calls it ("Storage", "Guild Storage"): the title of its window.
    pub name: String,
    pub items: Vec<net::inventory::Item>,
    /// How many kinds of things are in it, and how many fit.
    pub count: (u16, u16),
}

/// Which way a thing goes that the box asks how many of.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum Stowing {
    /// Out of the inventory into the storage: its place in the inventory.
    In(u16),
    /// Out of the storage into the inventory: its place in the storage.
    Out(u16),
}

/// The storage and its window.
#[derive(Default)]
pub(in crate::web) struct Storage {
    pub open: Option<Stored>,
    /// Which of the tabs down the window's side is open.
    pub tab: usize,
    /// Which tabs are shown in windows of their own beside it: a bit each (the client's `openTabs`).
    pub apart: u8,
    pub sorted: Sorted,
    /// The box of the three orders is open.
    pub sorting: bool,
    /// What was searched for, while the window of what was found is open.
    pub found: Option<String>,
    /// What the box that asks how many is open for, and how many there are.
    pub counting: Option<(Stowing, u16)>,
}
