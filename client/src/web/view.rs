//! The windows, as HTML laid over the game's own pictures.
//!
//! Sizes, places and colours are those of the original client (Ragexe 2026-02-19), read from its decompilation
//! (rodecomp: `UIWindowMgr.cpp`, `UINewSelectCharWnd.cpp`, `UINewMakeCharWnd.cpp`, `UIMessageBox.cpp`) and checked
//! against rustnarok-client. The registration and password windows have no original: the client has none.
//!
//! Text fields are real input elements, so phones' keyboards, paste and password managers work.

use super::app::{App, Ask, Game, Screen, epoch, escape};
use super::{data, figure};
use super::hud::{BOLD, Font, SMALL};
use super::parts::{at, picture, pressed, sent};
use crate::press::things::Form;
use crate::i18n::{Key, Lang};
use crate::net::packets::Character;

mod accounts;
mod characters;
mod creation;
mod game;

pub(super) use game::*;

/// The name shown over the login picture.
pub const SERVER_NAME: &str = "Ragnarok 2026";

/// The login picture of this client, stretched over the whole view as the original does.
pub const WALLPAPER: &str = "data/texture/유저인터페이스/t_login.jpg";

pub const SLOTS_PER_PAGE: u8 = 15;
/// How many hair styles a new character can choose from, and how many hair colours.
pub fn hair_styles(doram: bool) -> u16 {
    if doram { 6 } else { 23 }
}
pub const HAIR_COLOURS: u16 = 9;

/// The speaker of the sound switch, with waves when sound is on and crossed out when it is off.
fn speaker(sound: bool) -> String {
    [
        r#"<svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><path fill="currentColor" d="M2 6h3l4-3v10L5 10H2z"/>"#,
        r#"<path fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" d=""#,
        if sound { "M11 5.5a3.5 3.5 0 0 1 0 5M12.8 3.6a6 6 0 0 1 0 8.8" } else { "M11.2 6l3.6 4M14.8 6l-3.6 4" },
        r#""/></svg>"#
    ].concat()
}

/// One of the eight pictures of the frame around a chosen character.
pub(super) fn chosen_frame(frame: u8) -> String {
    picture(&format!("select_character_ver3/img_slot_select{frame}.bmp"))
}

fn background(name: &str) -> String {
    format!("background-image:url('{}')", picture(name))
}

/// A button's three pictures: as it is, under the pointer, and pressed.
fn states(normal: &str, over: &str, press: &str) -> String {
    format!("--n:url('{}');--o:url('{}');--p:url('{}')", picture(normal), picture(over), picture(press))
}

/// The pictures of a button whose files end in `_normal`, `_over` and `_press`.
fn states_of(base: &str) -> String {
    states(&format!("{base}_normal.bmp"), &format!("{base}_over.bmp"), &format!("{base}_press.bmp"))
}

/// The pictures of a button whose files are `name.bmp`, `name_a.bmp` (over) and `name_b.bmp` (pressed).
fn states_ab(base: &str) -> String {
    states(&format!("{base}.bmp"), &format!("{base}_a.bmp"), &format!("{base}_b.bmp"))
}

