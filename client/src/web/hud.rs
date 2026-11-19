//! The windows of the game itself: Basic Information, the hotkey bar, Status, Inventory, the skill list, and chat.
//!
//! Sizes, places, pictures and colours are the original client's, as rustnarok-client lists them
//! (`src/ui/basic_info.rs`, `hotkeys.rs`, `status.rs`, `inventory.rs`, `skills.rs`, `chat.rs`, `window.rs`; MIT OR
//! Apache-2.0). The inventory and the skill list are simpler than the original's: fixed sizes, the skills as one
//! list without the tree, and worn equipment shown among the items (until there is an Equipment window).
//!
//! Each window lives in an element of its own, so a value that changes redraws only its window: the chat's
//! input line is never rebuilt while someone types in it.

use crate::figures::grouped;
use crate::i18n::tr;
use super::app::{App, Shop, escape};
use super::data;
use super::parts::{at, dragged, dropped, picture, pressed, sent, used};
use crate::press::things::{Drag, Drop, Form, Leveled, Use};
use crate::net::items::{Hotkey, Item, Tab};
use crate::net::packets::parameter;

mod basic;
mod boxes;
mod choosing;
mod inventory;
mod equipment;
mod skills;
mod tabs;
mod status;
mod chat;
mod dialogs;
mod pets;
mod quests;
mod pvp;
mod statuses;
mod cooldowns;
mod hotkeys_set;
mod settings;
mod guide;
mod party;
mod guild;
mod homunculus;
mod mercenary;
mod skill_list;
mod player_menu;
mod exchange;
mod mail;
mod cart;
mod vending;
mod fight;
mod friends;
mod whispers;
mod viewequip;
mod unread;
mod scripted;
mod chat_rooms;
mod family;
mod broadcast;
mod marks;
mod trade;
mod storage;
mod lists;
mod emotions;
mod tips;
mod worldmap;
mod books;
pub(super) mod windows;

use boxes::*;
use lists::*;


/// How many chat lines are kept.
pub const CHAT_KEPT: usize = 240;

const BLACK: &str = "#000";
/// How many columns of items the inventory shows (its rows are as many as fit beside its tabs:
/// `App::inventory_body`).
const ITEM_COLUMNS: i32 = 7;

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

/// A button that is one of three of the client's pictures: as it stands, under the pointer, pressed. Every
/// picture button of the windows is written by this. `more` is anything else its tag says (what it tells when
/// pointed at), with the space before it.
fn pictured(x: i32, y: i32, (width, height): (i32, i32), [off, over, down]: [&str; 3], press: impl Into<crate::press::Press>, more: &str) -> String {
    format!(
        r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" {}{more}></button>"#,
        at(x, y, width, height),
        picture(off),
        picture(over),
        picture(down),
        pressed(press)
    )
}

/// One of the original's small buttons with three pictures: `name.bmp`, `name_a.bmp` under the pointer,
/// `name_b.bmp` pressed.
fn picture_press(x: i32, y: i32, name: &str, press: impl Into<crate::press::Press>) -> String {
    pictured(x, y, (42, 20), [&format!("{name}.bmp"), &format!("{name}_a.bmp"), &format!("{name}_b.bmp")], press, "")
}

/// A small picture button whose picture changes under the pointer.
fn press_button(x: i32, y: i32, size: i32, off: &str, on: &str, press: impl Into<crate::press::Press>) -> String {
    pictured(x, y, (size, size), [off, on, on], press, "")
}

