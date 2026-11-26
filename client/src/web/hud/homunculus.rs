//! The homunculus window (the client's `UIHomunInfoWnd`), what it asks, the list of the homunculus's skills,
//! and the menu on the homunculus itself.

use super::*;
use super::skill_list::Listed;
use crate::web::app::HomunculusAsk;
use crate::homunculus::{attack_speed, intimacy_word};
use crate::press::homunculus::{Order, Press};

/// A gauge of the homunculus window (`UIBarGraphHomun::Paint`), `width` by 9: the back with its ends, the
/// fill up to `now` of `most` (red under a quarter), and the two numbers in the middle, 2 above, as (1, 13).
fn gauge(x: i32, y: i32, width: i32, now: i64, most: i64) -> String {
    if most < now {
        return String::new();
    }
    let mut gauge = format!(
        "{}{}{}",
        art("basic_interface/gze_bg_left.bmp", x, y, 2, 9),
        art("basic_interface/gze_bg_mid.bmp", x + 2, y, width - 6, 9),
        art("basic_interface/gze_bg_right.bmp", x + width - 4, y, 4, 9)
    );
    if most > 0 && now > 0 {
        gauge.push_str(&filling(x, y, width - 8, now, most, Red::UnderQuarter));
    }
    if most > 0 {
        gauge.push_str(&numbers_over(x, y, width, now, most));
    }
    gauge
}

/// The eight numbers down the left of the window, by the words its picture has there (`homuninfo_bg.bmp`:
/// Atk, Matk, Hit, Critical, Def, Mdef, Flee, Aspd): each ends 79 across, 18 under the last, as (1, 14); the
/// last is the speed of its blows as the client reckons it from their delay.
pub(super) fn companion_numbers(stats: &[i32; 8]) -> String {
    stats
        .iter()
        .enumerate()
        .map(|(row, stat)| text_ending_in(
            FIGURES,
            79,
            280,
            24 + row as i32 * 18,
            BLACK,
            &if row == 7 { attack_speed(*stat) } else { *stat }.to_string()
        ))
        .collect()
}

impl App {
    /// The window (`UIHomunInfoWnd::OnCreate` and `Paint`), 280 by 208: its picture, the title 5 in and 2
    /// down; from 100 across the name (to type a new one, with the button that sets it), the level, Delete
    /// and Skill, the gauges of life and SP 157 wide, the experience to the next level over its bar, the
    /// hunger of 100 over its bar, Feed and the box of feeding by itself, the word of its intimacy; and in a
    /// white box under the picture, in red, that it earns a tenth of its owner's experience.
    pub(in crate::web) fn homunculus_window(&self) -> String {
        let Some(info) = &self.homunculus.info else { return String::new() };
        let mut window = art("basic_interface/homuninfo_bg.bmp", 0, 0, 280, 180);
        // `DrawFilledRoundedOutline(0, height - 40, width, 40, white)`; its rim is the plain windows' grey.
        window.push_str(&format!(
            r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-radius:3px;box-sizing:border-box"></div>"#,
            at(0, 168, 280, 40)
        ));
        window.push_str(&text(5, 2, BLACK, &Self::words(936, "Homunculus Info")));
        window.push_str(&cross(280, Press::Window));
        for (y, line, otherwise) in [(24, 407, "Name"), (47, 408, "Level"), (105, 1019, "EXP"), (131, 585, "Hunger"), (159, 586, "Intimacy")] {
            window.push_str(&text(100, y, BLACK, &Self::words(line, otherwise)));
        }
        window.push_str(&text(140, 159, BLACK, ":"));
        window.push_str(&companion_numbers(&info.stats));
        window.push_str(&field(&windows::HOMUNCULUS.field(), (135, 22, 80, 16), &info.name, 50, &format!("background:{FIELD_GROUND};box-sizing:border-box")));
        window.push_str(&picture_press(232, 21, "btn_rewrite", Press::Rename));
        window.push_str(&text(135, 47, BLACK, &info.level.to_string()));
        window.push_str(&picture_press(187, 44, "btn_del", Press::Delete));
        window.push_str(&picture_press(232, 44, "btn_skill", Press::Skills));
        window.push_str(&gauge(117, 73, 157, info.hp, info.max_hp));
        window.push_str(&gauge(117, 88, 157, info.sp, info.max_sp));
        // The experience the next level takes, ending 205 across, over the bar of how far it is.
        window.push_str(&text_ending_in(FIGURES, 205, 280, 104, BLACK, &info.next_exp.to_string()));
        window.push_str(&plain_bar(101, 119, 102, info.exp, info.next_exp, false));
        window.push_str(&text_ending_in(FIGURES, 200, 280, 130, BLACK, &format!("{} / 100", info.hunger)));
        window.push_str(&plain_bar(101, 145, 102, i64::from(info.hunger), 100, true));
        // Feed is 64 wide: the client makes it at 210, 155 and moves it to 210, 105.
        window.push_str(&picture_press_sized(210, 105, (64, 20), "btn_feed", Press::Feed));
        // The box of feeding by itself, 12 by 12 at 210, 142, and its words from 225, 136 (line 3279, whose
        // `\n` breaks the line).
        window.push_str(&check_words(210, 142, self.homunculus.autofeed, "", Press::AutoFeed));
        window.push_str(&format!(
            r#"<div class="a t" style="left:225px;top:136px;color:{BLACK};white-space:pre;line-height:12px">{}</div>"#,
            escape(&Self::words(3279, "Auto \\nFeeding").replace("\\n", "\n"))
        ));
        let (line, otherwise) = intimacy_word(info.intimacy);
        window.push_str(&text(150, 159, BLACK, Self::words(line, otherwise).trim()));
        window.push_str(&format!(
            r#"<div class="a t" style="left:8px;top:180px;color:#f00;white-space:pre;line-height:12px">{}</div>"#,
            escape(&Self::words(3284, "Homunculus receives \\n10% of master's experience.").replace("\\n", "\n"))
        ));
        window
    }

