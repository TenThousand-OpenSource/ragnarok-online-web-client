//! An NPC's shop: the question whether to buy or to sell, and the client's two windows of a shop, the list
//! (`UIItemShopWnd`) and the basket of what is bought (`UIItemPurchaseWnd`) or sold (`UIItemSellWnd`), with the
//! box that asks how many go across.

use super::*;
use crate::item_names::Named;
use crate::shops::name_width;
use crate::press::shops::Press;

/// A row of a shop's windows as they are drawn: the item, whether it is identified, its name, the two prices
/// (the shop's and this character's) and how many of it the row stands for.
struct Line {
    row: usize,
    id: u32,
    identified: bool,
    name: String,
    prices: (u32, u32),
    amount: u32,
}

/// A row's price as the client writes it (`UIItemShopWnd::Paint`): the one price, or the shop's and this
/// character's with an arrow between them; `unit` after it ("Z", and "Zeny" among what is sold).
fn priced((price, own): (u32, u32), unit: &str) -> String {
    if price == own { format!(" {} {unit}", grouped(i64::from(price))) } else { format!(" {price} -> {} {unit}", grouped(i64::from(own))) }
}

impl App {
    /// The shop's question (the client's `UIChooseSellBuyWnd`): its message box with the three buttons.
    pub(in crate::web) fn trade_window(&self) -> String {
        let Some(Shop::Choose(_)) = &self.shops.shop else { return String::new() };
        format!(
            "{}{}{}{}",
            art("win_msgbox.bmp", 0, 200, 280, 120),
            picture_press(144, 296, "btn_buy", Press::Buy),
            picture_press(189, 296, "btn_sell", Press::Sell),
            picture_press(234, 296, "btn_cancel", Press::Cancel)
        )
    }

    /// How many of a row of the shop's list are in the basket.
    pub(in crate::web) fn shop_kept(&self, row: usize) -> u32 {
        self.shops.basket.iter().find(|(kept, _)| *kept == row).map_or(0, |(_, amount)| u32::from(*amount))
    }

    /// The rows of the shop: what it sells, or what it takes of what is carried (how many of each are carried).
    fn shop_lines(&self) -> Vec<Line> {
        match &self.shops.shop {
            Some(Shop::Buy(offers)) => offers
                .iter()
                .enumerate()
                .map(|(row, offer)| {
                    let name = data::item_title(&Named { id: offer.id, kind: offer.kind, identified: true, ..Named::default() }, false);
                    Line { row, id: offer.id, identified: true, name, prices: (offer.price, offer.discounted), amount: 0 }
                })
                .collect(),
            Some(Shop::Sell(bids)) => bids
                .iter()
                .enumerate()
                .filter_map(|(row, bid)| {
                    let item = self.bag.items.iter().find(|item| item.index == bid.index)?;
                    Some(Line { row, id: item.id, identified: item.identified, name: Self::item_name(item), prices: (bid.price, bid.overcharged), amount: u32::from(item.amount) })
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    /// One row of either window (`item_row`): the name 36 from the cell, as wide as the client leaves it beside
    /// the price, the price ending 18 from the window's edge.
    fn shop_row(line: &Line, amount: Option<u32>, usual: i32, unit: &str, marks: &str) -> String {
        let resource = data::item_as(line.id, line.identified).map(|(_, resource)| resource);
        item_row(&Row {
            resource: resource.as_deref(),
            amount,
            name: &line.name,
            name_at: (36, name_width(line.prices.0, line.prices.0 != line.prices.1, usual)),
            aside: Some(Aside { html: &escape(&priced(line.prices, unit)), inset: 3, colour: "inherit" }),
            marks: &format!("{marks} {}", item_marks(line.id, line.identified)),
        })
    }

    /// The shop's list (`UIItemShopWnd`, 280 by 312): eight rows of what is sold, or of what is carried that
    /// the shop takes with how many are left of each, and at the bottom the box "Toggle Item Amount" (the
    /// client's line 295), which the client lets be ticked only while selling. The window takes back what is
    /// dragged out of the basket.
    pub(in crate::web) fn shop_window(&self) -> String {
        let selling = match &self.shops.shop {
            Some(Shop::Buy(_)) => false,
            Some(Shop::Sell(_)) => true,
            _ => return String::new(),
        };
        let rows: String = self
            .shop_lines()
            .iter()
            .filter_map(|line| {
                let left = line.amount.saturating_sub(self.shop_kept(line.row));
                let marks = format!("{} {}", dragged(Drag::Offer(line.row)), used(Use::ShopAdd(line.row)));
                (!selling || left > 0).then(|| Self::shop_row(line, selling.then_some(left), 100, "Z", &marks))
            })
            .collect();
        let title = if selling { Self::words(185, "Available Items to sell") } else { Self::words(186, "Shop Items") };
        format!(
            "<div {}>{}{}{}</div>",
            dropped(Drop::Shop),
            list_frame(280, 312, &title),
            list_of(40, 225, 8, &rows, ""),
            check_words(12, 292, self.shops.whole_stacks, &Self::words(295, "Toggle Item Amount."), Press::WholeStacks),
        )
    }

    /// The basket (`UIItemPurchaseWnd`, `UIItemSellWnd`, 280 by 152): three rows of what was put aside with
    /// how many of each, the sum under them ("Total : n Zeny", 12 in and 21 from the bottom), and its two
    /// buttons, Buy or Sell at 106 from the right edge and Cancel at 61, 24 from the bottom. The client writes
    /// "Z" after a price of what is bought and "Zeny" after one of what is sold.
    pub(in crate::web) fn shop_basket_window(&self) -> String {
        let selling = match &self.shops.shop {
            Some(Shop::Buy(_)) => false,
            Some(Shop::Sell(_)) => true,
            _ => return String::new(),
        };
        let lines = self.shop_lines();
        let kept: Vec<(&Line, u32)> = self.shops.basket.iter().filter_map(|(row, amount)| Some((lines.iter().find(|line| line.row == *row)?, u32::from(*amount)))).collect();
        let rows: String = kept
            .iter()
            .map(|(line, amount)| {
                let marks = format!("{} {}", dragged(Drag::Basket(line.row)), used(Use::ShopRemove(line.row)));
                Self::shop_row(line, Some(*amount), if selling { 100 } else { 110 }, if selling { "Zeny" } else { "Z" }, &marks)
            })
            .collect();
        let total: i64 = kept.iter().map(|(line, amount)| i64::from(line.prices.1) * i64::from(*amount)).sum();
        let (title, deal) = if selling { (Self::words(168, "Selling Items"), "btn_sell") } else { (Self::words(166, "Buying Items"), "btn_buy") };
        format!(
            "<div {}>{}{}{}{}{}</div>",
            dropped(Drop::Basket),
            list_frame(280, 152, &title),
            list_of(40, 225, 3, &rows, ""),
            total_line(131, total),
            picture_press(174, 128, deal, Press::Ok),
            picture_press(219, 128, "btn_cancel", Press::Cancel),
        )
    }

    /// How many of a row go across the shop's windows: the box that asks how many, with the row's name.
    pub(in crate::web) fn shop_count_window(&self) -> String {
        let Some((across, shown)) = self.shops.counting else { return String::new() };
        let (crate::web::app::Across::In(row) | crate::web::app::Across::Out(row)) = across;
        let name = self.shop_lines().into_iter().find(|line| line.row == row).map(|line| line.name).unwrap_or_default();
        count_box_of(&windows::SHOP_COUNT, &name, shown, Form::ShopCount, Press::CountOk)
    }
}
