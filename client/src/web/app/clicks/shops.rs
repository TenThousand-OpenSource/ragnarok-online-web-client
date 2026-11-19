//! An NPC's shop: its question, and what goes across its two windows, the list and the basket, as the client
//! moves it (`UIItemShopWnd`, `UIItemPurchaseWnd`, `UIItemSellWnd`, and the states 1 to 4 of
//! `UIItemDropCntWnd`).

use super::*;
use crate::shops::{Asked, MOST_BOUGHT};

impl App {
    /// A press of this system (`crate::press::shops`).
    pub(super) fn shops_pressed(&mut self, press: crate::press::shops::Press) {
        use crate::press::shops::Press;
        match press {
            Press::Buy | Press::Sell | Press::Cancel | Press::Ok => self.shop_answered(press),
            // The client's box can be ticked only while selling (`UIItemShopWnd::SendMsg`, `UM_SETTITLE`).
            Press::WholeStacks => {
                if matches!(self.shops.shop, Some(Shop::Sell(_))) {
                    self.shops.whole_stacks = !self.shops.whole_stacks;
                    self.redraw(&[&wnd::SHOP]);
                }
            }
            // The box that asks how many is a form, and its OK sends it.
            Press::CountOk => {}
        }
    }

    /// The shop's question is answered, or its basket's Buy, Sell or Cancel is pressed. The client sends the
    /// basket and closes both windows (`MM_REQ_PURCHASE`, `MM_REQ_SELL`); with nothing in the basket, and for
    /// Cancel, it sends no list. Whenever the window of the question, of what is bought or of what is sold
    /// closes, it tells the server the shop is left — CZ_NPC_TRADE_QUIT (`UIWindowMgr::DeleteWindow` for
    /// `WID_CHOOSESELLBUYWND`, `WID_ITEMPURCHASEWND` and `WID_ITEMSELLWND` sends `MM_NPC_TRADE_QUIT`) — after
    /// a deal as well, and after Buy or Sell of the question, whose window closes before the list comes.
    fn shop_answered(&mut self, press: crate::press::shops::Press) {
        use crate::press::shops::Press;
        let Some(shop) = self.shops.shop.take() else { return };
        let basket = std::mem::take(&mut self.shops.basket);
        self.shops.counting = None;
        let dealt = press == Press::Ok && !basket.is_empty();
        match (press, shop) {
            (Press::Buy, Shop::Choose(npc)) => self.send(&net::shops::deal(npc, false)),
            (Press::Sell, Shop::Choose(npc)) => self.send(&net::shops::deal(npc, true)),
            (_, Shop::Choose(_)) => {}
            (_, Shop::Buy(offers)) if dealt => {
                let wanted: Vec<(u32, u16)> = basket.iter().filter_map(|(row, amount)| Some((offers.get(*row)?.id, *amount))).collect();
                self.send(&net::shops::buy(&wanted));
            }
            (_, Shop::Sell(bids)) if dealt => {
                let sold: Vec<(u16, u16)> = basket.iter().filter_map(|(row, amount)| Some((bids.get(*row)?.index, *amount))).collect();
                self.send(&net::shops::sell(&sold));
            }
            _ => {}
        }
        self.send(&net::shops::quit());
        self.redraw(&[&wnd::TRADE, &wnd::SHOP, &wnd::SHOP_BASKET, &wnd::SHOP_COUNT]);
    }

    /// How many of a row of the list of what is sold to the shop are carried.
    fn shop_carried(&self, row: usize) -> u32 {
        let Some(Shop::Sell(bids)) = &self.shops.shop else { return 0 };
        bids.get(row).and_then(|bid| self.bag.at(bid.index)).map_or(0, |item| u32::from(item.amount))
    }

    /// A row was dragged out of the list and let go over the basket, or out of the basket and let go over
    /// the list (`DragAndDrop` of the three windows): one thing goes across at once, and so does a stack while
    /// the box "Toggle Item Amount" is ticked and something is sold; of more, the client asks how many. What
    /// is worn is bought one at a time without the question.
    pub(in crate::web::app) fn shop_let_go(&mut self, dragged: Drag, onto: Drop) {
        let selling = matches!(self.shops.shop, Some(Shop::Sell(_)));
        match (dragged, onto, &self.shops.shop) {
            (Drag::Offer(row), Drop::Basket, Some(Shop::Buy(offers))) => match offers.get(row) {
                Some(offer) if crate::shops::bought_singly(offer.kind) => self.shop_in(row, 1),
                Some(_) => self.shop_ask(Across::In(row), 1),
                None => {}
            },
            (Drag::Offer(row), Drop::Basket, Some(Shop::Sell(_))) => {
                let left = self.shop_carried(row).saturating_sub(self.shop_kept(row));
                if left == 1 || self.shops.whole_stacks { self.shop_in(row, left) } else { self.shop_ask(Across::In(row), left) }
            }
            (Drag::Basket(row), Drop::Shop, Some(_)) => {
                let kept = self.shop_kept(row);
                if kept == 1 || (selling && self.shops.whole_stacks) { self.shop_out(row, kept) } else { self.shop_ask(Across::Out(row), kept) }
            }
            _ => {}
        }
    }

