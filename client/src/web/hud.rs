//! The windows of the game itself. This file has the list of them and what they all are made of; a window is
//! drawn by its system's file here (`hud/<system>.rs`) and listed, with its place, in `hud/windows.rs`.
//!
//! What the windows share is in the files named for it, and is in scope in every file here through
//! `use super::*;`: `frames.rs` (art, title bar, grounds, button bar, system box, cross), `buttons.rs`
//! (picture buttons and the button with words), `texts.rs` (fonts, texts at a place, titles, coloured lines),
//! `checks.rs` (check boxes, radio buttons), `gauges.rs`, `items.rs` (an item's and a skill's picture, the
//! grid of cells), `fields.rs` (what is typed into), `colours.rs`; `boxes.rs` (the boxes that ask, menus,
//! the list to pick from) and `lists.rs` (the windows that list things a row to a thing).
//!
//! Sizes, places, pictures and colours are the original client's; each window's file says from where. Each
//! window lives in an element of its own, so a value that changes redraws only its window: the chat's input
//! line is never rebuilt while someone types in it.

use crate::numbers::grouped;
use crate::i18n::tr;
use super::app::{App, Shop, escape};
use super::data;
use super::parts::{at, dragged, dropped, picture, pressed, pressed_at, sent, stepped, tip, tip_at, tip_by_pointer, tip_in, used};
use crate::press::things::{Drag, Drop, Form, Leveled, Use};
use crate::net::inventory::{Hotkey, Item};
use crate::inventory::Tab;
use crate::net::packets::parameter;

mod basic;
mod boxes;
mod choosing;
mod inventory;
mod equipment;
mod skills;
mod tabs;
mod status;
mod chat;
mod talk;
mod esc;
mod messenger;
mod pets;
mod quests;
mod pvp;
mod statuses;
mod cooldowns;
mod hotkeys;
mod settings;
mod guide;
mod party;
mod guild;
mod homunculus;
mod mercenary;
mod skill_list;
mod player_menu;
mod exchange;
mod mail;
mod cart;
mod vending;
mod fight;
mod friends;
mod whispers;
mod viewequip;
mod unread;
mod scripted;
mod chat_rooms;
mod family;
mod broadcast;
mod marks;
mod shops;
mod storage;
mod lists;
mod emotions;
mod tips;
mod worldmap;
mod navigation;
mod books;
mod cashshop;
mod achievements;
mod reputation;
mod probability;
mod attendance;
mod bank;
mod instances;
mod battleground;
mod booking;
mod agency;
mod replay;
pub(super) mod windows;

// What the windows share, each kind of piece in its file.
mod buttons;
mod checks;
mod combos;
mod colours;
mod fields;
mod frames;
mod gauges;
mod items;
mod texts;

use boxes::*;
use buttons::*;
use checks::*;
use combos::*;
use colours::*;
use frames::*;
use lists::*;
use messenger::*;
use fields::*;
use gauges::*;
use items::*;
use texts::*;

// What the screens before the game and the app take from here by name.
pub(super) use frames::title_bar;
pub(super) use texts::{BOLD, Font, SMALL};

/// How many chat lines are kept.
pub const CHAT_KEPT: usize = 240;
/// The field a tab of the chat has its name typed anew in.
pub(in crate::web) const CHAT_TAB_NAME: &str = "chat-tab-name";

/// How many columns of items the inventory shows (its rows are as many as fit beside its tabs:
/// `App::inventory_body`).
const ITEM_COLUMNS: i32 = 7;

impl App {
    /// A line of the client's message table (`msgstringtable.txt`, counted from 0), in the player's language
    /// where this client has it; `otherwise` until the tables are there.
    pub(in crate::web) fn words(line: usize, otherwise: &str) -> String {
        tr(&data::message(line).unwrap_or_else(|| otherwise.to_owned())).to_owned()
    }

    fn value(&self, name: u16) -> i64 {
        // A dead character has no life left, whatever the server has already set aside for its return.
        if self.hero.dead && name == parameter::HP {
            return 0;
        }
        self.hero.params.get(&name).copied().unwrap_or(0)
    }
}
