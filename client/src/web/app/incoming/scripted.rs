//! What an NPC's script has the client show and play (`crate::net::scripted`): a picture over the screen, a
//! mark on the minimap, a bar that fills over someone, sounds, music, words over a head.

use super::*;
use crate::net::scripted::{self, Cutin};

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn scripted_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        let own = self.own_account();
        match id {
            // `MM_SHOW_IMAGE`: the picture of an NPC or of a scene, or the one shown goes.
            scripted::ZC_SHOW_IMAGE | scripted::ZC_SHOW_IMAGE2 => match scripted::cutin(id, packet) {
                Some(Cutin::Show(name, stand)) => self.cutin_shown(Some((name, stand))),
                Some(Cutin::Away) => self.cutin_shown(None),
                None => {}
            },
            scripted::ZC_COMPASS => self.compass_told(packet),
            // The bar over the player's own character (`ReceiveProgress`): when it is full the server is told.
            scripted::ZC_PROGRESS => {
                if let Some(lasts) = scripted::progress(packet) {
                    world::gauge(own, own, Some(f64::from(lasts)));
                    self.scripted.progress = Some(js_sys::Date::now() + f64::from(lasts));
                    after(lasts as i32 + 1, Action::Scripted);
                }
            }
            // The bar is taken away; the client answers this too while its bar was waited for
            // (`AM_DELETE_SKILL_RECHARGEGAGE`).
            scripted::ZC_PROGRESS_CANCEL => {
                world::gauge(own, own, None);
                if self.scripted.progress.take().is_some() {
                    self.send_on_map(&scripted::progress_done());
                }
            }
            // A bar over someone else, which nobody answers.
            scripted::ZC_PROGRESS_ACTOR => {
                if let Some((who, lasts)) = scripted::progress_of(packet) {
                    world::gauge(who, own, Some(f64::from(lasts)));
                }
            }
            scripted::ZC_SOUND => self.sound_told(packet),
            // `CGameMode::PlayBgm`: 0 plays the piece over and over, 1 once, 2 goes back to the map's music.
            scripted::ZC_PLAY_NPC_BGM => {
                if let Some((kind, name)) = scripted::music(packet) {
                    sound::piece((kind < 2).then_some((name, kind == 1)));
                }
            }
            // Words over a head, as what is said near hangs there, and not in the chat
            // (`ReceiveActorMessage7`: `AM_CHAT` alone).
            scripted::ZC_SHOWSCRIPT => {
                if let Some((who, words)) = scripted::shown_words(packet) {
                    world::said(who, own, &words);
                }
            }
            // `ReceiveEffectRemove`: an effect on someone ends.
            scripted::ZC_CLEAR_EFFECT => {
                if let Some((who, effect)) = scripted::cleared_effect(packet) {
                    world::effect_ends(effect, (who != own).then_some(who));
                }
            }
            _ => return false,
        }
        true
    }

    /// ZC_COMPASS (`ReceiveCompass`): a mark is put on the minimap under its number, or the one of that number
    /// taken away. One of kind 0 goes by itself after a while.
    fn compass_told(&mut self, packet: &[u8]) {
        let Some(told) = scripted::compass(packet) else { return };
        let visit = world::visit();
        if self.scripted.compass.0 != visit {
            self.scripted.compass = (visit, Default::default());
        }
        if told.kind == 2 {
            self.scripted.compass.1.remove(&told.id);
        } else {
            let until = (told.kind == 0).then(|| js_sys::Date::now() + scripted::COMPASS_SHOWN_FOR);
            self.scripted.compass.1.insert(told.id, Mark { cell: told.cell, colour: told.colour, until });
            if until.is_some() {
                after(scripted::COMPASS_SHOWN_FOR as i32 + 50, Action::Scripted);
            }
        }
        self.compass_marks();
    }

    /// ZC_SOUND (`CGameMode::ReceiveSound`): a sound once, from where its NPC stands; one that repeats, which
    /// plays at once from nowhere and then again after every wait (`clicks/scripted.rs`); or the end of the
    /// first one of that name that repeats. rAthena sends no wait: such a sound begins again as it ends
    /// (`sound::repeat`).
    fn sound_told(&mut self, packet: &[u8]) {
        let Some(told) = scripted::sound(packet) else { return };
        match told.act {
            0 => world::sounded(&told.name, told.who, self.own_account(), false),
            1 if told.term == 0 => sound::repeat(&told.name, true),
            1 => {
                sound::effect(&told.name, 1.0);
                self.scripted.repeated.push(Repeated { next: js_sys::Date::now() + f64::from(told.term), name: told.name, who: told.who, term: told.term });
                after(told.term as i32, Action::Scripted);
            }
            2 => {
                sound::repeat(&told.name, false);
                if let Some(place) = self.scripted.repeated.iter().position(|repeated| repeated.name == told.name) {
                    self.scripted.repeated.remove(place);
                }
            }
            _ => {}
        }
    }
}
