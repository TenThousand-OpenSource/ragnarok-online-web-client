//! The pet window: what the pet is called, how hungry and how close it is, and the commands given to it; the
//! list of eggs an incubator can hatch.

use super::*;
use crate::pets::{hunger_word, intimacy_word};
use crate::press::pets::{Order, Press};

impl App {
    /// The pet window: the client's `UIPetInfoWnd`, 280 by 180: the pet's picture 5 in and 21 down; from 100 across its name
    /// (to type a new one, with the button that sets it), level, hunger, intimacy with a bar of how far through
    /// that stage it is, and its accessory; `< Command >` opens the menu of feeding, performance, taking the
    /// accessory off and going back into the egg.
    pub(in crate::web) fn pet_window(&self) -> String {
        let Some(pet) = &self.pets.pet else { return String::new() };
        // `UIPetInfoWnd::Paint`: the title bar with nothing written in it, and the title 5 in and 2 down; under
        // it the panel every plain window has (`UIWindow::DrawPanelBackground`): the skin's colours 14,6 around
        // and 2,2 inside (grey 200 and white in the client's `colorchip.bmp`), the outline rounded at the bottom,
        // where the two corner pieces lie over it.
        let mut window = title_bar(280, "");
        window.push_str(&text(5, 2, BLACK, tr("Pet Info")));
        window.push_str(&cross(280, Press::Window));
        window.push_str(&panel_ground_round(0, 17, 280, 163));
        window.push_str(&art("sysboxs_ld.bmp", 0, 173, 7, 7));
        window.push_str(&art("sysboxs_rd.bmp", 273, 173, 7, 7));
        if let Some(file) = data::pet_picture(pet.job) {
            window.push_str(&format!(r#"<img class="a" alt="" draggable="false" style="left:5px;top:21px" src="{}">"#, picture(&format!("illust/{file}"))));
        }
        for (y, label) in [(24, "Name"), (44, "Level"), (69, "Hunger"), (89, "Intimacy"), (119, "Accessory")] {
            window.push_str(&text(100, y, BLACK, tr(label)));
        }
        window.push_str(&field("pet-name", (135, 24, 80, 16), &pet.name, 50, &format!("background:{FIELD_GROUND};box-sizing:border-box")));
        window.push_str(&picture_press(230, 21, "btn_rewrite", Press::Rename));
        window.push_str(&text(135, 44, BLACK, &pet.level.to_string()));
        // The box that opens the menu, and the arrow at its right end; the client's unseen button over it starts
        // 10 further left (120 by 18 at 150, 44).
        window.push_str(&format!(r#"<div class="a" style="{}" {}></div>"#, at(150, 44, 10, 18), pressed(Press::Menu)));
        window.push_str(&format!(
            r#"<div class="a t" style="{};background:#e6e6fa;text-align:center;line-height:18px;cursor:default" {}>&lt; Command &gt;</div>"#,
            at(160, 44, 92, 18),
            pressed(Press::Menu)
        ));
        window.push_str(&pictured_by(Convention::ABC, 252, 44, (18, 18), "basic_interface/txtbox_btn", Press::Menu, ""));
        window.push_str(&text(145, 69, BLACK, tr(hunger_word(pet.hunger))));
        let (close, through) = intimacy_word(pet.intimacy);
        window.push_str(&text(145, 89, BLACK, tr(close)));
        // The bar (`UIPetInfoBarGraph`, 120 by 9): its ends keep their size, the middle stretches; the blue
        // fills that many hundredths of what lies between its own ends.
        window.push_str(&art("basic_interface/gze_bg_left.bmp", 145, 105, 2, 9));
        window.push_str(&art("basic_interface/gze_bg_mid.bmp", 147, 105, 114, 9));
        window.push_str(&art("basic_interface/gze_bg_right.bmp", 261, 105, 4, 9));
        if through > 0 {
            window.push_str(&filling(145, 105, 112, i64::from(through), 100, Red::Never));
        }
        window.push_str(&text(100, 139, BLACK, tr(if pet.accessory == 0 { "Unequipped" } else { "Equipped" })));
        if self.pets.menu {
            let rows: String = [
                ("Feed Pet", Order::Feed),
                ("Performance", Order::Perform),
                ("Unequip Accessory", Order::Unequip),
                ("Return to Egg", Order::Egg)
            ]
                .iter()
                .map(|(words, order)| format!(
                    r#"<div class="t opt" style="height:16px;line-height:16px;padding-left:3px;cursor:default" {}>{}</div>"#,
                    pressed(Press::Do(*order)),
                    escape(tr(words))
                ))
                .collect();
            window.push_str(&format!(
                r#"<div class="a" style="{};background:#fff;border:1px solid #9a9a9a;box-sizing:content-box;z-index:2">{rows}</div>"#,
                at(160, 44, 110, 64)
            ));
        }
        window
    }

    /// The eggs an incubator can hatch (the client's `UIPetEggListWnd`): the picked one is hatched with OK (or
    /// with a double click).
    pub(in crate::web) fn egg_window(&self) -> String {
        let Some(eggs) = &self.pets.eggs else { return String::new() };
        let does = Picking { pick: |row| Press::EggPick(row).into(), used: Use::Egg, ok: Press::EggOk.into(), cancel: Press::EggCancel.into() };
        pick_list("Pet Egg List", &self.carried_rows(eggs), Some(self.pets.egg_picked), does)
    }

    /// The menu the right button opens on the player's own pet (the client's `OpenActorMenu`): its status,
    /// feeding, a trick, the accessory off, back into the egg; each a row 16 high, as the pet window's own.
    pub(in crate::web) fn pet_menu(&self) -> String {
        [
            ("Check Pet Status", Order::Info), ("Feed Pet", Order::Feed), ("Performance", Order::Perform),
            ("Unequip Accessory", Order::Unequip), ("Return to Egg", Order::Egg)
        ]
            .iter()
            .map(|(words, order)| menu_row(tr(words), Press::Do(*order)))
            .collect()
    }

    /// What the pet window asks before it acts, in the client's message box (280 by 120): whether to feed the
    /// pet, or whether to give it the name typed.
    pub(in crate::web) fn pet_question(&self) -> String {
        let words = match self.pets.asks {
            1 => tr("Are you sure that you want to feed your pet?").to_owned(),
            2 => tr("Pet name will be changed to {}, do you want to continue?").replace("{}", &self.pets.named),
            _ => return String::new(),
        };
        message_box(Asked::Sure, &escape(&words), Press::Yes, Press::No)
    }
}
