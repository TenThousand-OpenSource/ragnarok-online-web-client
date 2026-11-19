//! What is shown in the middle of the screen: an NPC's words and choices, the game options (and what is offered
//! to the dead), the eggs an incubator can hatch.

use super::*;
use crate::press::talk::Press;

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
    /// Redraws the NPC's window, where it is on the page.
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
    pub(in crate::web) fn dialog_window(&self) -> String {
        // The client's window of game options (`UIEscOptionWnd`): Escape opens and closes it, and it opens by
        // itself when the character dies. Alive, it offers the characters, the settings, the keys, leaving and
        // going back to the game; dead, the save point and going back to the game. 280 wide, the buttons 29 in
        // and 5 under the title, 23 apart.
        match self.esc {
            1 => {
                use crate::press::{Press, account, general, hotkeys, settings};
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
                window.push_str(&art("basic_interface/sys_base_off.bmp", 3, 3, 11, 11));
                window.push_str(&format!(
                    r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0;border-radius:0 0 3px 3px;box-sizing:border-box"></div>"#,
                    at(0, 17, 280, height - 17)
                ));
                for (row, (name, press)) in buttons.into_iter().enumerate() {
                    window.push_str(&pictured(29, 22 + row as i32 * 23, (221, 20), [&format!("{name}a.bmp"), &format!("{name}b.bmp"), &format!("{name}c.bmp")], press, ""));
                }
                return window;
            }
            // The save point is asked about first, in the client's message box.
            2 => {
                return message_box(Asked::Sure, tr("Do you really want to go back to your savepoint?"), crate::press::general::Press::EscYes, crate::press::general::Press::EscNo);
            }
            _ => {}
        }
        let Some(dialog) = &self.dialog else { return String::new() };
        let said: String = dialog.lines.iter().map(|line| format!("<div>{}</div>", linked(line))).collect();
        // How much lower than the first window one stands that the client puts so far of 480 down the screen.
        let stage = (crate::web::page_size().1 / crate::web::stage_scale()) as i32;
        let lower = |down: i32| stage * down / 480 - stage * 100 / 480;
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
        if !dialog.menu.is_empty() {
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
            // The menu: its list 12 in and 26 down, OK and Cancel in the corner.
            let top = lower(300);
            window.push_str(&format!(r#"{}<div class="a t scroll" style="{};background:#f0f0f0;white-space:normal">{choices}</div>"#, system_box(top, 280, 120), at(12, top + 12, 256, 78)));
            window.push_str(&picture_press(188, top + 96, "btn_ok", Press::Ok));
            window.push_str(&picture_press(233, top + 96, "btn_cancel", Press::Cancel));
        } else if let Some(words) = dialog.asks {
            let (top, width) = (lower(200), if words { 346 } else { 176 });
            window.push_str(&format!(
                r#"{}<input id="dialog-input" class="a in" style="{};background:#e8e8e8" type="{}" autocomplete="off" autofocus{}>"#,
                system_box(top, width, 46),
                at(15, top + 22, width - 77, 16),
                if words { "text" } else { "number" },
                if words { "" } else { r#" value="0" maxlength="12""# }
            ));
            window.push_str(&picture_press(width - 55, top + 14, "btn_ok", Press::Ok));
        }
        window
    }

    /// What is known of an item, opened with the right button: its large picture, what the server's database
    /// says of it (kind, weight, price and the like), and the client's own description under that.
    /// The eggs an incubator can hatch (the client's `UIPetEggListWnd`): the picked one is hatched with OK (or
    /// with a double click).
    pub(in crate::web) fn egg_window(&self) -> String {
        let Some(eggs) = &self.pets.eggs else { return String::new() };
        use crate::press::pets::Press;
        let does = Picking { pick: |row| Press::EggPick(row).into(), used: Use::Egg, ok: Press::EggOk.into(), cancel: Press::EggCancel.into() };
        self.item_list("Pet Egg List", eggs, Some(self.pets.egg_picked), does)
    }

    /// The items a magnifier can identify (the client's `UIItemIdentifyWnd`, titled with its message 521): the
    /// picked one is identified with OK (or with a double click); none is picked at first.
    pub(in crate::web) fn identify_window(&self) -> String {
        let Some(items) = &self.bag.identify else { return String::new() };
        use crate::press::inventory::Press;
        let does = Picking { pick: |row| Press::IdentifyPick(row).into(), used: Use::Identify, ok: Press::IdentifyOk.into(), cancel: Press::IdentifyCancel.into() };
        self.item_list("Identify Item", items, self.bag.identify_picked, does)
    }

    /// The rows of a list of things carried to pick one of (`pick_list`): each thing's picture and name, by
    /// where it is carried.
    fn item_list(&self, title: &str, places: &[u16], picked: Option<usize>, does: Picking) -> String {
        pick_list(title, &self.carried_rows(places), picked, does)
    }
}
