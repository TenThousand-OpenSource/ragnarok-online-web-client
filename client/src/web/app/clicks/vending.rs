//! A merchant's shop: making one's own out of the cart and closing it (`UIMerchantShopMakeWnd`,
//! `UIMerchantMirrorItemWnd`, `UIMerchantItemMyShopWnd`), the sign over a vendor's head
//! (`UIMerchantShopTitle`), and buying from a vendor (`UIMerchantItemShopWnd`, `UIMerchantItemPurchaseWnd`,
//! `UIMerchantItemPurchaseMSGWnd`), with the states 9 to 12 of the box that asks how many.

use super::*;
use crate::shops::{Asked, bought_singly, count_asked};
use crate::net::vending;
use crate::vending::MOST_AMOUNT;

/// How long the client waits before it asks for a vendor's list again (`MM_REQ_BUY_FROMMC`).
const ASK_AGAIN_MS: f64 = 1000.0;

impl App {
    /// A press of this system (`crate::press::vending`).
    pub(super) fn vending_pressed(&mut self, press: crate::press::vending::Press) {
        use crate::press::vending::Press;
        match press {
            // The window is a form, and its OK sends it (`stall_sent`).
            Press::StallOk | Press::CountOk => {}
            // Not opening after all tells the server so (`MM_REQ_OPENSTORE` with nought), which lets go of
            // the skill's use.
            Press::StallCancel => {
                if self.vending.making.take().is_some() {
                    self.send_on_map(&vending::open("", false, &[]));
                }
                (self.vending.asks, self.vending.counting) = (None, None);
                self.redraw(&wnd::VENDING_WINDOWS);
            }
            Press::Yes | Press::No => {
                if self.vending.asks.take().is_some() && press == Press::Yes {
                    self.stall_opened(true);
                }
                self.redraw(&[&wnd::VENDING_ASKS]);
            }
            // CZ_REQ_CLOSESTORE, with no question asked (`UIMerchantItemMyShopWnd`, at its command 201).
            Press::MineClose => {
                if self.vending.mine.take().is_some() {
                    self.send_on_map(&vending::close());
                }
                self.redraw(&[&wnd::VENDING_MINE]);
            }
            // One press on a sign asks for the list only with the client's `/shopping` on
            // (`UIMerchantShopTitle::OnLBtnDown`); two always do (`vending_used`).
            Press::Sign(vendor) => {
                if self.option("TT_SHOPPING_ON_OFF") {
                    self.vending_ask(vendor);
                }
            }
            Press::Buy => {
                if let Some(visit) = self.vending.visit.as_mut().filter(|visit| !visit.basket.is_empty()) {
                    visit.stage = Stage::Confirming;
                    self.redraw(&wnd::VENDING_WINDOWS);
                }
            }
            Press::Leave | Press::Refuse => {
                (self.vending.visit, self.vending.counting) = (None, None);
                self.redraw(&wnd::VENDING_WINDOWS);
            }
            Press::Confirm => {
                if let Some(visit) = self.vending.visit.as_mut().filter(|visit| visit.stage == Stage::Confirming) {
                    visit.stage = Stage::Sent;
                    let wanted: Vec<(u16, u16)> =
                        visit.basket.iter().filter_map(|(row, amount)| Some((visit.list.sold.get(*row)?.item.index, *amount))).collect();
                    let packet = vending::buy(visit.list.vendor, visit.list.shop, &wanted);
                    self.send_on_map(&packet);
                }
                self.redraw(&wnd::VENDING_WINDOWS);
            }
        }
    }

    /// A double click on a vendor's sign asks what the shop sells (`UIMerchantShopTitle::OnLBtnDblClk`).
    pub(in crate::web::app) fn vending_used(&mut self, vendor: u32) {
        self.vending_ask(vendor);
    }

