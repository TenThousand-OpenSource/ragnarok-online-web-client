//! The window of the attendance check (the client's `UICheckAttendanceWnd`).

use super::*;
use crate::attendance::{CELL, cell};
use crate::press::attendance::Press;

/// One of the client's colours of text, written as its code writes it (`0xBBGGRR`), as the page takes it.
fn colour(code: u32) -> String {
    format!("#{:02x}{:02x}{:02x}", code & 0xff, code >> 8 & 0xff, code >> 16 & 0xff)
}

/// A bold line with its middle in the middle of a stretch `width` wide that begins at `x` (`DrawCenteredText`
/// with a width).
fn centred(x: i32, y: i32, width: i32, size: u8, code: u32, words: &str) -> String {
    [
        format!(r#"<div class="a t" style="left:{x}px;top:{y}px;width:{width}px;text-align:center;white-space:nowrap;pointer-events:none;"#),
        format!(r#"color:{};{}">{}</div>"#, colour(code), Font(0, size, true).style(), escape(words))
    ].concat()
}

impl App {
    /// The cell of the day whose reward is to be claimed (`Paint`, `OnProcess`, `OnLBtnDown`): its two
    /// pictures change places every tenth of a second, and a third shows while it is held down. `marks` is
    /// what its tag says more.
    fn attendance_claim(&self, marks: &str) -> String {
        let (x, y) = cell(self.attendance.days);
        let [first, second, down] = ["a", "b", "press"].map(|name| picture(&format!("check_attendance/bt_slot_{name}.bmp")));
        [
            "<style>@keyframes attendance-blink{0%,49.99%{background-image:var(--n)}50%,100%{background-image:var(--o)}}".to_owned(),
            "[data-blinks]:not(:active){animation:attendance-blink .2s step-end infinite}</style>".to_owned(),
            format!(r#"<button type="button" class="a b" data-blinks style="{};--n:url('{first}');--o:url('{second}');"#, at(x, y, CELL.0, CELL.1)),
            format!(r#"--p:url('{down}')" {} {marks}></button>"#, pressed(Press::Claim))
        ].concat()
    }

    /// The attendance check (`UICheckAttendanceWnd`, 488 by 413 while the client's table of its banners is
    /// empty, as it is; `OnCreate`, `Paint`, `VisitChildren`): `check_attendance\attendance_bg.bmp`; the
    /// server's name in line 3465 of the message table at 41 down and the event's days in line 3466 at 70,
    /// both in the middle; the days in rows of five from (26, 95), each with its reward at (17, 14) of its
    /// cell and "Day n" (3467) 45 under its top; at the right what there is to do (3468) or what was done
    /// (3469), "Day" (3470) at 289 down with the days the event still runs at 309; over the days attended
    /// `bt_slot_off.tga`, and `bt_slot_complete.tga` over the last of them once today's was claimed; "Close"
    /// (3045) at (170, 376).
    pub(in crate::web) fn attendance_window(&self) -> String {
        let table = data::attendance();
        let server = self.account.as_ref().and_then(|account| account.char_servers.get(self.server_picked)).map(|server| server.name.clone());
        let mut window = art("check_attendance/attendance_bg.bmp", 0, 0, 488, 413);
        let filled = |line: usize, otherwise: &str, with: &[&str]| crate::numbers::filled(&Self::words(line, otherwise), with);
        window.push_str(&centred(0, 41, 488, 13, 0x203F66, &filled(3465, "Server %s Attendance Reward", &[&server.unwrap_or_default()])));
        let (first, last) = (crate::attendance::month_and_day(table.first), crate::attendance::month_and_day(table.last));
        let days = [first.0, first.1, last.0, last.1].map(|number| number.to_string());
        let period = filled(3466, "Event period: %02d/%02d ~ %2d/%2d 24:00", &[&days[0], &days[1], &days[2], &days[3]]);
        window.push_str(&centred(0, 70, 488, 12, 0x3950A4, &period));
        window.push_str(&centred(345, 289, 109, 14, 0x595939, &Self::words(3470, "Day")));
        if self.attendance.left > 0 {
            window.push_str(&centred(368, 309, 54, 22, 0xFFFFFF, &self.attendance.left.to_string()));
        }
        window.push_str(&text_in(Font(0, 12, true), 425, 318, &colour(0x595939), &Self::words(3471, "")));
        // The client cuts the line into pieces 112 wide and stands them, 15 apart, about the middle of 77.
        let says = if self.attendance.claimable {
            filled(3468, "Click the item to claim day %d reward", &[&(self.attendance.days + 1).to_string()])
        } else {
            filled(3469, "You have claimed day %d reward", &[&self.attendance.days.to_string()])
        };
        window.push_str(&[
            format!(r#"<div class="a t" style="{};display:flex;align-items:center;justify-content:center;text-align:center;"#, at(343, 95, 112, 77)),
            format!(r#"white-space:normal;line-height:15px;color:{};{}">"#, colour(0x613CA5), Font(0, 12, true).style()),
            format!("<span>{}</span></div>", escape(&says))
        ].concat());
        for (place, (_, item, many)) in table.rewards.iter().enumerate() {
            let (x, y) = cell(place);
            // Pointed at, a cell names its reward, and the right button opens the reward's description
            // (`OnMouseHover`, `OnRBtnDown`); the cell of the day to claim is the button that claims it.
            let marks = format!(r#"{} data-tip="{}""#, item_marks(*item, true), escape(&called(*item)));
            if self.attendance.claimable && place == self.attendance.days {
                window.push_str(&self.attendance_claim(&marks));
            } else {
                window.push_str(&format!(r#"<div class="a" style="{}" {marks}></div>"#, at(x, y, CELL.0, CELL.1)));
            }
            window.push_str(&format!(
                r#"<div class="a cell" style="left:{}px;top:{}px;pointer-events:none">{}{}</div>"#,
                x + 13,
                y + 10,
                Self::item_icon(*item, true),
                count(many)
            ));
            let label = filled(3467, "Day %d", &[&(place + 1).to_string()]);
            window.push_str(&centred(x, y + 45, CELL.0, 12, 0xA44F3B, &label));
        }
        for day in 0..self.attendance.days {
            let (x, y) = cell(day);
            let last = day + 1 == self.attendance.days && !self.attendance.claimable;
            let name = if last { "check_attendance/bt_slot_complete.tga" } else { "check_attendance/bt_slot_off.tga" };
            window.push_str(&format!(
                r#"<img class="a" src="{}" alt="" draggable="false" style="left:{x}px;top:{y}px;pointer-events:none">"#,
                picture(name)
            ));
        }
        window.push_str(&pictured(
            170,
            376,
            (146, 30),
            ["check_attendance/bt_ok_normal.bmp", "check_attendance/bt_ok_normal.bmp", "check_attendance/bt_ok_press.bmp"],
            Press::Close,
            ""
        ));
        window.push_str(&centred(170, 384, 146, 14, 0x313131, &Self::words(3045, "Close")));
        window
    }
}
