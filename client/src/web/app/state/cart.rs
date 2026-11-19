//! What the app keeps of the merchant's cart.

use std::collections::HashMap;

use crate::net::cart::Fill;
use crate::net::inventory::Item;

/// The skill Change Cart (`MC_CHANGECART`).
pub(in crate::web) const CHANGE_CART: u16 = 154;

/// The cart (the client's `UIMerchantItemWnd`, and the cart's kind its session keeps).
#[derive(Default)]
pub(in crate::web) struct Cart {
    /// Which cart the character pushes, as the state `EFST_ON_PUSH_CART` told: nought for none.
    pub kind: u8,
    /// What lies in it, each thing with its place there, and how full it is.
    pub items: Vec<Item>,
    pub fill: Fill,
    /// Its window is open.
    pub open: bool,
    /// What the box that asks how many is open for, and how many there are.
    pub counting: Option<(Carting, u16)>,
    /// The looks offered for the cart, while their window is open (`UISelectCartWnd`).
    pub looks: Option<Looks>,
}

/// Which way a thing goes that the cart's box asks how many of: each is a state of the client's
/// `UIItemDropCntWnd` (8 into the cart, 6 out of it into the inventory, 7 out of it into the storage).
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum Carting {
    /// Out of the inventory into the cart: its place in the inventory.
    In(u16),
    /// Out of the cart into the inventory: its place in the cart.
    Out(u16),
    /// Out of the storage into the cart: its place in the storage.
    FromStorage(u16),
    /// Out of the cart into the storage: its place in the cart.
    ToStorage(u16),
}

/// The looks offered for the cart.
#[derive(Default)]
pub(in crate::web) struct Looks {
    /// What the answer must name when the server offered them (ZC_SELECTCART); nought when the skill Change
    /// Cart opened the window.
    pub identity: u32,
    pub kinds: Vec<u8>,
    /// The first picture of each kind's sprite, once it is read, and where in it the cart stands.
    pub pictures: HashMap<u8, (String, (f32, f32))>,
}
