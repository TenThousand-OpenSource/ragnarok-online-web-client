//! The windows, as HTML laid over the game's own pictures.
//!
//! Sizes, places and colours are those of the original client (Ragexe 2026-02-19), read from its decompilation
//! (rodecomp: `UIWindowMgr.cpp`, `UINewSelectCharWnd.cpp`, `UINewMakeCharWnd.cpp`, `UIMessageBox.cpp`) and checked
//! against rustnarok-client. The registration and password windows have no original: the client has none.
//!
//! Text fields are real input elements, so phones' keyboards, paste and password managers work.

use super::app::{App, Ask, Game, Screen, epoch, escape};
use super::{data, figure};
use crate::i18n::{Key, Lang};
use crate::net::packets::Character;

/// The name shown over the login picture.
pub const SERVER_NAME: &str = "Ragnarok 2026";

const UI: &str = "data/texture/유저인터페이스/";
/// The login picture of this client, stretched over the whole view as the original does.
pub const WALLPAPER: &str = "data/texture/유저인터페이스/t_login.jpg";

pub const SLOTS_PER_PAGE: u8 = 15;
/// How many hair styles a new character can choose from, and how many hair colours.
pub fn hair_styles(doram: bool) -> u16 {
    if doram { 6 } else { 23 }
}
pub const HAIR_COLOURS: u16 = 9;

/// The speaker of the sound switch, with waves when sound is on and crossed out when it is off.
const SPEAKER_ON: &str = r#"<svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><path fill="currentColor" d="M2 6h3l4-3v10L5 10H2z"/><path fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" d="M11 5.5a3.5 3.5 0 0 1 0 5M12.8 3.6a6 6 0 0 1 0 8.8"/></svg>"#;
const SPEAKER_OFF: &str = r#"<svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><path fill="currentColor" d="M2 6h3l4-3v10L5 10H2z"/><path fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" d="M11.2 6l3.6 4M14.8 6l-3.6 4"/></svg>"#;

/// The URL of one of the interface's pictures, as a PNG.
fn picture(name: &str) -> String {
    format!("/image/{}", String::from(js_sys::encode_uri(&format!("{UI}{name}"))))
}

/// One of the eight pictures of the frame around a chosen character.
pub(super) fn chosen_frame(frame: u8) -> String {
    picture(&format!("select_character_ver3/img_slot_select{frame}.bmp"))
}

