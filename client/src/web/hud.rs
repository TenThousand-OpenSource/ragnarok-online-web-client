//! The windows of the game itself: Basic Information, the hotkey bar, Status, Inventory, the skill list, and chat.
//!
//! Sizes, places, pictures and colours are the original client's, as rustnarok-client lists them
//! (`src/ui/basic_info.rs`, `hotkeys.rs`, `status.rs`, `inventory.rs`, `skills.rs`, `chat.rs`, `window.rs`; MIT OR
//! Apache-2.0). The inventory and the skill list are simpler than the original's: fixed sizes, the skills as one
//! list without the tree, and worn equipment shown among the items (until there is an Equipment window).
//!
//! Each window lives in an element of its own, so a value that changes redraws only its window: the chat's
//! input line is never rebuilt while someone types in it.

use crate::i18n::tr;
use super::app::{App, Shop, escape};
use super::data;
use crate::net::items::{Hotkey, Item, Tab};
use crate::net::packets::parameter;

const UI: &str = "data/texture/유저인터페이스/";

/// How many chat lines are kept.
pub const CHAT_KEPT: usize = 240;

const BLACK: &str = "#000";
/// How many columns and rows of items the inventory shows, and how many skills the list shows.
const ITEM_COLUMNS: i32 = 7;
const ITEM_ROWS: i32 = 5;

fn picture(name: &str) -> String {
    format!("/image/{}", String::from(js_sys::encode_uri(&format!("{UI}{name}"))))
}

fn at(x: i32, y: i32, width: i32, height: i32) -> String {
    format!("left:{x}px;top:{y}px;width:{width}px;height:{height}px")
}

