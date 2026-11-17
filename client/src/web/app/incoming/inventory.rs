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
                }
                if packet.get(4) == Some(&trade::STORAGE) {
                    self.deal.storage = Some(Storage::default());
                }
            }
            items::ZC_INVENTORY_ITEMLIST_NORMAL => {
                self.bag.items.extend(items::stackable(packet));
                if let Some(storage) = &mut self.deal.storage {
                    storage.items.extend(items::stackable_of(packet, trade::STORAGE));
                }
                self.items_changed();
            }
            items::ZC_INVENTORY_ITEMLIST_EQUIP => {
                self.bag.items.extend(items::equipment(packet));
                if let Some(storage) = &mut self.deal.storage {
                    storage.items.extend(items::equipment_of(packet, trade::STORAGE));
                }
                self.items_changed();
            }
            // ZC_ITEMIDENTIFY_LIST (0x0177): `u16 length`, then the place in the inventory of each item not
            // identified. A magnifier was used: they are listed to pick the one to identify (the client's
            // `UIItemIdentifyWnd`).
            0x0177 => {
                self.bag.identify = Some(packet.get(4..).unwrap_or_default().chunks_exact(2).map(|index| u16::from_le_bytes([index[0], index[1]])).collect());
                self.bag.identify_picked = None;
                self.render();
            }
            // ZC_ACK_ITEMIDENTIFY (0x0179): `u16 index, u8 result`. Identified (0): the client says so in yellow
            // and the item is what it is from now on, name and picture; failed (1): it says so in red.
            0x0179 => {
                let index = packet.get(2..4).map_or(0, |index| u16::from_le_bytes([index[0], index[1]]));
                match packet.get(4) {
                    Some(0) => {
                        self.chat_line(data::message(491).unwrap_or_else(|| "Item appraisal has completed successfully.".to_owned()), "#ffff00");
                        if let Some(item) = self.bag.items.iter_mut().find(|item| item.index == index) {
                            item.identified = true;
                            // Its description, if open, is the identified one now.
                            if self.bag.info == Some((item.id, false)) {
                                self.bag.info = Some((item.id, true));
                            }
                        }
                        self.items_changed();
                    }
                    Some(1) => self.chat_line(data::message(492).unwrap_or_else(|| "Item appraisal has failed.".to_owned()), "#ff0000"),
                    _ => {}
                }
            }
            items::ZC_ITEM_PICKUP_ACK => {
                if let Some(gained) = items::gained(packet) {
                    // What was picked up is told in the chat and shown for a moment, as the original does.
                    self.bag.obtained = Some((gained.id, gained.amount, gained.identified));
                    self.bag.obtained_at = js_sys::Date::now();
                    match self.bag.items.iter_mut().find(|item| item.index == gained.index) {
                        Some(item) => item.amount += gained.amount,
                        None => self.bag.items.push(gained),
                    }
                    self.items_changed();
                    self.bag.told_obtained = false;
                    self.tell_obtained();
                }
            }
            items::ZC_USE_ITEM_ACK => {
                if let Some((index, item, who, left)) = items::used(packet) {
                    let own = Some(who) == self.account.as_ref().map(|account| account.account_id);
                    // What the item looks and sounds like when used, by the client's own tables.
                    if let Some((file, sound)) = item_effect(item) {
                        world::effect(file, (!own).then_some(who));
                        if let Some(sound) = sound {
                            sound::effect(sound, 1.0);
                        }
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
            items::ZC_DELETE_ITEM_FROM_BODY | items::ZC_ITEM_THROW_ACK => {
                if let Some((index, count)) = items::lost(id, packet) {
                    if let Some(item) = self.bag.items.iter_mut().find(|item| item.index == index) {
                        item.amount = item.amount.saturating_sub(count);
                    }
                    self.bag.items.retain(|item| item.amount > 0);
                    self.refresh_items();
                }
            }
            items::ZC_REQ_WEAR_EQUIP_ACK => {
                if let Some((index, place)) = items::put_on(packet) {
                    if let Some(item) = self.bag.items.iter_mut().find(|item| item.index == index) {
                        item.worn = place;
                    }
                    self.items_changed();
                } else {
                    // The server would not have it worn (another job's thing, too low a level): the client says so.
                    self.chat_line(crate::i18n::tr("You can't put this item on.").to_owned(), "#ff8484");
                }
            }
            items::ZC_REQ_TAKEOFF_EQUIP_ACK => {
                if let Some(index) = items::taken_off(packet) {
                    if let Some(item) = self.bag.items.iter_mut().find(|item| item.index == index) {
                        item.worn = 0;
                    }
                    self.items_changed();
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
            // ZC_EQUIP_ARROW (0x013c): `u16 index`. The ammunition in use (0: none).
            0x013c => {
                if packet.len() >= 4 {
                    let index = u16::from_le_bytes([packet[2], packet[3]]);
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
}