    /// A double click on a row: the client's two windows do something with it only while something is sold
    /// and the box "Toggle Item Amount" is ticked (`UIItemShopWnd::OnLBtnDblClk`, `UIItemSellWnd::OnLBtnDblClk`):
    /// then the row goes across as if it were dragged.
    pub(in crate::web::app) fn shop_used(&mut self, dragged: Drag, onto: Drop) {
        if !self.shops.whole_stacks || !matches!(self.shops.shop, Some(Shop::Sell(_))) {
            return;
        }
        self.shop_let_go(dragged, onto);
    }

    /// The box that asks how many is opened, with that number in it.
    fn shop_ask(&mut self, across: Across, shown: u32) {
        if shown > 0 {
            self.shops.counting = Some((across, shown));
            self.redraw(&[&wnd::SHOP_COUNT]);
        }
    }

    /// The number of the box was given (`UIItemDropCntWnd::SendMsg`, `UM_COMMAND`): what it means by what was
    /// asked (`crate::shops::count_asked`).
    pub(in crate::web::app) fn shop_counted(&mut self) {
        let Some((across, _)) = self.shops.counting.take() else { return };
        let typed = field(&wnd::SHOP_COUNT.field());
        let buying = matches!(self.shops.shop, Some(Shop::Buy(_)));
        match across {
            Across::In(row) if buying => {
                if let Some(amount) = crate::shops::count_asked(&typed, 0, Asked::Buying) {
                    self.shop_in(row, amount);
                }
            }
            Across::In(row) => {
                let left = self.shop_carried(row).saturating_sub(self.shop_kept(row));
                if let Some(amount) = crate::shops::count_asked(&typed, left, Asked::Moving) {
                    self.shop_in(row, amount);
                }
            }
            Across::Out(row) => {
                if let Some(amount) = crate::shops::count_asked(&typed, self.shop_kept(row), Asked::Moving) {
                    self.shop_out(row, amount);
                }
            }
        }
        self.redraw(&[&wnd::SHOP, &wnd::SHOP_BASKET, &wnd::SHOP_COUNT]);
    }

    /// So many of a row go into the basket (`UM_DROP` of `UIItemPurchaseWnd` and `UIItemSellWnd`). Of what is
    /// bought: a thing that is worn only once (the client's line 119), no more than 30000 of an item (its
    /// line 1704), and nothing the character's zeny do not pay for with what is in the basket already (55).
    fn shop_in(&mut self, row: usize, amount: u32) {
        let mut amount = amount;
        if let Some(Shop::Buy(offers)) = &self.shops.shop {
            let Some(offer) = offers.get(row).copied() else { return };
            let total: i64 = self.shops.basket.iter().filter_map(|(row, amount)| Some(i64::from(offers.get(*row)?.discounted) * i64::from(*amount))).sum();
            let kept = self.shop_kept(row);
            if crate::shops::bought_singly(offer.kind) && kept > 0 {
                return self.chat_line(Self::words(119, "Please avoid buying 2 of the same items at one time."), RED);
            }
            let (fits, limited) = crate::shops::into_basket(kept, amount);
            if limited {
                self.chat_line(Self::words(1704, "It is not possible to purchase the same item more than %d pieces at a time").replace("%d", &MOST_BOUGHT.to_string()), RED);
            }
            amount = fits;
            if amount > 0 && self.hero.zeny() < total + i64::from(amount) * i64::from(offer.discounted) {
                return self.chat_line(Self::words(55, "You do not have enough zeny."), RED);
            }
        }
        if amount > 0 {
            match self.shops.basket.iter_mut().find(|(kept, _)| *kept == row) {
                Some((_, kept)) => *kept = kept.saturating_add(amount as u16),
                None => self.shops.basket.push((row, amount as u16)),
            }
        }
        self.redraw(&[&wnd::SHOP, &wnd::SHOP_BASKET]);
    }

    /// So many of a row come back out of the basket (`UIItemShopWnd::SendMsg`, `UM_DROP`).
    fn shop_out(&mut self, row: usize, amount: u32) {
        for (kept, held) in &mut self.shops.basket {
            if *kept == row {
                *held = held.saturating_sub(amount.min(u32::from(u16::MAX)) as u16);
            }
        }
        self.shops.basket.retain(|(_, held)| *held > 0);
        self.redraw(&[&wnd::SHOP, &wnd::SHOP_BASKET]);
    }
}
