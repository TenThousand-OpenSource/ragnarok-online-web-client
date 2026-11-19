//! The Adventure Guide (the client's `CUIAdventureGuide`): what the game recommends to a character of its
//! level. The window, its tabs and its first page are here; the pages that list things are `guide/pages.rs`.

use super::*;
use crate::guide::{in_levels, kind_shown};
use crate::press::guide::Press;

mod pages;

/// The kinds of a tab: each its number, its name and the levels it is for.
type Kinds = Vec<(u32, String, u32, u32)>;

impl App {
    /// The window, 750 by 468: its picture under everything; the title bar; under it the character (its levels 46
    /// and 97 in, 6 down, its name 45 in and 22 down, in a space 161 by 41 at 1, 19) and the page's title (104 by
    /// 20 at 362, 28, bold, dark blue); the tabs down the left (98 wide and 40 apart from 9, 63); the page itself
    /// (627 by 387 at 122, 81; the first one 637 by 373 at 112, 63).
    pub(in crate::web) fn guide_window(&self) -> String {
        let guide = data::guide();
        let base_level = self.value(parameter::BASE_LEVEL);
        // The tabs: the guide's own first page, then its contents in their order.
        let tabs: Vec<&str> = std::iter::once(guide.menu.as_str()).chain(guide.tabs.iter().map(|(name, _)| name.as_str())).collect();
        let kinds = self.guide_kinds_offered(&guide, base_level);
        let picked_kind = self.guide_kind(&kinds, base_level);
        let mut window = self.guide_head(base_level);
        window.push_str(&self.guide_strip(&tabs, &kinds, picked_kind));
        let title = if self.guide.tab == 0 { guide.intro.as_str() } else { tabs.get(self.guide.tab).copied().unwrap_or_default() };
        // The page's title: `CUIAdventureGuide::OnCreate`, "Gulim" at 14, bold.
        window.push_str(&format!(
            r#"<div class="a t" style="{};text-align:center;{};color:#1b2689">{}</div>"#,
            at(312, 30, 204, 20),
            Font(0, 14, true).style(),
            escape(title)
        ));
        match self.guide.tab {
            0 => window.push_str(&Self::guide_intro(&guide)),
            // The monsters recommended for the character's level (the guide's second kind of page).
            2 => window.push_str(&self.guide_monsters(base_level)),
            1 | 3 | 4 => window.push_str(&self.guide_quests(&guide.quests[[0, 0, 0, 1, 2][self.guide.tab]], picked_kind, base_level)),
            5 | 6 => window.push_str(&self.guide_items(&guide.items[self.guide.tab - 5], picked_kind, base_level)),
            _ => {}
        }
        window
    }

    /// The window's picture, its title bar, and the character under it: its levels and its name.
    fn guide_head(&self, base_level: i64) -> String {
        let mut window = art("adventureguide/bg_guide.bmp", 0, 0, 750, 468);
        window.push_str(&title_bar(750, &data::message(4053).unwrap_or_else(|| "Adventure Guide".to_owned())));
        window.push_str(&cross(750, Press::Open));
        let job_level = self.value(parameter::JOB_LEVEL);
        window.push_str(&text(47, 25, BLACK, &format!("{} {base_level}", tr("Lv."))));
        window.push_str(&text(98, 25, BLACK, &format!("{} {job_level}", tr("Job"))));
        window.push_str(&text(46, 41, BLACK, &self.game.as_ref().map(|game| game.name.clone()).unwrap_or_default()));
        window
    }

    /// The kinds of the open tab that are offered to this character. Only the kinds with something for it
    /// are (the client's `CAdventureGuideMgr::IsAvailable`): a quest, an episode's part or a dungeon for its
    /// level; an item for its level that its job can wear.
    fn guide_kinds_offered(&self, guide: &data::Guide, base_level: i64) -> Kinds {
        let kinds: Kinds = match self.guide.tab {
            1 | 3 | 4 => guide.quests[[0, 0, 0, 1, 2][self.guide.tab]].kinds.clone(),
            5 | 6 => guide.items[self.guide.tab - 5].kinds.clone(),
            _ => Vec::new(),
        };
        let job = self.game.as_ref().map_or(0, |game| game.job);
        let for_level = |least: u32, most: u32| in_levels(least, most, base_level);
        let filled: std::collections::HashSet<u32> = match self.guide.tab {
            1 | 3 | 4 => guide.quests[[0, 0, 0, 1, 2][self.guide.tab]].list
                .iter()
                .filter(|quest| for_level(quest.min, quest.max))
                .map(|quest| quest.kind)
                .collect(),
            5 | 6 => guide.items[self.guide.tab - 5].list
                .iter()
                .filter(|item| for_level(item.min, item.max) && guide.wears(job, item))
                .map(|item| item.kind)
                .collect(),
            _ => Default::default(),
        };
        kinds.into_iter().filter(|(kind, _, least, most)| for_level(*least, *most) && filled.contains(kind)).collect()
    }

