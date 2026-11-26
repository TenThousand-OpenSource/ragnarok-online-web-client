//! What the app keeps of the cash shop.

use crate::cashshop::{Good, LISTS, Line};

/// What the shop asks or tells in a message box over its window (`UIWindowMgr::Confirm`).
pub(in crate::web) enum CashShopAsk {
    /// Something told, with OK alone: the words, and whether the box is titled "Error".
    Told(String, bool),
    /// Whether the purchase is meant: the words, and the free points it is to use.
    Buy(String, u32),
}

/// The cash shop (`UICashShopWnd`, and the lists `CSession::inventoryCategories` keeps for it).
pub(in crate::web) struct CashShop {
    pub open: bool,
    /// What the shop offers, list by list, as the server told; the last is what a search found.
    pub lists: [Vec<Good>; LISTS],
    pub tab: usize,
    pub page: usize,
    /// How many things a page shows: set as a tab is picked, and kept by a search (`m_itemsPerPage`).
    pub page_size: usize,
    /// The cash points and the free (kafra) points of the account, as the server last said.
    pub cash: u32,
    pub points: u32,
    /// What is to be bought, and the first kind of it that shows (`UICashShopWnd_Cart`).
    pub cart: Vec<Line>,
    pub top: usize,
    pub asks: Option<CashShopAsk>,
    /// What stands in the two fields, kept whenever something of the window is pressed: the window is drawn
    /// with them. The free points to use are "0" until typed over.
    pub sought: String,
    pub use_points: String,
    /// When the sale for one account was last asked for, by the page's clock: not again within five seconds.
    pub limited_asked: Option<f64>,
    /// The banner shown, and how many times a banner was shown since the window opened: a count that tells
    /// the turn that is due from one a pick has made late.
    pub banner: usize,
    pub banner_turns: u32,
}

impl Default for CashShop {
    fn default() -> Self {
        Self {
            open: false,
            lists: Default::default(),
            tab: 0,
            page: 0,
            page_size: crate::cashshop::page_size(0),
            cash: 0,
            points: 0,
            cart: Vec::new(),
            top: 0,
            asks: None,
            sought: String::new(),
            use_points: "0".to_owned(),
            limited_asked: None,
            banner: 0,
            banner_turns: 0,
        }
    }
}

impl CashShop {
    /// The things of the tab that is open (`UICashShopWnd::m_items`).
    pub fn shown(&self) -> &[Good] {
        &self.lists[self.tab.min(LISTS - 1)]
    }

    /// How many pages they make.
    pub fn pages(&self) -> usize {
        crate::cashshop::pages(self.shown().len(), self.page_size)
    }
}
