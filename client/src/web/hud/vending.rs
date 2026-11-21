//! A merchant's shop: the signs over the vendors' heads (`UIMerchantShopTitle`), the window a shop is made in
//! with what the cart has for it beside it (`UIMerchantShopMakeWnd`, `UIMerchantMirrorItemWnd`), one's own
//! shop while it is open (`UIMerchantItemMyShopWnd`), and what it asks. The buyer's windows are in
//! `vending/buying.rs`.

use super::*;
use crate::vending::{TITLE, after_commission, price_typed};
use crate::numbers::price_colour;
use crate::press::vending::Press;
use crate::web::app::{Crossing, VendingAsk};

mod buying;

/// A price in the colour the client gives it by its size, with the rim it gives its figures
/// (`UIWindow::DrawAmount`).
pub(super) fn price_written(price: u32) -> String {
    let (colour, rim) = price_colour(price);
    let rim = rim.map_or(String::new(), |rim| format!(";{}", outlined(rim)));
    format!(r#"<span style="color:{colour}{rim}">{}</span>"#, grouped(i64::from(price)))
}

impl App {
    /// The sign of a shop, 140 by 34 (`UIMerchantShopTitle::OnCreate`, `Paint`): a white plate with a grey
    /// line inside it, 28 high, the shop's picture (`shop.bmp`) 3 in and the title 31 in and 8 down in 106 by
    /// 15, cut with three dots where it does not fit and told whole when pointed at. The world keeps it over
    /// its vendor's head (`world::sign`). Two presses on it ask what the shop sells, and one does with the
    /// client's `/shopping` on (`OnLBtnDblClk`, `OnLBtnDown`); the hand shows over it.
    pub(in crate::web) fn shop_sign(vendor: u32, title: &str) -> String {
        // The client writes stars for a word that is not let be written (`UIMerchantShopTitle::SendMsg`,
        // `crate::manner`), and cuts a title at the first end of a line.
        let title = data::starred(title);
        let title = title.split('\r').next().unwrap_or_default();
        [
            format!(
                r#"<div class="a" style="{};cursor:pointer" {} {} title="{}">"#,
                at(0, 0, crate::web::world::SIGN.0, crate::web::world::SIGN.1),
                pressed(Press::Sign(vendor)),
                used(Use::Sign(vendor)),
                escape(title)
            ),
            format!(r#"<div class="a" style="{};background:#fff;border-radius:4px"></div>"#, at(0, 0, 140, 28)),
            format!(r#"<div class="a" style="{};border:1px solid #c3c3c3;border-radius:3px;box-sizing:border-box"></div>"#, at(2, 2, 136, 24)),
            format!(r#"<img class="a" style="left:3px;top:3px" src="{}" alt="" draggable="false" onerror="this.remove()">"#, picture("shop.bmp")),
            format!(r#"<div class="a t" style="{};overflow:hidden;white-space:nowrap;text-overflow:ellipsis;color:{BLACK}">"#, at(31, 8, 106, 15)),
            format!(r#"{}</div></div>"#, escape(title))
        ].concat()
    }

    /// How high the window of a shop being made is: 260, and from four things on 42 for each and 92 more
    /// (`UIMerchantShopMakeWnd::UpdateLayout`).
    pub(in crate::web) fn stall_height(&self) -> i32 {
        crate::vending::stall_height(self.vending.making.as_ref().map_or(0, |making| making.rows.len()))
    }

    /// The window a shop is made in (`UIMerchantShopMakeWnd`, 400 wide): its title (the client's line 224),
    /// "Name" (282) with the field of the shop's title, 36 characters at the most; a row of 42 to each thing
    /// put in: the thing with how many, its name, "Price: " (369) with the field of its price, and under
    /// that what the vendor is left of the price ("Total amount : ", 3222) in the price's colour; the red
    /// line 3223 above the bar, and OK and Cancel on it. A thing is put in by dragging it out of the window
    /// beside it, and taken out by dragging it back. What is under a price is as of the window's last
    /// drawing: the client writes it as the price is typed.
    ///
    /// Left out of the client's window: its button for a note of the shop (`bt_shopinfo.bmp`), "Import",
    /// which brings back the last shop's title and prices, and the box "Safe check for over 10 mil zeny".
    pub(in crate::web) fn stall_window(&self) -> String {
        let Some(making) = &self.vending.making else { return String::new() };
        let (width, height) = (400, self.stall_height());
        let rows: String = making
            .rows
            .iter()
            .enumerate()
            .filter_map(|(row, stalled)| {
                let item = self.cart.items.iter().find(|item| item.index == stalled.index)?;
                let top = 54 + 42 * row as i32;
                let left = price_typed(&stalled.price).filter(|price| *price > 0).map_or(String::new(), |price| {
                    format!(
                        r#"<div class="a t" style="left:235px;top:{}px;white-space:nowrap">{}{}</div>"#,
                        top + 18,
                        escape(&Self::words(3222, "Total amount : ")),
                        price_written(after_commission(price))
                    )
                });
                Some(format!(
                    r#"<div class="a" style="left:3px;top:{top}px;width:32px;height:32px;background:url('{}')">{}</div>
                       <div class="a t" style="left:39px;top:{}px;width:100px;line-height:12px;white-space:normal;overflow:hidden;max-height:36px">{}</div>
                       {}{}{left}"#,
                    picture("basic_interface/itemwin_mid.bmp"),
                    grid_cell(0, 1, item, stalled.amount, &dragged(Drag::Stalled(row)), &Self::item_name(item)),
                    top + 3,
                    escape(&Self::item_name(item)),
                    text(235, top + 3, BLACK, &Self::words(369, "Price: ")),
                    numeric_field(
                        &windows::stall_price_field(row),
                        (281, top, 94, 16),
                        &stalled.price,
                        14,
                        &format!("background:{FIELD_GROUND};text-align:right")
                    ),
                ))
            })
            .collect();
        format!(
            r#"<form {} {} class="a" style="{}">{}{}{}{}{rows}{}{}{}</form>"#,
            sent(Form::Stall),
            dropped(Drop::Stall),
            at(0, 0, width, height),
            title_bar(width, &Self::words(224, "Opening a stall")),
            panel(width, height),
            text(2, 25, BLACK, &Self::words(282, "Name")),
            field(&windows::VENDING_STALL.field(), (41, 22, width - 66, 16), &making.title, TITLE, &format!("background:{FIELD_GROUND}")),
            text(5, height - 43, RED, &Self::words(3223, "~ When selling items, total amount of zeny received ")),
            submit_press(width - 92, height - 24, "btn_ok", Press::StallOk),
            picture_press(width - 46, height - 24, "btn_cancel", Press::StallCancel),
        )
    }

    /// What the cart has for the shop being made (`UIMerchantMirrorItemWnd`, 280 by 120; its title the
    /// client's line 232): the cart's things, each with how many are not in the shop yet. A thing is taken up
    /// out of it towards the shop, and the window takes back what is dragged out of the shop.
    pub(in crate::web) fn stall_stock_window(&self) -> String {
        let Some(making) = &self.vending.making else { return String::new() };
        let cells: Vec<String> = self
            .cart
            .items
            .iter()
            .filter_map(|item| Some((item, item.amount.checked_sub(making.taken(item.index)).filter(|left| *left > 0)?)))
            .enumerate()
            .map(|(place, (item, left))| grid_cell(place, 7, item, left, &dragged(Drag::Stock(item.index)), &Self::item_name(item)))
            .collect();
        format!("<div {}>{}{}</div>", dropped(Drop::Stock), list_frame(280, 120, &Self::words(232, "Available Items for Vending")), grid_of(7, 2, &cells))
    }

    /// One's own shop while it is open (`UIMerchantItemMyShopWnd`, 280 by 120; its title the client's line
    /// 3237): what is left of each thing, which says its name and its price when pointed at, and "close" on
    /// the bar, 61 from the right and 24 from the bottom. Left out: the client's box "Notify when item sells
    /// out" (2642), which has a sale heard, and the list of what was sold that it opens as the shop closes.
    pub(in crate::web) fn vending_mine_window(&self) -> String {
        let Some(mine) = &self.vending.mine else { return String::new() };
        let cells: Vec<String> = mine
            .iter()
            .enumerate()
            .map(|(place, sold)| grid_cell(
                place,
                7,
                &sold.item,
                sold.item.amount,
                "",
                &format!("{} {} Zeny", Self::item_name(&sold.item), grouped(i64::from(sold.price)))
            ))
            .collect();
        format!(
            "{}{}{}",
            list_frame(280, 120, &Self::words(3237, "My Shop")),
            grid_of(7, 2, &cells),
            picture_press(280 - 61, 120 - 24, "btn_close", Press::MineClose)
        )
    }

    /// What the making of a shop asks, in the client's message box: whether a thing is to cost nothing (its
    /// line 604).
    pub(in crate::web) fn vending_question(&self) -> String {
        match self.vending.asks {
            Some(VendingAsk::Free) => message_box(
                Asked::Sure,
                &escape(&Self::words(604, "At least one item has its price set to 0 Zeny exists. Do you wish to continue?")),
                Press::Yes,
                Press::No
            ),
            None => String::new(),
        }
    }

    /// How many of a thing go into the shop being made, into the basket at a vendor's, or back: the box that
    /// asks how many, with the thing's name; one in it for what is bought, all there are for the rest.
    pub(in crate::web) fn vending_count_window(&self) -> String {
        let Some((across, most)) = self.vending.counting else { return String::new() };
        let carted = |index: u16| self.cart.items.iter().find(|item| item.index == index);
        let item = match across {
            Crossing::Stall(index) => carted(index),
            Crossing::Unstall(row) => self.vending.making.as_ref().and_then(|making| making.rows.get(row)).and_then(|stalled| carted(stalled.index)),
            Crossing::Basket(row) | Crossing::Unbasket(row) => self.vending.visit.as_ref().and_then(|visit| visit.list.sold.get(row)).map(|sold| &sold.item),
        };
        let shown = if matches!(across, Crossing::Basket(_)) { 1 } else { most };
        count_box_of(&windows::VENDING_COUNT, &item.map(Self::item_name).unwrap_or_default(), shown, Form::VendingCount, Press::CountOk)
    }
}