    /// The tabs down the window's left. Under the tab that is open hang its kinds, where it has them
    /// (`CUITab::CreateSubTabList`: 98 by 26 each, the picture's box 92 of them; the labels of both,
    /// `CUITab::OnCreate` and `CUISubTab::OnCreate`, the usual text in bold ("Gulim" at 12, `SetBold(1)`), in
    /// the middle, dark blue; a label longer than its tab is cut at the tab's edge), and the tabs after it
    /// move down by as much; what does not fit is scrolled to.
    fn guide_strip(&self, tabs: &[&str], kinds: &Kinds, picked_kind: Option<u32>) -> String {
        let bold = BOLD.style();
        let mut strip = String::new();
        for (index, name) in tabs.iter().enumerate() {
            let state = if index == self.guide.tab { "press" } else { "normal" };
            let press = pressed(Press::Tab(index));
            strip.push_str(&[
                format!(r#"<button type="button" class="t" style="display:block;width:98px;height:40px;padding:0;{bold};"#),
                format!(r#"color:#2c3c5f;background:url('{}') 0 0 / 100% 100% no-repeat;border:0;text-align:center;"#, picture(&format!(
                    "adventureguide/tab_btn_{state}.bmp"
                ))),
                format!(r#"line-height:40px;cursor:default" {press}>{}</button>"#, escape(name))
            ].concat());
            if index == self.guide.tab && !self.guide.folded {
                for (kind, name, ..) in kinds {
                    let state = if Some(*kind) == picked_kind { "press" } else { "normal" };
                    let press = pressed(Press::Kind(*kind));
                    strip.push_str(&[
                        format!(r#"<button type="button" class="t" style="display:block;width:98px;height:26px;padding:0 6px 0 0;"#),
                        format!(
                            r#"box-sizing:border-box;{bold};color:#2c3c5f;background:url('{}') 0 0 / 100% 100% no-repeat;border:0;"#,
                            picture(&format!("adventureguide/tab_btn_sub_{state}.bmp"))
                        ),
                        format!(r#"text-align:center;line-height:26px;cursor:default;white-space:nowrap;overflow:hidden" {press}>{}</button>"#, escape(name))
                    ].concat());
                }
            }
        }
        format!(r#"<div class="a scroll" style="{}">{strip}</div>"#, at(9, 63, 98, 372))
    }

    /// The first page (`CUIIntro`, 637 by 373 at 112, 63): the banner's frame, 630 by 216, with the banner
    /// itself in it, 583 by 162 at 22, 27; under it, at 10, 207, the guide's other pages, a row 603 by 44
    /// each: the page's name 32 in (bold, blue-grey) and what it is for 140 in (brown). In the client a press
    /// on a row does nothing (`CUIIntro::OnSelectContents` is empty); here, by the player's wish, it opens
    /// that page.
    fn guide_intro(guide: &data::Guide) -> String {
        let rows: String = guide
            .tabs
            .iter()
            .enumerate()
            .map(|(index, (name, about))| {
                [
                    format!(
                        r#"<div style="position:relative;width:603px;height:44px;cursor:pointer" {}>{}"#,
                        pressed(Press::Tab(index + 1)),
                        art("adventureguide/list_1.bmp", 0, 0, 603, 44)
                    ),
                    format!(r#"<div class="a t" style="left:32px;top:0;width:100px;line-height:44px;font-weight:bold;color:#485a81;"#),
                    format!(r#"white-space:nowrap">{}</div><div class="a t" style="left:140px;top:0;width:442px;line-height:44px;"#, escape(name)),
                    format!(r#"color:#5d432a;white-space:nowrap;overflow:hidden">{}</div></div>"#, escape(about))
                ].concat()
            })
            .collect();
        let mut page = art("adventureguide/bg_banner.bmp", 112, 63, 630, 216);
        page.push_str(&art("adventureguide/banner.bmp", 134, 90, 583, 162));
        page.push_str(&format!(r#"<div class="a viewer" style="{}">{rows}</div>"#, at(122, 270, 603, 166)));
        page
    }

    /// A line of the guide for the page: an item written into it as the client writes one into text (`^i[<id>]`)
    /// is shown by its picture, with its name under the pointer and its description on a right press.
    pub(super) fn guide_line(line: &str) -> String {
        let mut out = String::new();
        let mut rest = line;
        while let Some(start) = rest.find("^i[") {
            out.push_str(&escape(&rest[..start]));
            let after = &rest[start + 3..];
            let Some((id, end)) = after.find(']').and_then(|end| Some((after[..end].parse::<u32>().ok()?, end))) else {
                out.push_str("^i[");
                rest = after;
                continue;
            };
            out.push_str(&[
                format!(r#"<span class="cell" style="position:relative;display:inline-block;width:24px;height:24px;"#),
                format!(
                    r#"vertical-align:middle" data-info="{id}" data-tip="{}">{}</span>"#,
                    escape(&data::item(id).map(|(name, _)| name).unwrap_or_default()),
                    Self::item_icon(id, true)
                )
            ].concat());
            rest = &after[end + 1..];
        }
        out.push_str(&escape(rest));
        out
    }

    /// The kind shown of a tab's kinds: the one picked, or the first of those the character's level is shown.
    pub(in crate::web) fn guide_kind(&self, kinds: &[(u32, String, u32, u32)], level: i64) -> Option<u32> {
        kind_shown(kinds, self.guide.kind, level)
    }
}
