//! The inventory: what is carried, picked up, used, dropped, worn and taken off, and the hotkey bar.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn item_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // What the character carries. A list starts over with the packet that announces it.
            items::ZC_INVENTORY_START => {
                if packet.get(4) == Some(&0) {
                    self.bag.items.clear();
                    self.bag.favourites.clear();
                }
                if packet.get(4) == Some(&crate::net::storage::STORAGE) {
                    self.storage_begun(packet);
                }
            }
            items::ZC_INVENTORY_ITEMLIST_NORMAL => {
                self.bag.items.extend(items::stackable(packet));
                self.bag.favourites.extend(items::favourite_stacks(packet));
                self.storage_listed(id, packet);
                self.items_changed();
            }
            items::ZC_INVENTORY_ITEMLIST_EQUIP => {
                self.bag.items.extend(items::equipment(packet));
                self.bag.favourites.extend(items::favourite_equipment(packet));
                self.storage_listed(id, packet);
                self.items_changed();
            }
            // ZC_ITEMIDENTIFY_LIST (0x0177): `u16 length`, then the place in the inventory of each item not
            // identified. A magnifier was used: they are listed to pick the one to identify (the client's
            // `UIItemIdentifyWnd`).
            0x0177 => {
                self.bag.identify = Some(items::unidentified(packet));
                self.bag.identify_picked = None;
                self.render();
            }
            0x0179 => self.identified(packet),
            items::ZC_ITEM_PICKUP_ACK => self.picked_up(packet),
            items::ZC_USE_ITEM_ACK => self.item_used(packet),
            items::ZC_DELETE_ITEM_FROM_BODY | items::ZC_ITEM_THROW_ACK => self.item_lost(id, packet),
            items::ZC_REQ_WEAR_EQUIP_ACK => self.put_on(packet),
            items::ZC_REQ_TAKEOFF_EQUIP_ACK => {
                if let Some(index) = items::taken_off(packet) {
                    if let Some(item) = self.bag.items.iter_mut().find(|item| item.index == index) {
                        item.worn = 0;
                    }
                    self.items_changed();
                } else {
                    self.wearing_refused(items::take_off_refused(packet), false);
                }
            }
            // A thing is kept under the inventory's tab "Fav" now, or no more.
            items::ZC_INVENTORY_TAB => {
                if let Some((index, kept)) = items::favourite_moved(packet) {
                    self.bag.favourites.retain(|favourite| *favourite != index);
                    if kept {
                        self.bag.favourites.push(index);
                    }
                    self.redraw(&[&wnd::INVENTORY]);
                }
            }
            // The server sends the list twice on coming in: the hotkey bar (tab 0), then the second bar (tab 1),
            // which this client does not show yet. Only the first is this bar; taking the second for it too
            // emptied the bar at every login.
            items::ZC_SHORTCUT_KEY_LIST => {
                if packet.get(3..5) == Some(&[0, 0]) {
                    self.skills.hotkeys = items::hotkeys(packet);
                    // What the bar's skills are called and which pictures are theirs.
                    for hotkey in self.skills.hotkeys.clone().iter().filter(|hotkey| hotkey.skill && hotkey.id != 0) {
                        data::want_skill(hotkey.id as u16, || dispatch(Action::ItemsKnown));
                    }
                    self.items_changed();
                }
            }
            // ZC_EQUIP_ARROW (0x013c): the ammunition in use.
            0x013c => {
                if let Some(index) = items::ammunition(packet) {
                    for item in self.bag.items.iter_mut().filter(|item| item.kind == 10) {
                        item.worn = if item.index == index { 0x8000 } else { 0 };
                    }
                    self.refresh_items();
                }
            }
            _ => return false,
        }
        true
    }

    /// ZC_ACK_ITEMIDENTIFY (0x0179): `u16 index, u8 result`. Identified (0): the client says so in yellow
    /// and the item is what it is from now on, name and picture; failed (1): it says so in red.
    fn identified(&mut self, packet: &[u8]) {
        let (index, result) = items::identified(packet);
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
        if let Some(gained) = items::gained(packet) {
            self.bag.favourites.retain(|favourite| *favourite != gained.index);
            if items::favourite_gained(packet) {
                self.bag.favourites.push(gained.index);
            }
            // What was picked up is told in the chat and shown for a moment, as the original does.
            self.bag.obtained = Some((gained.id, gained.amount, gained.identified));
            self.bag.obtained_at = js_sys::Date::now();
            match self.bag.items.iter_mut().find(|item| item.index == gained.index) {
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
        if let Some((index, item, who, left)) = items::used(packet) {
            let own = Some(who) == self.account.as_ref().map(|account| account.account_id);
            // What the item looks and sounds like when used, by the client's own tables: its effect, launched
            // on the one who used it as any effect is, with that effect's sounds heard from there.
            if let Some(effect) = crate::effect_tables::item_effect(item) {
                world::burst(effect, (!own).then_some(who), None, 0.0);
            }
            if own {
                if let Some(item) = self.bag.items.iter_mut().find(|item| item.index == index) {
                    item.amount = left;
                }
                self.bag.items.retain(|item| item.amount > 0);
                self.refresh_items();
            }
        }
    }

    fn item_lost(&mut self, id: u16, packet: &[u8]) {
        if let Some((index, count)) = items::lost(id, packet) {
            // Something thrown away is told of in the chat, among the lines about items (the client's
            // `ZC_ITEM_THROW_ACK`: its message 1609, "You dropped %s (%d).", in its 0xFFFF00, which
            // read as blue, green, red is cyan).
            let thrown = self.bag.items.iter().find(|item| id == items::ZC_ITEM_THROW_ACK && item.index == index && count > 0);
            if let Some((name, line)) = thrown.and_then(|item| Some((data::item_as(item.id, item.identified)?.0, data::message(1609)?))) {
                self.chat_line_of(said::ITEM, crate::i18n::tr(&line).replacen("%s", &name, 1).replacen("%d", &count.to_string(), 1), CYAN);
            }
            if let Some(item) = self.bag.items.iter_mut().find(|item| item.index == index) {
                item.amount = item.amount.saturating_sub(count);
            }
            self.bag.items.retain(|item| item.amount > 0);
            self.refresh_items();
        }
    }

    fn put_on(&mut self, packet: &[u8]) {
        if let Some((index, place)) = items::put_on(packet) {
            if let Some(item) = self.bag.items.iter_mut().find(|item| item.index == index) {
                item.worn = place;
            }
            self.items_changed();
        } else {
            self.wearing_refused(items::wear_refused(packet), true);
        }
    }

    /// The server would not have a thing put on, or taken off: the client's line for its answer
    /// (`CGameMode::ReceiveWearEquipAck`, `ReceiveTakeOffEquipResult`: 1953 where the character does not meet
    /// what the thing asks, 372 for any other thing that cannot be put on), in red among the lines of what
    /// is worn. The colour and the kind of line are the ones this client had; the client's own were not read.
    fn wearing_refused(&mut self, result: Option<u8>, putting_on: bool) {
        let Some(line) = result.and_then(|result| items::refusal_line(result, putting_on)) else { return };
        let otherwise = if line == 372 { "You can't put this item on." } else { "Can't equip this item because you don't meet the requirements." };
        self.chat_line_of(said::EQUIPMENT, Self::words(line, otherwise), PALE_RED);
    }

    /// The weight carried, or the most that can be, changed from `was`, which was `before` hundredths of the
    /// most, to what it is now (`CGameMode`'s `SP_WEIGHT` and `SP_MAXWEIGHT`): the chat says when that passes the share from
    /// which nothing is recovered and 90 of a hundred, in red as it grows too heavy and in the client's
    /// 0xFFFF00 as it is light enough again.
    pub(super) fn weight_told(&mut self, name: u16, was: i64, before: i64) {
        use crate::net::items::weight;
        let value = |app: &Self, name: u16| app.hero.params.get(&name).copied().unwrap_or(0);
        // Until the most that can be carried is known there is no share to tell of; and of that most the
        // client tells only when it grew from one it knew.
        let most = value(self, packets::parameter::MAX_WEIGHT);
        if most == 0 || (name == packets::parameter::MAX_WEIGHT && (was == 0 || was >= most)) {
            return;
        }
        let after = weight::percent(value(self, packets::parameter::WEIGHT), value(self, packets::parameter::MAX_WEIGHT));
        let lines = if name == packets::parameter::WEIGHT { weight::weight_changed(before, after, self.notices.overweight.clamp(1, 100) as u8) } else { weight::most_grew(before, after) };
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
}
