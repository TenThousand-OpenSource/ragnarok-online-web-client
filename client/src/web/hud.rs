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

mod basic;
mod inventory;
mod equipment;
mod skills;
mod status;
mod chat;
mod dialogs;
mod pets;
mod hotkeys_set;
mod settings;
mod trade;

use basic::*;

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

impl App {
    fn value(&self, name: u16) -> i64 {
        // A dead character has no life left, whatever the server has already set aside for its return.
        if self.hero.dead && name == parameter::HP {
            return 0;
        }
        self.hero.params.get(&name).copied().unwrap_or(0)
    }

    /// Every window of the game, each in its own element. `view` is the stage's size.
    pub(super) fn hud(&self, view: (f64, f64)) -> String {
        // The client's own sum (`UINewChatWnd::ResizeRows`): the input line, the rows and three more, the tabs.
        let chat_lines = self.chat.rows;
        let chat_height = if chat_lines == 0 { 24 } else { 24 + (chat_lines as i32 + 3) * 14 + 16 };
        let hotkeys = if self.skills.hotkeys_hidden { String::new() } else { format!(r#"<div id="hotkeys" class="a hud" style="{}">{}</div>"#, at(230, 0, 290, (i32::from(self.skills.hotkey_rows) + 1) * 33 + 1), self.hotkeys()) };
        let status = if self.hero.status_open { format!(r#"<div id="status" class="a hud" style="{}">{}</div>"#, at(225, 285, 280, 141), self.status_window()) } else { String::new() };
        let inventory = if self.bag.open {
            format!(r#"<div id="inventory" class="a hud" style="{}">{}</div>"#, at(if self.deal.storage.is_some() { 515 } else { 225 }, 80, 40 + 32 * ITEM_COLUMNS + 16, 17 + 32 * ITEM_ROWS + 21), self.inventory_window())
        } else {
            String::new()
        };
        let skills = if self.skills.open { format!(r#"<div id="skills" class="a hud" style="{}">{}</div>"#, at(230, 60, 560, 400), self.skill_window()) } else { String::new() };
        // The original puts the NPC's window a fifth of the way down the screen, in the middle, and what it asks
        // under it.
        let dialog = format!(r#"<div id="dialog" class="a hud" style="left:{}px;top:{}px;width:280px">{}</div>"#, (view.0 as i32 - 280) / 2, view.1 as i32 * 100 / 480, self.dialog_window());
        let trade = format!(r#"<div id="trade" class="a hud" style="left:{}px;top:{}px;width:280px">{}</div>"#, (view.0 as i32 - 280) / 2, view.1 as i32 * 100 / 480, self.trade_window());
        let storage = format!(r#"<div id="storage" class="a hud" style="left:225px;top:80px;width:280px">{}</div>"#, self.storage_window());
        let throw = format!(r#"<div id="throw" class="a hud" style="left:{}px;top:{}px;width:182px">{}</div>"#, (view.0 as i32 - 182) / 2, view.1 as i32 * 200 / 480, self.throw_window());
        let equipment = if self.bag.equipment_open { format!(r#"<div id="equipment" class="a hud" style="left:560px;top:290px;width:280px">{}</div>"#, self.equipment_window()) } else { String::new() };
        let info = format!(
            r#"<div id="iteminfo" class="a hud" style="left:300px;top:230px;width:280px">{}</div><div id="skillinfo" class="a hud" style="left:330px;top:110px;width:304px">{}</div>"#,
            self.item_info_window(),
            self.skill_info_window()
        );
        // The list of eggs to hatch: 200 by 200, where the client opens it.
        let eggs = if self.pets.eggs.is_some() { format!(r#"<div id="eggs" class="a hud" style="left:{}px;top:{}px;width:200px">{}</div>"#, (view.0 as i32 - 200) / 2, view.1 as i32 * 200 / 480, self.egg_window()) } else { String::new() };
        // The pet window, where the client opens it: 185 across its 640-wide stage, 60 above 400 of 480 down.
        let pet = if self.pets.open && self.pets.pet.is_some() {
            format!(
                r#"<div id="pet" class="a hud" style="left:{}px;top:{}px;width:280px">{}</div><div id="pet-asks" class="a hud" style="left:{}px;top:{}px;width:280px">{}</div>"#,
                (view.0 as i32 - 640) / 2 + 185,
                view.1 as i32 * 400 / 480 - 60,
                self.pet_window(),
                (view.0 as i32 - 280) / 2,
                (view.1 as i32 - 120) / 2,
                self.pet_question()
            )
        } else {
            String::new()
        };
        // The slot machine of taming: 196 by 256 in the middle of the screen; a press on it starts the reels.
        let pet_menu = match self.pets.menu_at.filter(|_| self.pets.pet.is_some()) {
            Some((x, y)) => format!(r#"<div id="pet-menu" class="a hud" style="left:{x}px;top:{y}px;background:#fff;border:1px solid #9a9a9a;z-index:26">{}</div>"#, self.pet_menu()),
            None => String::new(),
        };
        // The Game Settings window, in the middle of the view, as the client puts it.
        let settings = if self.settings.open { format!(r#"<div id="settings" class="a hud" style="left:{}px;top:{}px;width:400px">{}</div>"#, ((view.0 as i32 - 400) / 2).max(0), ((view.1 as i32 - 350) / 2).max(0), self.settings_window()) } else { String::new() };
        // The Shortcut Settings window, in the middle of the view.
        let keys = if self.keys.open { format!(r#"<div id="keys" class="a hud" style="left:{}px;top:{}px;width:436px">{}</div>"#, ((view.0 as i32 - 436) / 2).max(0), ((view.1 as i32 - 432) / 2).max(0), self.key_window()) } else { String::new() };
        let taming = if self.pets.taming.is_some() {
            format!(
                r#"<div id="taming" class="a" style="{};cursor:pointer;z-index:30;pointer-events:auto" data-click="taming-shoot"><img id="taming-picture" alt="" draggable="false" style="position:absolute;left:0;top:0"></div>"#,
                at((view.0 as i32 - 196) / 2, (view.1 as i32 - 256) / 2, 196, 256)
            )
        } else {
            String::new()
        };
        let obtained = format!(r#"<div id="obtained" class="a" style="left:{}px;top:{}px;width:260px;pointer-events:none">{}</div>"#, (view.0 as i32 - 260) / 2, 75, self.obtained_note());
        // The menu hangs under the basic window, a little into it.
        let menu = format!(r#"<div id="menu" class="a hud" style="left:0;top:{}px;width:220px">{}</div>"#, if self.hero.basic_small { 55 } else { 134 } - 4, self.menu_icons());
        format!(
            r#"<div id="basic" class="a hud" style="left:0;top:0;width:220px">{}</div>{menu}{hotkeys}{status}{inventory}{skills}{storage}{equipment}{dialog}{trade}{info}{obtained}{throw}{taming}{eggs}{pet}{keys}{settings}{pet_menu}
               <div id="chat" class="a hud" data-locked="{}" style="{}">{}</div>"#,
            self.basic_info(),
            u8::from(self.chat.locked),
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
