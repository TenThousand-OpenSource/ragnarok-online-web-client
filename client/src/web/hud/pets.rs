//! The pet window: what the pet is called, how hungry and how close it is, and the commands given to it.

use super::*;
use crate::net::pets::{hunger_word, intimacy_word};
use crate::press::pets::{Order, Press};

impl App {
    /// Redraws the pet window, where it is on the page; a name being typed into it stays.
    /// The client's `UIPetInfoWnd`, 280 by 180: the pet's picture 5 in and 21 down; from 100 across its name
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
        window.push_str(&press_button(266, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::Window));
        window.push_str(&format!(
            r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0;border-radius:0 0 3px 3px;box-sizing:border-box"></div>"#,
            at(0, 17, 280, 163)
        ));
        window.push_str(&art("sysboxs_ld.bmp", 0, 173, 7, 7));
        window.push_str(&art("sysboxs_rd.bmp", 273, 173, 7, 7));
        if let Some(file) = data::pet_picture(pet.job) {
            window.push_str(&format!(r#"<img class="a" alt="" draggable="false" style="left:5px;top:21px" src="{}">"#, picture(&format!("illust/{file}"))));
        }
        for (y, label) in [(24, "Name"), (44, "Level"), (69, "Hunger"), (89, "Intimacy"), (119, "Accessory")] {
            window.push_str(&text(100, y, BLACK, tr(label)));
        }
        window.push_str(&format!(
            r#"<input id="pet-name" class="a in" style="{};background:#e8e8e8;box-sizing:border-box" type="text" maxlength="50" autocomplete="off" value="{}">"#,
            at(135, 24, 80, 16),
            escape(&pet.name)
        ));
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
        window.push_str(&pictured(252, 44, (18, 18), ["basic_interface/txtbox_btn_a.bmp", "basic_interface/txtbox_btn_b.bmp", "basic_interface/txtbox_btn_c.bmp"], Press::Menu, ""));
        window.push_str(&text(145, 69, BLACK, tr(hunger_word(pet.hunger))));
        let (close, through) = intimacy_word(pet.intimacy);
        window.push_str(&text(145, 89, BLACK, tr(close)));
        // The bar (`UIPetInfoBarGraph`, 120 by 9): its ends keep their size, the middle stretches; the blue
        // fills that many hundredths of what lies between its own ends.
        window.push_str(&art("basic_interface/gze_bg_left.bmp", 145, 105, 2, 9));
        window.push_str(&art("basic_interface/gze_bg_mid.bmp", 147, 105, 114, 9));
        window.push_str(&art("basic_interface/gze_bg_right.bmp", 261, 105, 4, 9));
        if through > 0 {
            let filled = 112 * through / 100;
            window.push_str(&art("basic_interface/gzeblue_left.bmp", 145, 105, 4, 8));
            window.push_str(&art("basic_interface/gzeblue_mid.bmp", 149, 105, filled, 8));
            window.push_str(&art("basic_interface/gzeblue_right.bmp", 149 + filled, 105, 4, 8));
        }
        window.push_str(&text(100, 139, BLACK, tr(if pet.accessory == 0 { "Unequipped" } else { "Equipped" })));
        if self.pets.menu {
            let rows: String = [("Feed Pet", Order::Feed), ("Performance", Order::Perform), ("Unequip Accessory", Order::Unequip), ("Return to Egg", Order::Egg)]
                .iter()
                .map(|(words, order)| format!(r#"<div class="t opt" style="height:16px;line-height:16px;padding-left:3px;cursor:default" {}>{}</div>"#, pressed(Press::Do(*order)), escape(tr(words))))
                .collect();
            window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border:1px solid #9a9a9a;box-sizing:content-box;z-index:2">{rows}</div>"#, at(160, 44, 110, 64)));
        }
        window
    }

    /// The menu the right button opens on the player's own pet (the client's `OpenActorMenu`): its status,
    /// feeding, a trick, the accessory off, back into the egg; each a row 16 high, as the pet window's own.
    pub(in crate::web) fn pet_menu(&self) -> String {
        [("Check Pet Status", Order::Info), ("Feed Pet", Order::Feed), ("Performance", Order::Perform), ("Unequip Accessory", Order::Unequip), ("Return to Egg", Order::Egg)]
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
