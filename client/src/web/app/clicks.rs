//! What each thing on the page does when pressed (its `data-click`).
//!
//! Each kind of thing has its own file here, named as its packets are in `incoming/` and its window in `hud/`.

use super::*;

mod account;
mod choosing;
mod talk;
mod status;
mod inventory;
mod skills;
mod chat;
mod shops;
mod storage;
mod pets;
mod homunculus;
mod mercenary;
mod quests;
mod hotkeys;
mod settings;
mod guide;
mod party;
mod guild;
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
mod emotions;
mod tips;
mod worldmap;
mod books;

impl App {
    /// Something on the page was pressed: the press its name and what went with it stand for (`crate::press`).
    pub(super) fn click(&mut self, what: &str, arg: &str) {
        // A name no press has does nothing: nothing on the page is drawn with one.
        if let Some(press) = crate::press::Press::parse(what, arg) {
            self.pressed(press);
        }
    }

    /// A press, handed to its system: from the page, or from the app itself doing what a button does.
    pub(in crate::web) fn pressed(&mut self, press: impl Into<crate::press::Press>) {
        use crate::press::Press;
        match press.into() {
            Press::General(press) => self.general_pressed(press),
            Press::Account(press) => self.account_pressed(press),
            Press::Talk(press) => self.talk_pressed(press),
            Press::Status(press) => self.status_pressed(press),
            Press::Inventory(press) => self.item_pressed(press),
            Press::Skills(press) => self.skill_pressed(press),
            Press::Chat(press) => self.chat_pressed(press),
            Press::Shops(press) => self.trade_pressed(press),
            Press::Storage(press) => self.storage_pressed(press),
            Press::Pets(press) => self.pet_pressed(press),
            Press::Homunculus(press) => self.homunculus_pressed(press),
            Press::Mercenary(press) => self.mercenary_pressed(press),
            Press::Quests(press) => self.quest_pressed(press),
            Press::Hotkeys(press) => self.key_pressed(press),
            Press::Settings(press) => self.settings_pressed(press),
            Press::Guide(press) => self.guide_pressed(press),
            Press::Party(press) => self.party_pressed(press),
            Press::Guild(press) => self.guild_pressed(press),
            Press::PlayerMenu(press) => self.player_menu_pressed(press),
            Press::Exchange(press) => self.exchange_pressed(press),
            Press::Mail(press) => self.mail_pressed(press),
            Press::Cart(press) => self.cart_pressed(press),
            Press::Vending(press) => self.vending_pressed(press),
            Press::Friends(press) => self.friend_pressed(press),
            Press::Whispers(press) => self.whisper_pressed(press),
            Press::ViewEquip(press) => self.viewequip_pressed(press),
            Press::Unread(press) => self.unread_pressed(press),
            Press::Scripted(press) => self.scripted_pressed(press),
            Press::ChatRooms(press) => self.chat_room_pressed(press),
            Press::Family(press) => self.family_pressed(press),
            Press::Emotions(press) => self.emotion_pressed(press),
            Press::WorldMap(press) => self.worldmap_pressed(press),
            Press::Tips(press) => self.tip_pressed(press),
            Press::Choosing(press) => self.choosing_pressed(press),
            Press::Fight(press) => self.fight_pressed(press),
            Press::Books(press) => self.book_pressed(press),
        }
    }

    /// A press that belongs to no one window (`crate::press::general`).
    fn general_pressed(&mut self, press: crate::press::general::Press) {
        use crate::press::Step;
        use crate::press::general::Press;
        match press {
            Press::Language(lang) => {
                self.lang = lang;
                crate::i18n::speak(lang);
                store("roweb.lang", Some(lang.code()));
                // The items and skills are asked for again, in the new language.
                data::forget_texts();
                let known: Vec<u32> = self.bag.items.iter().map(|item| item.id).chain(self.skills.hotkeys.iter().filter(|hotkey| !hotkey.skill).map(|hotkey| hotkey.id)).chain(self.bag.info.map(|(id, _)| id)).collect();
                data::want_items(known.into_iter(), || dispatch(Action::ItemsKnown));
                if let Some(skill) = self.skills.info {
                    data::want_skill(skill, || dispatch(Action::ItemsKnown));
                }
                self.render();
            }
            Press::Ok => {
                self.message = None;
                self.render();
            }
            // The minimap's plus and minus (the client's `UIMinimapZoomWnd`): nearer by a third of itself and
            // a quarter of that, back by a quarter, between whole and four times.
            Press::MapZoom(step) => {
                let zoom = self.minimap_zoom.max(1.0);
                self.minimap_zoom = if step == Step::More { (zoom + (zoom / 3.0 + zoom) * 0.25).min(4.0) } else { (zoom - zoom * 0.25).max(1.0) };
                self.render();
            }
            // The minimap's first button shows and hides the marks of who stands where in a town (the client's
            // `minimapMarkersVisible`).
            Press::MapMarks => {
                self.minimap_marks_off = !self.minimap_marks_off;
                self.render();
            }
            Press::Menu => {
                self.menu_closed = !self.menu_closed;
                self.render();
            }
            // The menu's icon (`UIMenuIconWnd`, `ID_OPTION`): the game's menu opens, or closes if it was open
            // (`ToggleWindow(WID_ESCOPTIONWND)`), as Escape with no window open does.
            Press::Options => {
                self.esc = u8::from(self.esc == 0);
                self.redraw(&[&wnd::DIALOG]);
            }
            Press::EscClose | Press::EscSave | Press::EscNo | Press::EscYes => {
                self.esc = match press {
                    Press::EscSave => 2,
                    Press::EscNo => 1,
                    _ => 0,
                };
                if press == Press::EscYes {
                    self.send(&npc::restart(false));
                }
                self.redraw(&[&wnd::DIALOG]);
            }
            Press::Sound => {
                self.sound = !self.sound;
                store("roweb.sound", Some(if self.sound { "on" } else { "off" }));
                self.tune();
                self.music_held = false;
                self.render();
                check_music_soon();
            }
        }
    }
}
