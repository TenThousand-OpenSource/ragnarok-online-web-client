//! What each thing on the page does when pressed: the press it stands for (`crate::press`, read back from its
//! `data-click`) is handed to its system.
//!
//! Each system has its own file here, named as its presses are in `press/`, its packets in `incoming/` and its
//! window in `hud/`; its function is `<file>_pressed`.

use super::*;

mod general;
mod account;
mod choosing;
mod talk;
mod status;
mod inventory;
mod skills;
mod chat;
mod shops;
mod storage;
mod pets;
mod homunculus;
mod mercenary;
mod quests;
mod hotkeys;
mod settings;
mod guide;
mod party;
mod guild;
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

impl App {
    /// Something on the page was pressed: the press its name and what went with it stand for (`crate::press`).
    pub(super) fn click(&mut self, what: &str, arg: &str) {
        // A name no press has does nothing: nothing on the page is drawn with one.
        if let Some(press) = crate::press::Press::parse(what, arg) {
            self.pressed(press);
        }
    }

    /// A press, handed to its system: from the page, or from the app itself doing what a button does.
    pub(in crate::web) fn pressed(&mut self, press: impl Into<crate::press::Press>) {
        use crate::press::Press;
        let press = press.into();
        // (A recording being played holds presses back, and one being made writes some down.)
        if !self.replay_lets(&press) {
            return;
        }
        match press {
            Press::General(press) => self.general_pressed(press),
            Press::Account(press) => self.account_pressed(press),
            Press::Talk(press) => self.talk_pressed(press),
            Press::Status(press) => self.status_pressed(press),
            Press::Inventory(press) => self.inventory_pressed(press),
            Press::Skills(press) => self.skills_pressed(press),
            Press::Chat(press) => self.chat_pressed(press),
            Press::Shops(press) => self.shops_pressed(press),
            Press::Storage(press) => self.storage_pressed(press),
            Press::Pets(press) => self.pets_pressed(press),
            Press::Homunculus(press) => self.homunculus_pressed(press),
            Press::Mercenary(press) => self.mercenary_pressed(press),
            Press::Quests(press) => self.quests_pressed(press),
            Press::Hotkeys(press) => self.hotkeys_pressed(press),
            Press::Settings(press) => self.settings_pressed(press),
            Press::Guide(press) => self.guide_pressed(press),
            Press::Party(press) => self.party_pressed(press),
            Press::Guild(press) => self.guild_pressed(press),
            Press::PlayerMenu(press) => self.player_menu_pressed(press),
            Press::Exchange(press) => self.exchange_pressed(press),
            Press::Mail(press) => self.mail_pressed(press),
            Press::Cart(press) => self.cart_pressed(press),
            Press::Vending(press) => self.vending_pressed(press),
            Press::Friends(press) => self.friends_pressed(press),
            Press::Whispers(press) => self.whispers_pressed(press),
            Press::ViewEquip(press) => self.viewequip_pressed(press),
            Press::Unread(press) => self.unread_pressed(press),
            Press::Scripted(press) => self.scripted_pressed(press),
            Press::ChatRooms(press) => self.chat_rooms_pressed(press),
            Press::Family(press) => self.family_pressed(press),
            Press::Emotions(press) => self.emotions_pressed(press),
            Press::WorldMap(press) => self.worldmap_pressed(press),
            Press::Tips(press) => self.tips_pressed(press),
            Press::Navigation(press) => self.navigation_pressed(press),
            Press::Choosing(press) => self.choosing_pressed(press),
            Press::Fight(press) => self.fight_pressed(press),
            Press::Books(press) => self.books_pressed(press),
            Press::CashShop(press) => self.cashshop_pressed(press),
            Press::Achievements(press) => self.achievements_pressed(press),
            Press::Reputation(press) => self.reputation_pressed(press),
            Press::Probability(press) => self.probability_pressed(press),
            Press::Attendance(press) => self.attendance_pressed(press),
            Press::Bank(press) => self.bank_pressed(press),
            Press::Instances(press) => self.instances_pressed(press),
            Press::Battleground(press) => self.battleground_pressed(press),
            Press::Booking(press) => self.booking_pressed(press),
            Press::Agency(press) => self.agency_pressed(press),
            Press::Replay(press) => self.replay_pressed(press),
        }
    }
}
