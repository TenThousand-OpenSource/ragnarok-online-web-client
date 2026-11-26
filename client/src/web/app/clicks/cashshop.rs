//! The cash shop: its window asked for and closed, its tabs and pages, the list of what is bought, a search,
//! the purchase (`crate::cashshop`, the client's `UICashShopWnd::SendMsg`).

use super::*;
use crate::cashshop::{self, Buying, FOUND, LIMITED};
use crate::press::Named;
use crate::press::cashshop::Press;

/// How long a banner shows before the next (`UICashShopWnd_Banner::OnProcess`), and how long the client waits
/// before it asks for the sale for one account again (`MM_GET_ACCOUNT_LIMITED_SALE_LIST`), in milliseconds.
const BANNER_SHOWS: i32 = 5000;
const LIMITED_WAITS: f64 = 5000.0;

/// Opens an address of the web beside the game (the client's `ShellExecute`), if it is one.
fn browse(address: &str) {
    if address.starts_with("https://") || address.starts_with("http://") {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target_and_features(address, "_blank", "noopener");
        }
    }
}

impl App {
    /// The key of the cash shop (`HK_CASHSHOPWND` of `UIWindowMgr`'s keys): the window is asked for, or
    /// closed if it is open. The key is used.
    pub(in crate::web::app) fn cashshop_key(&mut self) -> bool {
        self.pressed(if self.cashshop.open { Press::Close } else { Press::Window });
        true
    }

    /// As a map is entered while the client has none of the shop's lists, they are asked for (`CGameMode`'s
    /// start of a map, at `AreInventoryCategoriesEmpty`); the server tells them once.
    pub(in crate::web::app) fn cashshop_lists_wanted(&mut self) {
        if self.cashshop.lists.iter().all(Vec::is_empty) {
            self.send_on_map(&net::cashshop::ask_lists());
        }
    }

    /// The sale for one account is asked for, unless it was within the last five seconds: then the chat says
    /// how many seconds are left (`CGameMode::SendMsg`, `MM_GET_ACCOUNT_LIMITED_SALE_LIST`; line 3230).
    pub(in crate::web::app) fn cashshop_limited_wanted(&mut self) {
        let now = crate::web::now();
        if let Some(waited) = self.cashshop.limited_asked.map(|asked| now - asked).filter(|waited| *waited < LIMITED_WAITS) {
            let left = 5 - (waited / 1000.0) as u32;
            return self.chat_line(format!("{left}{}", Self::words(3230, " seconds left before it become usable again.")), "#00ffc8");
        }
        self.cashshop.limited_asked = Some(now);
        self.send_on_map(&net::cashshop::ask_limited());
    }

    /// The next banner is due five seconds from now, unless another is shown before.
    pub(in crate::web::app) fn cashshop_banner_shown(&mut self) {
        self.cashshop.banner_turns = self.cashshop.banner_turns.wrapping_add(1);
        if data::cashshop_banners().len() > 1 {
            let (what, with) = Press::BannerDue(self.cashshop.banner_turns).name();
            after(BANNER_SHOWS, Action::Click(what.to_owned(), with));
        }
    }

    /// What stands in the window's two fields is kept, so that the window drawn anew shows it.
    fn cashshop_fields_kept(&mut self) {
        if let Some(sought) = written(&wnd::CASH_SHOP.field()) {
            self.cashshop.sought = sought;
        }
        if let Some(points) = written(&wnd::cash_shop_points()) {
            self.cashshop.use_points = points;
        }
    }

    /// The shop says a line of the message table in a message box, with a number written into it where it
    /// asks for one.
    fn cashshop_says(&mut self, line: usize, number: Option<i32>) {
        let words = crate::numbers::filled(&Self::words(line, ""), &[&number.unwrap_or(0).to_string()]);
        self.cashshop.asks = Some(CashShopAsk::Told(words, false));
    }

    /// So many more of a thing go into the list of what is bought (`UICashShopWnd_Cart::SendMsg`,
    /// `UM_ADDITEM`): a new kind is shown at once, the list moved down by one.
    fn cashshop_put(&mut self, good: &cashshop::Good, amount: i32) {
        let kinds = self.cashshop.cart.len();
        if let Some(said) = cashshop::put(&mut self.cashshop.cart, good, amount) {
            let (line, number) = said.line();
            self.cashshop_says(line, number);
        }
        let moved = i32::from(self.cashshop.cart.len() > kinds);
        self.cashshop.top = cashshop::scrolled(self.cashshop.top, moved, self.cashshop.cart.len());
    }

    /// The line was searched with (`ID_SEARCH`): what was found is shown as a list of its own, with no tab
    /// picked; nothing found is told in a message box (line 1069).
    pub(in crate::web::app) fn cashshop_searched(&mut self) {
        self.cashshop_fields_kept();
        let sought = self.cashshop.sought.clone();
        if !sought.is_empty() {
            let found = cashshop::found(&self.cashshop.lists, &sought, |item| {
                data::item_title(&crate::item_names::Named { id: item, identified: true, ..Default::default() }, false)
            });
            let none = found.is_empty();
            self.cashshop.lists[FOUND] = found;
            if none {
                self.cashshop_says(1069, None);
            } else {
                (self.cashshop.tab, self.cashshop.page) = (FOUND, 0);
            }
        }
        self.redraw(&[&wnd::CASH_SHOP, &wnd::CASH_SHOP_ASKS]);
    }