    /// CZ_REQ_BUY_FROMMC, as `MM_REQ_BUY_FROMMC` sends it: not of oneself, not while a vendor's list is open,
    /// and no oftener than once a second.
    fn vending_ask(&mut self, vendor: u32) {
        let now = js_sys::Date::now();
        if vendor == self.own_account()
            || self.vending.visit.as_ref().is_some_and(|visit| visit.stage != Stage::Sent)
            || now - self.vending.asked_at < ASK_AGAIN_MS
        {
            return;
        }
        self.vending.asked_at = now;
        self.send_on_map(&vending::ask(vendor));
    }

    /// What was typed into the window of the shop being made is kept, before the window is drawn anew or
    /// read: the title and each row's price.
    pub(in crate::web::app) fn stall_kept(&mut self) {
        let Some(making) = &mut self.vending.making else { return };
        if element(&wnd::VENDING_STALL.field()).is_none() {
            return;
        }
        making.title = field(&wnd::VENDING_STALL.field());
        for (row, stalled) in making.rows.iter_mut().enumerate() {
            stalled.price = field(&wnd::stall_price_field(row));
        }
    }

    /// The windows of the shops are drawn anew for something that changed outside them (a name that arrived,
    /// the cart's things): what is being typed into a shop being made is kept first.
    pub(in crate::web::app) fn vending_redrawn(&mut self) {
        self.stall_kept();
        self.redraw(&wnd::VENDING_WINDOWS);
    }

    /// Something was dragged out of its window and let go over one of a shop's: a thing of the cart goes into
    /// the shop being made or back out of it, a row of a vendor's list into the basket or back out of it.
    pub(in crate::web::app) fn vending_let_go(&mut self, dragged: Drag, onto: Drop) {
        match (dragged, onto) {
            (Drag::Stock(index), Drop::Stall) => {
                let Some(making) = &self.vending.making else { return };
                let left = self.cart.items.iter().find(|item| item.index == index).map_or(0, |item| item.amount).saturating_sub(making.taken(index));
                self.vending_across(Crossing::Stall(index), u32::from(left), left == 1);
            }
            (Drag::Stalled(row), Drop::Stock) => {
                let Some(amount) = self.vending.making.as_ref().and_then(|making| making.rows.get(row)).map(|stalled| stalled.amount) else { return };
                self.vending_across(Crossing::Unstall(row), u32::from(amount), amount == 1);
            }
            (Drag::Vended(row), Drop::Wants) => {
                let Some((kind, left, kept)) = self.vending.visit.as_ref().and_then(
                    |visit| Some((
                        visit.list.sold.get(row)?.item.kind,
                        visit.list.sold[row].item.amount.saturating_sub(visit.kept(row)),
                        visit.kept(row)
                    ))
                ) else { return };
                // A thing that is worn goes in singly, and only once (the client's line 119).
                if bought_singly(kind) && kept > 0 {
                    return self.chat_line(Self::words(119, "Please avoid buying 2 of the same items at one time."), RED);
                }
                self.vending_across(Crossing::Basket(row), u32::from(left), bought_singly(kind));
            }
            (Drag::Wanted(row), Drop::Vendor) => {
                let Some(kept) = self.vending.visit.as_ref().map(|visit| visit.kept(row)) else { return };
                self.vending_across(Crossing::Unbasket(row), u32::from(kept), kept == 1);
            }
            _ => {}
        }
    }

    /// A thing goes across: at once when it goes singly, else the box asks how many of what there are.
    fn vending_across(&mut self, across: Crossing, most: u32, singly: bool) {
        if most == 0 {
            return;
        }
        if singly {
            return self.vending_moved(across, 1);
        }
        self.vending.counting = Some((across, most));
        self.redraw(&[&wnd::VENDING_COUNT]);
    }

