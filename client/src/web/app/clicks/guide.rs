//! The Adventure Guide.

use super::*;
use crate::web::app::packets::parameter;

impl App {
    /// What a press on one of these does; `false` when the thing pressed is not of this kind.
    pub(super) fn guide_click(&mut self, what: &str, arg: &str) -> bool {
        match what {
            // The guide's icon in the menu, and the window's own close button.
            "guide" => {
                self.guide.open = !self.guide.open;
                self.guide_wants();
                self.render();
            }
            "guide-tab" => {
                let tab = arg.parse().unwrap_or(0);
                // The tab that is open already folds its kinds away, or unfolds them; the page stays.
                if tab == self.guide.tab {
                    self.guide.folded = !self.guide.folded;
                    self.render();
                    return true;
                }
                self.guide.tab = tab;
                self.guide.folded = false;
                self.guide.picked = None;
                self.guide.kind = None;
                self.guide_wants();
                self.render();
            }
            "guide-kind" => {
                self.guide.kind = arg.parse().ok();
                self.guide.picked = None;
                self.guide_wants();
                self.render();
            }
            "guide-pick" => {
                self.guide.picked = Some(arg.to_owned());
                self.refresh_guide();
            }
            _ => return false,
        }
        true
    }

    /// Redraws the window where it is on the page; its list keeps its place.
    pub(in crate::web) fn refresh_guide(&self) {
        let Some(window) = element("guide") else { return };
        let scrolled = element("guide-list").map(|list| list.scroll_top());
        window.set_inner_html(&self.guide_window());
        if let (Some(scrolled), Some(list)) = (scrolled, element("guide-list")) {
            list.set_scroll_top(scrolled);
        }
    }

    /// Asks for what the monsters' page shows and is not here yet: the monsters' pictures (the first picture of
    /// each one's standing) and the names and pictures of the items they are shown with.
    fn guide_wants(&mut self) {
        if !self.guide.open {
            return;
        }
        let level = self.hero.params.get(&parameter::BASE_LEVEL).copied().unwrap_or(1);
        let guide = data::guide();
        // The items of a page of weapons or armour: their names, pictures and descriptions.
        if let Some(page) = self.guide.tab.checked_sub(5).and_then(|at| guide.items.get(at)) {
            let job = self.game.as_ref().map_or(0, |game| game.job);
            let filled: std::collections::HashSet<u32> = page.list.iter().filter(|item| (i64::from(item.min)..=i64::from(item.max)).contains(&level) && guide.wears(job, item)).map(|item| item.kind).collect();
            let kinds: Vec<(u32, String, u32, u32)> = page.kinds.iter().filter(|kind| filled.contains(&kind.0)).cloned().collect();
            let kind = self.guide_kind(&kinds, level);
            data::want_items(page.list.iter().filter(|item| Some(item.kind) == kind && (i64::from(item.min)..=i64::from(item.max)).contains(&level) && guide.wears(job, item)).flat_map(|item| {
                // The item itself, and those written into the lines of where it is had from (`^i[<id>]`).
                let written = item.from.iter().flat_map(|line| line.split("^i[").skip(1).filter_map(|rest| rest.split(']').next()?.parse::<u32>().ok()).collect::<Vec<u32>>());
                std::iter::once(item.id).chain(written).collect::<Vec<u32>>()
            }), || dispatch(Action::ItemsKnown));
        }
        if self.guide.tab != 2 {
            return;
        }
        let shown: Vec<&data::GuideMonster> = guide.monsters.iter().filter(|monster| (i64::from(monster.min)..=i64::from(monster.max)).contains(&level)).collect();
        data::want_items(shown.iter().flat_map(|monster| monster.rewards.iter().copied()), || dispatch(Action::ItemsKnown));
        let wanted: Vec<(u32, String)> = shown
            .iter()
            .filter(|monster| monster.id != 0 && !self.guide.pictures.contains_key(&monster.id) && self.guide.asked.insert(monster.id))
            .filter_map(|monster| Some((monster.id, data::sprite(u16::try_from(monster.id).ok()?)?)))
            .collect();
        if wanted.is_empty() {
            return;
        }
        wasm_bindgen_futures::spawn_local(async move {
            for (id, sprite) in wanted {
                if let Ok(picture) = figure::frame_picture(&format!("몬스터/{sprite}"), 0, 0).await {
                    dispatch(Action::GuideRead(id, picture));
                }
            }
        });
    }
}
