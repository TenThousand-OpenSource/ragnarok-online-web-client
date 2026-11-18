//! The Adventure Guide (the client's `CUIAdventureGuide`): what the game recommends to a character of its level.

use super::*;
use crate::press::guide::Press;

/// The client's words for a monster's element, race and size, by what the server's database calls them: lines
/// of the client's message table (those its navigation shows a monster with).
fn element_line(element: &str) -> usize {
    match element {
        "Water" => 2263,
        "Earth" => 2264,
        "Fire" => 2265,
        "Wind" => 2266,
        "Poison" => 2267,
        "Holy" => 2268,
        "Dark" => 2269,
        "Ghost" => 2270,
        "Undead" => 2271,
        _ => 2272,
    }
}

fn race_line(race: &str) -> usize {
    match race {
        "Undead" => 2276,
        "Brute" => 2277,
        "Plant" => 2278,
        "Insect" => 2279,
        "Fish" => 2280,
        "Demon" => 2281,
        "Demihuman" => 2282,
        "Angel" => 2283,
        "Dragon" => 2284,
        _ => 2285,
    }
}

fn size_line(size: &str) -> usize {
    match size {
        "Medium" => 2273,
        "Large" => 2274,
        _ => 2275,
    }
}

impl App {
    /// The window, 750 by 468: its picture under everything; the title bar; under it the character (its levels 46
    /// and 97 in, 6 down, its name 45 in and 22 down, in a space 161 by 41 at 1, 19) and the page's title (104 by
    /// 20 at 362, 28, bold, dark blue); the tabs down the left (98 wide and 40 apart from 9, 63); the page itself
    /// (627 by 387 at 122, 81; the first one 637 by 373 at 112, 63).
    pub(in crate::web) fn guide_window(&self) -> String {
        let guide = data::guide();
        let mut window = art("adventureguide/bg_guide.bmp", 0, 0, 750, 468);
        window.push_str(&title_bar(750, &data::message(4053).unwrap_or_else(|| "Adventure Guide".to_owned())));
        window.push_str(&press_button(736, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::Open));
        let (base_level, job_level) = (self.value(parameter::BASE_LEVEL), self.value(parameter::JOB_LEVEL));
        window.push_str(&text(47, 25, BLACK, &format!("{} {base_level}", tr("Lv."))));
        window.push_str(&text(98, 25, BLACK, &format!("{} {job_level}", tr("Job"))));
        window.push_str(&text(46, 41, BLACK, &self.game.as_ref().map(|game| game.name.clone()).unwrap_or_default()));
        // The tabs: the guide's own first page, then its contents in their order.
        let tabs: Vec<&str> = std::iter::once(guide.menu.as_str()).chain(guide.tabs.iter().map(|(name, _)| name.as_str())).collect();
        // Under the tab that is open hang its kinds, where it has them (`CUITab::CreateSubTabList`: 98 by 26
        // each, the picture's box 92 of them, so the label is written a little smaller here than the client's
        // narrower font writes it, to fit the same; the labels of both, `CUITab::OnCreate` and `CUISubTab::OnCreate`, bold, in the middle, dark blue), and the tabs after it move down by as much; what does not fit is scrolled to.
        let kinds: Vec<(u32, String, u32, u32)> = match self.guide.tab {
            1 | 3 | 4 => guide.quests[[0, 0, 0, 1, 2][self.guide.tab]].kinds.clone(),
            5 | 6 => guide.items[self.guide.tab - 5].kinds.clone(),
            _ => Vec::new(),
        };
        // Only the kinds with something for this character are offered (the client's `CAdventureGuideMgr::
        // IsAvailable`): a quest, an episode's part or a dungeon for its level; an item for its level that its
        // job can wear.
        let job = self.game.as_ref().map_or(0, |game| game.job);
        let for_level = |least: u32, most: u32| (i64::from(least)..=i64::from(most)).contains(&base_level);
        let filled: std::collections::HashSet<u32> = match self.guide.tab {
            1 | 3 | 4 => guide.quests[[0, 0, 0, 1, 2][self.guide.tab]].list.iter().filter(|quest| for_level(quest.min, quest.max)).map(|quest| quest.kind).collect(),
            5 | 6 => guide.items[self.guide.tab - 5].list.iter().filter(|item| for_level(item.min, item.max) && guide.wears(job, item)).map(|item| item.kind).collect(),
            _ => Default::default(),
        };
        let kinds: Vec<(u32, String, u32, u32)> = kinds.into_iter().filter(|(kind, _, least, most)| for_level(*least, *most) && filled.contains(kind)).collect();
        let picked_kind = self.guide_kind(&kinds, base_level);
        let mut strip = String::new();
        for (index, name) in tabs.iter().enumerate() {
            let state = if index == self.guide.tab { "press" } else { "normal" };
            let press = pressed(Press::Tab(index));
            strip.push_str(&format!(
                r#"<button type="button" class="t" style="display:block;width:98px;height:40px;padding:0;font-weight:bold;color:#2c3c5f;background:url('{}') 0 0 / 100% 100% no-repeat;border:0;text-align:center;line-height:40px;cursor:default" {press}>{}</button>"#,
                picture(&format!("adventureguide/tab_btn_{state}.bmp")),
                escape(name)
            ));
            if index == self.guide.tab && !self.guide.folded {
                for (kind, name, ..) in &kinds {
                    let state = if Some(*kind) == picked_kind { "press" } else { "normal" };
                    let press = pressed(Press::Kind(*kind));
                    strip.push_str(&format!(
                        r#"<button type="button" class="t" style="display:block;width:98px;height:26px;padding:0 6px 0 0;box-sizing:border-box;font-weight:bold;font-size:10.5px;letter-spacing:-0.3px;color:#2c3c5f;background:url('{}') 0 0 / 100% 100% no-repeat;border:0;text-align:center;line-height:26px;cursor:default;white-space:nowrap;overflow:hidden" {press}>{}</button>"#,
                        picture(&format!("adventureguide/tab_btn_sub_{state}.bmp")),
                        escape(name)
                    ));
                }
            }
        }
        window.push_str(&format!(r#"<div class="a scroll" style="{}">{strip}</div>"#, at(9, 63, 98, 372)));
        let title = if self.guide.tab == 0 { guide.intro.as_str() } else { tabs.get(self.guide.tab).copied().unwrap_or_default() };
        window.push_str(&format!(r#"<div class="a t" style="{};text-align:center;font-weight:bold;font-size:14px;color:#1b2689">{}</div>"#, at(312, 30, 204, 20), escape(title)));
        match self.guide.tab {
            // The first page (`CUIIntro`, 637 by 373 at 112, 63): the banner's frame, 630 by 216, with the
            // banner itself in it, 583 by 162 at 22, 27; under it, at 10, 207, the guide's other pages, a row 603
            // by 44 each: the page's name 32 in (bold, blue-grey) and what it is for 140 in (brown). In the
            // client a press on a row does nothing (`CUIIntro::OnSelectContents` is empty); here, by the
            // player's wish, it opens that page.
            0 => {
                let rows: String = guide
                    .tabs
                    .iter()
                    .enumerate()
                    .map(|(index, (name, about))| {
                        format!(
                            r#"<div style="position:relative;width:603px;height:44px;cursor:pointer" {}>{}<div class="a t" style="left:32px;top:0;width:100px;line-height:44px;font-weight:bold;color:#485a81;white-space:nowrap">{}</div><div class="a t" style="left:140px;top:0;width:442px;line-height:44px;color:#5d432a;white-space:nowrap;overflow:hidden">{}</div></div>"#,
                            pressed(Press::Tab(index + 1)),
                            art("adventureguide/list_1.bmp", 0, 0, 603, 44),
                            escape(name),
                            escape(about)
                        )
                    })
                    .collect();
                window.push_str(&art("adventureguide/bg_banner.bmp", 112, 63, 630, 216));
                window.push_str(&art("adventureguide/banner.bmp", 134, 90, 583, 162));
                window.push_str(&format!(r#"<div class="a viewer" style="{}">{rows}</div>"#, at(122, 270, 603, 166)));
            }
            // The monsters recommended for the character's level (the guide's second kind of page).
            2 => window.push_str(&self.guide_monsters(base_level)),
            1 | 3 | 4 => window.push_str(&self.guide_quests(&guide.quests[[0, 0, 0, 1, 2][self.guide.tab]], picked_kind, base_level)),
            5 | 6 => window.push_str(&self.guide_items(&guide.items[self.guide.tab - 5], picked_kind, base_level)),
            _ => {}
        }
        window
    }

    /// A line of the guide for the page: an item written into it as the client writes one into text (`^i[<id>]`)
    /// is shown by its picture, with its name under the pointer and its description on a right press.
    fn guide_line(line: &str) -> String {
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
            let known = data::item(id);
            out.push_str(&format!(
                r#"<span class="cell" style="position:relative;display:inline-block;width:24px;height:24px;vertical-align:middle" data-info="{id}" data-tip="{}">{}</span>"#,
                escape(&known.as_ref().map(|(name, _)| name.clone()).unwrap_or_default()),
                Self::icon(known.as_ref().map(|(_, resource)| resource.as_str()))
            ));
            rest = &after[end + 1..];
        }
        out.push_str(&escape(rest));
        out
    }

    /// The kind shown of a tab's kinds: the one picked, or the first of those the character's level is shown.
    pub(in crate::web) fn guide_kind(&self, kinds: &[(u32, String, u32, u32)], level: i64) -> Option<u32> {
        let shown = |kind: &&(u32, String, u32, u32)| (i64::from(kind.2)..=i64::from(kind.3)).contains(&level);
        kinds.iter().filter(shown).find(|kind| Some(kind.0) == self.guide.kind).or_else(|| kinds.iter().find(shown)).map(|kind| kind.0)
    }

    /// A page of quests, of an episode's parts or of dungeons (`CUIQuest`): a list 627 by 355 of rows 603 by 97.
    /// In a row (`CUIQuestInfo`): at 25, 13 the title in bold and under it the first line of what the client's
    /// tip says of it; at 25, 73 what must be done before it; at 400, 5, in a space 200 by 87, in bold, what
    /// the guide shows as its reward. Those of the kind picked and for the character's level are listed. (The
    /// client also stamps what is done and offers to find the way there; this client has neither yet.)
    fn guide_quests(&self, page: &data::GuidePage<data::GuideQuest>, kind: Option<u32>, level: i64) -> String {
        let rows: String = page
            .list
            .iter()
            .filter(|quest| Some(quest.kind) == kind && (i64::from(quest.min)..=i64::from(quest.max)).contains(&level))
            .map(|quest| {
                let key = quest.id.to_string();
                let picked = self.guide.picked.as_deref() == Some(key.as_str());
                let rewards: String = quest.rewards.iter().map(|line| format!("<div>{}</div>", Self::guide_line(line))).collect();
                let press = pressed(Press::Pick(key));
                format!(
                    r#"<div style="position:relative;width:603px;height:97px" {press}>{}<div class="a t" style="{};white-space:normal;overflow:hidden;line-height:18px"><b>{}</b><br>{}</div><div class="a t" style="{};white-space:nowrap;overflow:hidden;color:#666">{}</div><div class="a t" style="{};white-space:normal;overflow:hidden;line-height:18px;font-weight:bold">{rewards}</div></div>"#,
                    art(&format!("adventureguide/list_2_{}.bmp", if picked { "press" } else { "normal" }), 0, 0, 603, 97),
                    at(25, 13, 360, 57),
                    escape(&quest.title),
                    escape(&quest.says),
                    at(25, 73, 360, 13),
                    escape(&quest.needs),
                    at(400, 9, 190, 83),
                )
            })
            .collect();
        format!(r#"<div id="guide-list" class="a viewer" style="{}">{rows}</div>"#, at(122, 81, 627, 355))
    }

    /// A page of weapons or of armour (`CUIItem`): a list 627 by 355 of rows 603 by 128. In a row
    /// (`CUIItemInfo`): the item's large picture, 75 by 100 at 45, 14; its name in bold at 165, 13 (blue); under
    /// it, 173 by 97 at 165, 26, the lines of its description that name a property (those the client marks in
    /// grey; the English descriptions mark them in blue instead), seven at most; at 338, 26 whether it can be refined, enchanted and enchanted by grade (blue when it
    /// can, red when not); at 480, 5, in a space 108 wide, where it is had from.
    fn guide_items(&self, page: &data::GuidePage<data::GuideItem>, kind: Option<u32>, level: i64) -> String {
        // The client's own words for these six lines are lines 4034 to 4039 of its message table, which the
        // table this client reads does not reach; these stand in for them.
        let guide = data::guide();
        let job = self.game.as_ref().map_or(0, |game| game.job);
        const CAN: [(&str, &str); 3] = [("Refinable", "Not refinable"), ("Enchantable", "Not enchantable"), ("Grade enchantable", "Not grade enchantable")];
        let rows: String = page
            .list
            .iter()
            .filter(|item| Some(item.kind) == kind && (i64::from(item.min)..=i64::from(item.max)).contains(&level) && guide.wears(job, item))
            .map(|item| {
                let key = item.id.to_string();
                let picked = self.guide.picked.as_deref() == Some(key.as_str());
                let known = data::item(item.id);
                let large = known.as_ref().filter(|(_, resource)| !resource.is_empty()).map(|(_, resource)| format!(r#"<img src="{}" alt="" draggable="false" onerror="this.remove()">"#, picture(&format!("collection/{resource}.bmp")))).unwrap_or_default();
                let lines: String = data::about(item.id).map(|(lines, _)| lines).unwrap_or_default().iter().filter(|line| line.contains("^777777") || line.to_ascii_lowercase().contains("^0000cc")).take(7).map(|line| format!("<div>{}</div>", coloured(line))).collect();
                let can: String = item.can.iter().zip(CAN).map(|(can, (yes, no))| format!(r#"<div style="color:{}">{}</div>"#, if *can { "#6464c8" } else { "#c86464" }, escape(tr(if *can { yes } else { no })))).collect();
                let from: String = item.from.iter().map(|line| format!("<div>{}</div>", Self::guide_line(line))).collect();
                let press = pressed(Press::Pick(key));
                format!(
                    r#"<div style="position:relative;width:603px;height:128px" {press} data-info="{}">{}<div class="a" style="{}">{large}</div><div class="a t" style="left:165px;top:13px;font-weight:bold;color:#526bce;white-space:nowrap">{}</div><div class="a t" style="{};white-space:nowrap;overflow:hidden;line-height:14px">{lines}</div><div class="a t" style="{};white-space:normal;line-height:18px">{can}</div><div class="a t" style="{};white-space:normal;overflow:hidden;line-height:16px">{from}</div></div>"#,
                    item.id,
                    art(&format!("adventureguide/list_3_{}.bmp", if picked { "press" } else { "normal" }), 0, 0, 603, 128),
                    at(45, 14, 75, 100),
                    escape(&known.as_ref().map(|(name, _)| name.clone()).unwrap_or_default()),
                    at(165, 30, 173, 93),
                    at(338, 32, 115, 90),
                    at(480, 11, 108, 112),
                )
            })
            .collect();
        format!(r#"<div id="guide-list" class="a viewer" style="{}">{rows}</div>"#, at(122, 81, 627, 355))
    }

    /// The monsters' page (`CUIMonster`): a list 627 by 355 of rows 603 by 128. In a row (`CUIMonsterInfo`): the
    /// monster's picture in a space 124 square at 16, 2; its name 165 in and 19 down (blue); under it, from 39
    /// down, its element with the element's level, its race, its size, and that it is a boss when it is one; at
    /// 480, 5, in a space 108 wide, the pictures of the items the guide shows as its reward.
    fn guide_monsters(&self, level: i64) -> String {
        let guide = data::guide();
        let rows: String = guide
            .monsters
            .iter()
            .filter(|monster| (i64::from(monster.min)..=i64::from(monster.max)).contains(&level))
            .map(|monster| {
                let picked = self.guide.picked.as_deref() == Some(monster.key.as_str());
                let mut row = art(&format!("adventureguide/list_3_{}.bmp", if picked { "press" } else { "normal" }), 0, 0, 603, 128);
                // The picture stands on the middle of its space's floor, and what is larger than the space is
                // cut at its edges.
                if let Some((url, size, feet)) = self.guide.pictures.get(&monster.id) {
                    row.push_str(&format!(
                        r#"<div class="a" style="{};overflow:hidden"><img src="{url}" alt="" draggable="false" style="position:absolute;left:{:.0}px;top:{:.0}px;width:{}px;height:{}px"></div>"#,
                        at(16, 2, 124, 124),
                        62.0 - feet.0,
                        86.0 - feet.1,
                        size.0,
                        size.1
                    ));
                }
                // One the server's database does not know is shown by the name the guide knows it by, as the
                // client shows one its own tables lack.
                if monster.id == 0 {
                    row.push_str(&format!(r#"<div class="a t" style="left:165px;top:19px;color:#526bce">{}</div>"#, escape(&monster.key)));
                } else {
                    row.push_str(&format!(r#"<div class="a t" style="left:165px;top:19px;color:#526bce">{}</div>"#, escape(&monster.name)));
                    let line = |line: usize, otherwise: &str| data::message(line).unwrap_or_else(|| otherwise.to_owned());
                    let mut lines = vec![
                        line(element_line(&monster.element), &format!("{} %d", monster.element)).replace("%d", &monster.element_level.to_string()),
                        line(race_line(&monster.race), &monster.race),
                        line(size_line(&monster.size), &monster.size),
                    ];
                    if monster.boss {
                        lines.push(line(2230, "Boss"));
                    }
                    for (at, words) in lines.iter().enumerate() {
                        row.push_str(&text(165, 45 + at as i32 * 18, if monster.boss && at == 3 { "#ff6464" } else { BLACK }, words));
                    }
                }
                let rewards: String = monster
                    .rewards
                    .iter()
                    .map(|item| {
                        let known = data::item(*item);
                        format!(
                            r#"<div class="cell" style="position:relative;display:inline-block;width:24px;height:24px;margin:0 3px 3px 0" data-info="{item}" title="{}">{}</div>"#,
                            escape(&known.as_ref().map(|(name, _)| name.clone()).unwrap_or_default()),
                            Self::icon(known.as_ref().map(|(_, resource)| resource.as_str()))
                        )
                    })
                    .collect();
                row.push_str(&format!(r#"<div class="a" style="{};line-height:0">{rewards}</div>"#, at(480, 11, 108, 112)));
                format!(r#"<div style="position:relative;width:603px;height:128px" {}>{row}</div>"#, pressed(Press::Pick(monster.key.clone())))
            })
            .collect();
        format!(r#"<div id="guide-list" class="a viewer" style="{}">{rows}</div>"#, at(122, 81, 627, 355))
    }
}
