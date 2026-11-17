//! NPC shops and the Kafra's storage.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn trade_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // The Kafra's storage.
            trade::ZC_NOTIFY_STOREITEM_COUNTINFO => {
                if let Some(count) = trade::storage_count(packet) {
                    self.storage.get_or_insert_with(Storage::default).count = count;
                    // With the storage open the inventory is too, beside it.
                    self.inventory_open = true;
                    self.items_changed();
                    self.render();
                }
            }
            trade::ZC_ADD_ITEM_TO_STORE => {
                if let (Some(storage), Some(added)) = (&mut self.storage, trade::stored(packet)) {
                    match storage.items.iter_mut().find(|item| item.index == added.index) {
                        Some(item) => item.amount = item.amount.saturating_add(added.amount),
                        None => storage.items.push(added),
                    }
                    self.items_changed();
                }
            }
            trade::ZC_DELETE_ITEM_FROM_STORE => {
                if let (Some(storage), Some((index, amount))) = (&mut self.storage, trade::taken(packet)) {
                    if let Some(item) = storage.items.iter_mut().find(|item| item.index == index) {
                        item.amount = item.amount.saturating_sub(amount.min(u32::from(u16::MAX)) as u16);
                    }
                    storage.items.retain(|item| item.amount > 0);
                    self.items_changed();
                }
            }
            trade::ZC_CLOSE_STORE => {
                self.storage = None;
                self.render();
            }
            // An NPC's shop.
            trade::ZC_SELECT_DEALTYPE => {
                if let Some(npc) = npc::npc(id, packet) {
                    self.shop = Some(Shop::Choose(npc));
                    self.refresh_trade();
                }
            }
            trade::ZC_PC_PURCHASE_ITEMLIST => {
                let offers = trade::offers(packet);
                data::want_items(offers.iter().map(|offer| offer.id), || dispatch(Action::ItemsKnown));
                self.basket.clear();
                self.shop_picked = None;
                self.shop = Some(Shop::Buy(offers));
                self.refresh_trade();
            }
            trade::ZC_PC_SELL_ITEMLIST => {
                self.basket.clear();
                self.shop_picked = None;
                self.shop = Some(Shop::Sell(trade::bids(packet)));
                self.refresh_trade();
            }
            trade::ZC_PC_PURCHASE_RESULT | trade::ZC_PC_SELL_RESULT => {
                let words = match (id, packet.get(2).copied().unwrap_or(0)) {
                    (trade::ZC_PC_PURCHASE_RESULT, 0) => "The deal has successfully completed.",
                    (trade::ZC_PC_PURCHASE_RESULT, 1) => "You do not have enough zeny.",
                    (trade::ZC_PC_PURCHASE_RESULT, 2) => "You are over your weight limit.",
                    (trade::ZC_PC_PURCHASE_RESULT, _) => "You cannot carry any more items.",
                    (_, 0) => "The deal has successfully completed.",
                    _ => "The deal has failed.",
                };
                self.chat_line(words.to_owned(), "#ffff00");
            }
            _ => return false,
        }
        true
    }
}
