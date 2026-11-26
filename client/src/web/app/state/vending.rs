//! What the app keeps of merchants' shops: the signs in sight, a shop being made, one's own shop while it is
//! open, and a vendor's list with the basket of what is bought from it.

use std::collections::BTreeMap;

use crate::net::vending::{Listed, Sold};

/// Merchants' shops.
#[derive(Default)]
pub(in crate::web) struct Vending {
    /// The signs of the vendors in sight (`UIMerchantShopTitle`): the title, by the vendor's account.
    pub signs: BTreeMap<u32, String>,
    /// The shop being made (`UIMerchantShopMakeWnd`).
    pub making: Option<Stall>,
    /// One's own shop while it is open (`UIMerchantItemMyShopWnd`): what is left to sell.
    pub mine: Option<Vec<Sold>>,
    /// The title one's own shop was last opened with (`g_session.storeTitle`, kept as the opening is sent):
    /// its window is titled with it.
    pub title: String,
    /// The vendor whose list is open (`UIMerchantItemShopWnd`, `UIMerchantItemPurchaseWnd`).
    pub visit: Option<Visit>,
    /// What the box that asks how many is open for, and how many there are.
    pub counting: Option<(Crossing, u32)>,
    /// What is asked before a shop is opened.
    pub asks: Option<VendingAsk>,
    /// When the list of a vendor was last asked for: the client asks no oftener than once a second.
    pub asked_at: f64,
}

/// A shop being made.
#[derive(Default)]
pub(in crate::web) struct Stall {
    /// How many kinds of things it may sell.
    pub slots: u16,
    /// What was typed for its title, as of the last press in the window; between presses the page's field
    /// has it. The prices of the rows are kept the same way.
    pub title: String,
    pub rows: Vec<Stalled>,
}

/// A row of a shop being made: a thing of the cart (its place there), how many of it are sold, and the price
/// as it was typed.
pub(in crate::web) struct Stalled {
    pub index: u16,
    pub amount: u16,
    pub price: String,
}

/// A vendor's list, and what was put into the basket: `(row of the list, amount)`.
pub(in crate::web) struct Visit {
    pub list: Listed,
    /// What the vendor is called, if the world knew as the list came: the list's title has it.
    pub name: Option<String>,
    pub basket: Vec<(usize, u16)>,
    pub stage: Stage,
}

/// How far buying from a vendor has come.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum Stage {
    /// The list and the basket are open.
    Choosing,
    /// The basket's Buy was pressed: the window that asks whether to buy for that sum is open
    /// (`UIMerchantItemPurchaseMSGWnd`), and the two lists are put away.
    Confirming,
    /// The purchase was sent: nothing is open, and the list is kept only to name a thing the server says was
    /// not bought.
    Sent,
}

/// What the box that asks how many is asked of, in a shop: each is a state of the client's `UIItemDropCntWnd`.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum Crossing {
    /// A thing of the cart (its place there) goes into the shop being made (state 9).
    Stall(u16),
    /// A row of the shop being made goes back (state 10).
    Unstall(usize),
    /// A row of a vendor's list goes into the basket (state 12).
    Basket(usize),
    /// A row comes back out of the basket (state 11).
    Unbasket(usize),
}

/// What the making of a shop asks before it acts, in the client's message box.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum VendingAsk {
    /// A thing costs nothing: go on?
    Free,
}

impl Stall {
    /// How many of the thing at that place of the cart are in the shop being made.
    pub fn taken(&self, index: u16) -> u16 {
        self.rows.iter().filter(|row| row.index == index).map(|row| row.amount).sum()
    }
}

impl Visit {
    /// How many of a row of the list are in the basket.
    pub fn kept(&self, row: usize) -> u16 {
        self.basket.iter().find(|(kept, _)| *kept == row).map_or(0, |(_, amount)| *amount)
    }

    /// What everything in the basket costs.
    pub fn total(&self) -> u64 {
        self.basket.iter().filter_map(|(row, amount)| Some(u64::from(self.list.sold.get(*row)?.price) * u64::from(*amount))).sum()
    }
}
