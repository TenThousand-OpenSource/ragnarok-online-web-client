//! What the two sides of the client's `UIMessengerGroupWnd` share, the Party side (`hud/party.rs`) and the
//! Friend side (`hud/friends.rs`): what the window has under its title, the strip its bar of buttons stands
//! on, and a button of that bar.

use super::*;

/// A button of the bar under the list: 26 by 20, its three pictures `name.bmp`, `name_a.bmp`, `name_b.bmp`.
pub(in crate::web::hud) fn bar_button(x: i32, y: i32, name: &str, press: impl Into<crate::press::Press>, tip: &str) -> String {
    pictured_by(Convention::AB, x, y, (26, 20), &format!("basic_interface/{name}"), press, &format!(r#" data-tip="{}""#, escape(tip)))
}

/// The strip the bar's buttons stand on, `y` down a window that wide (`UIMessengerGroupWnd::Paint`: its two
/// ends 7 wide and the middle between them).
pub(in crate::web::hud) fn bar_back(width: i32, y: i32) -> String {
    format!(
        r#"{}<div class="a" style="{};background:url('{}') 0 0 repeat-x"></div>{}"#,
        art("basic_interface/mesbtn_left.bmp", 0, y, 7, 20),
        at(7, y, width - 14, 20),
        picture("basic_interface/mesbtn_mid.bmp"),
        art("basic_interface/mesbtn_right.bmp", width - 7, y, 7, 20)
    )
}

impl App {
    /// What the window has under its title on both of its sides, the Party one and the Friend one
    /// (`hud/friends.rs`): the button by the cross that makes it small, or large again (`ID_MINI`,
    /// `ID_RESTORE_PARTY_LIST`), the cross, the white ground of the list, and the bar its tabs stand on.
    pub(in crate::web::hud) fn messenger_frame(&self, width: i32, height: i32) -> String {
        [
            if self.party.small {
                pictured_by(Convention::OutOverPress, width - 28, 3, (10, 10), "renewalparty/btn_titlebar_maximum", Press::Size, "")
            } else {
                press_button(width - 28, 3, 11, "basic_interface/sys_base_off.bmp", "basic_interface/sys_base_on.bmp", Press::Size)
            },
            cross(width, Press::Window),
            format!(r#"<div class="a" style="{};background:#fff;border:1px solid #c0c0c0;box-sizing:border-box"></div>"#, at(0, 17, width, height - 38)),
            button_bar(width, height),
        ]
        .concat()
    }
}
