//! The pages of the Adventure Guide that list things: quests (and an episode's parts, and dungeons), weapons
//! and armour, and monsters.

use super::*;

use crate::guide::{element_line, race_line, size_line};

impl App {
    /// A page of quests, of an episode's parts or of dungeons (`CUIQuest`): a list 627 by 355 of rows 603 by 97.
    /// In a row (`CUIQuestInfo`): at 25, 13 the title in bold and under it the first line of what the client's
    /// tip says of it; at 25, 73 what must be done before it; at 400, 5, in a space 200 by 87, in bold, what
    /// the guide shows as its reward. Those of the kind picked and for the character's level are listed. (The
    /// client also stamps what is done and offers to find the way there; this client has neither yet.)
    pub(super) fn guide_quests(&self, page: &data::GuidePage<data::GuideQuest>, kind: Option<u32>, level: i64) -> String {
        let rows: String = page
            .list
            .iter()
            .filter(|quest| Some(quest.kind) == kind && in_levels(quest.min, quest.max, level))
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
    pub(super) fn guide_items(&self, page: &data::GuidePage<data::GuideItem>, kind: Option<u32>, level: i64) -> String {
        // The client's own words for these six lines are lines 4034 to 4039 of its message table, which the
        // table this client reads does not reach; these stand in for them.
        let guide = data::guide();
        let job = self.game.as_ref().map_or(0, |game| game.job);
        const CAN: [(&str, &str); 3] = [("Refinable", "Not refinable"), ("Enchantable", "Not enchantable"), ("Grade enchantable", "Not grade enchantable")];
        let rows: String = page
            .list
            .iter()
            .filter(|item| Some(item.kind) == kind && in_levels(item.min, item.max, level) && guide.wears(job, item))
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
    pub(super) fn guide_monsters(&self, level: i64) -> String {
        let guide = data::guide();
        let rows: String = guide
            .monsters
            .iter()
            .filter(|monster| in_levels(monster.min, monster.max, level))
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
                        format!(
                            r#"<div class="cell" style="position:relative;display:inline-block;width:24px;height:24px;margin:0 3px 3px 0" data-info="{item}" title="{}">{}</div>"#,
                            escape(&data::item(*item).map(|(name, _)| name).unwrap_or_default()),
                            Self::item_icon(*item, true)
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