fn at(x: i32, y: i32, width: i32, height: i32) -> String {
    format!("left:{x}px;top:{y}px;width:{width}px;height:{height}px")
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

fn click(what: &str, arg: impl std::fmt::Display) -> String {
    format!(r#"data-click="{what}" data-arg="{arg}""#)
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
            .map(|lang| format!(r#"<button type="button" class="{}" {}>{}</button>"#, if lang == self.lang { "on" } else { "" }, click("lang", lang.code()), lang.code().to_uppercase()))
            .collect();
        // The server's name stands over the login picture; the larger windows cover that place.
        let titled = matches!(self.screen, Screen::Login | Screen::ServerSelect | Screen::Register | Screen::Recover | Screen::Reset(_));
        let title = if titled { format!(r#"<div class="title">{}</div>"#, escape(SERVER_NAME)) } else { String::new() };
        let speaker = format!(
            r#"<button type="button" class="{}" title="{}" aria-label="{}" {}>{}</button>"#,
            if self.sound { "on" } else { "" },
            self.text(Key::Sound),
            self.text(Key::Sound),
            click("sound", ""),
            if self.sound { SPEAKER_ON } else { SPEAKER_OFF }
        );
        let hint = if self.music_held { format!(r#"<span class="hint">{}</span>"#, self.text(Key::SoundHint)) } else { String::new() };
        // Leaving the game: the original has this in its Esc menu, which is not made yet.
        let leave = if matches!(self.screen, Screen::Game) { format!(r#"<button type="button" {}>{}</button>"#, click("logout", ""), self.text(Key::LogOut)) } else { String::new() };
        // In the game the corner is the minimap's, as in the client: leaving is in the window of game options
        // (Escape), the sound and the language in Game Settings.
        if matches!(self.screen, Screen::Game) {
            return format!("{window}{}", self.overlay(view));
        }
        format!(r#"{title}<div class="lang">{hint}{leave}{speaker}{languages}</div>{window}{}"#, self.overlay(view))
    }

    /// The box over everything else: a message to confirm, or the wait for a server.
    fn overlay(&self, view: (f64, f64)) -> String {
        let place = (((view.0 - 640.0) / 2.0 + 185.0).floor(), ((view.1 - 480.0) / 2.0 + 180.0).floor());
        let inner = if let Some(message) = &self.message {
            format!(
                r#"<p class="a" style="{}">{}</p>{}"#,
                at(20, 40, 240, 52),
                escape(message),
                button(&at(235, 96, 42, 20), &states_ab("btn_ok"), &format!("{} autofocus", click("ok", "")), "")
            )
        } else if let Some(ask) = self.ask {
            let name = |char_id: u32| self.characters.iter().find(|character| character.char_id == char_id).map(|character| escape(&character.name)).unwrap_or_default();
            let (text, field) = match ask {
                Ask::Reserve(char_id) => (crate::i18n::fill(self.text(Key::AskDelete), &[&name(char_id)]), String::new()),
                Ask::Delete(char_id) => (crate::i18n::fill(self.text(Key::AskDeleteNow), &[&name(char_id)]), String::new()),
                Ask::Birthdate(_) => (
                    self.text(Key::AskBirthdate).to_owned(),
                    format!(r#"<input class="a in boxed" id="birthdate" inputmode="numeric" maxlength="6" autocomplete="off" autofocus style="{}">"#, at(20, 70, 240, 18)),
                ),
            };
            let focus = if field.is_empty() { "autofocus" } else { "" };
            format!(
                r#"<p class="a tight" style="{}">{text}</p>{field}{}{}"#,
                at(14, 24, 252, 66),
                button(&at(189, 96, 42, 20), &states_ab("btn_ok"), &format!("{} {focus}", click("yes", "")), ""),
                button(&at(235, 96, 42, 20), &states_ab("btn_cancel"), &click("no", ""), "")
            )
        } else if let Some(wait) = self.wait {
            format!(
                r#"<p class="a" style="{}">{}</p>{}"#,
                at(10, 22, 260, 60),
                self.text(wait),
                button(&at(119, 95, 42, 20), &states_ab("btn_cancel"), &click("cancel-wait", ""), "")
            )
        } else {
            return String::new();
        };
        format!(r#"<div class="veil"></div>{}"#, window(view, (280, 120), place, "top", "win_msgbox.bmp", &inner))
    }

    fn login_window(&self, view: (f64, f64)) -> String {
        let field = |id: &str, y: i32, extra: &str| format!(r#"<input class="a in centre" id="{id}" maxlength="23" style="{}" {extra}>"#, at(15, y, 153, 16));
        let checkbox = if self.save_id { "login_interface/checkbox_on.bmp" } else { "login_interface/checkbox_off.bmp" };
        let inner = format!(
            r#"<form data-submit="login">
                <div class="a combo" style="{combo}">{service}</div>
                {id}{password}
                <button type="button" class="a check" style="{check};{check_picture}" {save}></button>
                <div class="a small" style="{save_label};color:#333">{save_text}</div>
                {restore}{register}
                <button class="a b" style="{login};{login_pictures}" aria-label="Login"></button>
            </form>"#,
            combo = at(13, 10, 160, 22),
            service = self.text(Key::Service),
            id = field("user", 40, &format!(r#"name="username" autocomplete="username" autofocus value="{}""#, escape(&self.saved_id))),
            password = field("password", 64, &format!(r#"name="password" type="password" autocomplete="current-password" placeholder="{}""#, self.text(Key::Password))),
            check = at(56, 85, 12, 12),
            check_picture = background(checkbox),
            save = click("save-id", ""),
            save_label = at(73, 85, 110, 14),
            save_text = self.text(Key::SaveId),
            restore = button(&at(13, 104, 67, 21), &states_of("login_interface/bt_close"), &click("to-recover", ""), self.text(Key::Restore)),
            register = button(&at(87, 104, 84, 21), &states_of("login_interface/bt_join"), &click("to-register", ""), self.text(Key::Register)),
            login = at(192, 32, 84, 84),
            login_pictures = states_of("login_interface/bt_start"),
        );
        window(view, (301, 132), login_place(view), "", "login_interface/bg_login.tga", &inner)
    }

    /// A window for a form the original client does not have, in the look of its newer windows.
    fn form(&self, view: (f64, f64), height: i32, submit: &str, title: Key, rows: &str) -> String {
        let buttons = format!(
            r#"<button class="a b" style="{};{pictures}">{}</button>{}"#,
            at(12, height - 36, 131, 24),
            self.text(Key::Ok),
            button(&at(158, height - 36, 131, 24), &states_of("select_character_ver3/bt_info"), &click("to-login", ""), self.text(Key::Cancel)),
            pictures = states_of("select_character_ver3/bt_info"),
        );
        let inner = format!(r#"<form data-submit="{submit}"><h1>{}</h1><div class="rows">{rows}</div>{buttons}</form>"#, self.text(title));
        let place = login_place(view);
        // Taller than the login window: moved up so it ends where that one ends.
        window(view, (301, height), (place.0, place.1 + 132.0 - f64::from(height)), "panel", "", &inner)
    }

    fn row(&self, label: Key, input: &str) -> String {
        format!(r#"<label><span>{}</span>{input}</label>"#, self.text(label))
    }

    fn register_window(&self, view: (f64, f64)) -> String {
        let rows = [
            self.row(Key::Id, r#"<input id="reg-id" maxlength="23" autocomplete="username" autofocus>"#),
            self.row(Key::PasswordLabel, r#"<input id="reg-password" type="password" maxlength="23" autocomplete="new-password">"#),
            self.row(Key::RepeatPassword, r#"<input id="reg-repeat" type="password" maxlength="23" autocomplete="new-password">"#),
            self.row(Key::Email, r#"<input id="reg-email" type="email" maxlength="39" autocomplete="email">"#),
            self.row(Key::Sex, &format!(r#"<select id="reg-sex"><option value="M">{}</option><option value="F">{}</option></select>"#, self.text(Key::Male), self.text(Key::Female))),
        ];
        self.form(view, 214, "register", Key::RegisterTitle, &rows.concat())
    }

    fn recover_window(&self, view: (f64, f64)) -> String {
        let rows = [
            format!(r#"<p>{}</p>"#, self.text(Key::RestoreHint)),
            self.row(Key::Id, r#"<input id="rec-id" maxlength="23" autocomplete="username" autofocus>"#),
            self.row(Key::Email, r#"<input id="rec-email" type="email" maxlength="39" autocomplete="email">"#),
        ];
        self.form(view, 196, "recover", Key::RestoreTitle, &rows.concat())
    }

    fn reset_window(&self, view: (f64, f64)) -> String {
        let rows = [
            self.row(Key::PasswordLabel, r#"<input id="new-password" type="password" maxlength="23" autocomplete="new-password" autofocus>"#),
            self.row(Key::RepeatPassword, r#"<input id="new-repeat" type="password" maxlength="23" autocomplete="new-password">"#),
        ];
        self.form(view, 136, "reset", Key::ResetTitle, &rows.concat())
    }

    pub(super) fn character_in(&self, slot: u8) -> Option<&Character> {
        self.characters.iter().find(|character| character.slot == slot)
    }

    /// The servers to play on, each with how many are playing there now: the original's window (280 by 200, its
    /// list 12 in and 22 down, OK and Cancel in the corner).
    fn server_select_window(&self, view: (f64, f64)) -> String {
        let servers = self.account.as_ref().map(|account| &account.char_servers[..]).unwrap_or_default();
        let lines: String = servers
            .iter()
            .enumerate()
            .map(|(index, server)| {
                let picked = if index == self.server_picked { "background:#c6d6f7;" } else { "" };
                format!(
                    r#"<div style="{picked}height:16px;line-height:16px;padding:0 4px;cursor:default;white-space:nowrap" {} data-open-server="{index}">{} ({})</div>"#,
                    click("server", index),
                    escape(&server.name),
                    server.users
                )
            })
            .collect();
        let inner = format!(
            r#"{}<div class="a" style="{};background:#fff;border:1px solid #c3c3c3;border-top:0;box-sizing:border-box"></div>
               <div class="a" style="{};background:#f0f0f0;overflow-y:auto;font:12px Tahoma,Arial,sans-serif;color:#000">{lines}</div>{}{}"#,
            super::hud::title_bar(280, self.text(Key::SelectServer)),
            at(0, 17, 280, 183),
            at(12, 22, 256, 145),
            button(&at(189, 176, 42, 20), &states_ab("btn_ok"), &format!("{} autofocus", click("server-ok", "")), ""),
            button(&at(234, 176, 42, 20), &states_ab("btn_cancel"), &click("server-cancel", ""), ""),
        );
        let place = (((view.0 - 640.0) / 2.0 + 185.0).floor(), (view.1 * 300.0 / 480.0 - 80.0).floor());
        window(view, (280, 200), place, "", "", &inner)
    }

    fn char_select_window(&self, view: (f64, f64)) -> String {
        let mut inner = String::new();
        for slot in 0..SLOTS_PER_PAGE {
            let (x, y) = (i32::from(slot % 5) * 157 + 10, i32::from(slot / 5) * 195 + 21);
            let selected = slot == self.selected;
            let character = self.character_in(slot);
            if slot >= self.slots && character.is_none() {
                inner.push_str(&format!(r#"<div class="a" style="{};{}"></div>"#, at(x, y, 157, 195), background("select_character_ver3/img_slot_lock.bmp")));
                continue;
            }
            let base = if character.is_some() { "select_character_ver3/img_slot" } else { "select_character_ver3/img_slot2" };
            inner.push_str(&button(&at(x, y, 157, 195), &states_of(base), &format!(r#"{} data-open="{slot}""#, click("slot", slot)), ""));
            match (character, selected) {
                // The frame around a chosen character runs through eight pictures.
                (Some(_), true) => {
                    // All eight lie on one another and show in turn: a picture swapped in its place would blink
                    // while the browser makes it ready.
                    let frames: String = (0..8).map(|frame| format!(r#"<i style="background-image:url('{}')"></i>"#, chosen_frame(frame))).collect();
                    inner.push_str(&format!(r#"<div class="a chosen" style="{}">{frames}</div>"#, at(x - 2, y - 1, 161, 197)));
                }
                (None, true) => inner.push_str(&format!(r#"<div class="a pass" style="{};{}"></div>"#, at(x, y, 157, 195), background("select_character_ver3/img_slot2_select.bmp"))),
                _ => {}
            }
            if let Some(character) = character {
                inner.push_str(&format!(
                    r#"<canvas class="a pass" id="pc{slot}" width="{}" height="{}" style="left:{x}px;top:{y}px"></canvas>
                       <img class="a pass" style="left:{}px;top:{}px" src="{}" alt="" onerror="this.remove()">
                       <div class="a pass name" style="{}">{}</div>"#,
                    figure::WIDTH,
                    figure::HEIGHT,
                    x + 118,
                    y + 14,
                    picture(&format!("renewalparty/icon_jobs_{}.bmp", character.job)),
                    at(x, y + 167, 157, 14),
                    escape(&character.name),
                ));
                // When a character waiting to be deleted can be: red until then, blue from then on.
                if let Some(&when) = self.deletions.get(&character.char_id) {
                    let (colour, lines) = if when <= epoch() {
                        ("#00f", [self.text(Key::DeletionReady).to_owned(), String::new()])
                    } else {
                        let date = js_sys::Date::new(&(when * 1000.0).into());
                        ("#f00", [format!("{}/{}", date.get_month() + 1, date.get_date()), format!("{:02}:{:02}", date.get_hours(), date.get_minutes())])
                    };
                    for (line, text) in lines.iter().enumerate() {
                        inner.push_str(&format!(r#"<div class="a pass small centre" style="{};color:{colour}">{text}</div>"#, at(x + 20, y + 19 + 12 * line as i32, 96, 12)));
                    }
                }
            }
        }

        // The panel on the right: the chosen character's details, and what can be done with it.
        let chosen = self.character_in(self.selected);
        inner.push_str(&format!(r#"<div class="a" style="{};{}"></div>"#, at(809, 26, 173, 575), background("select_character_ver3/img_info.bmp")));
        if let Some(character) = chosen {
            let mut lines = vec![(116, character.map.clone()), (133, data::job_name(character.job)), (150, character.base_level.to_string()), (167, character.base_exp.to_string())];
            lines.push((184, character.hp.to_string()));
            lines.push((201, character.sp.to_string()));
            for (index, stat) in character.stats.iter().enumerate() {
                lines.push((218 + 17 * index as i32, stat.to_string()));
            }
            for (y, value) in lines {
                inner.push_str(&format!(r#"<div class="a small value" style="{}">{}</div>"#, at(868, y, 107, 15), escape(&value)));
            }
        }
        let info = states_of("select_character_ver3/bt_info");
        // The two buttons under the details change with the chosen character: one not being deleted can be, one
        // waiting can be kept after all, and one whose wait is over can be deleted for good.
        let deletion = chosen.and_then(|character| self.deletions.get(&character.char_id)).map(|&when| when <= epoch());
        let (first, second) = match (chosen, deletion) {
            (None, _) => (("disabled".to_owned(), Key::DeleteCharacter), ("disabled".to_owned(), Key::Notice)),
            (Some(_), None) => ((click("delete", ""), Key::DeleteCharacter), ("disabled".to_owned(), Key::Notice)),
            (Some(_), Some(false)) => ((click("cancel-delete", ""), Key::CancelDeletion), ("disabled".to_owned(), Key::Notice)),
            (Some(_), Some(true)) => ((click("cancel-delete", ""), Key::CancelDeletion), (click("delete-now", ""), Key::Delete)),
        };
        inner.push_str(&button(&at(830, 322, 131, 24), &info, &first.0, self.text(first.1)));
        inner.push_str(&button(&at(830, 351, 131, 24), &info, &second.0, self.text(second.1)));
        let chosen = chosen.filter(|_| deletion.is_none());
        let start = if chosen.is_some() {
            states("select_character_ver3/bt_gamestart_on.bmp", "select_character_ver3/bt_gamestart_over.bmp", "select_character_ver3/bt_gamestart_press.bmp")
        } else {
            let off = "select_character_ver3/bt_gamestart_off.bmp";
            states(off, off, off)
        };
        inner.push_str(&format!(
            r#"<button type="button" class="a b start" style="{};{start}" {} {}>{}</button>"#,
            at(816, 407, 165, 110),
            click("start", ""),
            if chosen.is_some() { "" } else { "disabled" },
            self.text(Key::GameStart)
        ));
        // One page: both arrows are off.
        for (x, side) in [(830, "left"), (939, "right")] {
            inner.push_str(&format!(r#"<div class="a" style="{};{}"></div>"#, at(x, 558, 21, 28), background(&format!("select_character_ver3/bt_{side}_off.bmp"))));
        }
        inner.push_str(&format!(r#"<div class="a list-title" style="{}">{}</div>"#, at(830, 538, 131, 14), self.text(Key::CharacterList)));
        inner.push_str(&format!(r#"<div class="a page" style="{}">1 / 1</div>"#, at(851, 560, 88, 22)));
        inner.push_str(&button(&at(945, 5, 17, 18), &states_of("select_character_ver3/bt_close2"), &click("logout", ""), ""));

        let size = (997, 626);
        window(view, size, centred(view, size), "", "select_character_ver3/bg_back1.tga", &inner)
    }

    fn char_create_window(&self, view: (f64, f64)) -> String {
        let make = &self.make;
        let folder = "make_character_ver2";
        let mut inner = format!(r#"<div class="a" style="{};color:#5f5f5f">{}</div>"#, at(44, 17, 300, 14), self.text(Key::CharacterCreation));

        // The two races: the chosen one is lit.
        for (y, race, doram, name, about) in [(36, "human", false, Key::Human, Key::HumanAbout), (202, "doram", true, Key::Doram, Key::DoramAbout)] {
            let lit = if doram == make.doram { "on" } else { "off" };
            inner.push_str(&format!(
                r#"<button type="button" class="a race" style="{};{}" {}><b>{}</b><p>{}</p></button>"#,
                at(21, y, 364, 166),
                background(&format!("{folder}/img_{race}_{lit}.bmp")),
                click("race", race),
                self.text(name),
                self.text(about)
            ));
        }

        for (x, sex, name) in [(437, 1u8, "male"), (500, 0u8, "female")] {
            let base = format!("{folder}/bt_{name}");
            let pictures = if make.sex == sex {
                let on = format!("{base}_on.bmp");
                states(&on, &on, &on)
            } else {
                states(&format!("{base}_off.bmp"), &format!("{base}_over.bmp"), &format!("{base}_press.bmp"))
            };
            inner.push_str(&button(&at(x, 59, 62, 25), &pictures, &click("sex", sex), ""));
        }
        inner.push_str(&button(&at(426, 179, 22, 23), &states_of(&format!("{folder}/bt_leftturn")), &click("turn", -1), ""));
        inner.push_str(&button(&at(551, 179, 22, 23), &states_of(&format!("{folder}/bt_rightturn")), &click("turn", 1), ""));
        // The new character, feet at (500, 228).
        inner.push_str(&format!(r#"<canvas class="a pass" id="pc-make" width="{}" height="{}" style="left:420px;top:91px"></canvas>"#, figure::WIDTH, figure::HEIGHT));
        inner.push_str(&format!(r#"<input class="a in" id="char-name" maxlength="23" autocomplete="off" autofocus style="{}">"#, at(440, 268, 120, 20)));

        inner.push_str(&format!(r#"<div class="a small" style="{};color:#404040">{}</div>"#, at(609, 44, 150, 14), self.text(Key::HairStyle)));
        let thumbnails = match (make.doram, make.sex == 1) {
            (false, true) => "",
            (false, false) => "_girl",
            (true, true) => "_doramboy",
            (true, false) => "_doramgirl",
        };
        for index in 0..24u16 {
            let place = at(i32::from(index % 4) * 38 + 606, i32::from(index / 4) * 37 + 59, 36, 37);
            let style = index + 1;
            if style > hair_styles(make.doram) {
                inner.push_str(&format!(r#"<div class="a" style="{place};{}"></div>"#, background(&format!("{folder}/img_hairstyle_none.bmp"))));
                continue;
            }
            let frame = if style == make.hair_style {
                let select = format!("{folder}/bt_hairstyle_select.bmp");
                states(&select, &select, &select)
            } else {
                states_of(&format!("{folder}/bt_hairstyle"))
            };
            let thumbnail = format!(r#"<img src="{}" alt="">"#, picture(&format!("{folder}/img_hairstyle{thumbnails}{style:02}.bmp")));
            inner.push_str(&button(&place, &frame, &click("hair", style), &thumbnail));
        }

        inner.push_str(&format!(r#"<div class="a small" style="{};color:#404040">{}</div>"#, at(609, 285, 150, 14), self.text(Key::HairColor)));
        for colour in 0..HAIR_COLOURS {
            let base = format!("{folder}/color{:02}", colour + 1);
            let pictures = if colour == make.hair_colour {
                let on = format!("{base}_on.bmp");
                states(&on, &on, &on)
            } else {
                states(&format!("{base}_off.bmp"), &format!("{base}_over.bmp"), &format!("{base}_press.bmp"))
            };
            inner.push_str(&button(&at(i32::from(colour % 5) * 27 + 619, i32::from(colour / 5) * 24 + 306, 22, 22), &pictures, &click("colour", colour), ""));
        }

        let make_pictures = states_of(&format!("{folder}/bt_make"));
        inner.push_str(&button(&at(25, 379, 152, 27), &make_pictures, &format!(r#"{} data-big"#, click("back", "")), self.text(Key::Return)));
        inner.push_str(&format!(r#"<button class="a b" data-big style="{};{make_pictures}">{}</button>"#, at(616, 379, 152, 27), self.text(Key::Create)));
        inner.push_str(&button(&at(768, 9, 17, 16), &states_of(&format!("{folder}/bt_close")), &click("back", ""), ""));

        let size = (794, 422);
        let place = centred(view, size);
        let inner = format!(r#"<form data-submit="create">{inner}</form>"#);
        window(view, size, (place.0, place.1 - 70.0), "", &format!("{folder}/bg_makebg.bmp"), &inner)
    }

    /// In the game: the game's own windows over the world, and the minimap in its corner with the character's
    /// place under it.
    fn game_window(&self, view: (f64, f64)) -> String {
        let Some(game) = &self.game else { return String::new() };
        // A map name goes into a URL: only what map names are made of passes.
        let map: String = game.map.chars().filter(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '@')).collect();
        // Under the minimap, as the original has it: the character's cell, and the minimap's buttons (which do
        // nothing yet).
        // Plus and minus bring the map nearer and take it back (the client's `UIMinimapZoomWnd`); the others
        // (the marks, the world map) do nothing yet.
        let buttons: String = [("object", 1, ""), ("plus", 73, "+"), ("minus", 87, "-"), ("mini", 101, ""), ("viewon", 115, "")]
            .iter()
            .map(|(name, x, zoom)| {
                let press = if zoom.is_empty() { String::new() } else { format!(r#" data-click="map-zoom" data-arg="{zoom}""#) };
                format!(
                    r#"<img src="{}" alt="" draggable="false" style="position:absolute;left:{x}px;top:0;{}" onerror="this.remove()"{press}>"#,
                    picture(&format!("minimap/i_{name}_1.bmp")),
                    if zoom.is_empty() { "" } else { "pointer-events:auto;cursor:pointer" }
                )
            })
            .collect();
        // The buttons under the minimap show when the player has them so (the client's `/minimap`).
        let buttons = if self.option("TT_SHOW_MINIMAP_BUTTON_ONOFF") { buttons } else { String::new() };
        let (pictured, marked) = minimap_place(game, self.minimap_zoom).unwrap_or_else(|| (String::new(), "display:none".to_owned()));
        format!(
            r#"<div class="minimap" style="overflow:hidden"><img id="minimap" src="{}" alt="" style="{pictured}" onerror="this.remove()"><i id="marker" style="{}"></i></div>
               <div class="mapname"><span id="position"> {}  {}</span>{buttons}</div>{}"#,
            picture(&format!("map/{map}.bmp")),
            marked,
            game.x,
            game.y,
            self.hud(view),
        )
    }
}

/// How the minimap's picture and the character's mark on it are placed, as styles. The minimap is a square
/// picture with the map in its middle, north up; nearer than whole (the client's zoom, 1 to 4) it shows the
/// part around the character, as much of it as fits, never past the picture's edge (`CGameMode::DrawMiniMap`).
pub(super) fn minimap_place(game: &Game, zoom: f32) -> Option<(String, String)> {
    let (width, height) = game.map_size?;
    let zoom = zoom.clamp(1.0, 4.0);
    let scale = 100.0 / width.max(height) as f32;
    let (x, y) = (50.0 + (f32::from(game.x) - width as f32 / 2.0) * scale, 50.0 - (f32::from(game.y) - height as f32 / 2.0) * scale);
    // What is shown is a square of the picture, this many hundredths of it a side, from here.
    let shown = 100.0 / zoom;
    let from = |at: f32| (at - shown / 2.0).clamp(0.0, 100.0 - shown);
    let (left, top) = (from(x), from(y));
    Some((
        format!("position:absolute;width:{:.2}%;height:{:.2}%;left:{:.2}%;top:{:.2}%", zoom * 100.0, zoom * 100.0, -left * zoom, -top * zoom),
        format!("left:{:.2}%;top:{:.2}%", (x - left) * zoom, (y - top) * zoom),
    ))
}
