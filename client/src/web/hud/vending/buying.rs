//! Buying from a vendor: the vendor's list (`UIMerchantItemShopWnd`), the basket (`UIMerchantItemPurchaseWnd`)
//! and the window that asks whether to buy for the sum (`UIMerchantItemPurchaseMSGWnd`).

use super::*;

impl App {
    /// A row of either list (`item_row`) with what stands at its right end written after it: a price in its
    /// colour, which a row of the lists has no place for.
    fn vending_row(item: &Item, amount: u16, name_width: i32, right: &str, marks: &str) -> String {
        let resource = data::item_as(item.id, item.identified).map(|(_, resource)| resource);
        let row = item_row(&Row {
            resource: resource.as_deref(),
            amount: Some(u32::from(amount)),
            name: &Self::item_name(item),
            name_at: (36, name_width),
            price: None,
            marks: &format!("{marks} {}", item_marks(item.id, item.identified)),
        });
        format!(r#"{}<div class="a t" style="right:18px;top:12px;white-space:nowrap;color:#000">{right}</div></div>"#, row.strip_suffix("</div>").unwrap_or(&row))
    }

    /// How many rows the vendor's list shows: one to a thing, four at the least (`UM_REFRESHITEM`), and here
    /// eight at the most, past which it scrolls (the client lets the window be drawn out to 336 high).
    pub(in crate::web) fn vendor_rows(&self) -> i32 {
        self.vending.visit.as_ref().map_or(4, |visit| visit.list.sold.len().clamp(4, 8) as i32)
    }

    /// The vendor's list (`UIMerchantItemShopWnd`, 280 wide, 32 a row and 56 more): its title the client's
    /// line 227 with the vendor's name after a dash, and a row to each thing: what is left of it that is not
    /// in the basket, its name, and its price in the price's colour before a "Z". A row is taken up out of it
    /// towards the basket, and the window takes back what is dragged out of the basket.
    pub(in crate::web) fn vendor_window(&self) -> String {
        let Some(visit) = &self.vending.visit else { return String::new() };
        let rows: String = visit
            .list
            .sold
            .iter()
            .enumerate()
            .filter_map(|(row, sold)| {
                let left = sold.item.amount.checked_sub(visit.kept(row)).filter(|left| *left > 0)?;
                Some(Self::vending_row(&sold.item, left, 110, &format!(" {} Z", price_written(sold.price)), &dragged(Drag::Vended(row))))
            })
            .collect();
        let shop = Self::words(227, "Merchant Shop");
        let title = visit.name.as_ref().map_or(shop.clone(), |name| format!("{shop} - {name}"));
        let rows_shown = self.vendor_rows();
        format!("<div {}>{}{}</div>", dropped(Drop::Vendor), list_frame(280, rows_shown * 32 + 56, &title), list_of(40, 225, rows_shown, &rows, ""))
    }

    /// The basket (`UIMerchantItemPurchaseWnd`, 280 by 120; its title the client's line 228): two rows of
    /// what was put in with how many of each and what one costs, the sum under them ("Total : n Zeny", 12 in
    /// and 21 from the bottom), Buy 106 from the right edge and Cancel 61, 24 from the bottom.
    pub(in crate::web) fn vendor_basket_window(&self) -> String {
        let Some(visit) = &self.vending.visit else { return String::new() };
        let rows: String = visit
            .basket
            .iter()
            .filter_map(|(row, amount)| {
                let sold = visit.list.sold.get(*row)?;
                Some(Self::vending_row(&sold.item, *amount, 90, &format!(" {} Zeny", grouped(i64::from(sold.price))), &dragged(Drag::Wanted(*row))))
            })
            .collect();
        format!(
            "<div {}>{}{}{}{}{}</div>",
            dropped(Drop::Wants),
            list_frame(280, 120, &Self::words(228, "Buying Items")),
            list_of(40, 225, 2, &rows, ""),
            text(12, 99, BLACK, &format!("Total : {} Zeny", grouped(visit.total() as i64))),
            picture_press(174, 96, "btn_buy", Press::Buy),
            picture_press(219, 96, "btn_cancel", Press::Leave),
        )
    }

    /// The window that asks whether to buy (`UIMerchantItemPurchaseMSGWnd`, 300 by 150): its title the
    /// client's line 2326, then 10 in and 20, 40 and 60 down "Below is the total cost:" (2327), the sum with
    /// " Zeny" (2328) and "Press buy to confirm." (2329), and Buy and Cancel as on the basket.
    pub(in crate::web) fn vendor_confirm_window(&self) -> String {
        let Some(visit) = &self.vending.visit else { return String::new() };
        format!(
            "{}{}{}{}{}{}{}",
            title_bar(300, &Self::words(2326, "Confirm Deal")),
            panel(300, 150),
            text(10, 20, BLACK, &Self::words(2327, "Below is the total cost:")),
            text(10, 40, BLACK, &format!("{}{}", grouped(visit.total() as i64), Self::words(2328, " Zeny"))),
            text(10, 60, BLACK, &Self::words(2329, "Press buy to confirm.")),
            picture_press(194, 126, "btn_buy", Press::Confirm),
            picture_press(239, 126, "btn_cancel", Press::Refuse),
        )
    }
}
