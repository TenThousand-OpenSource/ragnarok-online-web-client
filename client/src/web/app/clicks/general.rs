//! What belongs to no one window: the language, the sound, the minimap's buttons, the game's menu.

use super::*;

impl App {
    /// A press that belongs to no one window (`crate::press::general`).
    pub(super) fn general_pressed(&mut self, press: crate::press::general::Press) {
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
                self.minimap_zoom = crate::minimap::zoomed(self.minimap_zoom, step == Step::More);
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
                    self.send(&crate::net::packets::restart(false));
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
