//! What is pressed of what an NPC's script shows, and the times such things run by: the bar over the
//! character filling, a sound that repeats, a mark on the minimap that goes by itself.

use super::*;
use crate::net::scripted;

impl App {
    /// A press of this system (`crate::press::scripted`).
    pub(super) fn scripted_pressed(&mut self, press: crate::press::scripted::Press) {
        match press {
            // The cross of a picture (`UIIllustWnd::SendMsg`, `ID_CLOSE`: `MM_SHOW_IMAGE` with no picture).
            crate::press::scripted::Press::CutinClose => self.cutin_shown(None),
        }
    }

    /// Shows a picture over the screen, or takes the one there away. The window is put where the picture's
    /// kind has it as it opens.
    pub(in crate::web::app) fn cutin_shown(&mut self, cutin: Option<(String, scripted::Stand)>) {
        self.scripted.cutin = cutin;
        let style = wnd::cutin_style(self.scripted.cutin.as_ref().map(|(_, stand)| *stand));
        self.open_at(&wnd::CUTIN, &style);
    }

    /// A time something of a script runs by has come (`Action::Scripted`): the bar over the character is
    /// full, a sound that repeats is due, a mark on the minimap has been shown long enough. Each is looked at
    /// by the clock, so a time that comes late or twice does no harm.
    pub(in crate::web::app) fn scripted_due(&mut self) {
        let time = js_sys::Date::now();
        // The bar is full: the server is told, and the script goes on (`AM_DELETE_SKILL_RECHARGEGAGE` on the
        // player's own character sends CZ_PROGRESS).
        if self.scripted.progress.is_some_and(|full| full <= time) {
            self.scripted.progress = None;
            self.send_on_map(&scripted::progress_done());
        }
        // A sound that repeats (`CGameMode::UpdateLoopSounds`): from where its NPC stands, and not at all
        // while that one is out of sight; one of no NPC from nowhere.
        let own = self.own_account();
        let mut soonest: Option<f64> = None;
        for repeated in &mut self.scripted.repeated {
            if repeated.next <= time {
                match repeated.who {
                    0 => sound::effect(&repeated.name, 1.0),
                    who => world::sounded(&repeated.name, who, own, true),
                }
                repeated.next = time + f64::from(repeated.term);
            }
            soonest = Some(soonest.map_or(repeated.next, |soonest| soonest.min(repeated.next)));
        }
        if let Some(soonest) = soonest {
            after((soonest - time).max(1.0) as i32, Action::Scripted);
        }
        let marks = self.scripted.compass.1.len();
        self.scripted.compass.1.retain(|_, mark| mark.until.is_none_or(|until| until > time));
        if self.scripted.compass.1.len() != marks {
            self.compass_marks();
        }
    }

    /// Writes the script's marks on the minimap anew.
    pub(in crate::web::app) fn compass_marks(&self) {
        if let Some(marks) = element("compass-marks") {
            marks.set_inner_html(&self.minimap_compass());
        }
    }

    /// The character was put elsewhere (`CGameMode::InitializeState`, and a new `CGameMode` for another
    /// map): the marks on the minimap and the bar go; on another map the picture, the sounds that repeat and
    /// the script's music go too.
    pub(in crate::web::app) fn scripted_moved(&mut self, same_map: bool) {
        self.scripted.compass.1.clear();
        self.scripted.progress = None;
        world::gauge(self.own_account(), self.own_account(), None);
        if !same_map {
            self.scripted.repeated.clear();
            sound::repeats_end();
            sound::piece(None);
            if self.scripted.cutin.take().is_some() {
                self.redraw(&[&wnd::CUTIN]);
            }
        }
    }
}
