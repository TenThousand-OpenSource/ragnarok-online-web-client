//! The Shortcut Settings window: every key of the client, and the setting of another in its place.

use super::*;
use crate::hotkeys;
use crate::press::Step;
use crate::press::hotkeys::Press;
use crate::web::app::PAGE;

impl App {
    /// Redraws the window where it is on the page.
    /// The client's `UIHotKeyWnd`, 436 by 432: four tabs (Hotkey Bar, Hotkey Bar 2, Interface, Macros) under
    /// the title; two columns of eighteen rows, 20 high, each row a name 135 wide and its keys on a tinted
    /// 80; a row pressed is picked, and the next key pressed becomes its key (Escape takes it away); under
    /// them Reset at the left and OK, Cancel and Close at the right, on the button bar. Tables longer than a
    /// page (the interface) have the arrows at the top right.
    pub(in crate::web) fn key_window(&self) -> String {
        let table = hotkeys::TABS[self.keys.tab.min(3)];
        let listed = self.listed_keys(table);
        let pages = listed.len().div_ceil(PAGE).max(1);
        let page = self.keys.page.min(pages - 1);
        let mut window = title_bar(436, "Shortcut Settings");
        window.push_str(&art("basic_interface/sys_base_off.bmp", 3, 3, 11, 11));
        window.push_str(&press_button(422, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::Close));
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0;box-sizing:border-box"></div>"#, at(0, 17, 436, 386)));
        window.push_str(&art("basic_interface/btnbar_left2.bmp", 0, 403, 21, 29));
        window.push_str(&art("basic_interface/btnbar_mid2.bmp", 21, 403, 394, 29));
        window.push_str(&art("basic_interface/btnbar_right2.bmp", 415, 403, 21, 29));
        // The tabs, 17 high from 3 in and 20 down; the one open stands white on the frame of the rows.
        let mut x = 3;
        for (tab, name) in ["Hotkey Bar", "Hotkey Bar 2", "Interface", "Macros"].iter().enumerate() {
            let wide = if tab == 1 { 84 } else { 72 };
            let open = tab == self.keys.tab;
            let press = pressed(Press::Tab(tab));
            window.push_str(&format!(
                r#"<div class="a t" style="{};text-align:center;line-height:17px;cursor:default;border:1px solid #c0c0c0;border-bottom:0;box-sizing:border-box;background:{}" {press}>{}</div>"#,
                at(x, 20, wide, if open { 20 } else { 17 }),
                if open { "#fff" } else { "#e8e8e8" },
                escape(tr(name))
            ));
            x += wide;
        }
        window.push_str(&format!(r#"<div class="a" style="{};border:1px solid #c0c0c0;box-sizing:border-box;pointer-events:none"></div>"#, at(3, 40, 430, 360)));
        window.push_str(&format!(r#"<div class="a" style="{};background:#c0c0c0"></div>"#, at(218, 40, 1, 360)));
        // The keys' tint is the table's: the bar's, the second bar's, or the others' (the client's colours).
        let tint = match table {
            hotkeys::table::BAR => "#d6efff",
            hotkeys::table::BAR_2 => "#ffd6ef",
            _ => "#feffdf",
        };
        for (row, place) in listed.iter().skip(page * PAGE).take(PAGE).enumerate() {
            let (name, _) = data::hotkeys(table).get(*place).cloned().unwrap_or_default();
            let chord = self.bound(table, *place);
            let keys = match hotkeys::written(chord, self.keys.mac, data::key_name) {
                keys if keys.is_empty() => tr("Not Assigned").to_owned(),
                keys => keys,
            };
            let (left, y) = (if row < 18 { 3 } else { 218 }, 40 + (row % 18) as i32 * 20);
            let picked = self.keys.row == Some(row);
            let press = pressed(Press::Row(row));
            window.push_str(&format!(
                r#"<div class="a t" style="{};line-height:19px;text-align:center;overflow:hidden;white-space:nowrap;cursor:default;border-bottom:1px solid #c0c0c0;box-sizing:border-box;background:{}" {press}>{}</div><div class="a t" style="{};line-height:19px;text-align:center;overflow:hidden;white-space:nowrap;cursor:default;border-left:1px solid #c0c0c0;border-bottom:1px solid #c0c0c0;box-sizing:border-box;background:{tint}" {press}>{}</div>"#,
                at(left, y, 135, 20),
                if picked { "#ffe1e1" } else { "transparent" },
                escape(tr(&name)),
                at(left + 135, y, 80, 20),
                escape(&keys)
            ));
        }
        if pages > 1 {
            window.push_str(&press_button(356, 25, 11, "basic_interface/arw_left.bmp", "basic_interface/arw_left.bmp", Press::Page(Step::Less)));
            window.push_str(&press_button(416, 25, 11, "basic_interface/arw_right.bmp", "basic_interface/arw_right.bmp", Press::Page(Step::More)));
            // Which page of how many, in the smaller text in bold (`UIHotKeyWnd::Paint`: (0, 11), bold).
            window.push_str(&format!(r#"<div class="a t" style="{};{};text-align:center">{} / {pages}</div>"#, at(367, 23, 49, 14), Font(0, 11, true).style(), page + 1));
        }
        // What the window says: how a key is set while a row is picked, or why the last key was not taken.
        let says = self.keys.notice.clone().or_else(|| self.keys.row.map(|_| tr("Press a key to assign. Pressing 'ESC' will remove the assigned key.").to_owned()));
        if let (Some(says), Some(row)) = (says, self.keys.row) {
            let y = 40 + ((row % 18) as i32 - 1).max(0) * 20 + if row % 18 == 0 { 20 } else { 0 };
            window.push_str(&format!(
                r#"<div class="a t" style="left:18px;top:{y}px;width:400px;text-align:center;color:{};background:rgba(0,0,0,.75);line-height:20px;pointer-events:none;z-index:2">{}</div>"#,
                if self.keys.notice.is_some() { "#ffff00" } else { "#fff" },
                escape(&says)
            ));
        }
        window.push_str(&picture_press(5, 407, "btn_reset", Press::Reset));
        window.push_str(&picture_press(297, 407, "btn_ok", Press::Ok));
        window.push_str(&picture_press(344, 407, "btn_cancel", Press::Cancel));
        window.push_str(&picture_press(390, 407, "btn_close", Press::Close));
        window
    }
}
