//! The mercenary window (the client's `UIMerInfoWnd`), what it asks, the list of the mercenary's skills, and
//! the menu on the mercenary itself.

use super::*;
use super::skill_list::Listed;
use crate::mercenary::{expiry, title};
use crate::press::mercenary::{Order, Press};

impl App {
    /// The window (`UIMerInfoWnd::OnCreate` and `Paint`), 280 by 180: the homunculus window's picture, the
    /// title by which mercenary it is; from 100 across its name (over a grey copy of itself a pixel to the
    /// right) and level, the buttons that dismiss it and open its skills, its life and SP in the picture's
    /// own gauges (85 wide) with the numbers beside them, when its contract ends in red, the faith of its
    /// guild, how many were called and how many it has killed.
    pub(in crate::web) fn mercenary_window(&self) -> String {
        let Some(info) = &self.mercenary.info else { return String::new() };
        let mut window = art("basic_interface/homuninfo_bg.bmp", 0, 0, 280, 180);
        let (line, otherwise) = title(self.mercenary.job);
        window.push_str(&text(5, 2, BLACK, &Self::words(line, otherwise)));
        window.push_str(&cross(280, Press::Window));
        for (x, y, line, otherwise) in [(100, 24, 407, "Name"), (100, 47, 408, "Level"), (100, 111, 1184, "Expiration"), (100, 135, 1185, "Loyalty"), (100, 159, 1186, "Summons"), (230, 133, 1187, "Kill")] {
            window.push_str(&text(x, y, BLACK, &Self::words(line, otherwise)));
        }
        window.push_str(&super::homunculus::companion_numbers(&info.stats));
        window.push_str(&text(136, 24, "#646464", &info.name));
        window.push_str(&text(135, 24, BLACK, &info.name));
        window.push_str(&text(135, 47, BLACK, &info.level.to_string()));
        window.push_str(&picture_press(187, 44, "btn_fired", Press::Fire));
        window.push_str(&picture_press(232, 44, "btn_skill", Press::Skills));
        // `UIBarGraphPlayer`, 85 by 9: its own back is Basic Information's, 135 wide, of which the window
        // shows 85; the fill and the numbers are drawn as that window's gauges are (`App::gauge`), cut there.
        for (y, now, most) in [(73, info.hp, info.max_hp), (88, info.sp, info.max_sp)] {
            window.push_str(&mercenary_gauge(117, y, i64::from(now), i64::from(most)));
            window.push_str(&text_in(FIGURES, 218, y - 3, BLACK, &format!("{now} / {most}")));
        }
        // The end of its contract, as the client's line 1260 writes a time, in the player's own hours.
        let ends = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(f64::from(info.expires) * 1000.0));
        window.push_str(&text(155, 111, RED, &expiry(&Self::words(1260, "%m/%d %H:%M"), (ends.get_month() + 1, ends.get_date(), ends.get_hours(), ends.get_minutes()))));
        window.push_str(&text_in(FIGURES, 170, 133, BLACK, &info.faith.to_string()));
        window.push_str(&text_in(FIGURES, 170, 157, BLACK, &info.calls.to_string()));
        window.push_str(&text_in(FIGURES, 245, 157, BLACK, &info.kills.to_string()));
        window
    }

    /// Whether to dismiss the mercenary, in the client's message box (its line 1370, with its colours).
    pub(in crate::web) fn mercenary_question(&self) -> String {
        if !self.mercenary.asks {
            return String::new();
        }
        message_box(Asked::Sure, &coloured(&Self::words(1370, "^ff0000Deleting a Mercenary Soldier^000000 will also delete his growth history. Do you really want to proceed with the deletion?")), Press::Yes, Press::No)
    }

    /// The list of its skills (`UISkillListWnd`, its `skillTab` 1): no points to spend, nothing to raise.
    pub(in crate::web) fn mercenary_skills(&self) -> String {
        let does = Listed { close: Press::Skills.into(), pick: |row| Press::SkillPick(row).into(), uses: Press::SkillUse.into(), level: |step| Press::SkillLevel(step).into(), raise: None };
        self.skill_list(&Self::words(1262, "Mercenary Skill List"), &self.mercenary.skills, self.mercenary.skill_picked, None, does)
    }

    /// The menu the right button opens on the player's own mercenary (the client's `OpenActorMenu`): its
    /// window and standing by (the homunculus's lines 1112 and 1114).
    pub(in crate::web) fn mercenary_menu(&self) -> String {
        [(1112, "View Info", Order::Info), (1114, "Stand By", Order::StandBy)].iter().map(|(line, otherwise, order)| menu_row(&Self::words(*line, otherwise), Press::Do(*order))).collect()
    }
}

/// A gauge of the mercenary window (`UIBarGraphPlayer::Paint` at 85 by 9): the fill up to `now` of `most`, red
/// under a quarter, over the frame the window's picture already has there. The numbers the gauge itself
/// writes in its middle are the same the window writes beside it, and are left to the window.
fn mercenary_gauge(x: i32, y: i32, now: i64, most: i64) -> String {
    if most <= 0 || now > most || now <= 0 {
        return String::new();
    }
    filling(x, y, 77, now, most, Red::UnderQuarter)
}
