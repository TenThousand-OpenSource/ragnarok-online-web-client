//! The Tipbox.

use super::*;

/// Where the browser keeps that the Tipbox is not to open as the game is entered (the client's
/// `SHOWTIPSATSTARTUP`).
const KEPT: &str = "roweb.tips-off";

impl App {
    /// Reads what the browser keeps.
    pub(in crate::web::app) fn load_tips(&mut self) {
        self.tips.not_at_start = stored(KEPT).is_some();
    }

    /// Opens the window on a tip picked by lot, as the client does.
    fn open_tips(&mut self) {
        let tips = data::tips();
        if tips.is_empty() {
            return;
        }
        let lot = (js_sys::Math::random() * tips.len() as f64) as usize;
        (self.tips.tip, self.tips.page, self.tips.found, self.tips.open) = (tips[lot.min(tips.len() - 1)].id, 0, None, true);
    }

    /// As the game is first entered the Tipbox opens, unless the player has it not (`UITipboxWnd`, the box
    /// "Open on Startup").
    pub(in crate::web::app) fn tips_at_start(&mut self) {
        if !std::mem::replace(&mut self.tips.started, true) && !self.tips.not_at_start {
            self.open_tips();
        }
    }

    /// A press of this system (`crate::press::tips`).
    pub(super) fn tips_pressed(&mut self, press: crate::press::tips::Press) {
        use crate::press::Step;
        use crate::press::tips::Press;
        match press {
            // The menu's icon, Alt+D and `/tip`: it opens, or closes if it was open.
            Press::Window => {
                if self.tips.open {
                    self.tips.open = false;
                } else {
                    self.open_tips();
                }
                self.render();
            }
            Press::Page(step) => {
                let pages = data::tips().iter().find(|tip| tip.id == self.tips.tip).map_or(1, |tip| tip.pages.len().max(1));
                // The arrows go round: after the last page the first, before the first the last.
                self.tips.page = if step == Step::More { (self.tips.page + 1) % pages } else { (self.tips.page + pages - 1) % pages };
                self.redraw(&[&wnd::TIPS]);
            }
            // Tip 0 is no tip: Escape closes the window with it.
            Press::Tip(0) => {
                self.tips.open = false;
                self.render();
            }
            Press::Tip(tip) => {
                if data::tips().iter().any(|known| known.id == tip) {
                    (self.tips.tip, self.tips.page, self.tips.found) = (tip, 0, None);
                    self.redraw(&[&wnd::TIPS]);
                }
            }
            Press::Startup => {
                self.tips.not_at_start = !self.tips.not_at_start;
                store(KEPT, self.tips.not_at_start.then_some("1"));
                self.redraw(&[&wnd::TIPS]);
            }
        }
    }

    /// What was typed into the window's line is looked for in the tips' titles and texts, whatever the size
    /// of its letters; an empty line brings the tip back.
    pub(in crate::web::app) fn tip_searched(&mut self) {
        let sought = field(&wnd::TIPS.field()).trim().to_lowercase();
        self.tips.found = (!sought.is_empty()).then(
            || data::tips()
                .iter()
                .filter(
                    |tip| tip.title.to_lowercase().contains(&sought) || tip.pages.iter().any(|page| page.to_lowercase().contains(&sought))
                )
                .map(|tip| tip.id)
                .collect()
        );
        self.redraw(&[&wnd::TIPS]);
    }
}
