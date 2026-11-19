//! What is shown in the middle of the screen: an NPC's words, its choices and what it asks to be typed. The
//! game options, which stand in the same place, are `dialogs/esc.rs`.

use super::*;
use crate::press::talk::Press;
use crate::web::app::Dialog;

mod esc;

/// A line an NPC says, as HTML: its colours (`coloured`), and each link written into it (`crate::links`) as
/// the client's viewer shows one, its words underlined in the colour of links (`CreateLinkWindow`). A link to
/// an item opens what is known of the item when pressed, one to a tip opens the Tipbox on it; a line of the
/// message table named in the text is written out. A place for the navigation, a quest and an address of the
/// web show their words and are not pressed: this client has no navigation, and it opens no address a
/// server names.
fn linked(line: &str) -> String {
    use crate::links::{self, Kind};
    let (marked, found) = links::parted(line);
    if found.is_empty() {
        return coloured(line);
    }
    links::put_back(&coloured(&marked), &found, |link| {
        let does = match (link.kind, link.number()) {
            (Kind::Message, number) => return escape(&number.and_then(|line| data::message(line as usize)).unwrap_or_default()),
            (Kind::Item, Some(item)) => format!(" {}", pressed(Press::Item(item))),
            (Kind::Tip, Some(tip)) => format!(" {}", pressed(crate::press::tips::Press::Tip(tip))),
            _ => String::new(),
        };
        let colour = if link.kind == Kind::Url { links::URL_COLOUR } else { links::LINK_COLOUR };
        format!(r#"<span style="color:{colour};text-decoration:underline;cursor:{}"{does}>{}</span>"#, if does.is_empty() { "default" } else { "pointer" }, escape(&link.words))
    })
}

impl App {
    /// What an NPC says, with "next" or "close" under it; below it, the menu to choose from or the line to type
    /// into. The original's windows, in the frame of the system boxes: `UISayDialogWnd` is 280 by 180, its text
    /// (`UITagTextViewer`) on light grey 10 from the edges and 40 short of the height, begun 3 in and 4 down,
    /// broken at 65 less than the window's width, each line 6 under the one before (`CCreateLineInfo`), the
    /// button 4 from the right and 24 from the bottom; `UIChooseWnd` is 280 by 120. The client puts the first
    /// 100 of 480 down the screen and the menu 300 of 480 down it (`UIWindowMgr::MakeWindow`,
    /// `ScaleByScreenHeight`): so much lower the menu stands here, by the stage's height. The boxes that ask
    /// for a number or for words (`UINpcEditDialogWnd`, 176 by 46, and `UINpcTextEditDialogWnd`, 346 by 46)
    /// stand 200 of 480 down, over the foot of what is said, as in the client: the line 15,22 and 77 short of
    /// the box's width on its grey, OK 55 from the right and 32 from the bottom. (That their frame is that
    /// of the system boxes is taken from `UIPasswordWnd`, which is made alike.)
    ///
    /// The game options and what they ask stand in this window's place while they are open (`esc_window`).
    pub(in crate::web) fn dialog_window(&self) -> String {
        if let Some(options) = self.esc_window() {
            return options;
        }
        let Some(dialog) = &self.dialog else { return String::new() };
        // How much lower than the first window one stands that the client puts so far of 480 down the screen.
        let stage = (crate::web::page_size().1 / crate::web::stage_scale()) as i32;
        let lower = |down: i32| stage * down / 480 - stage * 100 / 480;
        let mut window = Self::said_box(dialog);
        if !dialog.menu.is_empty() {
            window.push_str(&Self::choices_box(dialog, lower(300)));
        } else if let Some(words) = dialog.asks {
            window.push_str(&Self::asked_box(words, lower(200)));
        }
        window
    }

    /// What the NPC says, and "next" or "close" under it.
    fn said_box(dialog: &Dialog) -> String {
        let said: String = dialog.lines.iter().map(|line| format!("<div>{}</div>", linked(line))).collect();
        let mut window = format!(
            r#"{}<div class="a t said scroll" style="{};background:#f2f2f2;padding:1px 42px 0 3px;line-height:{}px;box-sizing:border-box;white-space:normal;user-select:text">{said}</div>"#,
            system_box(0, 280, 180),
            at(10, 10, 260, 140),
            crate::font::cell(0, 12) + 6
        );
        match dialog.button {
            Some(true) => window.push_str(&picture_press(232, 156, "btn_next", Press::Next)),
            Some(false) => window.push_str(&picture_press(232, 156, "btn_close", Press::Close)),
            None => {}
        }
        window
    }

    /// The menu to choose from, `top` down: its list 12 in and 26 down, OK and Cancel in the corner.
    fn choices_box(dialog: &Dialog, top: i32) -> String {
        let choices: String = dialog
            .menu
            .iter()
            .enumerate()
            .filter(|(_, choice)| !choice.is_empty())
            .map(|(index, choice)| {
                let picked = if index == dialog.picked { "background:#c6d6f7;" } else { "" };
                format!(r#"<div style="{picked}padding:1px 3px;cursor:default" {}>{}</div>"#, pressed(Press::Pick(index)), linked(choice))
            })
            .collect();
        let mut window = format!(r#"{}<div class="a t scroll" style="{};background:#f0f0f0;white-space:normal">{choices}</div>"#, system_box(top, 280, 120), at(12, top + 12, 256, 78));
        window.push_str(&picture_press(188, top + 96, "btn_ok", Press::Ok));
        window.push_str(&picture_press(233, top + 96, "btn_cancel", Press::Cancel));
        window
    }

    /// The box that asks for words (`words`) or for a number, `top` down, with its OK.
    fn asked_box(words: bool, top: i32) -> String {
        let width = if words { 346 } else { 176 };
        let mut window = format!(
            r#"{}<input id="dialog-input" class="a in" style="{};background:#e8e8e8" type="{}" autocomplete="off" autofocus{}>"#,
            system_box(top, width, 46),
            at(15, top + 22, width - 77, 16),
            if words { "text" } else { "number" },
            if words { "" } else { r#" value="0" maxlength="12""# }
        );
        window.push_str(&picture_press(width - 55, top + 14, "btn_ok", Press::Ok));
        window
    }
}