    /// What the window asks before it acts, in the client's message box: whether to feed the homunculus
    /// (line 601), to give it the name typed (2977) or to delete it (955); the colours the lines carry
    /// (`^ff0000`) are written as the client writes them.
    pub(in crate::web) fn homunculus_question(&self) -> String {
        let words = match &self.homunculus.asks {
            Some(HomunculusAsk::Feed) => Self::words(601, "Are you sure that you want to feed your pet?"),
            Some(HomunculusAsk::Rename(name)) => {
                Self::words(2977, "Are you sure you want to change your Homunculus's name to ^0000ff%s^000000?").replace("%s", name)
            }
            Some(HomunculusAsk::Delete) => Self::words(955, "You have decided to delete this Homunculus. \
                When deleted, the homunculus and its history will be deleted and they cannot be restored in the \
                future. Would you like to continue?"),
            None => return String::new(),
        };
        confirm_box(&words, Press::Yes, Press::No)
    }

    /// The list of its skills (`UISkillListWnd`, its `skillTab` 0), with the points it has to spend.
    pub(in crate::web) fn homunculus_skills(&self) -> String {
        let points = self.homunculus.info.as_ref().map_or(0, |info| info.skill_points);
        let does = Listed {
            close: Press::Skills.into(),
            pick: |row| Press::SkillPick(row).into(),
            uses: Press::SkillUse.into(),
            level: |step| Press::SkillLevel(step).into(),
            raise: Some(|skill| Press::SkillRaise(skill).into())
        };
        self.skill_list(&Self::words(937, "Homunculus Skill List"), &self.homunculus.skills, self.homunculus.skill_picked, Some(points), does)
    }

    /// The menu the right button opens on the player's own homunculus (the client's `OpenActorMenu`): its
    /// window, feeding, standing by (lines 1112 to 1114).
    pub(in crate::web) fn homunculus_menu(&self) -> String {
        [(1112, "View Info", Order::Info), (1113, "Feed", Order::Feed), (1114, "Stand By", Order::StandBy)]
            .iter()
            .map(|(line, otherwise, order)| menu_row(&Self::words(*line, otherwise), Press::Do(*order)))
            .collect()
    }
}
