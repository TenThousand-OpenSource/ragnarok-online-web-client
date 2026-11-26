//! The inventory: what is carried, picked up, used, dropped, worn and taken off. (The hotkey bar:
//! `incoming/hotkeys.rs`.)

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn inventory_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            net::inventory::ZC_INVENTORY_START => self.inventory_begun(packet),
            net::inventory::ZC_INVENTORY_ITEMLIST_NORMAL => {
                self.bag.items.extend(net::inventory::stackable(packet));
                self.bag.favourites.extend(net::inventory::favourite_stacks(packet));
                self.storage_listed(id, packet);
                self.items_changed();
            }
            net::inventory::ZC_INVENTORY_ITEMLIST_EQUIP => {
                self.bag.items.extend(net::inventory::equipment(packet));
                self.bag.favourites.extend(net::inventory::favourite_equipment(packet));
                self.storage_listed(id, packet);
                self.items_changed();
            }
            net::inventory::ZC_ITEMIDENTIFY_LIST => self.identify_listed(packet),
            net::inventory::ZC_ACK_ITEMIDENTIFY => self.identified(packet),
            net::inventory::ZC_ITEM_PICKUP_ACK => self.picked_up(packet),
            net::inventory::ZC_USE_ITEM_ACK => self.item_used(packet),
            net::inventory::ZC_DELETE_ITEM_FROM_BODY | net::inventory::ZC_ITEM_THROW_ACK => self.item_lost(id, packet),
            net::inventory::ZC_REQ_WEAR_EQUIP_ACK => self.put_on(packet),
            net::inventory::ZC_REQ_TAKEOFF_EQUIP_ACK => self.taken_off(packet),
            net::inventory::ZC_INVENTORY_TAB => self.favourite_moved(packet),
            net::inventory::ZC_EQUIP_ARROW => self.ammunition_told(packet),
            // The lists of one kind are all sent: the client draws that kind's windows anew
            // (`CGameMode::RefreshItemListWindow`). Here they are drawn as each list arrives.
            net::inventory::ZC_INVENTORY_END => {}
            // The places the inventory has over its hundred (`CGameMode::ReceiveExtraInventorySlots`) and the
            // things of the equipment switch (`ReceiveSwapEquipmentList`), both sent as the game is entered:
            // the client keeps them for the inventory expansion and the switch window, neither of the classic
            // game (`BACKLOG.md`). Nothing is shown of them.
            net::inventory::ZC_EXTEND_BODYITEM_SIZE | net::inventory::ZC_SEND_SWAP_EQUIPITEM_INFO => {}
            _ => return false,
        }
        true
    }

    /// What the character carries. A list starts over with the packet that announces it.
    fn inventory_begun(&mut self, packet: &[u8]) {
        if net::inventory::list_kind(packet) == Some(0) {
            self.bag.items.clear();
            self.bag.favourites.clear();
        }
        if net::inventory::list_kind(packet) == Some(crate::net::storage::STORAGE) {
            self.storage_begun(packet);
        }
    }

    /// ZC_ITEMIDENTIFY_LIST (0x0177): `u16 length`, then the place in the inventory of each item not
    /// identified. A magnifier was used: they are listed to pick the one to identify (the client's
    /// `UIItemIdentifyWnd`).
    fn identify_listed(&mut self, packet: &[u8]) {
        self.bag.identify = Some(net::inventory::unidentified(packet));
        self.bag.identify_picked = None;
        self.render();
    }

    /// ZC_REQ_TAKEOFF_EQUIP_ACK: a thing was taken off, or the server would not have it.
    fn taken_off(&mut self, packet: &[u8]) {
        if let Some(index) = net::inventory::taken_off(packet) {
            if let Some(item) = self.bag.at_mut(index) {
                item.worn = 0;
            }
            self.items_changed();
        } else {
            self.wearing_refused(net::inventory::take_off_refused(packet), false);
        }
    }

    /// A thing is kept under the inventory's tab "Fav" now, or no more.
    fn favourite_moved(&mut self, packet: &[u8]) {
        if let Some((index, kept)) = net::inventory::favourite_moved(packet) {
            self.bag.favourites.retain(|favourite| *favourite != index);
            if kept {
                self.bag.favourites.push(index);
            }
            self.redraw(&[&wnd::INVENTORY]);
        }
    }

    /// ZC_EQUIP_ARROW (0x013c): the ammunition in use.
    fn ammunition_told(&mut self, packet: &[u8]) {
        if let Some(index) = net::inventory::ammunition(packet) {
            for item in self.bag.items.iter_mut().filter(|item| item.kind == 10) {
                item.worn = if item.index == index { 0x8000 } else { 0 };
            }
            self.refresh_items();
        }
    }

    /// ZC_ACK_ITEMIDENTIFY (0x0179): `u16 index, u8 result`. Identified (0): the client says so in yellow
    /// and the item is what it is from now on, name and picture; failed (1): it says so in red.
    fn identified(&mut self, packet: &[u8]) {
        let (index, result) = net::inventory::identified(packet);
        match result {
            Some(0) => {
                self.chat_line(data::message(491).unwrap_or_else(|| "Item appraisal has completed successfully.".to_owned()), YELLOW);
                if let Some(item) = self.bag.items.iter_mut().find(|item| item.index == index) {
                    item.identified = true;
                    // Its description, if open, is the identified one now.
                    if self.bag.info == Some((item.id, false)) {
                        self.bag.info = Some((item.id, true));
                    }
                }
                self.items_changed();
            }
            Some(1) => self.chat_line(data::message(492).unwrap_or_else(|| "Item appraisal has failed.".to_owned()), RED),
            _ => {}
        }
    }

    fn picked_up(&mut self, packet: &[u8]) {
        if let Some(gained) = net::inventory::gained(packet) {
            self.bag.favourites.retain(|favourite| *favourite != gained.index);
            if net::inventory::favourite_gained(packet) {
                self.bag.favourites.push(gained.index);
            }
            // What was picked up is told in the chat and shown for a moment, as the original does.
            self.bag.obtained = Some((crate::item_names::Named::from(&gained), gained.amount));
            self.bag.obtained_at = js_sys::Date::now();
            match self.bag.at_mut(gained.index) {
                Some(item) if item.id == gained.id => item.amount += gained.amount,
                // Another thing in a place the server has given anew: what lay there is gone without a word
                // (what was given away in a deal that is being made: `incoming/exchange.rs`).
                Some(item) => *item = gained,
                None => self.bag.items.push(gained),
            }
            self.items_changed();
            self.bag.told_obtained = false;
            self.tell_obtained();
        }
    }

    fn item_used(&mut self, packet: &[u8]) {
        if let Some(net::inventory::Used { index, item, who, left }) = net::inventory::used(packet) {
            let own = Some(who) == self.account.as_ref().map(|account| account.account_id);
            // What the item looks and sounds like when used, by the client's own tables: its effect, launched
            // on the one who used it as any effect is, with that effect's sounds heard from there.
            if let Some(effect) = crate::effect_tables::item_effect(item) {
                world::burst(effect, (!own).then_some(who), None, 0.0);
            }
            if own {
                if let Some(item) = self.bag.at_mut(index) {
                    item.amount = left;
                }
                self.bag.items.retain(|item| item.amount > 0);
                self.refresh_items();
            }
        }
    }

    fn item_lost(&mut self, id: u16, packet: &[u8]) {
        if let Some((index, count)) = net::inventory::lost(id, packet) {
            // Something thrown away is told of in the chat, among the lines about items (the client's
            // `ZC_ITEM_THROW_ACK`: its message 1609, "You dropped %s (%d).", in its 0xFFFF00, which
            // read as blue, green, red is cyan).
            let thrown = self.bag.items.iter().find(|item| id == net::inventory::ZC_ITEM_THROW_ACK && item.index == index && count > 0);
            if let Some((name, line)) = thrown.and_then(|item| Some((data::item_as(item.id, item.identified)?.0, data::message(1609)?))) {
                self.chat_line_of(said::ITEM, crate::i18n::tr(&line).replacen("%s", &name, 1).replacen("%d", &count.to_string(), 1), CYAN);
            }
            if let Some(item) = self.bag.at_mut(index) {
                item.amount = item.amount.saturating_sub(count);
            }
            self.bag.items.retain(|item| item.amount > 0);
            self.refresh_items();
        }
    }

    fn put_on(&mut self, packet: &[u8]) {
        if let Some((index, place)) = net::inventory::put_on(packet) {
            if let Some(item) = self.bag.at_mut(index) {
                item.worn = place;
            }
            self.items_changed();
        } else {
            self.wearing_refused(net::inventory::wear_refused(packet), true);
        }
    }

    /// The server would not have a thing put on, or taken off: the client's line for its answer
    /// (`CGameMode::ReceiveWearEquipAck`, `ReceiveTakeOffEquipResult`: 1953 where the character does not meet
    /// what the thing asks, 372 for any other thing that cannot be put on), in red among the lines of what
    /// is worn. The colour and the kind of line are the ones this client had; the client's own were not read.
    fn wearing_refused(&mut self, result: Option<u8>, putting_on: bool) {
        let Some(line) = result.and_then(|result| net::inventory::refusal_line(result, putting_on)) else { return };
        let otherwise = if line == 372 { "You can't put this item on." } else { "Can't equip this item because you don't meet the requirements." };
        self.chat_line_of(said::EQUIPMENT, Self::words(line, otherwise), PALE_RED);
    }

    /// The weight carried, or the most that can be, changed from `was`, which was `before` hundredths of the
    /// most, to what it is now (`CGameMode`'s `SP_WEIGHT` and `SP_MAXWEIGHT`): the chat says when that passes the share from
    /// which nothing is recovered and 90 of a hundred, in red as it grows too heavy and in the client's
    /// 0xFFFF00 as it is light enough again.
    pub(super) fn weight_told(&mut self, name: u16, was: i64, before: i64) {
        use crate::inventory::weight;
        let value = |app: &Self, name: u16| app.hero.params.get(&name).copied().unwrap_or(0);
        // Until the most that can be carried is known there is no share to tell of; and of that most the
        // client tells only when it grew from one it knew.
        let most = value(self, packets::parameter::MAX_WEIGHT);
        if most == 0 || (name == packets::parameter::MAX_WEIGHT && (was == 0 || was >= most)) {
            return;
        }
        let after = weight::percent(value(self, packets::parameter::WEIGHT), value(self, packets::parameter::MAX_WEIGHT));
        let lines = if name == packets::parameter::WEIGHT {
            weight::weight_changed(before, after, self.notices.overweight.clamp(1, 100) as u8)
        } else {
            weight::most_grew(before, after)
        };
        for said in lines {
            let otherwise = match said.line {
                307 => "Attacks and skills cannot be used when your Weight is over 90%.",
                308 => "Your HP/SP are now being restored naturally.",
                309 => "Attack and Skills are now available.",
                _ => "Your weight limit is over %d, HP and SP will not recover naturally.",
            };
            let words = Self::words(said.line, otherwise);
            let words = if said.with_share { words.replace("%d", &self.notices.overweight.to_string()) } else { words };
            self.chat_line_of(said::STATUS, words, if said.warning { RED } else { CYAN });
        }
    }

    /// Asks what the items carried and on the bar are called, and redraws their windows.
    pub(in crate::web::app) fn items_changed(&self) {
        let stored = self.storage.open.iter().flat_map(|storage| storage.items.iter().map(|item| item.id));
        let ids = self.bag.items
            .iter()
            .map(|item| item.id)
            .chain(self.skills.hotkeys.iter().filter(|hotkey| !hotkey.skill).map(|hotkey| hotkey.id))
            .chain(stored);
        data::want_items(ids, || dispatch(Action::ItemsKnown));
        // The weapon in hand (bit 2 of where a thing is worn) is drawn on the character.
        world::own_weapon(self.bag.items.iter().find(|item| item.worn & 2 != 0).map_or(0, |item| item.id));
        self.refresh_items();
        self.paint_figures();
    }

    /// Tells of the thing just picked up, once its name is known: a line in the chat, and a note on the screen.
    /// The name is the whole one, "+1 Knife [3]" (`AddPickedUpItemEx` and `UpdateReceivedItemEx`,
    /// `GameModePackets.cpp`: `MSI_GET_ITEM` filled with `item.DisplayName(0)`): a smith's refining gives the
    /// thing anew, and the line says how far it is refined now.
    pub(in crate::web::app) fn tell_obtained(&mut self) {
        let Some((named, amount)) = self.bag.obtained.clone().filter(|_| !self.bag.told_obtained) else { return };
        if data::item_as(named.id, named.identified).is_none() {
            return;
        }
        let name = data::item_title(&named, false);
        self.bag.told_obtained = true;
        self.chat_line_of(said::ITEM, format!("{name} - {amount} {}", crate::i18n::tr("obtained.")), YELLOW);
        if let Some(note) = element("obtained") {
            note.set_inner_html(&self.obtained_note());
        }
    }

    /// The gateway told what some items are called: whatever names an item is written anew.
    pub(in crate::web::app) fn items_known(&mut self) {
        self.chat_links_named();
        self.tell_obtained();
        self.party_obtained_said();
        self.mvp_item_said();
        self.tell_fed();
        self.tell_homunculus_fed();
        self.refresh_items();
        // The shop's list and basket name what they hold, as the question's window does.
        self.redraw(&[&wnd::TRADE, &wnd::SHOP, &wnd::SHOP_BASKET]);
        self.redraw(&[&wnd::GUIDE, &wnd::TIPS]);
        self.redraw(&[&wnd::ATTENDANCE, &wnd::CASH_SHOP]);
        self.redraw(&[&wnd::QUEST_PANE]);
        self.redraw(&[&wnd::MAIL_READ, &wnd::MAIL_WRITE]);
        self.vending_redrawn();
        self.redraw(&[&wnd::VIEW_EQUIP]);
        self.redraw(&[&wnd::CHOOSE_PICTURED, &wnd::CHOOSE_LINE]);
        self.paint_figures();
    }
}
