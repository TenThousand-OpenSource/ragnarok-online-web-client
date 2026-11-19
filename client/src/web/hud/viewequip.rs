//! Looking at another player's equipment: the window that shows it (the client's `UIEquipWnd` with
//! `m_viewOther`, 280 by 167), and the box of the player's own Equipment window that lets others look. The
//! places of the things worn are drawn by the Equipment window's own code (`hud/equipment.rs`).

use super::*;
use crate::press::viewequip::Press;

impl App {
    /// The window (`UIEquipWnd::Paint` and `OnCreate` with `m_viewOther`): whose equipment it is in the
    /// title (line 1361 of the message table), the strip of the tabs, the client's `equipwin_bg.bmp` 37 down
    /// with the other's character on it, and the things worn in their places, to be looked at only. (Of the
    /// client's two tabs, the equipment and the costumes, the first is made, as in the Equipment window.)
    pub(in crate::web) fn view_equip_window(&self) -> String {
        let Some(seen) = &self.viewequip.seen else { return String::new() };
        let mut window = title_bar(280, &Self::words(1361, "%s's Equipment").replace("%s", &seen.name));
        window.push_str(&press_button(266, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::Close));
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border-left:1px solid #c8c8c8;border-right:1px solid #c8c8c8;box-sizing:border-box"></div>"#, at(0, 17, 280, 20)));
        window.push_str(&art("basic_interface/equipwin_bg.bmp", 0, 37, 280, 130));
        window.push_str(&format!(
            r#"<div class="a" style="{};overflow:hidden;pointer-events:none"><canvas id="pc-viewequip" width="{}" height="{}" style="position:absolute;left:0;top:-25px"></canvas></div>"#,
            at(61, 37, super::super::figure::WIDTH as i32, 130),
            super::super::figure::WIDTH,
            super::super::figure::HEIGHT
        ));
        window.push_str(&Self::worn_places(&seen.items, false));
        window
    }

    /// The box of the player's own Equipment window, `y` down it (`UIEquipWnd::OnCreate`: `allowViewCheck`,
    /// 10 in and 15 above the window's foot): "Show Equipment", ticked while others may look.
    pub(in crate::web) fn equipment_open_box(&self, y: i32) -> String {
        check_words(10, y, self.viewequip.allowed, &Self::words(1362, "Show Equipment"), Press::Allow)
    }

    /// The other's character, as the window shows it: standing, facing the screen, with the weapon worn.
    pub(in crate::web) fn seen_figure(&self) -> Option<crate::web::figure::Figure> {
        let seen = self.viewequip.seen.as_ref()?;
        Some(crate::web::figure::Figure {
            job: seen.job,
            sex: seen.sex,
            hair_style: seen.hair_style,
            hair_colour: seen.hair_colour,
            headgear: seen.headgear,
            body_palette: seen.body_palette,
            weapon: seen.items.iter().find(|item| item.worn & 2 != 0).map_or(0, |item| item.id),
            shield: seen.items.iter().find(|item| item.worn & 0x22 == 0x20).map_or(0, |item| item.id),
            action: 0,
            frame: 0,
        })
    }
}