/// A piece of the window's art at its place and size (stretched to it).
fn art(name: &str, x: i32, y: i32, width: i32, height: i32) -> String {
    format!(r#"<div class="a" style="{};background:url('{}') 0 0 / 100% 100% no-repeat"></div>"#, at(x, y, width, height), picture(name))
}

/// The frame of the original's system boxes around a space `width` by `height`, `top` down: corners and edges
/// of 14-pixel pieces, white inside.
fn system_box(top: i32, width: i32, height: i32) -> String {
    let piece = |name: &str, x: i32, y: i32, w: i32, h: i32, repeat: &str| {
        format!(r#"<div class="a" style="{};background:url('{}') 0 0 {repeat}"></div>"#, at(x, top + y, w, h), picture(&format!("sysbox_{name}.bmp")))
    };
    [
        format!(r#"<div class="a" style="{};background:#fff"></div>"#, at(14, top + 14, width - 28, height - 28)),
        piece("mu", 14, 0, width - 28, 14, "repeat-x"),
        piece("md", 14, height - 14, width - 28, 14, "repeat-x"),
        piece("lm", 0, 14, 14, height - 28, "repeat-y"),
        piece("rm", width - 14, 14, 14, height - 28, "repeat-y"),
        piece("lu", 0, 0, 14, 14, "no-repeat"),
        piece("ru", width - 14, 0, 14, 14, "no-repeat"),
        piece("ld", 0, height - 14, 14, 14, "no-repeat"),
        piece("rd", width - 14, height - 14, 14, 14, "no-repeat"),
    ]
    .concat()
}

/// One of the original's small buttons with three pictures: `name.bmp`, `name_a.bmp` under the pointer,
/// `name_b.bmp` pressed.
fn picture_button(x: i32, y: i32, name: &str, click: &str) -> String {
    format!(
        r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="{click}"></button>"#,
        at(x, y, 42, 20),
        picture(&format!("{name}.bmp")),
        picture(&format!("{name}_a.bmp")),
        picture(&format!("{name}_b.bmp"))
    )
}

/// A line an NPC says, as HTML: `^RRGGBB` in it changes the colour of what follows.
fn coloured(line: &str) -> String {
    let mut html = String::new();
    let mut open = false;
    let mut rest = line;
    while let Some(at) = rest.find('^') {
        html.push_str(&escape(&rest[..at]));
        let code = rest.get(at + 1..at + 7).filter(|code| code.len() == 6 && code.bytes().all(|byte| byte.is_ascii_hexdigit()));
        match code {
            Some(code) => {
                if std::mem::replace(&mut open, true) {
                    html.push_str("</span>");
                }
                html.push_str(&format!(r#"<span style="color:#{code}">"#));
                rest = &rest[at + 7..];
            }
            None => {
                html.push('^');
                rest = &rest[at + 1..];
            }
        }
    }
    html.push_str(&escape(rest));
    if open {
        html.push_str("</span>");
    }
    if html.is_empty() { "&nbsp;".to_owned() } else { html }
}

/// A line of text with its top-left corner at a place.
fn text(x: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="left:{x}px;top:{y}px;color:{colour}">{}</div>"#, escape(words))
}

/// A line of text ending at `right`, in a window `width` wide.
fn text_ending(right: i32, width: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="right:{}px;top:{y}px;color:{colour}">{}</div>"#, width - right, escape(words))
}

/// A small picture button whose picture changes under the pointer.
fn button(x: i32, y: i32, size: i32, off: &str, on: &str, click: &str, arg: &str) -> String {
    format!(
        r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="{click}" data-arg="{arg}"></button>"#,
        at(x, y, size, size),
        picture(off),
        picture(on),
        picture(on)
    )
}

/// The standard title bar: caps that keep their size and a middle that stretches, the title in black over a
/// white copy of itself.
pub(super) fn title_bar(width: i32, title: &str) -> String {
    format!(
        "{}{}{}{}",
        art("basic_interface/titlebar_left.bmp", 0, 0, 12, 17),
        art("basic_interface/titlebar_mid.bmp", 12, 0, width - 24, 17),
        art("basic_interface/titlebar_right.bmp", width - 12, 0, 12, 17),
        format!(r#"<div class="a t titled" style="left:17px;top:1px">{}</div>"#, escape(tr(title)))
    )
}

/// A number with commas between its thousands.
fn grouped(number: i64) -> String {
    let digits = number.unsigned_abs().to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if number < 0 { format!("-{grouped}") } else { grouped }
}

impl App {
    fn value(&self, name: u16) -> i64 {
        // A dead character has no life left, whatever the server has already set aside for its return.
        if self.dead && name == parameter::HP {
            return 0;
        }
        self.params.get(&name).copied().unwrap_or(0)
    }

    /// Every window of the game, each in its own element. `view` is the stage's size.
    pub(super) fn hud(&self, view: (f64, f64)) -> String {
        // The client's own sum (`UINewChatWnd::ResizeRows`): the input line, the rows and three more, the tabs.
        let chat_lines = self.chat_rows;
        let chat_height = if chat_lines == 0 { 24 } else { 24 + (chat_lines as i32 + 3) * 14 + 16 };
        let hotkeys = if self.hotkeys_hidden { String::new() } else { format!(r#"<div id="hotkeys" class="a hud" style="{}">{}</div>"#, at(230, 0, 290, (i32::from(self.hotkey_rows) + 1) * 33 + 1), self.hotkeys()) };
        let status = if self.status_open { format!(r#"<div id="status" class="a hud" style="{}">{}</div>"#, at(225, 285, 280, 141), self.status_window()) } else { String::new() };
        let inventory = if self.inventory_open {
            format!(r#"<div id="inventory" class="a hud" style="{}">{}</div>"#, at(if self.storage.is_some() { 515 } else { 225 }, 80, 40 + 32 * ITEM_COLUMNS + 16, 17 + 32 * ITEM_ROWS + 21), self.inventory_window())
        } else {
            String::new()
        };
        let skills = if self.skills_open { format!(r#"<div id="skills" class="a hud" style="{}">{}</div>"#, at(230, 60, 560, 400), self.skill_window()) } else { String::new() };
        // The original puts the NPC's window a fifth of the way down the screen, in the middle, and what it asks
        // under it.
        let dialog = format!(r#"<div id="dialog" class="a hud" style="left:{}px;top:{}px;width:280px">{}</div>"#, (view.0 as i32 - 280) / 2, view.1 as i32 * 100 / 480, self.dialog_window());
        let trade = format!(r#"<div id="trade" class="a hud" style="left:{}px;top:{}px;width:280px">{}</div>"#, (view.0 as i32 - 280) / 2, view.1 as i32 * 100 / 480, self.trade_window());
        let storage = format!(r#"<div id="storage" class="a hud" style="left:225px;top:80px;width:280px">{}</div>"#, self.storage_window());
        let throw = format!(r#"<div id="throw" class="a hud" style="left:{}px;top:{}px;width:182px">{}</div>"#, (view.0 as i32 - 182) / 2, view.1 as i32 * 200 / 480, self.throw_window());
        let equipment = if self.equipment_open { format!(r#"<div id="equipment" class="a hud" style="left:560px;top:290px;width:280px">{}</div>"#, self.equipment_window()) } else { String::new() };
        let info = format!(
            r#"<div id="iteminfo" class="a hud" style="left:300px;top:230px;width:280px">{}</div><div id="skillinfo" class="a hud" style="left:330px;top:110px;width:304px">{}</div>"#,
            self.item_info_window(),
            self.skill_info_window()
        );
        // The slot machine of taming: 196 by 256 in the middle of the screen; a press on it starts the reels.
        let taming = if self.taming.is_some() {
            format!(
                r#"<div id="taming" class="a" style="{};cursor:pointer;z-index:30;pointer-events:auto" data-click="taming-shoot"><img id="taming-picture" alt="" draggable="false" style="position:absolute;left:0;top:0"></div>"#,
                at((view.0 as i32 - 196) / 2, (view.1 as i32 - 256) / 2, 196, 256)
            )
        } else {
            String::new()
        };
        let obtained = format!(r#"<div id="obtained" class="a" style="left:{}px;top:{}px;width:260px;pointer-events:none">{}</div>"#, (view.0 as i32 - 260) / 2, 75, self.obtained_note());
        // The menu hangs under the basic window, a little into it.
        let menu = format!(r#"<div id="menu" class="a hud" style="left:0;top:{}px;width:220px">{}</div>"#, if self.basic_small { 55 } else { 134 } - 4, self.menu_icons());
        format!(
            r#"<div id="basic" class="a hud" style="left:0;top:0;width:220px">{}</div>{menu}{hotkeys}{status}{inventory}{skills}{storage}{equipment}{dialog}{trade}{info}{obtained}{throw}{taming}
               <div id="chat" class="a hud" data-locked="{}" style="{}">{}</div>"#,
            self.basic_info(),
            u8::from(self.chat_locked),
            at(0, view.1 as i32 - chat_height, 600, chat_height),
            self.chat_window(chat_lines, chat_height)
        )
    }

    /// Redraws the windows that show the character's values, where they are on the page.
    pub(super) fn refresh_values(&self) {
        let document = web_sys::window().and_then(|window| window.document());
        let Some(document) = document else { return };
        super::world::own_life(self.value(parameter::HP), self.value(parameter::MAX_HP), self.value(parameter::SP), self.value(parameter::MAX_SP));
        if let Some(basic) = document.get_element_by_id("basic") {
            basic.set_inner_html(&self.basic_info());
        }
        if let Some(status) = document.get_element_by_id("status") {
            status.set_inner_html(&self.status_window());
        }
    }

    /// Redraws the NPC's window, where it is on the page.
    pub(super) fn refresh_dialog(&self) {
        if let Some(dialog) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("dialog")) {
            dialog.set_inner_html(&self.dialog_window());
            if let Ok(Some(text)) = dialog.query_selector(".said") {
                text.set_scroll_top(text.scroll_height());
            }
        }
    }

    /// Redraws the shop's window, where it is on the page.
    pub(super) fn refresh_trade(&self) {
        if let Some(trade) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("trade")) {
            trade.set_inner_html(&self.trade_window());
        }
    }

    /// What an NPC says, with "next" or "close" under it; below it, the menu to choose from or the line to type
    /// into. The original's windows (`UISayDialogWnd`, `UIChooseWnd`): 280 by 120, in the frame of the system
    /// boxes, the text on light grey 10 from the edges.
    fn dialog_window(&self) -> String {
        // The client's window of game options (`UIEscOptionWnd`): Escape opens and closes it, and it opens by
        // itself when the character dies. Alive, it offers the characters, the settings, the keys, leaving and
        // going back to the game; dead, the save point and going back to the game. 280 wide, the buttons 29 in
        // and 5 under the title, 23 apart.
        match self.esc {
            1 => {
                let buttons: &[(&str, &str)] =
                    if self.dead { &[("esc_10", "esc-save"), ("esc_02", "esc-close")] } else { &[("esc_01", "logout"), ("esc_001", ""), ("esc_08", ""), ("esc_09", "exit"), ("esc_02", "esc-close")] };
                let height = 17 + if self.dead { 53 } else { 122 };
                let mut window = title_bar(280, "Game Options");
                window.push_str(&art("basic_interface/sys_base_off.bmp", 3, 3, 11, 11));
                window.push_str(&format!(
                    r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0;border-radius:0 0 3px 3px;box-sizing:border-box"></div>"#,
                    at(0, 17, 280, height - 17)
                ));
                for (row, (name, click)) in buttons.iter().enumerate() {
                    window.push_str(&format!(
                        r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="{click}"></button>"#,
                        at(29, 22 + row as i32 * 23, 221, 20),
                        picture(&format!("{name}a.bmp")),
                        picture(&format!("{name}b.bmp")),
                        picture(&format!("{name}c.bmp"))
                    ));
                }
                return window;
            }
            // The save point is asked about first, in the client's message box.
            2 => {
                return format!(
                    r#"{}<div class="a t" style="{};white-space:normal">{}</div>{}{}"#,
                    art("win_msgbox.bmp", 0, 0, 280, 120),
                    at(12, 30, 256, 56),
                    tr("Do you really want to go back to your savepoint?"),
                    picture_button(189, 96, "btn_ok", "esc-yes"),
                    picture_button(235, 96, "btn_cancel", "esc-no")
                );
            }
            _ => {}
        }
        let Some(dialog) = &self.dialog else { return String::new() };
        let said: String = dialog.lines.iter().map(|line| format!("<div>{}</div>", coloured(line))).collect();
        let mut window = format!(
            r#"{}<div class="a t said scroll" style="{};background:#f2f2f2;padding:2px 4px;box-sizing:border-box;white-space:normal;user-select:text">{said}</div>"#,
            system_box(0, 280, 120),
            at(10, 10, 260, 80)
        );
        match dialog.button {
            Some(true) => window.push_str(&picture_button(232, 96, "btn_next", "dialog-next")),
            Some(false) => window.push_str(&picture_button(232, 96, "btn_close", "dialog-close")),
            None => {}
        }
        if !dialog.menu.is_empty() {
            let choices: String = dialog
                .menu
                .iter()
                .enumerate()
                .filter(|(_, choice)| !choice.is_empty())
                .map(|(index, choice)| {
                    let picked = if index == dialog.picked { "background:#c6d6f7;" } else { "" };
                    format!(r#"<div style="{picked}padding:1px 3px;cursor:default" data-click="dialog-pick" data-arg="{index}">{}</div>"#, coloured(choice))
                })
                .collect();
            // The menu: its list 12 in and 26 down, OK and Cancel in the corner.
            window.push_str(&format!(r#"{}<div class="a t scroll" style="{};background:#f0f0f0;white-space:normal">{choices}</div>"#, system_box(124, 280, 120), at(12, 136, 256, 78)));
            window.push_str(&picture_button(188, 220, "btn_ok", "dialog-ok"));
            window.push_str(&picture_button(233, 220, "btn_cancel", "dialog-cancel"));
        } else if let Some(words) = dialog.asks {
            window.push_str(&format!(
                r#"{}<input id="dialog-input" class="a in" style="{};background:#fff;border:1px solid #9a9a9a" type="{}" autocomplete="off" autofocus>"#,
                system_box(124, 280, 120),
                at(12, 160, 256, 18),
                if words { "text" } else { "number" }
            ));
            window.push_str(&picture_button(233, 220, "btn_ok", "dialog-ok"));
        }
        window
    }

    /// The shop: the original's box asking whether to buy or to sell (`win_msgbox.bmp` with its three buttons),
    /// then the list of what is sold or what is taken, with how many of each. The lists are simpler than the
    /// original's two windows with a basket: one list, a number to each line.
    fn trade_window(&self) -> String {
        let Some(shop) = &self.shop else { return String::new() };
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
                    let item = self.inventory.iter().find(|item| item.index == *index)?;
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
        let aside = |row: usize| self.basket.iter().find(|(kept, _)| *kept == row).map_or(0, |(_, amount)| *amount);
        let list: String = (0..rows.len())
            .filter_map(|row| {
                let left = rows[row].3.map(|have| have.saturating_sub(aside(row)));
                (left != Some(0)).then(|| line(row, left.filter(|left| *left > 1), self.shop_picked == Some(row), &format!(r#"data-use="shop-add:{row}""#)))
            })
            .collect();
        let kept: String = self.basket.iter().map(|(row, amount)| line(*row, Some(*amount).filter(|amount| *amount > 1), false, &format!(r#"data-use="shop-remove:{row}""#))).collect();
        let total: i64 = self.basket.iter().filter_map(|(row, amount)| Some(i64::from(rows.get(*row)?.2) * i64::from(*amount))).sum();
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

    /// Redraws the window that asks how many to drop, where it is on the page, and puts the typing into it.
    pub(super) fn refresh_throw(&self) {
        let Some(window) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("throw")) else { return };
        window.set_inner_html(&self.throw_window());
        if let Some(count) = window.query_selector("input").ok().flatten().and_then(|count| wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlInputElement>(count).ok()) {
            let _ = count.focus();
            count.select();
        }
    }

    /// How many of a stack to drop: the original's small box (182 by 46) with the item's name, the number to
    /// type over (all of them at first) and OK.
    fn throw_window(&self) -> String {
        let Some((index, most)) = self.throwing else { return String::new() };
        let name = self.inventory.iter().find(|item| item.index == index).map(Self::item_name).unwrap_or_default();
        format!(
            r#"{}<form data-submit="throw" class="a" style="{}">
                 <div class="a t" style="left:15px;top:6px;width:104px;overflow:hidden;white-space:nowrap">{}</div>
                 <input id="throw-count" class="a in" style="{};background:#e8e8e8" type="number" min="1" max="{most}" value="{most}">
                 {}
               </form>"#,
            system_box(0, 182, 46),
            at(0, 0, 182, 46),
            escape(&name),
            at(15, 22, 80, 16),
            picture_button(127, 14, "btn_ok", "throw-ok").replace(r#"type="button""#, r#"type="submit""#),
        )
    }

    /// What the character wears, laid out as the original's window (its sizes as rustnarok-client lists them):
    /// two columns of five places, the character between them. A double click on a place takes the thing off.
    fn equipment_window(&self) -> String {
        // Where a thing is worn (the server's bits) and where that shows: column and row.
        const PLACES: [(u32, i32, i32); 10] = [(256, 0, 0), (512, 1, 0), (1, 0, 1), (16, 1, 1), (2, 0, 2), (32, 1, 2), (4, 0, 3), (64, 1, 3), (8, 0, 4), (128, 1, 4)];
        let mut window = title_bar(280, "Equipment");
        window.push_str(&button(266, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "equipment", ""));
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border-left:1px solid #c8c8c8;border-right:1px solid #c8c8c8;box-sizing:border-box"></div>"#, at(0, 17, 280, 20)));
        window.push_str(&art("basic_interface/equipwin_bg2.bmp", 0, 37, 280, 150));
        // The character stands on the shadow drawn into the picture.
        window.push_str(&format!(
            r#"<div class="a" style="{};overflow:hidden;pointer-events:none"><canvas id="pc-equip" width="{}" height="{}" style="position:absolute;left:0;top:-25px"></canvas></div>"#,
            at(61, 37, super::figure::WIDTH as i32, 150),
            super::figure::WIDTH,
            super::figure::HEIGHT
        ));
        for (place, column, row) in PLACES {
            // A thing worn in two places (a two-handed weapon) shows in both.
            let Some(item) = self.inventory.iter().find(|item| item.worn & place != 0) else { continue };
            let resource = data::item(item.id).map(|(_, resource)| resource);
            let (icon_x, name_x) = if column == 0 { (6, 33) } else { (251, 172) };
            window.push_str(&format!(
                r#"<div class="a cell" style="left:{icon_x}px;top:{}px;width:24px;height:24px" data-use="item:{}" data-info="{}" title="{}"><div style="position:absolute;left:-4px;top:-4px">{}</div></div>
                   <div class="a t" style="left:{name_x}px;top:{}px;width:74px;line-height:12px;white-space:normal;overflow:hidden;max-height:24px;{}">{}</div>"#,
                38 + row * 26,
                item.index,
                item.id,
                escape(&Self::item_name(item)),
                Self::icon(resource.as_deref()),
                40 + row * 26,
                if column == 0 { "" } else { "text-align:right" },
                escape(&Self::item_name(item)),
            ));
        }
        // The ammunition in use (arrows): its picture at 115, 64 under the character's hands and its name under
        // it, 40 wide, as the client's window has them. A double click takes it off, as with anything worn.
        if let Some(item) = self.inventory.iter().find(|item| item.kind == 10 && item.worn != 0) {
            let resource = data::item(item.id).map(|(_, resource)| resource);
            window.push_str(&format!(
                r#"<div class="a cell" style="left:115px;top:64px;width:24px;height:24px" data-use="item:{}" data-info="{}" title="{} : {} {}"><div style="position:absolute;left:-4px;top:-4px">{}</div><span class="n">{}</span></div>
                   <div class="a t" style="left:107px;top:88px;width:40px;line-height:12px;text-align:center;white-space:nowrap;overflow:hidden">{}</div>"#,
                item.index,
                item.id,
                escape(&Self::item_name(item)),
                item.amount,
                tr("ea"),
                Self::icon(resource.as_deref()),
                item.amount,
                escape(&Self::item_name(item)),
            ));
        }
        window
    }

    /// The note of a thing just picked up: its picture and "name - N obtained", fading after a moment.
    pub(super) fn obtained_note(&self) -> String {
        let Some((id, amount)) = self.obtained else { return String::new() };
        let Some((name, resource)) = data::item(id) else { return String::new() };
        // It shows for five seconds from the moment the thing was picked up, however often the page is drawn
        // meanwhile: drawn anew, it goes on from where it was.
        let shown = (js_sys::Date::now() - self.obtained_at) / 1000.0;
        if !(0.0..5.0).contains(&shown) {
            return String::new();
        }
        // The client's `UINotifyItemObtainWnd`: a system box 32 high, the item's picture 13 in and 5 down, the
        // words from 41 in and 10 down; it stays five seconds.
        let words = format!("{} - {amount} {}", escape(&name), tr("obtained."));
        let width = 41 + words.chars().count() as i32 * 6 + 12;
        format!(
            r#"<div class="note" style="position:relative;display:inline-block;width:{width}px;height:32px;text-align:left;animation-delay:-{shown:.2}s">{}<div class="a cell" style="left:9px;top:1px">{}</div><div class="a t" style="left:41px;top:10px;white-space:nowrap">{words}</div></div>"#,
            system_box(0, width, 32),
            Self::icon(Some(&resource)),
        )
    }

    /// What is known of an item, opened with the right button: its large picture, what the server's database
    /// says of it (kind, weight, price and the like), and the client's own description under that.
    /// What an item is, as the client shows it (`UIItemCollectionWnd`): its box of 280 by 120 with the
    /// collection's background, the item's large picture at 10, 11, its name at 90, 10 and its description in a
    /// field of 182 by 80 at 91, 31 that scrolls.
    fn item_info_window(&self) -> String {
        let Some(id) = self.item_info else { return String::new() };
        let (name, resource) = data::item(id).unwrap_or_else(|| (format!("{} {id}", tr("Item")), String::new()));
        let (description, _) = data::about(id).unwrap_or_default();
        let description: String = description.iter().map(|line| format!("<div>{}</div>", coloured(line))).collect();
        let large = if resource.is_empty() { String::new() } else { format!(r#"<img src="{}" alt="" onerror="this.remove()">"#, picture(&format!("collection/{resource}.bmp"))) };
        format!(
            r#"{}{}<div class="a" style="{}">{large}</div>
               <div class="a t" style="{};white-space:nowrap;overflow:hidden">{}</div>
               <div class="a t viewer" style="{};white-space:normal;user-select:text">{description}</div>{}"#,
            system_box(0, 280, 120),
            art("basic_interface/collection_bg.bmp", 0, 0, 280, 120),
            at(10, 11, 75, 100),
            at(90, 10, 170, 14),
            escape(&name),
            at(91, 31, 182, 80),
            button(266, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "item-info-close", ""),
        )
    }

    /// What a skill does, as the client shows it (`UISkillDescribeWnd`): a white box 304 wide inside a rounded
    /// line, as tall as the client's description of the skill needs at 18 pixels a line (the first line is the
    /// skill's name), with the close button in its corner. Only a text taller than the screen scrolls.
    fn skill_info_window(&self) -> String {
        let Some(id) = self.skill_info else { return String::new() };
        let lines = data::skill_text(id).unwrap_or_else(|| vec![data::skill(id).map_or_else(|| tr("Unknown Skill").to_owned(), |(_, name, _)| name)]);
        let height = 18 * lines.len() as i32 + 4 + 6;
        let description: String = lines.iter().map(|line| format!(r#"<div style="height:18px;line-height:18px;white-space:nowrap">{}</div>"#, coloured(line))).collect();
        format!(
            r#"<div style="position:relative;width:304px;height:{height}px;max-height:calc(100vh - 50px);background:#fff;border:1px solid #c3c3c3;border-radius:4px;box-sizing:border-box;overflow:hidden">
               <div class="t viewer" style="position:absolute;left:3px;top:3px;right:3px;bottom:3px;user-select:text">{description}</div>{}</div>"#,
            button(304 - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "skill-info-close", ""),
        )
    }

    /// The Kafra's storage, laid out as the original's: tabs for the kinds of things down the left side, each
    /// thing a line with its picture, how many and its name, and how full the storage is with "close" at the
    /// bottom. A double click on a line takes the whole stack out; while the storage is open, a double click in
    /// the inventory puts the whole stack in.
    fn storage_window(&self) -> String {
        const TABS: [&str; 8] = ["Item", "Cash", "Armor", "Wpn", "Cstm", "Ammo", "Card", "Misc"];
        const LINES: i32 = 11;
        let Some(storage) = &self.storage else { return String::new() };
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
            let open = if index == self.storage_tab { " open" } else { "" };
            window.push_str(&format!(
                r#"<button type="button" class="a side{open}" style="{}" data-click="storage-tab" data-arg="{index}"><i>{}</i></button>"#,
                at(2, 19 + index as i32 * 34, 16, 34),
                tr(label)
            ));
        }
        let lines: String = storage
            .items
            .iter()
            .filter(|item| tab_of(item) == self.storage_tab)
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

    /// Redraws the windows that show items and skills, where they are on the page.
    pub(super) fn refresh_items(&self) {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else { return };
        for (id, html) in [("inventory", self.inventory_window()), ("skills", self.skill_window()), ("hotkeys", self.hotkeys()), ("storage", self.storage_window()), ("iteminfo", self.item_info_window()), ("skillinfo", self.skill_info_window()), ("equipment", self.equipment_window())] {
            if let Some(element) = document.get_element_by_id(id) {
                // The lists keep their place when they are redrawn.
                let scrolled = element.query_selector(".scroll").ok().flatten().map_or(0, |list| list.scroll_top());
                element.set_inner_html(&html);
                if let Ok(Some(list)) = element.query_selector(".scroll") {
                    list.set_scroll_top(scrolled);
                }
            }
        }
        // The windows that show the character were written anew, and their canvases with them: the character
        // is drawn on them again.
        self.paint_figures();
    }

    /// Redraws the chat's lines, the newest at the bottom.
    pub(super) fn refresh_chat(&self) {
        if let Some(log) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("chat-lines")) {
            log.set_inner_html(&self.chat_lines(self.chat_rows));
        }
    }

    /// A gauge of the Basic Information window: its back, the fill up to `now` of `most` (red when under a
    /// quarter), and the two numbers over it.
    fn gauge(&self, x: i32, y: i32, now: i64, most: i64) -> String {
        let mut gauge = art("basic_interface/gze_bg1.bmp", x, y, 135, 9);
        // Values that make no sense (more than the most) are not drawn at all.
        if most <= 0 || now > most {
            return gauge;
        }
        let colour = if now * 4 < most { "red" } else { "blue" };
        let fill = (127 * now.max(0) / most) as i32;
        gauge.push_str(&art(&format!("basic_interface/gze{colour}_left.bmp"), x, y, 4, 8));
        gauge.push_str(&art(&format!("basic_interface/gze{colour}_mid.bmp"), x + 4, y, fill, 8));
        gauge.push_str(&art(&format!("basic_interface/gze{colour}_right.bmp"), x + 4 + fill, y, 4, 8));
        gauge.push_str(&format!(r#"<div class="a t centre" style="{};white-space:pre">{now:3}  /  {most:3}</div>"#, at(x, y - 2, 135, 13)));
        gauge
    }

    /// An experience bar: how much of the way to the next level is done.
    fn experience(&self, x: i32, y: i32, now: i64, next: i64) -> String {
        if next <= 0 {
            return String::new();
        }
        let fill = (110 * now.clamp(0, next) / next) as i32;
        format!(
            r#"<div class="a" style="{};background:#fff;border:1px solid #afafaf" title="{:.1} %"><div style="width:{fill}px;height:4px;background:#567fcf"></div></div>"#,
            at(x, y, 112, 6),
            now as f64 * 100.0 / next as f64
        )
    }

    fn percent(now: i64, most: i64) -> String {
        if most > 0 { format!("{}%", now * 100 / most) } else { String::new() }
    }

    /// The strip of icons that open the windows, as the original lays it out: five to a row, under a bar that
    /// folds it away. The icons of windows not made yet are there, faded, and do nothing.
    fn menu_icons(&self) -> String {
        const ICONS: [(&str, &str, &str); 23] = [
            ("status", "status", "Status (Alt+A)"),
            ("equip", "equipment", "Equipment (Alt+Q)"),
            ("item", "inventory", "Inventory (Alt+E)"),
            ("skill", "skills", "Skills (Alt+S)"),
            ("party", "", "Party (Alt+Z)"),
            ("guild", "", "Guild (Alt+G)"),
            ("battle", "", "Battlegrounds"),
            ("quest", "", "Quests (Alt+U)"),
            ("map", "", "World map (Ctrl+~)"),
            ("navigation", "", "Navigation"),
            ("option", "", "Options (Alt+O)"),
            ("bank", "", "Bank"),
            ("rec", "", "Replay"),
            ("mail", "", "Mail"),
            ("achievement", "", "Achievements"),
            ("tip", "", "Tips"),
            ("shop", "", "Cash shop"),
            ("keyboard", "", "Shortcut settings"),
            ("attendance", "", "Attendance"),
            ("adventurerAgency", "", "Adventurer agency"),
            ("repute", "", "Reputation"),
            ("adventureguide", "", "Adventure guide"),
            ("probability", "", "Probabilities"),
        ];
        let bar = |name: &str| {
            format!(
                r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="menu" title="Menu"></button>"#,
                at(0, 4, 219, 9),
                picture(&format!("menu_icon/bt_menu_{name}normal.bmp")),
                picture(&format!("menu_icon/bt_menu_{name}over.bmp")),
                picture(&format!("menu_icon/bt_menu_{name}press.bmp"))
            )
        };
        if self.menu_closed {
            return bar("");
        }
        let mut strip = bar("close_");
        for (position, (name, click, title)) in ICONS.iter().enumerate() {
            let name = if *name == "status" && self.game.as_ref().is_some_and(|game| data::is_doram(game.job)) { "status_doram" } else { name };
            let (x, y) = ((position % 5 * 42 + 9) as i32, (position / 5 * 43 + 23) as i32);
            let (normal, pressed) = (picture(&format!("menu_icon/bt_{name}.bmp")), picture(&format!("menu_icon/bt_{name}_press.bmp")));
            let dim = if click.is_empty() { " dim" } else { "" };
            strip.push_str(&format!(
                r#"<button type="button" class="a b{dim}" style="{};--n:url('{normal}');--o:url('{normal}');--p:url('{pressed}')" data-click="{click}" title="{}"></button>"#,
                at(x, y, 32, 34),
                tr(title)
            ));
        }
        strip
    }

    fn basic_info(&self) -> String {
        let Some(game) = &self.game else { return String::new() };
        let job = data::job_name(game.job);
        let (hp, max_hp, sp, max_sp) = (self.value(parameter::HP), self.value(parameter::MAX_HP), self.value(parameter::SP), self.value(parameter::MAX_SP));
        let (base_level, job_level) = (self.value(parameter::BASE_LEVEL), self.value(parameter::JOB_LEVEL));
        let (base_exp, next_base) = (self.value(parameter::BASE_EXP), self.value(parameter::NEXT_BASE_EXP));
        let (job_exp, next_job) = (self.value(parameter::JOB_EXP), self.value(parameter::NEXT_JOB_EXP));

        if self.basic_small {
            // The small form: a title bar with the name, and two lines.
            let exp = if next_base > 0 { base_exp as f64 * 100.0 / next_base as f64 } else { 0.0 };
            let low = |now: i64, most: i64| if now * 4 < most { "#f00" } else { BLACK };
            return format!(
                r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0"></div>{}{}{}
                   <div class="a t" style="left:9px;top:36px"><span style="color:{}">HP. {hp} / {max_hp}</span> | <span style="color:{}">SP. {sp} / {max_sp}</span></div>"#,
                at(0, 17, 220, 38),
                title_bar(220, &game.name),
                button(206, 3, 11, "basic_interface/sys_mini_off.bmp", "basic_interface/sys_mini_on.bmp", "basic-size", ""),
                text(9, 20, BLACK, &format!("{lv}{base_level:2} / {job} / {lv}{job_level:2} / {} {exp:.1} %", tr("Exp."), lv = tr("Lv."))),
                low(hp, max_hp),
                low(sp, max_sp),
            );
        }

        let mut window = art("basic_interface/basewin_bg2.bmp", 0, 0, 220, 134);
        window.push_str(&format!(r#"<div class="a t titled" style="left:17px;top:2px">{}</div>"#, tr("Basic Information")));
        window.push_str(&button(206, 3, 11, "basic_interface/sys_mini_off.bmp", "basic_interface/sys_mini_on.bmp", "basic-size", ""));
        window.push_str(&text(9, 22, BLACK, &game.name));
        window.push_str(&text(9, 36, BLACK, &job));
        window.push_str(&text(17, 51, BLACK, "HP"));
        window.push_str(&text(17, 65, BLACK, "SP"));
        window.push_str(&self.gauge(35, 53, hp, max_hp));
        window.push_str(&self.gauge(35, 68, sp, max_sp));
        window.push_str(&text(173, 51, BLACK, &Self::percent(hp, max_hp)));
        window.push_str(&text(173, 65, BLACK, &Self::percent(sp, max_sp)));
        window.push_str(&text(17, 84, BLACK, &format!("{} {base_level}", tr("Base Lv."))));
        window.push_str(&text(17, 96, BLACK, &format!("{} {job_level}", tr("Job Lv."))));
        window.push_str(&self.experience(85, 89, base_exp, next_base));
        window.push_str(&self.experience(85, 101, job_exp, next_job));
        // Weight is kept in tenths; carrying half of what one can is already too much to recover by resting.
        let (weight, max_weight) = (self.value(parameter::WEIGHT), self.value(parameter::MAX_WEIGHT));
        let heavy = if max_weight > 0 && weight * 2 >= max_weight { "#f00" } else { BLACK };
        window.push_str(&format!(
            r#"<div class="a t" style="right:5px;top:116px;white-space:pre"><span style="color:{heavy}">{} : {:3} / {:3}</span>  {} : {}</div>"#,
            tr("Weight"),
            weight / 10,
            max_weight / 10,
            tr("Zeny"),
            grouped(self.value(parameter::ZENY))
        ));
        window
    }

    /// The picture of an item or a skill, or nothing while its name is not known.
    fn icon(resource: Option<&str>) -> String {
        match resource {
            // An item this client's tables do not have is shown as the original shows it: an apple.
            Some(resource) if resource.is_empty() => Self::icon(Some("사과")),
            Some(resource) => format!(r#"<img src="{}" alt="" draggable="false" onerror="this.remove()">"#, picture(&format!("item/{resource}.bmp"))),
            None => String::new(),
        }
    }

    /// How many of an item the character carries without wearing them.
    fn carried(&self, id: u32) -> u32 {
        self.inventory.iter().filter(|item| item.id == id && item.worn == 0).map(|item| u32::from(item.amount)).sum()
    }

    /// The bar of slots for skills and items: two rows of nine. F1 to F9 use the first row; an icon dragged from
    /// the inventory or the skill list fills a slot, and a right click empties it.
    /// The hotkey bar as the client draws it (`UIShortCutWnd`): one to four rows of nine, each row its frame 33
    /// under the last with its number at the left; no key is written on a slot (the client names the key only
    /// when a slot is pointed at). The buttons at the right take a row away and add one.
    fn hotkeys(&self) -> String {
        let rows = i32::from(self.hotkey_rows) + 1;
        let mut bar = String::new();
        for row in 0..rows {
            bar.push_str(&art("shortcut/shortcut_frame.bmp", 0, row * 33, 290, 34));
            bar.push_str(&text(6, row * 33 + 13, "rgb(74,74,47)", &(row + 1).to_string()));
            for column in 0..9 {
                let slot = (row * 9 + column) as usize;
                let hotkey = self.hotkeys.get(slot).copied().unwrap_or_default();
                let (icon, count, title) = self.hotkey_look(hotkey);
                let key = if row == 0 { format!("F{} ", column + 1) } else { String::new() };
                let skill = if hotkey.skill && hotkey.id != 0 { format!(r#" data-skill="{}""#, hotkey.id) } else { String::new() };
                bar.push_str(&format!(
                    r#"<div class="a slot" style="{}" data-slot="{slot}" data-use="slot:{slot}"{skill} title="{key}{}">{icon}{count}</div>"#,
                    at(16 + 29 * column, row * 33 + 4, 24, 24),
                    escape(&title),
                ));
            }
        }
        for (name, arg, y) in [("minus", "-", rows * 33 - 28), ("plus", "+", rows * 33 - 13)] {
            bar.push_str(&format!(
                r#"<img class="a" style="right:2px;top:{y}px;cursor:pointer" src="{}" alt="" draggable="false" data-click="hotkey-rows" data-arg="{arg}">"#,
                picture(&format!("shortcut/btn_{name}_normal.bmp"))
            ));
        }
        bar
    }

    /// A hotkey's picture, the number under it, and what it says when pointed at.
    fn hotkey_look(&self, hotkey: Hotkey) -> (String, String, String) {
        if hotkey.id == 0 {
            return (String::new(), String::new(), String::new());
        }
        if hotkey.skill {
            let known = data::skill(hotkey.id as u16);
            let icon = Self::icon(known.as_ref().map(|(resource, ..)| resource.to_lowercase()).as_deref());
            let name = known.map_or_else(|| format!("{} {}", tr("Skill"), hotkey.id), |(_, name, _)| name);
            (icon, format!(r#"<span class="n">{}</span>"#, hotkey.count), format!("{name} Lv {}", hotkey.count))
        } else {
            let known = data::item(hotkey.id);
            let carried = self.carried(hotkey.id);
            // An item that is used up stays on the bar, without its picture, until there is more of it.
            let icon = if carried == 0 { String::new() } else { Self::icon(known.as_ref().map(|(_, resource)| resource.as_str())) };
            let name = known.map_or_else(|| format!("{} {}", tr("Item"), hotkey.id), |(name, _)| name);
            (icon, if carried == 0 { String::new() } else { format!(r#"<span class="n">{carried}</span>"#) }, format!("{name} : {carried} {}", tr("ea")))
        }
    }

    /// An item's name as the game shows it: its refinement before it, its free slots after it.
    fn item_name(item: &Item) -> String {
        let name = data::item(item.id).map_or_else(|| format!("{} {}", tr("Item"), item.id), |(name, _)| name);
        if item.refine > 0 { format!("+{} {name}", item.refine) } else { name }
    }

    fn inventory_window(&self) -> String {
        let width = 40 + 32 * ITEM_COLUMNS + 16;
        let body = 32 * ITEM_ROWS;
        let mut window = title_bar(width, "Inventory");
        window.push_str(&button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "inventory", ""));
        // The body: the tabs' strip, the cells, and the pieces at their sides.
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border-left:1px solid #c3c3c3"></div>"#, at(0, 17, 40, body)));
        window.push_str(&format!(r#"<div class="a" style="{};background:url('{}') 0 0 repeat-y"></div>"#, at(20, 17, 20, body), picture("basic_interface/itemwin_left.bmp")));
        window.push_str(&format!(r#"<div class="a" style="{};background:url('{}') 0 0 repeat-y"></div>"#, at(width - 20, 17, 20, body), picture("basic_interface/itemwin_right.bmp")));
        for (index, (tab, label)) in [(Tab::Usable, "Use"), (Tab::Equipment, "Eqp"), (Tab::Other, "Etc")].into_iter().enumerate() {
            let open = if tab == self.inventory_tab { " open" } else { "" };
            let letters: String = tr(label).chars().map(|letter| format!("<span>{letter}</span>")).collect();
            window.push_str(&format!(
                r#"<button type="button" class="a tab{open}" style="{}" data-click="inventory-tab" data-arg="{index}">{letters}</button>"#,
                at(3, 18 + index as i32 * 51, 17, 52)
            ));
        }
        let mut cells = String::new();
        // What is worn is in the Equipment window, as in the original.
        let shown: Vec<&Item> = self.inventory.iter().filter(|item| item.tab() == self.inventory_tab && item.worn == 0).collect();
        for (place, item) in shown.iter().enumerate() {
            let (column, row) = (place as i32 % ITEM_COLUMNS, place as i32 / ITEM_COLUMNS);
            let resource = data::item(item.id).map(|(_, resource)| resource);
            let worn = if item.worn != 0 { " worn" } else { "" };
            cells.push_str(&format!(
                r#"<div class="a cell{worn}" style="left:{}px;top:{}px" draggable="true" data-drag="item:{}" data-use="{}:{}" data-info="{info}" title="{} : {} {}">{}<span class="n">{}</span></div>"#,
                column * 32,
                row * 32,
                item.id,
                if self.storage.is_some() { "put" } else { "item" },
                item.index,
                escape(&Self::item_name(item)),
                item.amount,
                tr("ea"),
                Self::icon(resource.as_deref()),
                item.amount,
                info = item.id
            ));
        }
        // Rows of empty cells below the last item, so the window is never shorter than it shows.
        let rows = (shown.len().div_ceil(ITEM_COLUMNS as usize) as i32).max(ITEM_ROWS);
        window.push_str(&format!(
            r#"<div class="a scroll" style="{};background:url('{}')"><div style="position:relative;height:{}px">{cells}</div></div>"#,
            at(40, 17, 32 * ITEM_COLUMNS, body),
            picture("basic_interface/itemwin_mid.bmp"),
            rows * 32
        ));
        let footer = 17 + body;
        window.push_str(&art("basic_interface/btnbar_left.bmp", 0, footer, 21, 21));
        window.push_str(&art("basic_interface/btnbar_mid.bmp", 21, footer, width - 42, 21));
        window.push_str(&art("basic_interface/btnbar_right.bmp", width - 21, footer, 21, 21));
        let kinds = self.inventory.iter().filter(|item| item.worn == 0).count();
        window.push_str(&text(35, footer + 5, if kinds >= 100 { "#f00" } else { "#232323" }, &format!("{kinds} / 100")));
        window
    }

    /// The skill window as the client draws it (`UINewSkillListWnd`, its tree of skills): 560 by 400, the tabs of
    /// the jobs down its left, the skills of the job in seven columns 68 apart and rows 56 apart, each with its
    /// name over it and its level under it, the character standing in the corner.
    fn skill_window(&self) -> String {
        const TABS: i32 = 20;
        let (width, height) = (560, 400);
        let job = self.own_figure().job;
        let pages = data::skill_pages(job);
        let page = self.skill_tab.min(pages.len().saturating_sub(1));
        let mut window = String::new();
        // The body: white between the bars, inside two rounded lines.
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border:1px solid #c3c3c3;border-radius:4px;box-sizing:border-box"></div>"#, at(TABS, 0, width - TABS, height)));
        window.push_str(&format!(r#"<div class="a" style="{};border:1px solid #c3c3c3;border-radius:3px;box-sizing:border-box"></div>"#, at(TABS + 2, 2, width - TABS - 4, height - 4)));
        window.push_str(&art("basic_interface/skill_upbar_left.bmp", TABS, 0, 12, 20));
        window.push_str(&art("basic_interface/skill_upbar_mid.bmp", TABS + 12, 0, width - TABS - 24, 20));
        window.push_str(&art("basic_interface/skill_upbar_right.bmp", width - 12, 0, 12, 20));
        window.push_str(&art("basic_interface/skill_botbar_left.bmp", TABS, height - 32, 12, 32));
        window.push_str(&art("basic_interface/skill_botbar_mid.bmp", TABS + 12, height - 32, width - TABS - 24, 32));
        window.push_str(&art("basic_interface/skill_botbar_right.bmp", width - 12, height - 32, 12, 32));
        window.push_str(&format!(r#"<div class="a t titled" style="left:40px;top:5px">{}</div>"#, tr("Skill Tree")));
        window.push_str(&art("basic_interface/sys_base_off.bmp", 22, 5, 11, 11));
        window.push_str(&art("basic_interface/sys_mini_off.bmp", width - 30, 5, 11, 11));
        window.push_str(&button(width - 16, 5, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "skills", ""));
        // The tabs: one for each job the character grew through.
        for tab in 0..pages.len() {
            let open = if tab == page { " open" } else { "" };
            window.push_str(&format!(
                r#"<button type="button" class="a side{open}" style="{};font:11px Tahoma,Arial,sans-serif" data-click="skill-tab" data-arg="{tab}">{}</button>"#,
                at(0, 22 + tab as i32 * 50, TABS + 1, 48),
                tr(["1st", "2nd", "3rd", "4th"].get(tab).copied().unwrap_or(""))
            ));
        }
        // The cells, empty ones first.
        for cell in 0..42 {
            window.push_str(&art("basic_interface/no_skill.bmp", cell % 7 * 68 + 76, cell / 7 * 56 + 44, 24, 24));
        }
        let points = self.value(parameter::SKILL_POINTS);
        for (place, id) in pages.get(page).into_iter().flatten().copied().filter(|(place, _)| *place < 42) {
            let (x, y) = (i32::from(place % 7) * 68 + 72, i32::from(place / 7) * 56);
            let known = data::skill(id);
            let learned = self.skills.iter().find(|skill| skill.id == id);
            let level = learned.map_or(0, |skill| skill.level);
            let name = known.as_ref().map_or_else(|| format!("{} {id}", tr("Skill")), |(_, name, _)| name.clone());
            let most = known.as_ref().map_or(level, |(.., most)| *most);
            let raisable = learned.is_some_and(|skill| skill.raisable) && points > 0 && level < most;
            // A skill that can be raised stands on light blue; the others on white.
            let ground = if raisable { "background:#afc4f0;outline:1px solid #afc4f0;border-radius:2px" } else { "background:#fff" };
            window.push_str(&format!(r#"<div class="a" style="{};{ground}"></div>"#, at(x + 3, y + 43, 26, 26)));
            // Only a skill that is learned and does something can be used and put on the bar.
            let usable = learned.is_some_and(|skill| skill.level > 0 && !skill.passive());
            let drag = if usable { format!(r#"draggable="true" data-drag="skill:{id}:{level}" data-use="skill:{id}""#) } else { String::new() };
            let dim = if level == 0 { "filter:grayscale(1) opacity(.55);" } else { "" };
            let icon = Self::icon(known.as_ref().map(|(resource, ..)| resource.to_lowercase()).as_deref());
            window.push_str(&format!(r#"<div class="a" style="{};{dim}" {drag} data-skill="{id}" title="{}">{icon}</div>"#, at(x + 4, y + 44, 24, 24), escape(&name)));
            let shade = if level == 0 { "#787878" } else { "#666666" };
            // The name, over the cell: as much of it as fits in 48 pixels.
            window.push_str(&format!(
                r#"<div class="a" style="{};font:11px Tahoma,Arial,sans-serif;color:{shade};text-align:center;white-space:nowrap;overflow:hidden;pointer-events:none">{}</div>"#,
                at(x - 8, y + 30, 48, 13),
                escape(&name)
            ));
            if level > 0 {
                let shown = if learned.is_some_and(|skill| !skill.passive()) { format!("{level} / {level}") } else { level.to_string() };
                window.push_str(&format!(
                    r#"<div class="a" style="{};font:bold 11px Tahoma,Arial,sans-serif;color:{shade};text-align:center;white-space:nowrap;text-shadow:-1px 0 #fff,1px 0 #fff,0 -1px #fff,0 1px #fff;pointer-events:none">{shown}</div>"#,
                    at(x - 8, y + 70, 48, 13)
                ));
            }
            if raisable {
                window.push_str(&format!(
                    r#"<button type="button" class="a" style="{};padding:0;border:0;background:none;font:bold 11px Tahoma,Arial,sans-serif;color:#f00;cursor:pointer" data-click="raise-skill" data-arg="{id}" title="{}">+</button>"#,
                    at(x + 30, y + 42, 12, 14),
                    tr("Raise")
                ));
            }
        }
        // The character stands in the corner, its feet 25 from the bottom.
        window.push_str(&format!(
            r#"<div class="a" style="{};overflow:hidden;pointer-events:none"><canvas id="pc-skill" width="{}" height="{}" style="position:absolute;left:{}px;top:{}px"></canvas></div>"#,
            at(0, height - 130, 100, 130),
            super::figure::WIDTH,
            super::figure::HEIGHT,
            48 - 80,
            130 - 25 - 137
        ));
        window.push_str(&text(80, height - 23, BLACK, &format!("{} : {points}", tr("Skill Point"))));
        for (name, x) in [("btn_apply", width - 100), ("btn_reset", width - 50)] {
            window.push_str(&format!(r#"<div class="a" style="{};background:url('{}');opacity:.5"></div>"#, at(x, height - 27, 42, 20), picture(&format!("{name}.bmp"))));
        }
        window
    }

    fn status_window(&self) -> String {
        let width = 280;
        let mut window = title_bar(width, "Status");
        window.push_str(&button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "status", ""));
        window.push_str(&art("statuswnd/w_statwin_bg.bmp", 0, 17, 280, 124));
        let points = self.value(parameter::STATUS_POINTS);
        for stat in 0..6u16 {
            let top = 23 + 16 * i32::from(stat);
            let (value, bonus, cost) = (self.value(parameter::STR + stat), self.stat_bonus[usize::from(stat)], self.value(parameter::STR_COST + stat));
            window.push_str(&text(36, top, BLACK, &value.to_string()));
            if bonus != 0 {
                window.push_str(&text(if value >= 100 { 54 } else { 49 }, top, BLACK, &format!("{bonus:+}")));
            }
            // A stat can be raised while the points for it are there.
            if cost > 0 && cost <= points {
                window.push_str(&button(78, top + 1, 11, "basic_interface/arw_right.bmp", "basic_interface/arw_right.bmp", "raise", &stat.to_string()));
            }
            if cost > 0 {
                window.push_str(&text_ending(101, width, top, BLACK, &cost.to_string()));
            }
        }
        let pair = |first: u16, second: u16| format!("{} + {}", self.value(first), self.value(second));
        // The server keeps the delay between attacks; the window shows the speed it means.
        let attack_speed = (2000 - self.value(parameter::ATTACK_DELAY)) / 10;
        let left = [pair(parameter::ATK, parameter::ATK_BONUS), pair(parameter::MATK_MIN, parameter::MATK_MAX), self.value(parameter::HIT).to_string(), self.value(parameter::CRITICAL).to_string()];
        let right = [
            pair(parameter::DEF, parameter::DEF_BONUS),
            pair(parameter::MDEF, parameter::MDEF_BONUS),
            pair(parameter::FLEE, parameter::FLEE_BONUS),
            attack_speed.to_string(),
            points.to_string(),
        ];
        for (row, value) in left.iter().enumerate() {
            window.push_str(&text_ending(189, width, 22 + 16 * row as i32, BLACK, value));
        }
        for (row, value) in right.iter().enumerate() {
            window.push_str(&text_ending(275, width, 22 + 16 * row as i32, BLACK, value));
        }
        window
    }

    fn chat_lines(&self, lines: usize) -> String {
        // Every line kept is there: the window shows the last, and scrolls back through the others. The second
        // tab keeps to what the game itself says (things picked up, deals, refusals), not what people say.
        let _ = lines;
        self.chat
            .iter()
            .filter(|(_, colour)| self.chat_tab == 0 || !matches!(colour.as_str(), "#0f0" | "#fff"))
            .map(|(words, colour)| format!(r#"<div style="color:{colour}">{}</div>"#, escape(words)))
            .collect()
    }

    /// The chat, as the client's `UINewChatWnd` lays it out, 600 wide: the tabs at the top (71 wide, 16 high,
    /// the open one lighter), the small buttons at the right of them and the lock that lets the window be
    /// dragged; the see-through list between its white and grey lines; and the input line with the box for
    /// whom to whisper to, the two edits and the round buttons. With no rows only the input line is left.
    fn chat_window(&self, lines: usize, height: i32) -> String {
        const TABS: [&str; 2] = ["Public Log", "Battle Log"];
        const TAB: i32 = 71;
        let base = height - 24;
        let mut window = String::new();
        if lines > 0 {
            let tabs = TABS.len() as i32 * TAB;
            for (index, name) in TABS.iter().enumerate() {
                let open = index == self.chat_tab;
                window.push_str(&format!(
                    r#"<button type="button" class="a chat-tab" style="{};background:rgba(0,0,0,{})" data-click="chat-tab" data-arg="{index}">{}</button>"#,
                    at(4 + TAB * index as i32, if open { 1 } else { 2 }, TAB - 1, if open { 16 } else { 15 }),
                    if open { ".4" } else { ".6" },
                    tr(name),
                ));
            }
            // The list's see-through back, and the lines around it.
            window.push_str(&format!(r#"<div class="a" style="{};background:rgba(0,0,0,.4)"></div>"#, at(4, 17, 600 - 8, height - 16 - 24 - 1)));
            window.push_str(&format!(r#"<div class="a" style="{};background:#fff"></div>"#, at(3, 16, 1, height - 16)));
            window.push_str(&format!(r#"<div class="a" style="{};background:#fff"></div>"#, at(tabs + 3, 16, 600 - tabs - 10, 1)));
            window.push_str(&format!(r#"<div class="a" style="{};background:#8e8e8e"></div>"#, at(600 - 4, 18, 1, height - 18)));
            window.push_str(&format!(
                r#"<div id="chat-log" class="a" style="{}"><div id="chat-lines">{}</div></div>"#,
                at(5, 18, 600 - 8, height - 16 - 24 - 4),
                self.chat_lines(lines)
            ));
            // From the right: the lock, then the small buttons, each 2 from the next (`ApplyHeaderLayout`).
            let lock = if self.chat_locked { ("lock_dragwnd", 9) } else { ("unlock_dragwnd", 15) };
            window.push_str(&format!(
                r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="chat-lock" title="Lock the window in place"></button>"#,
                at(600 - 20, 4, lock.1, 9),
                picture(&format!("basic_interface/{}.bmp", lock.0)),
                picture(&format!("basic_interface/{}.bmp", lock.0)),
                picture(&format!("basic_interface/{}.bmp", lock.0)),
            ));
            let mut right = 600 - 20;
            for (name, width, tall, click, title) in [("wnd_mini", 8, 8, "chat-mini", "Hide the log"), ("battle_option", 8, 8, "", "Add a tab"), ("stickoff", 14, 9, "", "Detach"), ("battle_option2", 9, 9, "", "Options")] {
                right -= width + 2;
                window.push_str(&format!(
                    r#"<button type="button" class="a b{}" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="{click}" title="{title}"></button>"#,
                    if click.is_empty() { " dim" } else { "" },
                    at(right, 4, width, tall),
                    picture(&format!("basic_interface/{name}_a.bmp")),
                    picture(&format!("basic_interface/{name}_b.bmp")),
                    picture(&format!("basic_interface/{name}_b.bmp")),
                    title = tr(title),
                ));
            }
        }
        window.push_str(&format!(r#"<div class="a" style="{};background:#8e8e8e"></div>"#, at(4, base, 600 - 8, 1)));
        window.push_str(&format!(
            r#"<form data-submit="chat" class="a" style="{};background:url('{}') 0 0 no-repeat">
                <input class="a in" id="chat-to" maxlength="23" autocomplete="off" style="{};background:#f0f0f0" title="Whisper to">
                <div class="a" style="{};background:url('{}') 0 0 no-repeat"></div>
                <input class="a in" id="chat-input" maxlength="70" autocomplete="off" style="{}">
                <button type="button" class="a b" style="{};--n:url('{off}');--o:url('{on}');--p:url('{on}')" data-click="chat-size" title="Change the size (F10)"></button>
                <button type="button" class="a b" style="{};--n:url('{off}');--o:url('{on}');--p:url('{on}')" data-click="chat-mini" title="Hide or show the log"></button>
                <button type="submit" hidden></button>
            </form>"#,
            at(0, base, 600, 24),
            picture("basic_interface/dialog_bg.bmp"),
            at(6, 4, 90, 16),
            at(97, 3, 8, 18),
            picture("basic_interface/dialog_btn0.bmp"),
            at(110, 4, 460, 16),
            at(572, 7, 11, 11),
            at(586, 7, 11, 11),
            off = picture("basic_interface/sys_base_off.bmp"),
            on = picture("basic_interface/sys_base_on.bmp"),
        ));
        window
    }
}

#[cfg(test)]
mod tests {
    use super::grouped;

    #[test]
    fn groups_thousands() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1,000");
        assert_eq!(grouped(1234567), "1,234,567");
        assert_eq!(grouped(-45000), "-45,000");
    }
}
