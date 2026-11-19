//! The storage, the Kafra's or a guild's: opened, filled, added to, taken from and closed.

use super::*;
use crate::net::storage;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn storage_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            storage::ZC_NOTIFY_STOREITEM_COUNTINFO => {
                if let Some(count) = storage::storage_count(packet) {
                    self.storage.open.get_or_insert_with(Stored::default).count = count;
                    // With the storage open the inventory is too, beside it.
                    self.bag.open = true;
                    self.items_changed();
                    self.render();
                }
            }
            storage::ZC_ADD_ITEM_TO_STORE => {
                if let (Some(stored), Some(added)) = (&mut self.storage.open, storage::stored(packet)) {
                    match stored.items.iter_mut().find(|item| item.index == added.index) {
                        Some(item) => item.amount = item.amount.saturating_add(added.amount),
                        None => stored.items.push(added),
                    }
                    self.items_changed();
                }
            }
            storage::ZC_DELETE_ITEM_FROM_STORE => {
                if let (Some(stored), Some((index, amount))) = (&mut self.storage.open, storage::taken(packet)) {
                    if let Some(item) = stored.items.iter_mut().find(|item| item.index == index) {
                        item.amount = item.amount.saturating_sub(amount.min(u32::from(u16::MAX)) as u16);
                    }
                    stored.items.retain(|item| item.amount > 0);
                    self.items_changed();
                }
            }
            // The client closes the windows of the tabs with it and keeps which were open
            // (`UIItemStoreWnd::SendMsg`, `ID_CLOSE`), and closes the window of what was found.
            storage::ZC_CLOSE_STORE => {
                self.storage.open = None;
                self.storage.counting = None;
                self.storage.found = None;
                self.storage.sorting = false;
                self.render();
            }
            _ => return false,
        }
        true
    }

    /// A list of the storage begins (ZC_INVENTORY_START of that kind): the storage is empty again, under the
    /// name the server gives it.
    pub(super) fn storage_begun(&mut self, packet: &[u8]) {
        self.storage.open = Some(Stored { name: storage::list_name(packet).unwrap_or_default(), ..Stored::default() });
    }

    /// The things of a list of the storage (ZC_INVENTORY_ITEMLIST_NORMAL, ZC_INVENTORY_ITEMLIST_EQUIP).
    pub(super) fn storage_listed(&mut self, id: u16, packet: &[u8]) {
        if let Some(stored) = &mut self.storage.open {
            stored.items.extend(if id == net::inventory::ZC_INVENTORY_ITEMLIST_NORMAL { net::inventory::stackable_of(packet, storage::STORAGE) } else { net::inventory::equipment_of(packet, storage::STORAGE) });
        }
    }
}
