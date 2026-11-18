//! What is shown in the middle of the screen: an NPC's words and choices, the game options (and what is offered
//! to the dead), the eggs an incubator can hatch.

use super::*;
use crate::press::talk::Press;

/// What a list of items to pick one from does: a press on a row, a double click on it, OK and Cancel.
struct Picking {
    pick: fn(usize) -> crate::press::Press,
    used: fn(usize) -> Use,
    ok: crate::press::Press,
    cancel: crate::press::Press,
}

impl App {
    /// Redraws the NPC's window, where it is on the page.
    /// What an NPC says, with "next" or "close" under it; below it, the menu to choose from or the line to type
    /// into. The original's windows (`UISayDialogWnd`, `UIChooseWnd`): 280 by 120, in the frame of the system
    /// boxes, the text on light grey 10 from the edges.
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
                    window.push_str(&format!(
                        r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" {}></button>"#,
                        at(29, 22 + row as i32 * 23, 221, 20),
                        picture(&format!("{name}a.bmp")),
                        picture(&format!("{name}b.bmp")),
                        picture(&format!("{name}c.bmp")),
                        pressed(press)
                    ));
                }
                return window;
            }
            // The save point is asked about first, in the client's message box.
            2 => {
                return format!(
                    r#"{}<div class="a t" style="{};white-space:normal">{}</div>{}{}"#,
                    art("win_msgbox.bmp", 0, 0, 280, 120),
                    at(12, 30, 256, 56),
                    tr("Do you really want to go back to your savepoint?"),
                    picture_press(189, 96, "btn_ok", crate::press::general::Press::EscYes),
                    picture_press(235, 96, "btn_cancel", crate::press::general::Press::EscNo)
                );
            }
            _ => {}
        }
        let Some(dialog) = &self.dialog else { return String::new() };
        let said: String = dialog.lines.iter().map(|line| format!("<div>{}</div>", coloured(line))).collect();
        let mut window = format!(
            r#"{}<div class="a t said scroll" style="{};background:#f2f2f2;padding:2px 4px;box-sizing:border-box;white-space:normal;user-select:text">{said}</div>"#,
            system_box(0, 280, 120),
            at(10, 10, 260, 80)
        );
        match dialog.button {
            Some(true) => window.push_str(&picture_press(232, 96, "btn_next", Press::Next)),
            Some(false) => window.push_str(&picture_press(232, 96, "btn_close", Press::Close)),
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
                    format!(r#"<div style="{picked}padding:1px 3px;cursor:default" {}>{}</div>"#, pressed(Press::Pick(index)), coloured(choice))
                })
                .collect();
            // The menu: its list 12 in and 26 down, OK and Cancel in the corner.
            window.push_str(&format!(r#"{}<div class="a t scroll" style="{};background:#f0f0f0;white-space:normal">{choices}</div>"#, system_box(124, 280, 120), at(12, 136, 256, 78)));
            window.push_str(&picture_press(188, 220, "btn_ok", Press::Ok));
            window.push_str(&picture_press(233, 220, "btn_cancel", Press::Cancel));
        } else if let Some(words) = dialog.asks {
            window.push_str(&format!(
                r#"{}<input id="dialog-input" class="a in" style="{};background:#fff;border:1px solid #9a9a9a" type="{}" autocomplete="off" autofocus>"#,
                system_box(124, 280, 120),
                at(12, 160, 256, 18),
                if words { "text" } else { "number" }
            ));
            window.push_str(&picture_press(233, 220, "btn_ok", Press::Ok));
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

    /// The client's window that lists items to pick one of (`UIItemIdentifyWnd` and those made of it), 200 by
    /// 200: four rows 32 apart from 34 down, each with the item's picture 8 in and its name 40 in, the picked one
    /// on a band 30 high that starts 5 in and 3 above; OK and Cancel 92 and 46 from the right, 24 from the bottom.
    fn item_list(&self, title: &str, places: &[u16], picked: Option<usize>, does: Picking) -> String {
        let mut window = title_bar(200, title);
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border-left:1px solid #c8c8c8;border-right:1px solid #c8c8c8;box-sizing:border-box"></div>"#, at(0, 17, 200, 162)));
        let rows: String = places
            .iter()
            .enumerate()
            .filter_map(|(row, index)| {
                let item = self.bag.items.iter().find(|item| item.index == *index)?;
                let resource = data::item_as(item.id, item.identified).map(|(_, resource)| resource);
                let band = if picked == Some(row) { r#"<div style="position:absolute;left:5px;top:0;width:176px;height:30px;background:#c6d6f7"></div>"# } else { "" };
                Some(format!(
                    r#"<div style="position:relative;height:32px;cursor:default" {} {}>{band}<div style="position:absolute;left:8px;top:3px;width:24px;height:24px">{}</div><div class="t" style="position:absolute;left:40px;top:3px;white-space:nowrap">{}</div></div>"#,
                    pressed((does.pick)(row)),
                    used((does.used)(row)),
                    Self::icon(resource.as_deref()),
                    escape(&Self::item_name(item))
                ))
            })
            .collect();
        window.push_str(&format!(r#"<div class="a scroll" style="{}">{rows}</div>"#, at(1, 31, 198, 128)));
        window.push_str(&art("basic_interface/btnbar_left.bmp", 0, 179, 21, 21));
        window.push_str(&art("basic_interface/btnbar_mid.bmp", 21, 179, 158, 21));
        window.push_str(&art("basic_interface/btnbar_right.bmp", 179, 179, 21, 21));
        window.push_str(&picture_press(108, 176, "btn_ok", does.ok));
        window.push_str(&picture_press(154, 176, "btn_cancel", does.cancel));
        window
    }
}