/// One of the client's small boxes with its words beside it (`UICheckBox`: `checkbox_1.bmp` ticked,
/// `checkbox_0.bmp` not, the words 12 to the right of where the box begins), pressed to switch it.
fn check_words(x: i32, y: i32, on: bool, words: &str, press: impl Into<crate::press::Press>) -> String {
    format!(
        r#"<div class="a t" style="left:{x}px;top:{y}px;height:12px;line-height:12px;padding-left:12px;white-space:nowrap;cursor:default;color:{BLACK};background:url('{}') 0 1px no-repeat" {}>{}</div>"#,
        picture(if on { "checkbox_1.bmp" } else { "checkbox_0.bmp" }),
        pressed(press),
        escape(words)
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

/// How the client asks for a text: the family, the height and whether it is bold, as `UIWindow::DrawTextAt`
/// and the other drawing functions take them, and `UIStaticText::SetFont` and `CUIStaticText::SetFont` with
/// `SetFontHeight` and `SetBold` (whose "Gulim" is family 0: the first of `FontCache`'s names). A window that
/// writes a text in anything but the usual (0, 12) says so with one of these, never with a size in pixels.
#[derive(Clone, Copy)]
pub(super) struct Font(pub u8, pub u8, pub bool);

/// The usual text in bold.
pub(super) const BOLD: Font = Font(0, 12, true);
/// The smaller text: how many of an item there are (`DrawItemWithQuantity`), the counts under the lists.
pub(super) const SMALL: Font = Font(0, 11, false);
/// What Basic Information and Status write their words and numbers in (`UIBasicInfoWnd`, `UIStatusWnd::Paint`).
const FIGURES: Font = Font(1, 14, false);

impl Font {
    /// The style of a text asked for so: its size and the line it stands in (`crate::font::style`), and its
    /// weight. The face is the page's, also in a button, which would otherwise take the browser's own.
    pub(super) fn style(self) -> String {
        format!("font-family:inherit;{};font-weight:{}", crate::font::style(self.0, self.1), if self.2 { "bold" } else { "normal" })
    }
}

/// A line of text with its top-left corner at a place, in the usual font.
fn text(x: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="left:{x}px;top:{y}px;color:{colour}">{}</div>"#, escape(words))
}

/// A line of text with its top-left corner at a place, in the font the client asks for there.
fn text_in(font: Font, x: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="left:{x}px;top:{y}px;color:{colour};{}">{}</div>"#, font.style(), escape(words))
}

/// A line of text ending at `right`, in a window `width` wide, in the font the client asks for there.
fn text_ending_in(font: Font, right: i32, width: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="right:{}px;top:{y}px;color:{colour};{}">{}</div>"#, width - right, font.style(), escape(words))
}

/// A line of text ending at `right`, in a window `width` wide.
fn text_ending(right: i32, width: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="right:{}px;top:{y}px;color:{colour}">{}</div>"#, width - right, escape(words))
}

/// How many of a thing there are, written in the corner of its cell: the client's smaller text
/// (`DrawItemWithQuantity`, `CUIGameItemWithNum::Paint` and the hotkey bar's `DrawRightOutlinedText`: (0, 11)).
fn count(amount: impl std::fmt::Display) -> String {
    format!(r#"<span class="n" style="{}">{amount}</span>"#, SMALL.style())
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

/// The white panel every plain window has under its title (`UIWindow::DrawPanelBackground`), with the bar its
/// OK and Cancel stand on at the bottom (`DrawButtonBar`).
fn panel(width: i32, height: i32) -> String {
    format!(
        r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0;box-sizing:border-box"></div>{}{}{}"#,
        at(0, 17, width, height - 17),
        art("basic_interface/btnbar_left.bmp", 0, height - 21, 21, 21),
        art("basic_interface/btnbar_mid.bmp", 21, height - 21, width - 42, 21),
        art("basic_interface/btnbar_right.bmp", width - 21, height - 21, 21, 21)
    )
}

impl App {
    /// A line of the client's message table (`msgstringtable.txt`, counted from 0), in the player's language
    /// where this client has it; `otherwise` until the tables are there.
    pub(in crate::web) fn words(line: usize, otherwise: &str) -> String {
        tr(&data::message(line).unwrap_or_else(|| otherwise.to_owned())).to_owned()
    }

    fn value(&self, name: u16) -> i64 {
        // A dead character has no life left, whatever the server has already set aside for its return.
        if self.hero.dead && name == parameter::HP {
            return 0;
        }
        self.hero.params.get(&name).copied().unwrap_or(0)
    }
}
