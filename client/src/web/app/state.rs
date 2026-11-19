//! What the app keeps, in groups: one for each part of the game, each in a file of its system's name in
//! `state/`. Here is what the way in keeps (the game entered, a question asked, the character being made).
//! `App` itself, which holds one of each group, is in `app.rs`.

use super::*;

mod ai;
mod books;
mod broadcast;
mod cart;
mod chat;
mod chat_rooms;
mod choosing;
mod commands;
mod cooldowns;
mod emotions;
mod exchange;
mod family;
mod fight;
mod friends;
mod guide;
mod guild;
mod homunculus;
mod inventory;
mod link;
mod mail;
mod marks;
mod mercenary;
mod notices;
mod party;
mod pets;
mod player_menu;
mod pvp;
mod quests;
mod scripted;
mod settings;
mod shops;
mod skills;
mod status;
mod statuses;
mod storage;
mod talk;
mod tips;
mod unread;
mod vending;
mod viewequip;
mod whispers;
mod worldmap;

// Every system's group, by its own names; the books' under names that say whose they are.
pub(in crate::web) use {ai::*, broadcast::*, cart::*, chat::*, chat_rooms::*, choosing::*, commands::*, cooldowns::*};
pub(in crate::web) use {emotions::*, exchange::*, family::*, fight::*, friends::*, guide::*, guild::*, homunculus::*};
pub(in crate::web) use {inventory::*, link::*, mail::*, marks::*, mercenary::*, notices::*, party::*, pets::*};
pub(in crate::web) use {player_menu::*, pvp::*, quests::*, scripted::*, settings::*, shops::*, skills::*, status::*};
pub(in crate::web) use {statuses::*, storage::*, talk::*, tips::*, unread::*, vending::*, viewequip::*, whispers::*};
pub(in crate::web) use worldmap::*;
pub(in crate::web) use books::{Aloud as BookAloud, BOOK_SAID, BookButton, BookNews, Books, OpenBook, Reading};

pub(in crate::web) struct Game {
    pub name: String,
    pub job: u16,
    pub map: String,
    pub x: u16,
    pub y: u16,
    /// The map's size in cells, once it is drawn: for the minimap.
    pub map_size: Option<(usize, usize)>,
}

/// A question over the window, waiting for OK or Cancel; each is about the character with that ID.
#[derive(Clone, Copy)]
pub(in crate::web) enum Ask {
    /// Start the wait for a deletion?
    Reserve(u32),
    /// Delete for good?
    Delete(u32),
    /// The same, for an account that has a date of birth to confirm it with.
    Birthdate(u32),
    /// A key set in the Shortcut Settings window is another entry's already: take it from that one? The entry
    /// it is set for, the one that has it (each a table and a place), and the key.
    Rebind((usize, usize), (usize, usize), Chord),
}

/// The character being made.
pub(in crate::web) struct Make {
    pub doram: bool,
    pub sex: u8,
    pub hair_style: u16,
    pub hair_colour: u16,
    /// Which way the preview looks: 0 is south, then clockwise.
    pub direction: u8,
}

impl Default for Make {
    fn default() -> Self {
        Self { doram: false, sex: 1, hair_style: 1, hair_colour: 0, direction: 0 }
    }
}
