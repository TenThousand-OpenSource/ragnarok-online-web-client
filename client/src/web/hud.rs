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
use super::parts::{at, dragged, picture, pressed, sent, used};
use crate::press::things::{Drag, Form, Leveled, Use};
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
mod guide;
mod trade;
pub(super) mod windows;

use basic::*;


/// How many chat lines are kept.
pub const CHAT_KEPT: usize = 240;

const BLACK: &str = "#000";
/// How many columns and rows of items the inventory shows, and how many skills the list shows.
const ITEM_COLUMNS: i32 = 7;
const ITEM_ROWS: i32 = 5;

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
        // White under the pieces too, a little inside the frame: where the page is laid out between pixels (a
        // box centred in an odd width, a zoomed page) the pieces part by a hair, and the ground must not show.
        format!(r#"<div class="a" style="{};background:#fff;border-radius:2px"></div>"#, at(3, top + 3, width - 6, height - 6)),
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
fn picture_press(x: i32, y: i32, name: &str, press: impl Into<crate::press::Press>) -> String {
    format!(
        r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" {}></button>"#,
        at(x, y, 42, 20),
        picture(&format!("{name}.bmp")),
        picture(&format!("{name}_a.bmp")),
        picture(&format!("{name}_b.bmp")),
        pressed(press)
    )
}

/// A small picture button whose picture changes under the pointer.
fn press_button(x: i32, y: i32, size: i32, off: &str, on: &str, press: impl Into<crate::press::Press>) -> String {
    format!(r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" {}></button>"#, at(x, y, size, size), picture(off), picture(on), picture(on), pressed(press))
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
