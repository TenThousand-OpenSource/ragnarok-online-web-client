//! The Status window: the six stats, what they give, and the points to raise them.

use super::*;
use crate::press::status::Press;

impl App {
    pub(in crate::web) fn status_window(&self) -> String {
        let width = 280;
        let mut window = title_bar(width, "Status");
        window.push_str(&cross(width, Press::Window));
        window.push_str(&art("statuswnd/w_statwin_bg.bmp", 0, 17, 280, 124));
        let points = self.value(parameter::STATUS_POINTS);
        // Every number of the window is written as (1, 14) (`UIStatusWnd::Paint`).
        for stat in 0..6u16 {
            let top = 23 + 16 * i32::from(stat);
            let (value, bonus, cost) = (self.value(parameter::STR + stat), self.hero.stat_bonus[usize::from(stat)], self.value(parameter::STR_COST + stat));
            window.push_str(&text_in(FIGURES, 36, top, BLACK, &value.to_string()));
            // Beside a stat of three figures the bonus is written smaller, further right and 2 lower, as (1, 12).
            if bonus != 0 && value >= 100 {
                window.push_str(&text_in(Font(1, 12, false), 54, top + 2, BLACK, &format!("{bonus:+}")));
            } else if bonus != 0 {
                window.push_str(&text_in(FIGURES, 49, top, BLACK, &format!("{bonus:+}")));
            }
            // A stat can be raised while the points for it are there.
            if cost > 0 && cost <= points {
                window.push_str(&press_button(78, top + 1, 11, "basic_interface/arw_right.bmp", "basic_interface/arw_right.bmp", Press::Raise(stat)));
            }
            if cost > 0 {
                window.push_str(&text_ending_in(FIGURES, 101, width, top, BLACK, &cost.to_string()));
            }
        }
        let pair = |first: u16, second: u16| format!("{} + {}", self.value(first), self.value(second));
        // The server keeps the delay between attacks; the window shows the speed it means.
        let attack_speed = (2000 - self.value(parameter::ATTACK_DELAY)) / 10;
        let left = [pair(parameter::ATK, parameter::ATK_BONUS), pair(parameter::MATK_MIN, parameter::MATK_MAX), self.value(parameter::HIT).to_string(), self.value(parameter::CRITICAL).to_string()];
        let right = [
            pair(parameter::DEF, parameter::DEF_BONUS),
            pair(parameter::MDEF, parameter::MDEF_BONUS),
            pair(parameter::FLEE, parameter::FLEE_BONUS),
            attack_speed.to_string(),
            points.to_string(),
        ];
        for (row, value) in left.iter().enumerate() {
            window.push_str(&text_ending_in(FIGURES, 189, width, 22 + 16 * row as i32, BLACK, value));
        }
        for (row, value) in right.iter().enumerate() {
            window.push_str(&text_ending_in(FIGURES, 275, width, 22 + 16 * row as i32, BLACK, value));
        }
        window
    }
}
