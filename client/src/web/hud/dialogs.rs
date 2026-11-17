//! What is shown in the middle of the screen: an NPC's words and choices, the game options (and what is offered
//! to the dead), the eggs an incubator can hatch.

use super::*;

impl App {
    /// Redraws the NPC's window, where it is on the page.
    pub(in crate::web) fn refresh_dialog(&self) {
        if let Some(dialog) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("dialog")) {
            dialog.set_inner_html(&self.dialog_window());
            if let Ok(Some(text)) = dialog.query_selector(".said") {
                text.set_scroll_top(text.scroll_height());
            }
        }
    }

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
                let buttons: &[(&str, &str)] =
                    if self.hero.dead { &[("esc_10", "esc-save"), ("esc_02", "esc-close")] } else { &[("esc_01", "logout"), ("esc_001", "settings"), ("esc_08", "keys"), ("esc_09", "exit"), ("esc_02", "esc-close")] };
                let height = 17 + if self.hero.dead { 53 } else { 122 };
                let mut window = title_bar(280, "Game Options");
                window.push_str(&art("basic_interface/sys_base_off.bmp", 3, 3, 11, 11));
                window.push_str(&format!(
                    r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0;border-radius:0 0 3px 3px;box-sizing:border-box"></div>"#,
                    at(0, 17, 280, height - 17)
                ));
                for (row, (name, click)) in buttons.iter().enumerate() {
                    window.push_str(&format!(
                        r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="{click}"></button>"#,
                        at(29, 22 + row as i32 * 23, 221, 20),
                        picture(&format!("{name}a.bmp")),
                        picture(&format!("{name}b.bmp")),
                        picture(&format!("{name}c.bmp"))
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
                    picture_button(189, 96, "btn_ok", "esc-yes"),
                    picture_button(235, 96, "btn_cancel", "esc-no")
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
            Some(true) => window.push_str(&picture_button(232, 96, "btn_next", "dialog-next")),
            Some(false) => window.push_str(&picture_button(232, 96, "btn_close", "dialog-close")),
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
                    format!(r#"<div style="{picked}padding:1px 3px;cursor:default" data-click="dialog-pick" data-arg="{index}">{}</div>"#, coloured(choice))
                })
                .collect();
            // The menu: its list 12 in and 26 down, OK and Cancel in the corner.
            window.push_str(&format!(r#"{}<div class="a t scroll" style="{};background:#f0f0f0;white-space:normal">{choices}</div>"#, system_box(124, 280, 120), at(12, 136, 256, 78)));
            window.push_str(&picture_button(188, 220, "btn_ok", "dialog-ok"));
            window.push_str(&picture_button(233, 220, "btn_cancel", "dialog-cancel"));
        } else if let Some(words) = dialog.asks {
            window.push_str(&format!(
                r#"{}<input id="dialog-input" class="a in" style="{};background:#fff;border:1px solid #9a9a9a" type="{}" autocomplete="off" autofocus>"#,
                system_box(124, 280, 120),
                at(12, 160, 256, 18),
                if words { "text" } else { "number" }
            ));
            window.push_str(&picture_button(233, 220, "btn_ok", "dialog-ok"));
        }
        window
    }

    /// What is known of an item, opened with the right button: its large picture, what the server's database
    /// says of it (kind, weight, price and the like), and the client's own description under that.
    /// The eggs an incubator can hatch (the client's `UIPetEggListWnd`): the picked one is hatched with OK (or
    /// with a double click).
    pub(in crate::web) fn egg_window(&self) -> String {
        let Some(eggs) = &self.pets.eggs else { return String::new() };
        self.item_list("Pet Egg List", eggs, Some(self.pets.egg_picked), "egg")
    }

    /// The items a magnifier can identify (the client's `UIItemIdentifyWnd`, titled with its message 521): the
    /// picked one is identified with OK (or with a double click); none is picked at first.
    pub(in crate::web) fn identify_window(&self) -> String {
        let Some(items) = &self.bag.identify else { return String::new() };
        self.item_list("Identify Item", items, self.bag.identify_picked, "identify")
    }

    /// The client's window that lists items to pick one of (`UIItemIdentifyWnd` and those made of it), 200 by
    /// 200: four rows 32 apart from 34 down, each with the item's picture 8 in and its name 40 in, the picked one
    /// on a band 30 high that starts 5 in and 3 above; OK and Cancel 92 and 46 from the right, 24 from the bottom.
    fn item_list(&self, title: &str, places: &[u16], picked: Option<usize>, list: &str) -> String {
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
                    r#"<div style="position:relative;height:32px;cursor:default" data-click="{list}-pick" data-arg="{row}" data-use="{list}:{row}">{band}<div style="position:absolute;left:8px;top:3px;width:24px;height:24px">{}</div><div class="t" style="position:absolute;left:40px;top:3px;white-space:nowrap">{}</div></div>"#,
                    Self::icon(resource.as_deref()),
                    escape(&Self::item_name(item))
                ))
            })
            .collect();
        window.push_str(&format!(r#"<div class="a scroll" style="{}">{rows}</div>"#, at(1, 31, 198, 128)));
        window.push_str(&art("basic_interface/btnbar_left.bmp", 0, 179, 21, 21));
        window.push_str(&art("basic_interface/btnbar_mid.bmp", 21, 179, 158, 21));
        window.push_str(&art("basic_interface/btnbar_right.bmp", 179, 179, 21, 21));
        window.push_str(&picture_button(108, 176, "btn_ok", &format!("{list}-ok")));
        window.push_str(&picture_button(154, 176, "btn_cancel", &format!("{list}-cancel")));
        window
    }
}