    /// The number of the box was given. Into the shop being made and back out of the basket a number under
    /// one counts as one (the box's states 9 and 11); into the basket any number is taken and cut to what
    /// may be bought (state 12 and the basket's `UM_DROP`).
    pub(in crate::web::app) fn vending_counted(&mut self) {
        let Some((across, most)) = self.vending.counting.take() else { return };
        let typed = field(&wnd::VENDING_COUNT.field());
        let amount = match across {
            Crossing::Stall(_) | Crossing::Unbasket(_) => count_asked(&typed, most, Asked::Storing),
            Crossing::Unstall(_) => count_asked(&typed, most, Asked::Moving),
            Crossing::Basket(_) => count_asked(&typed, most, Asked::Buying),
        };
        self.redraw(&[&wnd::VENDING_COUNT]);
        if let Some(amount) = amount {
            self.vending_moved(across, amount);
        }
    }

    fn vending_moved(&mut self, across: Crossing, amount: u32) {
        match across {
            Crossing::Stall(index) => self.stall_in(index, amount),
            Crossing::Unstall(row) => self.stall_out(row, amount),
            Crossing::Basket(row) => self.basket_in(row, amount),
            Crossing::Unbasket(row) => {
                if let Some(visit) = &mut self.vending.visit {
                    if let Some(kept) = visit.basket.iter_mut().find(|(kept, _)| *kept == row) {
                        kept.1 = kept.1.saturating_sub(amount.min(u32::from(u16::MAX)) as u16);
                    }
                    visit.basket.retain(|(_, amount)| *amount > 0);
                }
                self.redraw(&wnd::VENDING_WINDOWS);
            }
        }
    }

    /// So many of a thing of the cart go into the shop being made (`UIMerchantShopMakeWnd`, `UM_DROP`): no
    /// more than 30000 of one thing (the client's line 1705), to the row the thing already has, or to a new
    /// one while the shop has room; with no room it is let go of and nothing happens.
    fn stall_in(&mut self, index: u16, amount: u32) {
        self.stall_kept();
        let Some(making) = &mut self.vending.making else { return };
        let before = u32::from(making.taken(index));
        let amount = amount.min(MOST_AMOUNT.saturating_sub(before));
        if before + amount >= MOST_AMOUNT {
            let line = Self::words(1705, "It is not possible to purchase the same item more than %d pieces at a time")
                .replacen("%d", &MOST_AMOUNT.to_string(), 1);
            self.chat_line(line, RED);
        }
        let Some(making) = &mut self.vending.making else { return };
        let room = making.rows.len() < usize::from(making.slots);
        match making.rows.iter_mut().find(|stalled| stalled.index == index) {
            Some(stalled) => stalled.amount += amount as u16,
            None if room && amount > 0 => making.rows.push(Stalled { index, amount: amount as u16, price: "0".to_owned() }),
            None => {}
        }
        self.redraw(&wnd::VENDING_WINDOWS);
    }

    /// So many of a row of the shop being made go back; an emptied row is taken out, and the rows under it
    /// move up with their prices.
    fn stall_out(&mut self, row: usize, amount: u32) {
        self.stall_kept();
        let Some(making) = &mut self.vending.making else { return };
        if let Some(stalled) = making.rows.get_mut(row) {
            stalled.amount = stalled.amount.saturating_sub(amount.min(u32::from(u16::MAX)) as u16);
        }
        making.rows.retain(|stalled| stalled.amount > 0);
        self.redraw(&wnd::VENDING_WINDOWS);
    }

