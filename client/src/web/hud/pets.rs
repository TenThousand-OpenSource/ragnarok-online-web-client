//! The pet window: what the pet is called, how hungry and how close it is, and the commands given to it.

use super::*;
use crate::net::pets::{hunger_word, intimacy_word};

impl App {
    /// Redraws the pet window, where it is on the page; a name being typed into it stays.
    pub(in crate::web) fn refresh_pet(&self) {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else { return };
        use wasm_bindgen::JsCast;
        let name = || document.get_element_by_id("pet-name").and_then(|input| input.dyn_into::<web_sys::HtmlInputElement>().ok());
        let typed = document.active_element().filter(|active| active.id() == "pet-name").and_then(|_| name()).map(|input| input.value());
        if let Some(window) = document.get_element_by_id("pet") {
            window.set_inner_html(&self.pet_window());
        }
        if let (Some(typed), Some(input)) = (typed, name()) {
            input.set_value(&typed);
            let _ = input.focus();
        }
        if let Some(asked) = document.get_element_by_id("pet-asks") {
            asked.set_inner_html(&self.pet_question());
        }
    }

    /// The client's `UIPetInfoWnd`, 280 by 180: the pet's picture 5 in and 21 down; from 100 across its name
    /// (to type a new one, with the button that sets it), level, hunger, intimacy with a bar of how far through
    /// that stage it is, and its accessory; `< Command >` opens the menu of feeding, performance, taking the
    /// accessory off and going back into the egg.
    pub(in crate::web) fn pet_window(&self) -> String {
        let Some(pet) = &self.pet else { return String::new() };
        let mut window = title_bar(280, "Pet Info");
        window.push_str(&button(266, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "pet", ""));
        window.push_str(&format!(
            r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0;border-radius:0 0 3px 3px;box-sizing:border-box"></div>"#,
            at(0, 17, 280, 163)
        ));
        if let Some(file) = data::pet_picture(pet.job) {
            window.push_str(&format!(r#"<img class="a" alt="" draggable="false" style="left:5px;top:21px" src="{}">"#, picture(&format!("illust/{file}"))));
        }
        for (y, label) in [(24, "Name"), (44, "Level"), (69, "Hunger"), (89, "Intimacy"), (119, "Accessory")] {
            window.push_str(&text(100, y, BLACK, tr(label)));
        }
        window.push_str(&format!(
            r#"<input id="pet-name" class="a in" style="{};background:#e8e8e8;box-sizing:border-box" type="text" maxlength="23" autocomplete="off" value="{}">"#,
            at(135, 24, 80, 16),
            escape(&pet.name)
        ));
        window.push_str(&picture_button(230, 21, "btn_rewrite", "pet-rename"));
        window.push_str(&text(135, 44, BLACK, &pet.level.to_string()));
        // The box that opens the menu, and the arrow at its right end.
        window.push_str(&format!(
            r#"<div class="a t" style="{};background:#e6e6fa;text-align:center;line-height:18px;cursor:default" data-click="pet-menu">&lt; Command &gt;</div>"#,
            at(160, 44, 92, 18)
        ));
        window.push_str(&format!(
            r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="pet-menu"></button>"#,
            at(252, 44, 18, 18),
            picture("basic_interface/txtbox_btn_a.bmp"),
            picture("basic_interface/txtbox_btn_b.bmp"),
            picture("basic_interface/txtbox_btn_c.bmp")
        ));
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
        if self.pet_menu {
            let rows: String = [("Feed Pet", "feed"), ("Performance", "perform"), ("Unequip Accessory", "unequip"), ("Return to Egg", "egg")]
                .iter()
                .map(|(words, order)| format!(r#"<div class="t opt" style="height:16px;line-height:16px;padding-left:3px;cursor:default" data-click="pet-do" data-arg="{order}">{}</div>"#, escape(tr(words))))
                .collect();
            window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border:1px solid #9a9a9a;box-sizing:content-box;z-index:2">{rows}</div>"#, at(160, 44, 110, 64)));
        }
        window
    }

    /// What the pet window asks before it acts, in the client's message box (280 by 120): whether to feed the
    /// pet, or whether to give it the name typed.
    pub(in crate::web) fn pet_question(&self) -> String {
        let words = match self.pet_asks {
            1 => tr("Are you sure that you want to feed your pet?").to_owned(),
            2 => tr("Pet name will be changed to {}, do you want to continue?").replace("{}", &self.pet_named),
            _ => return String::new(),
        };
        format!(
            r#"{}<div class="a t" style="{};white-space:normal">{}</div>{}{}"#,
            art("win_msgbox.bmp", 0, 0, 280, 120),
            at(12, 30, 256, 56),
            escape(&words),
            picture_button(189, 96, "btn_ok", "pet-yes"),
            picture_button(235, 96, "btn_cancel", "pet-no")
        )
    }
}
