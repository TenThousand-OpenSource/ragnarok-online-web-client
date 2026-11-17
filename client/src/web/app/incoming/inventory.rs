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
            items::ZC_ITEM_PICKUP_ACK => {
                if let Some(gained) = items::gained(packet) {
                    // What was picked up is told in the chat and shown for a moment, as the original does.
                    self.bag.obtained = Some((gained.id, gained.amount));
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
            items::ZC_SHORTCUT_KEY_LIST => {
                self.skills.hotkeys = items::hotkeys(packet);
                self.items_changed();
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
