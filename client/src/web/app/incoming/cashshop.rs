//! What the server says of the cash shop (`crate::net::cashshop`): its window opened, its lists, the answer to
//! a purchase.

use super::*;
use crate::cashshop::LISTS;
use crate::net::cashshop;

impl App {
    /// The window is made (`UIWindowMgr::EnsureWindow`, `WID_CASHSHOP_WND`; `UICashShopWnd::OnCreate`, and
    /// `UM_SELECT_CASHSHOP_TAB`, `UM_SET_CASHSHOP_BALANCE` of `CGameMode::ReceiveCashShopBanner`): on its
    /// first tab, or on the one the server names when that is not the first; with the points the server
    /// tells, nothing in the list of what is bought and no free points to use; the sale for one account asked
    /// for, and the first banner shown.
    fn cashshop_opened(&mut self, cash: u32, points: u32, tab: u32) {
        let lists = std::mem::take(&mut self.cashshop.lists);
        let (limited_asked, banner_turns) = (self.cashshop.limited_asked, self.cashshop.banner_turns);
        self.cashshop = CashShop { open: true, lists, cash, points, limited_asked, banner_turns, ..CashShop::default() };
        let tab = tab as usize;
        if tab != 0 && tab < LISTS {
            (self.cashshop.tab, self.cashshop.page_size) = (tab, crate::cashshop::page_size(tab));
        }
        self.cashshop_limited_wanted();
        self.cashshop_banner_shown();
        self.render();
    }

    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn cashshop_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // `CGameMode::ReceiveCashShopBanner`: an open window closes; otherwise it opens.
            cashshop::ZC_SE_CASHSHOP_OPEN => {
                let Some((cash, points, tab)) = cashshop::opened(packet) else { return true };
                if self.cashshop.open {
                    self.cashshop_closed();
                } else {
                    self.cashshop_opened(cash, points, tab);
                }
            }
            // `CGameMode::ReceiveTypedItemList`: the things are added to their list, and named once the
            // gateway has told of them.
            cashshop::ZC_ACK_SCHEDULER_CASHITEM => {
                let Some((tab, goods)) = cashshop::listed(packet) else { return true };
                data::want_items(goods.iter().map(|good| good.item), || dispatch(Action::ItemsKnown));
                if let Some(list) = self.cashshop.lists.get_mut(usize::from(tab)) {
                    list.extend(goods);
                }
                self.redraw(&[&wnd::CASH_SHOP]);
            }
            // `CGameMode::ReceiveRequestResult`: bought, the points are as the server says and the list of
            // what is bought is empty again; refused, the chat says why.
            cashshop::ZC_SE_PC_BUY_CASHITEM_RESULT => {
                let Some(bought) = cashshop::bought(packet) else { return true };
                let answer = crate::cashshop::answer(bought.result);
                if answer.done && self.cashshop.open {
                    (self.cashshop.cash, self.cashshop.points) = (bought.cash, bought.points);
                    self.cashshop.cart.clear();
                    (self.cashshop.top, self.cashshop.use_points) = (0, "0".to_owned());
                }
                if let Some(line) = answer.line {
                    let words = Self::words(line, "");
                    if answer.boxed {
                        let words = format!("- {} : {words}", bought.result);
                        self.cashshop.asks = Some(CashShopAsk::Told(words.clone(), true));
                        self.chat_line(words, RED);
                    } else if !words.is_empty() {
                        self.chat_line(words, RED);
                    }
                }
                self.redraw(&[&wnd::CASH_SHOP, &wnd::CASH_SHOP_ASKS]);
                set_field(&wnd::cash_shop_points(), &self.cashshop.use_points);
            }
            _ => return false,
        }
        true
    }
}
