//! Double clicks on things, things dragged out of a window and let go over another, the right button on a
//! thing that has a menu: which system sees to each.

use super::*;

impl App {
    /// Something was double-clicked, or a key or the mouse used a slot of the hotkey bar: each kind of thing is
    /// handed to its system.
    pub(in crate::web::app) fn used(&mut self, what: Use) {
        match what {
            // A row of a shop goes across as if it were dragged (`clicks/shops.rs`).
            Use::ShopAdd(row) => self.shop_used(Drag::Offer(row), Drop::Basket),
            Use::ShopRemove(row) => self.shop_used(Drag::Basket(row), Drop::Shop),
            Use::Put(index) => self.storage_put_used(index),
            Use::Take(index) => self.storage_let_out(index),
            // A double click on a friend opens the talk with that friend (`clicks/friends.rs`).
            Use::Friend(row) => self.friend_talk(row),
            // A double click on a vendor's sign asks what the shop sells (`clicks/vending.rs`).
            Use::Sign(vendor) => self.vending_used(vendor),
            // A double click on a row of a list to pick from is picking it and pressing OK (the client's
            // `UIItemIdentifyWnd::OnLBtnDblClk`).
            Use::Egg(row) => {
                self.pressed(crate::press::pets::Press::EggPick(row));
                return self.pressed(crate::press::pets::Press::EggOk);
            }
            Use::Identify(row) => {
                self.pressed(crate::press::inventory::Press::IdentifyPick(row));
                return self.pressed(crate::press::inventory::Press::IdentifyOk);
            }
            Use::Choice(row) => return self.choice_used(row),
            Use::Room(id) => return self.room_used(id),
            Use::Emotion(emotion) => return self.emotion_shown(emotion),
            // A card asks what it can be set into (`clicks/choosing.rs`).
            Use::Item(index) if self.card_used(index) => return,
            Use::Item(index) => self.use_item(index),
            Use::Skill(id) => self.use_skill(id, None),
            Use::Slot(slot) => self.use_slot(slot),
        }
    }

    /// Something was dragged out of its window and let go over a window that takes things: a shop's two, or
    /// the storage's.
    pub(super) fn let_go(&mut self, dragged: Drag, onto: Drop) {
        match onto {
            Drop::Shop | Drop::Basket => self.shop_let_go(dragged, onto),
            Drop::Storage => self.storage_let_go(dragged),
            Drop::Cart => self.cart_let_go(dragged),
            Drop::Stall | Drop::Stock | Drop::Vendor | Drop::Wants => self.vending_let_go(dragged, onto),
            Drop::InventoryTab(tab) => self.tab_let_go(dragged, tab),
        }
    }

    /// The right button was pressed on a thing in a window that has a menu: the menu of its system opens there.
    pub(super) fn menu_on(&mut self, what: things::Menu, at: (f32, f32)) {
        match what {
            things::Menu::Member(row) => self.party_menu_opens(row, at),
            things::Menu::GuildMember(row) => self.guild_menu_opens(GuildMenu::Member(row), at),
            things::Menu::GuildRelated(slot) => self.guild_related_pressed(slot, at),
            things::Menu::GuildCastle(line) => self.guild_castle_menu_opens(line),
            things::Menu::Friend(row) => self.friend_menu_opens(row, at),
            things::Menu::Whisper(place) => self.whisper_menu_opens(place, at),
            things::Menu::Room => self.room_menu_opens(None, at),
            things::Menu::RoomMember(row) => self.room_menu_opens(Some(row), at),
        }
    }
}
