//! The game options (the client's `UIEscOptionWnd`), which stand where an NPC's window stands, and the
//! question they ask of the dead.

use super::*;

impl App {
    /// The client's window of game options (`UIEscOptionWnd`), or what it asks, while either is open: Escape
    /// opens and closes it, and it opens by itself when the character dies. Alive, it offers the characters,
    /// the settings, the keys, leaving and going back to the game; dead, the save point and going back to the
    /// game. 280 wide, the buttons 29 in and 5 under the title, 23 apart.
    pub(super) fn esc_window(&self) -> Option<String> {
        use crate::press::{Press, account, general, hotkeys, settings};
        match self.esc {
            1 => {
                let buttons: Vec<(&str, Press)> = if self.hero.dead {
                    vec![("esc_10", general::Press::EscSave.into()), ("esc_02", general::Press::EscClose.into())]
                } else {
                    vec![
                        ("esc_01", account::Press::Logout.into()),
                        ("esc_001", settings::Press::Open.into()),
                        ("esc_08", hotkeys::Press::Open.into()),
                        ("esc_09", account::Press::Exit.into()),
                        ("esc_02", general::Press::EscClose.into()),
                    ]
                };
                let height = 17 + if self.hero.dead { 53 } else { 122 };
                let mut window = title_bar(280, "Game Options");
                window.push_str(&base_button(3, 3));
                window.push_str(&panel_ground_round(0, 17, 280, height - 17));
                for (row, (name, press)) in buttons.into_iter().enumerate() {
                    window.push_str(&pictured(29, 22 + row as i32 * 23, (221, 20), [&format!("{name}a.bmp"), &format!("{name}b.bmp"), &format!("{name}c.bmp")], press, ""));
                }
                Some(window)
            }
            // The save point is asked about first, in the client's message box.
            2 => Some(message_box(Asked::Sure, tr("Do you really want to go back to your savepoint?"), general::Press::EscYes, general::Press::EscNo)),
            _ => None,
        }
    }
}
