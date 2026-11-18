//! An NPC's shop (buying and selling) and the Kafra's storage.

use super::*;

impl App {
    /// Redraws the shop's window, where it is on the page.
    pub(in crate::web) fn refresh_trade(&self) {
        if let Some(trade) = crate::web::by_id("trade") {
            trade.set_inner_html(&self.trade_window());
        }
    }

    /// The shop: the original's box asking whether to buy or to sell (`win_msgbox.bmp` with its three buttons),
    /// then the list of what is sold or what is taken, with how many of each. The lists are simpler than the
    /// original's two windows with a basket: one list, a number to each line.
    pub(in crate::web) fn trade_window(&self) -> String {
        let Some(shop) = &self.deal.shop else { return String::new() };
        let rows: Vec<(u32, String, u32, Option<u16>)> = match shop {
            Shop::Choose(_) => {
                return format!(
                    "{}{}{}{}",
                    art("win_msgbox.bmp", 0, 200, 280, 120),
                    picture_button(144, 296, "btn_buy", "shop-buy"),
                    picture_button(189, 296, "btn_sell", "shop-sell"),
                    picture_button(234, 296, "btn_cancel", "shop-cancel")
                );
            }
            Shop::Buy(offers) => offers.iter().map(|offer| (offer.id, data::item(offer.id).map_or_else(|| format!("{} {}", tr("Item"), offer.id), |(name, _)| name), offer.price, None)).collect(),
            Shop::Sell(bids) => bids
                .iter()
                .filter_map(|(index, price)| {
                    let item = self.bag.items.iter().find(|item| item.index == *index)?;
                    Some((item.id, Self::item_name(item), *price, Some(item.amount)))
                })
                .collect(),
        };
        let selling = matches!(shop, Shop::Sell(_));
        // One line of either window: the thing in its cell with how many, its name, its price at the right.
        let line = |row: usize, amount: Option<u16>, picked: bool, attributes: &str| {
            let Some((id, name, price, _)) = rows.get(row) else { return String::new() };
            let resource = data::item(*id).map(|(_, resource)| resource);
            // The line pressed last is the one picked: the page keeps that itself, so the list is not redrawn
            // between the two presses of a double click.
            let _ = picked;
            let ink = "inherit";
            format!(
                r#"<div class="row" tabindex="-1" style="position:relative;height:32px;cursor:default" {attributes}>
                     <div class="a cell" style="left:3px;top:3px;width:26px;height:26px;background:#f7f7f7;border:1px solid #b0b0b0;box-sizing:border-box;overflow:hidden"><div style="position:absolute;left:-4px;top:-4px">{}</div>{}</div>
                     <div class="a t" style="left:36px;top:9px;color:{ink}">{}</div>
                     <div class="a t" style="right:8px;top:9px;color:{ink}">{} Z</div>
                   </div>"#,
                Self::icon(resource.as_deref()),
                amount.map_or(String::new(), |amount| format!(r#"<span class="n">{amount}</span>"#)),
                escape(name),
                grouped(i64::from(*price)),
            )
        };
        // What is still to be had of each line: all of it when buying, what is carried less what is put aside.
        let aside = |row: usize| self.deal.basket.iter().find(|(kept, _)| *kept == row).map_or(0, |(_, amount)| *amount);
        let list: String = (0..rows.len())
            .filter_map(|row| {
                let left = rows[row].3.map(|have| have.saturating_sub(aside(row)));
                (left != Some(0)).then(|| line(row, left.filter(|left| *left > 1), self.deal.picked == Some(row), &format!(r#"data-use="shop-add:{row}""#)))
            })
            .collect();
        let kept: String = self.deal.basket.iter().map(|(row, amount)| line(*row, Some(*amount).filter(|amount| *amount > 1), false, &format!(r#"data-use="shop-remove:{row}""#))).collect();
        let total: i64 = self.deal.basket.iter().filter_map(|(row, amount)| Some(i64::from(rows.get(*row)?.2) * i64::from(*amount))).sum();
        let frame = |x: i32, y: i32, title: &str, lines: i32, body: &str| {
            let footer = 17 + lines * 32;
            format!(
                r#"<div class="a" style="left:{x}px;top:{y}px;width:280px;height:{}px">{}<div class="a scroll" style="{};background:#fff;border-left:1px solid #c3c3c3;border-right:1px solid #c3c3c3;box-sizing:border-box">{body}</div>{}{}{}"#,
                footer + 21,
                title_bar(280, title),
                at(0, 17, 280, lines * 32),
                art("basic_interface/btnbar_left.bmp", 0, footer, 21, 21),
                art("basic_interface/btnbar_mid.bmp", 21, footer, 238, 21),
                art("basic_interface/btnbar_right.bmp", 259, footer, 21, 21),
            )
        };
        let (have, taken, deal) = if selling { ("Available Items to sell", "Selling Items", "btn_sell") } else { ("Available Items to buy", "Buying Items", "btn_buy") };
        // The list at the left and what is put aside at the right, as the original lays its two windows out.
        format!(
            r#"{}<div class="a t" style="left:8px;top:{}px">{} <input id="shop-amount" class="in" style="width:44px;height:14px;background:#fff;border:1px solid #9a9a9a" type="number" min="1" placeholder="{}"></div></div>{}<div class="a t" style="left:8px;top:{}px">Total : {} Zeny</div>{}{}</div>"#,
            frame(-150, 0, tr(have), 9, &list),
            17 + 9 * 32 + 3,
            tr("Amount"),
            if selling { "all" } else { "1" },
            frame(140, 180, tr(taken), 3, &kept),
            17 + 3 * 32 + 4,
            grouped(total),
            picture_button(188, 17 + 3 * 32 + 1, deal, "shop-ok"),
            picture_button(233, 17 + 3 * 32 + 1, "btn_cancel", "shop-cancel"),
        )
    }

    /// The Kafra's storage, laid out as the original's: tabs for the kinds of things down the left side, each
    /// thing a line with its picture, how many and its name, and how full the storage is with "close" at the
    /// bottom. A double click on a line takes the whole stack out; while the storage is open, a double click in
    /// the inventory puts the whole stack in.
    pub(in crate::web) fn storage_window(&self) -> String {
        const TABS: [&str; 8] = ["Item", "Cash", "Armor", "Wpn", "Cstm", "Ammo", "Card", "Misc"];
        const LINES: i32 = 11;
        let Some(storage) = &self.deal.storage else { return String::new() };
        // Which tab a thing is under, by the server's kind of item.
        let tab_of = |item: &Item| match item.kind {
            0 | 2 | 11 => 0,
            18 => 1,
            4 | 8 | 12 => 2,
            5 => 3,
            10 => 5,
            6 => 6,
            _ => 7,
        };
        let (width, body) = (280, 32 * LINES);
        let mut window = title_bar(width, "Storage");
        window.push_str(&button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "storage-close", ""));
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border-left:1px solid #c3c3c3;border-right:1px solid #c3c3c3;box-sizing:border-box"></div>"#, at(0, 17, width, body)));
        for (index, label) in TABS.iter().enumerate() {
            let open = if index == self.deal.storage_tab { " open" } else { "" };
            window.push_str(&format!(
                r#"<button type="button" class="a side{open}" style="{}" data-click="storage-tab" data-arg="{index}"><i>{}</i></button>"#,
                at(2, 19 + index as i32 * 34, 16, 34),
                tr(label)
            ));
        }
        let lines: String = storage
            .items
            .iter()
            .filter(|item| tab_of(item) == self.deal.storage_tab)
            .map(|item| {
                let resource = data::item(item.id).map(|(_, resource)| resource);
                let id = item.id;
                format!(
                    r#"<div class="row" tabindex="-1" style="position:relative;height:32px;cursor:default" data-use="take:{}" data-info="{id}">
                         <div class="a cell" style="left:0;top:1px;width:30px;height:30px;background:#fff;border:1px solid #7b94ce;border-radius:4px;box-sizing:border-box;overflow:hidden"><div style="position:absolute;left:-2px;top:-2px">{}</div><span class="n">{}</span></div>
                         <div class="a t" style="left:40px;top:9px;color:inherit">{}</div>
                       </div>"#,
                    item.index,
                    Self::icon(resource.as_deref()),
                    item.amount,
                    escape(&Self::item_name(item)),
                )
            })
            .collect();
        window.push_str(&format!(r#"<div class="a scroll" style="{}">{lines}</div>"#, at(40, 17, width - 46, body)));
        let footer = 17 + body;
        window.push_str(&art("basic_interface/btnbar_left.bmp", 0, footer, 21, 26));
        window.push_str(&art("basic_interface/btnbar_mid.bmp", 21, footer, width - 42, 26));
        window.push_str(&art("basic_interface/btnbar_right.bmp", width - 21, footer, 21, 26));
        window.push_str(&text(10, footer + 7, "#232323", &format!("{}/{}", storage.count.0.max(storage.items.len() as u16), storage.count.1)));
        window.push_str(&picture_button(width - 48, footer + 3, "btn_close", "storage-close"));
        window
    }
}