fn button(place: &str, pictures: &str, attributes: &str, label: &str) -> String {
    format!(r#"<button type="button" class="a b" style="{place};{pictures}" {attributes}>{label}</button>"#)
}

/// A window of the original's size at the original's place, shrunk to fit a smaller view.
fn window(view: (f64, f64), size: (i32, i32), place: (f64, f64), class: &str, backdrop: &str, inner: &str) -> String {
    let (width, height) = (f64::from(size.0), f64::from(size.1));
    let scale = (view.0 / width).min(view.1 / height).min(1.0);
    let left = place.0.min(view.0 - width * scale).max(0.0).floor();
    let top = place.1.min(view.1 - height * scale).max(0.0).floor();
    let backdrop = if backdrop.is_empty() { String::new() } else { background(backdrop) };
    format!(
        r#"<div class="w {class}" style="left:{left}px;top:{top}px;width:{}px;height:{}px;transform:scale({scale});{backdrop}">{inner}</div>"#,
        size.0, size.1
    )
}

/// Where the login window, and what stands in for it, goes.
fn login_place(view: (f64, f64)) -> (f64, f64) {
    (((view.0 - 640.0) / 2.0 + 185.0).floor(), (view.1 * 300.0 / 480.0).floor())
}

fn centred(view: (f64, f64), size: (i32, i32)) -> (f64, f64) {
    (((view.0 - f64::from(size.0)) / 2.0).floor(), ((view.1 - f64::from(size.1)) / 2.0).floor())
}

impl App {
    fn text(&self, key: Key) -> &'static str {
        key.text(self.lang)
    }

    pub(super) fn html(&self, view: (f64, f64)) -> String {
        let window = match &self.screen {
            Screen::Login => self.login_window(view),
            Screen::Register => self.register_window(view),
            Screen::Recover => self.recover_window(view),
            Screen::Reset(_) => self.reset_window(view),
            Screen::ServerSelect => self.server_select_window(view),
            Screen::CharSelect => self.char_select_window(view),
            Screen::CharCreate(_) => self.char_create_window(view),
            Screen::Game => self.game_window(view),
        };
        let languages: String = Lang::ALL
            .into_iter()
            .map(|lang| format!(
                r#"<button type="button" class="{}" {}>{}</button>"#,
                if lang == self.lang { "on" } else { "" },
                pressed(crate::press::general::Press::Language(lang)),
                lang.code().to_uppercase()
            ))
            .collect();
        // The server's name stands over the login picture; the larger windows cover that place.
        let titled = matches!(self.screen, Screen::Login | Screen::ServerSelect | Screen::Register | Screen::Recover | Screen::Reset(_));
        let title = if titled { format!(r#"<div class="title">{}</div>"#, escape(SERVER_NAME)) } else { String::new() };
        let speaker = format!(
            r#"<button type="button" class="{}" title="{}" aria-label="{}" {}>{}</button>"#,
            if self.sound { "on" } else { "" },
            self.text(Key::Sound),
            self.text(Key::Sound),
            pressed(crate::press::general::Press::Sound),
            speaker(self.sound)
        );
        let hint = if self.music_held { format!(r#"<span class="hint">{}</span>"#, self.text(Key::SoundHint)) } else { String::new() };
        // In the game the corner is the minimap's, as in the client: leaving is in the window of game options
        // (Escape), the sound and the language in Game Settings.
        if matches!(self.screen, Screen::Game) {
            return format!("{window}{}", self.overlay(view));
        }
        format!(r#"{title}<div class="lang">{hint}{speaker}{languages}</div>{window}{}"#, self.overlay(view))
    }

    /// The box over everything else: a message to confirm, or the wait for a server.
    fn overlay(&self, view: (f64, f64)) -> String {
        let place = (((view.0 - 640.0) / 2.0 + 185.0).floor(), ((view.1 - 480.0) / 2.0 + 180.0).floor());
        let inner = if let Some(message) = &self.message {
            format!(
                r#"<p class="a" style="{}">{}</p>{}"#,
                at(20, 40, 240, 52),
                escape(message),
                button(&at(235, 96, 42, 20), &states_ab("btn_ok"), &format!("{} autofocus", pressed(crate::press::general::Press::Ok)), "")
            )
        } else if let Some(ask) = self.ask {
            let name = |char_id: u32| self.characters
                .iter()
                .find(|character| character.char_id == char_id)
                .map(|character| escape(&character.name))
                .unwrap_or_default();
            let (text, field) = match ask {
                Ask::Reserve(char_id) => (crate::i18n::fill(self.text(Key::AskDelete), &[&name(char_id)]), String::new()),
                Ask::Delete(char_id) => (crate::i18n::fill(self.text(Key::AskDeleteNow), &[&name(char_id)]), String::new()),
                // The client's message 1489, with what the entry that has the key is called.
                Ask::Rebind(_, (table, place), _) => (
                    escape(
                        &data::message(1489)
                            .unwrap_or_else(|| "This key is already registered to [[%s]]. Bind it to this function instead?".to_owned())
                            .replace("%s", &data::hotkeys(table).get(place).map(|(name, _)| name.clone()).unwrap_or_default())
                    ),
                    String::new(),
                ),
                Ask::Birthdate(_) => (
                    self.text(Key::AskBirthdate).to_owned(),
                    format!(
                        r#"<input class="a in boxed" id="birthdate" inputmode="numeric" maxlength="6" autocomplete="off" autofocus style="{}">"#,
                        at(20, 70, 240, 18)
                    ),
                ),
            };
            let focus = if field.is_empty() { "autofocus" } else { "" };
            format!(
                r#"<p class="a tight" style="{}">{text}</p>{field}{}{}"#,
                at(14, 24, 252, 66),
                button(&at(189, 96, 42, 20), &states_ab("btn_ok"), &format!("{} {focus}", pressed(crate::press::account::Press::Yes)), ""),
                button(&at(235, 96, 42, 20), &states_ab("btn_cancel"), &pressed(crate::press::account::Press::No), "")
            )
        } else if let Some(wait) = self.wait {
            format!(
                r#"<p class="a" style="{}">{}</p>{}"#,
                at(10, 22, 260, 60),
                self.text(wait),
                button(&at(119, 95, 42, 20), &states_ab("btn_cancel"), &pressed(crate::press::account::Press::CancelWait), "")
            )
        } else {
            return String::new();
        };
        format!(r#"<div class="veil"></div>{}"#, window(view, (280, 120), place, "top", "win_msgbox.bmp", &inner))
    }
}