    /// "Purchase" (`ID_BUY`): the client refuses in a message box, or asks whether it is meant.
    fn cashshop_buying(&mut self) {
        let points: i64 = self.cashshop.use_points.trim().parse().unwrap_or(0);
        match cashshop::buying(&self.cashshop.cart, i64::from(self.cashshop.cash), points) {
            Buying::Nothing => {}
            Buying::Refused(line) => self.cashshop_says(line, None),
            Buying::Asked(line, numbers) => {
                let numbers: Vec<String> = numbers.iter().map(i64::to_string).collect();
                let with: Vec<&str> = numbers.iter().map(String::as_str).collect();
                self.cashshop.asks = Some(CashShopAsk::Buy(crate::numbers::filled(&Self::words(line, ""), &with), points as u32));
            }
        }
    }

    /// The window is put away, and the server told (`UIWindowMgr::CloseWindow`, `WID_CASHSHOP_WND`).
    pub(in crate::web::app) fn cashshop_closed(&mut self) {
        if std::mem::take(&mut self.cashshop.open) {
            self.cashshop.asks = None;
            self.send_on_map(&net::cashshop::close());
        }
        self.render();
    }

    /// A press of this system (`crate::press::cashshop`).
    pub(super) fn cashshop_pressed(&mut self, press: Press) {
        use crate::press::Step;
        self.cashshop_fields_kept();
        match press {
            // `CGameMode::SendMsg`, `MM_REQUEST_CASH_SHOP`: asked of the server with tab 0, unless an NPC's
            // shop is open (`IsNavigationActive`). The answer opens the window, or closes it if it is open.
            Press::Window => {
                if !matches!(self.shops.shop, Some(Shop::Buy(_) | Shop::Sell(_))) {
                    self.send_on_map(&net::cashshop::open(0));
                }
                return;
            }
            Press::Close => return self.cashshop_closed(),
            // `ID_RADIOBTN`: another list is shown from its first page; the tab already open stays as it is.
            Press::Tab(tab) => {
                if cashshop::TABS.contains(&tab) && tab != self.cashshop.tab {
                    (self.cashshop.tab, self.cashshop.page, self.cashshop.page_size) = (tab, 0, cashshop::page_size(tab));
                }
            }
            Press::Page(turn) => self.cashshop.page = cashshop::turned(self.cashshop.page, self.cashshop.pages(), turn),
            Press::Add(place) => {
                if let Some(good) = self.cashshop.shown().get(self.cashshop.page * self.cashshop.page_size + place).cloned() {
                    self.cashshop_put(&good, 1);
                }
            }
            Press::More(item) | Press::Less(item) => {
                if let Some(good) = self.cashshop.cart.iter().find(|line| line.good.item == item).map(|line| line.good.clone()) {
                    self.cashshop_put(&good, if matches!(press, Press::More(_)) { 1 } else { -1 });
                }
            }
            Press::Take(item) => {
                cashshop::taken(&mut self.cashshop.cart, item);
                self.cashshop.top = cashshop::scrolled(self.cashshop.top, 0, self.cashshop.cart.len());
            }
            Press::Scroll(step) => {
                self.cashshop.top = cashshop::scrolled(self.cashshop.top, if step == Step::More { 1 } else { -1 }, self.cashshop.cart.len())
            }
            Press::Search => return self.cashshop_searched(),
            // "Refresh" is for the sale for one account alone (`m_refreshButton->Enable`).
            Press::Refresh => {
                if self.cashshop.tab == LIMITED {
                    self.cashshop_limited_wanted();
                }
                return;
            }
            // `ID_CHARGE`: the address of line 3301 of the message table is opened.
            Press::Charge => return browse(&Self::words(3301, "")),
            Press::Buy => self.cashshop_buying(),
            Press::Yes => {
                if let Some(CashShopAsk::Buy(_, points)) = self.cashshop.asks.take() {
                    let kinds: Vec<(u32, u32, u16)> =
                        self.cashshop.cart.iter().map(|line| (line.good.item, line.amount as u32, line.good.bought_as())).collect();
                    self.send_on_map(&net::cashshop::buy(points, &kinds));
                }
            }
            Press::No => self.cashshop.asks = None,
            // `UICashShopWnd_Banner::OnLBtnUp`: the banner's page.
            Press::Banner => return browse(data::cashshop_banners().get(self.cashshop.banner).map_or("", |banner| &banner.1)),
            Press::BannerPicked(banner) => {
                self.cashshop.banner = banner;
                self.cashshop_banner_shown();
            }
            Press::BannerDue(turn) => {
                if !self.cashshop.open || turn != self.cashshop.banner_turns {
                    return;
                }
                self.cashshop.banner = (self.cashshop.banner + 1) % data::cashshop_banners().len().max(1);
                self.cashshop_banner_shown();
            }
        }
        self.redraw(&[&wnd::CASH_SHOP, &wnd::CASH_SHOP_ASKS]);
    }
}
