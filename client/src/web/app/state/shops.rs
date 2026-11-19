//! What the app keeps of dealing with an NPC's shop.

use super::*;

/// An NPC's shop, as far as the dealing has come.
pub(in crate::web) enum Shop {
    /// The shop asks whether to buy or to sell.
    Choose(u32),
    Buy(Vec<trade::Offer>),
    /// What the shop takes of what is carried.
    Sell(Vec<trade::Bid>),
}

/// What the box that asks how many is asked of, in a shop: each is a state of the client's `UIItemDropCntWnd`.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum Across {
    /// A row of the list goes into the basket of what is bought (state 1) or sold (state 2).
    In(usize),
    /// A row comes back out of the basket (states 3 and 4).
    Out(usize),
}

/// Dealing with an NPC's shop.
#[derive(Default)]
pub(in crate::web) struct Deal {
    pub shop: Option<Shop>,
    /// What was put into the basket to buy or to sell: `(row of the list, amount)`.
    pub basket: Vec<(usize, u16)>,
    /// What the box that asks how many is open for, with the number it begins with.
    pub counting: Option<(Across, u32)>,
    /// The list's box "Toggle Item Amount" (the client's `dragAllItems`): ticked, a stack goes across whole
    /// without the question, and a double click sends it.
    pub whole_stacks: bool,
}
