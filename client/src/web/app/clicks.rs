//! What each thing on the page does when pressed (its `data-click`).
//!
//! Each kind of thing has its own file here, named as its packets are in `incoming/` and its window in `hud/`.

use super::*;

mod account;
mod talk;
mod status;
mod inventory;
mod skills;
mod chat;
mod shops;
mod pets;

impl App {
    pub(super) fn click(&mut self, what: &str, arg: &str) {
        match what {
            "lang" => {
                if let Some(lang) = Lang::from_code(arg) {
                    self.lang = lang;
                    crate::i18n::speak(lang);
                    store("roweb.lang", Some(lang.code()));
                    // The items and skills are asked for again, in the new language.
                    data::forget_texts();
                    let known: Vec<u32> = self.bag.items.iter().map(|item| item.id).chain(self.skills.hotkeys.iter().filter(|hotkey| !hotkey.skill).map(|hotkey| hotkey.id)).chain(self.bag.info).collect();
                    data::want_items(known.into_iter(), || dispatch(Action::ItemsKnown));
                    if let Some(skill) = self.skills.info {
                        data::want_skill(skill, || dispatch(Action::ItemsKnown));
                    }
                    self.render();
                }
            }
            "ok" => {
                self.message = None;
                self.render();
            }
            "menu" => {
                self.menu_closed = !self.menu_closed;
                self.render();
            }
            "esc-close" | "esc-save" | "esc-no" | "esc-yes" => {
                self.esc = match what {
                    "esc-save" => 2,
                    "esc-no" => 1,
                    _ => 0,
                };
                if what == "esc-yes" {
                    if let Some(socket) = &self.socket {
                        socket.send(&npc::restart(false));
                    }
                }
                self.refresh_dialog();
            }
            "sound" => {
                self.sound = !self.sound;
                store("roweb.sound", Some(if self.sound { "on" } else { "off" }));
                self.tune();
                self.music_held = false;
                self.render();
                check_music_soon();
            }
            _ => {
                let _ = self.account_click(what, arg)
                    || self.talk_click(what, arg)
                    || self.status_click(what, arg)
                    || self.item_click(what, arg)
                    || self.skill_click(what, arg)
                    || self.chat_click(what, arg)
                    || self.trade_click(what, arg)
                    || self.pet_click(what, arg);
            }
        }
    }
}
