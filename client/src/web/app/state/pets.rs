//! What the app keeps of the pets: the player's pet and its window, the eggs, the slot machine of taming.

use crate::net::pets;

/// The slot machine shown when a monster is being tamed (the client's `UIPetTamingDeceiveWnd`): the pictures
/// of `slotmachine.spr` run in turn; a press starts the reels and asks the server, whose answer has them slow
/// down and stop on the picture of the catch or of the miss.
pub(in crate::web) struct Taming {
    /// The monster being tamed.
    pub(in crate::web::app) target: u32,
    /// Which action of the sprite plays: 0 waiting, 1 the reels turning.
    pub(in crate::web::app) state: usize,
    /// The client's counters: the picture shown is `tick / delay`.
    pub(in crate::web::app) tick: i32,
    pub(in crate::web::app) delay: i32,
    pub(in crate::web::app) longest: i32,
    /// The picture to stop on (-1: none yet).
    pub(in crate::web::app) wanted: i32,
    pub(in crate::web::app) started: bool,
    /// When it came to rest: it closes two seconds later.
    pub(in crate::web::app) rested: Option<f64>,
    /// The pictures of both actions, once read: each a data URL, its size and where its origin is.
    pub frames: [Vec<(String, (usize, usize), (f32, f32))>; 2],
    /// The picture on the page now.
    pub shown: (usize, usize),
    /// The page's timer that drives it.
    pub(in crate::web::app) timer: i32,
}

/// The pseudo-skill the client aims with when a monster is to be tamed (`Zc_Start_Capture`).
pub(in crate::web::app) const TAME: u16 = 10000;

/// The player's pet: catching one, hatching it, and its window.
#[derive(Default)]
pub(in crate::web) struct Pets {
    /// The slot machine of taming a monster, while it is open.
    pub taming: Option<Taming>,
    /// The pet eggs offered for hatching (their places in the inventory), while the incubator's list is open,
    /// and which of them is picked.
    pub eggs: Option<Vec<u16>>,
    pub egg_picked: usize,
    /// The pet, while one is out of its egg, and the unit on the map that is it.
    pub pet: Option<pets::Pet>,
    pub unit: u32,
    /// The pet window is open; its menu of commands is; what it asks before acting (1 whether to feed, 2
    /// whether to give the name typed, kept in `named`).
    pub open: bool,
    pub menu: bool,
    pub asks: u8,
    pub named: String,
    /// The answer to feeding the pet (whether it ate, and its food), until the chat has told of it: the food's
    /// name may arrive a moment later.
    pub fed: Option<(bool, u32)>,
    /// The pet window was opened by itself for this pet's low intimacy: it is, once for a pet.
    pub alerted: bool,
    /// The menu of the pet, opened with the right button on it: where on the stage.
    pub menu_at: Option<(i32, i32)>,
    /// How hungry the pet was before its hunger last changed: what it feels on being fed goes by that.
    pub hunger_before: i32,
    /// When the pet last told of its master's danger (milliseconds since 1970): at most once in five seconds.
    pub warned_at: f64,
}
