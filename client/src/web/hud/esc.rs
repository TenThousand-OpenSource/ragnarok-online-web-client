//! The game options (the client's `UIEscOptionWnd`), and the question they ask of the dead, which stands where
//! an NPC's window stands.

use super::*;

impl App {
    /// Whether the dead character may rise where it lies (the menu's third shape, `UIEscOptionWnd`'s mode 2):
    /// it carries one of the things that raise (`CGameMode::HasResurrectionItem`) on a map that lets them
    /// (`crate::pvp::Zone::raises_with`). (The client asks the same of the state 580, which no classic skill
    /// gives: it is not looked at here.)
    fn rises(&self) -> bool {
        self.pvp.zone.raises_with(self.bag.items.iter().map(|item| (item.id, item.amount)))
    }

    /// The menu's buttons from the top, each the name of its pictures and what it does (`LayoutButtons`).
    fn esc_buttons(&self) -> Vec<(&'static str, crate::press::Press)> {
        use crate::press::{account, general, hotkeys, settings};
        if self.hero.dead {
            let rise = self.rises().then(|| ("esc_05", general::Press::EscRise.into()));
            return rise.into_iter().chain([("esc_10", general::Press::EscSave.into()), ("esc_02", general::Press::EscClose.into())]).collect();
        }
        vec![
            ("esc_01", account::Press::Logout.into()),
            ("esc_001", settings::Press::Open.into()),
            ("esc_08", hotkeys::Press::Open.into()),
            ("esc_09", account::Press::Exit.into()),
            ("esc_02", general::Press::EscClose.into()),
        ]
    }

    /// The client's window of game options (`UIEscOptionWnd`): Escape opens and closes it, and it opens by
    /// itself when the character dies (or closes, if it was open). Alive, it offers the characters, the
    /// settings, the keys, leaving and going back to the game (122 high under the title); dead, the save point
    /// and going back to the game (53), and over them rising where one lies for one who carries a Token of
    /// Siegfried (76). 280 wide, the buttons 29 in and 5 under the title, 23
    /// apart. The title is line 1483 of the message table and, in brackets, the character's number in the
    /// client's other figures (`UIEscOptionWnd::Paint`, `ShortcutFilename`). The shape is that of the moment
    /// the window opened in the client (its constructor); here it follows the character, which differs only
    /// for a menu left open while someone raises the dead.
    pub(in crate::web) fn esc_window(&self) -> String {
        let buttons = self.esc_buttons();
        let height = 17 + 7 + 23 * buttons.len() as i32;
        let code = crate::numbers::coded(self.own_character());
        let mut window = title_bar(280, &format!("{} ({code})", Self::words(1483, "Game Options")));
        window.push_str(&base_button(3, 3));
        window.push_str(&panel_ground_round(0, 17, 280, height - 17));
        for (row, (name, press)) in buttons.into_iter().enumerate() {
            window.push_str(&pictured(
                29,
                22 + row as i32 * 23,
                (221, 20),
                [&format!("{name}a.bmp"), &format!("{name}b.bmp"), &format!("{name}c.bmp")],
                press,
                ""
            ));
        }
        window
    }

    /// The save point is asked about first, in the client's message box (line 1548 of the message table,
    /// `UIEscOptionWnd::SendMsg`).
    pub(super) fn esc_question(&self) -> Option<String> {
        use crate::press::general::Press;
        let words = Self::words(1548, "Do you really want to go back to your savepoint?");
        (self.esc == 2).then(|| confirm_box(&words, Press::EscYes, Press::EscNo))
    }
}