    /// So many of a row of a vendor's list go into the basket (`UIMerchantItemPurchaseWnd`, `UM_DROP`): no
    /// more than the vendor has left (the client's line 1706), no more than 30000 (1704), and nothing that
    /// the zeny in hand does not pay for with what is in the basket already (55); each said in red.
    fn basket_in(&mut self, row: usize, amount: u32) {
        let Some(visit) = &self.vending.visit else { return };
        let Some(sold) = visit.list.sold.get(row) else { return };
        let (kept, price, basket) = (u32::from(visit.kept(row)), u64::from(sold.price), visit.total());
        let left = u32::from(sold.item.amount).saturating_sub(kept);
        let mut said = Vec::new();
        let mut amount = amount;
        if amount > left {
            amount = left;
            said.push(Self::words(1706, "Can purchase upto %d pieces of the same item at a time.").replacen("%d", &left.to_string(), 1));
        }
        if kept + amount > MOST_AMOUNT {
            amount = MOST_AMOUNT.saturating_sub(kept);
            said.push(
                Self::words(1704, "It is not possible to purchase the same item more than %d pieces at a time")
                    .replacen("%d", &MOST_AMOUNT.to_string(), 1)
            );
        }
        let zeny = self.hero.zeny().max(0) as u64;
        if zeny < price * u64::from(amount) + basket {
            amount = 0;
            said.push(Self::words(55, "You do not have enough zeny."));
        }
        for line in said {
            self.chat_line(line, RED);
        }
        let Some(visit) = self.vending.visit.as_mut().filter(|_| amount > 0) else { return };
        match visit.basket.iter_mut().find(|(kept, _)| *kept == row) {
            Some(kept) => kept.1 += amount as u16,
            None => visit.basket.push((row, amount as u16)),
        }
        self.redraw(&wnd::VENDING_WINDOWS);
    }

    /// The window of the shop being made was sent with its OK.
    pub(in crate::web::app) fn stall_sent(&mut self) {
        self.stall_kept();
        self.stall_opened(false);
    }

    /// The shop is asked to be opened, after the client's checks in its order (`UIMerchantShopMakeWnd`, at
    /// `ID_OK`): every price a number (its line 602), something to sell (2494), nothing unidentified (603), a
    /// thing that costs nothing only if the player says so (604, asked unless `free` says it was), a title
    /// (225) with no word that is not let be written (14), and no more zeny in all than a character can hold
    /// (3826). The client says these in message boxes; here the chat says them, in red.
    fn stall_opened(&mut self, free: bool) {
        let Some(making) = &self.vending.making else { return };
        let prices: Option<Vec<u32>> = making.rows.iter().map(|stalled| crate::vending::price_typed(&stalled.price)).collect();
        let unidentified = making.rows.iter().any(|stalled| self.cart.items.iter().any(|item| item.index == stalled.index && !item.identified));
        let zeny = self.hero.zeny().max(0) as u64;
        let refused = match &prices {
            None => Some((602, "Only the numbers (0~9) are available.")),
            Some(prices) if prices.is_empty() => Some((2494, "This item you want to sell is not registered. Please register the item first.")),
            Some(_) if unidentified => Some((603, "You cannot sell unidentified items.")),
            Some(prices) if prices.contains(&0) && !free => {
                self.vending.asks = Some(VendingAsk::Free);
                return self.redraw(&[&wnd::VENDING_ASKS]);
            }
            Some(_) if making.title.trim().is_empty() => Some((225, "Please enter the stall's name")),
            // A title with a word that is not let be written (`g_forbiddenWords.Allows`, `crate::manner`).
            Some(_) if data::foul(&making.title) => Some((14, "Foul Language Detected.")),
            Some(prices) if zeny + prices.iter().zip(&making.rows).map(
                |(price, stalled)| u64::from(*price) * u64::from(stalled.amount)
            ).sum::<u64>() > i32::MAX as u64 => {
                Some((3826, "The total amount of items to sell exceeds the amount of Zeny you can have."))
            }
            Some(_) => None,
        };
        if let Some((line, otherwise)) = refused {
            return self.chat_line(Self::words(line, otherwise), RED);
        }
        let things: Vec<(u16, u16, u32)> =
            making.rows.iter().zip(prices.unwrap_or_default()).map(|(stalled, price)| (stalled.index, stalled.amount, price)).collect();
        let packet = vending::open(making.title.trim(), true, &things);
        self.send_on_map(&packet);
    }
}
