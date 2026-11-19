//! Merchants' shops: what the map server tells of them. What the chat says of each answer, and in which
//! colour, is the client's (`GameModePackets.cpp`: the handlers of 0x012d, 0x0a28, 0x0b40, 0x0b3d and 0x0135,
//! `ProcessVendingSale`, `UIMerchantShopMakeWnd::PrintSellList`).

use super::*;
use crate::net::vending::{self, Opened};

/// The colours the client writes in what it says of one's own shop (its 0x96FFFF, 0x5AFFFF, 0x78FFFF and
/// 0xFFFF64, which it writes blue first), and of a purchase that failed (0x6464FF).
const LIST_HEAD: &str = "#ffff96";
const LIST_LINE: &str = "#ffff5a";
const LIST_FOOT: &str = "#ffff78";
const SOLD: &str = PALE_CYAN;
const NOT_BOUGHT: &str = "#ff6464";
/// What it says that everything is sold in (0x00FF00).
const SOLD_OUT: &str = "#00ff00";

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other. Two packets of other systems
    /// are overheard and left to their own readers: entering the game, where the shops of the character
    /// before are forgotten, and someone going out of sight, whose sign goes too.
    pub(super) fn vending_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            packets::ZC_ACCEPT_ENTER | packets::ZC_NPCACK_MAPMOVE => {
                self.vending = Vending::default();
                self.redraw(&wnd::VENDING_WINDOWS);
                return false;
            }
            net::actors::ZC_NOTIFY_VANISH => {
                // (One struck down stays in sight, and so does its sign until the server takes it down.)
                if let Some((who, false)) = net::actors::vanished(packet) {
                    self.vending_sign(who, None);
                }
                return false;
            }
            vending::ZC_OPENSTORE => {
                if let Some(slots) = vending::slots(packet) {
                    self.vending.making = Some(Stall { slots, ..Stall::default() });
                    self.vending.counting = None;
                    self.redraw(&wnd::VENDING_WINDOWS);
                }
            }
            vending::ZC_ACK_OPENSTORE2 => match vending::opened(packet) {
                Some(Opened::Open) => self.vending_said_open(),
                Some(Opened::Failed) => self.vending_not_opened(2639, "Failed to open stalls."),
                Some(Opened::NotHere) => self.vending_not_opened(3229, "Unable to open a shop at the current location"),
                None => {}
            },
            // What one's own shop sells: an empty list closes it.
            vending::ZC_PC_PURCHASE_MYITEMLIST => {
                if let Some(sold) = vending::mine(packet) {
                    data::want_items(sold.iter().map(|sold| sold.item.id), || dispatch(Action::ItemsKnown));
                    self.vending.making = None;
                    self.vending.mine = (!sold.is_empty()).then_some(sold);
                    self.redraw(&wnd::VENDING_WINDOWS);
                }
            }
            vending::ZC_DELETEITEM_FROM_MCSTORE => {
                if let Some((index, amount)) = vending::sale(packet) {
                    self.vending_sold(index, amount);
                }
            }
            vending::ZC_STORE_ENTRY => {
                if let Some((who, title)) = vending::sign(packet) {
                    self.vending_sign(who, Some(title));
                }
            }
            vending::ZC_DISAPPEAR_ENTRY => {
                if let Some(who) = vending::sign_gone(packet) {
                    self.vending_sign(who, None);
                }
            }
            vending::ZC_PC_PURCHASE_ITEMLIST_FROMMC => {
                if let Some(list) = vending::listed(packet) {
                    data::want_items(
                        list.sold.iter().flat_map(
                            |sold| std::iter::once(sold.item.id).chain(crate::item_names::Named::from(&sold.item).cards_set())
                        ),
                        || dispatch(Action::ItemsKnown)
                    );
                    self.vending.visit = Some(Visit { name: world::player_name(list.vendor), list, basket: Vec::new(), stage: Stage::Choosing });
                    self.vending.counting = None;
                    self.redraw(&wnd::VENDING_WINDOWS);
                }
            }
            vending::ZC_PC_PURCHASE_RESULT_FROMMC => {
                if let Some((index, amount, result)) = vending::purchase_result(packet) {
                    // The client names the thing by where it now lies in the inventory, which the server's
                    // place in the vendor's cart is not: it is named here by the list last looked at.
                    let name = self.vending.visit
                        .as_ref()
                        .and_then(|visit| visit.list.sold.iter().find(|sold| sold.item.index == index))
                        .map(|sold| Self::item_name(&sold.item))
                        .unwrap_or_default();
                    if let Some(words) = crate::vending::purchase_said(result, &name, amount, Self::words) {
                        self.chat_line(words, if result == 7 { RED } else { NOT_BOUGHT });
                    }
                    // The windows were put away as Buy was confirmed; the list is forgotten now.
                    if self.vending.visit.as_ref().is_some_and(|visit| visit.stage == Stage::Sent) {
                        self.vending.visit = None;
                    }
                }
            }
            _ => return false,
        }
        true
    }

    /// A vendor's sign is put up, or taken down.
    fn vending_sign(&mut self, who: u32, title: Option<String>) {
        self.vending_sign_told(who, title.as_deref());
        match title {
            Some(title) => self.vending.signs.insert(who, title),
            None => self.vending.signs.remove(&who),
        };
    }

    /// The world keeps a sign over its vendor's head, unless the client's `/showshop` is off
    /// (`TT_SHOW_MERCHANT_SHOP_ON_OFF`), which shows none.
    fn vending_sign_told(&self, who: u32, title: Option<&str>) {
        let shown = title.filter(|_| self.option("TT_SHOW_MERCHANT_SHOP_ON_OFF")).map(|title| Self::shop_sign(who, title));
        world::sign(who, self.own_account(), world::Board::Shop, shown);
    }

    /// The options were switched: every sign known is put up, or taken down, by what they say now.
    pub(in crate::web) fn vending_signs_told(&self) {
        for (who, title) in &self.vending.signs {
            self.vending_sign_told(*who, Some(title));
        }
    }

    /// The shop is open: the chat lists what it sells (`PrintSellList`), each thing with what the vendor is
    /// left of its price, and the zeny in hand.
    fn vending_said_open(&mut self) {
        let Some(sold) = self.vending.mine.clone() else { return };
        self.chat_line("=== SALE LIST ===".to_owned(), LIST_HEAD);
        for thing in &sold {
            self.chat_line(format!("    {}  :  {}  Zeny", Self::item_name(&thing.item), crate::numbers::grouped(i64::from(thing.price))), LIST_LINE);
            self.chat_line(
                format!("Commission : 1 EA = {} Zeny", crate::numbers::grouped(i64::from(crate::vending::after_commission(thing.price)))),
                LIST_LINE
            );
        }
        let zeny = self.hero.zeny();
        self.chat_line(format!("Current Zeny : {} Z", crate::numbers::grouped(zeny)), LIST_FOOT);
    }

    /// The shop was not opened: its windows are put away and the chat says why. (The client says the first
    /// of the two in a message box.)
    fn vending_not_opened(&mut self, line: usize, otherwise: &str) {
        self.vending.making = None;
        self.vending.mine = None;
        self.redraw(&wnd::VENDING_WINDOWS);
        self.chat_line(Self::words(line, otherwise), RED);
    }

    /// So many of a thing of one's own shop were sold (`ProcessVendingSale`): the chat says so; with nothing
    /// left it says that too, and the shop is closed.
    fn vending_sold(&mut self, index: u16, amount: u16) {
        let Some(mine) = &mut self.vending.mine else { return };
        let Some(sold) = mine.iter_mut().find(|sold| sold.item.index == index) else { return };
        sold.item.amount = sold.item.amount.saturating_sub(amount);
        let name = Self::item_name(&sold.item);
        mine.retain(|sold| sold.item.amount > 0);
        let out = mine.is_empty();
        self.chat_line(Self::words(231, "%s %d sold.").replacen("%s", &name, 1).replacen("%d", &amount.to_string(), 1), SOLD);
        if out {
            self.chat_line(Self::words(311, "Items are sold out."), SOLD_OUT);
            return self.pressed(crate::press::vending::Press::MineClose);
        }
        self.redraw(&[&wnd::VENDING_MINE]);
    }
}
