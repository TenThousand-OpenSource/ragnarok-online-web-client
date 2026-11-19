//! A letter of the mail on the screen: the one being read (the client's `UIRodexContentsWnd`) and the one being
//! written (`UIRodexComposeWnd`). Both are 300 by 400 over the same picture, with the five places for things
//! and the zeny in the same spots.

use super::*;
use crate::net::packets::parameter;

/// The five places for things, 34 apart, each a shadow (`shadow_item_off.bmp`).
fn places() -> String {
    (0..mail::MOST_ITEMS as i32).map(|place| art(&rodex("shadow_item_off"), 26 + 34 * place, 319, 32, 18)).collect()
}

/// A thing in one of the places: the shadow lit, the thing's picture with how many there are, its name when
/// pointed at. `drag` is what taking it out of its place carries, if it can be taken.
fn thing(place: usize, item: &Item, drag: Option<Drag>) -> String {
    let x = 30 + 34 * place as i32;
    let resource = data::item_as(item.id, item.identified).map(|(_, resource)| resource);
    format!(
        r#"{}<div class="a" style="{}" {} data-info="{}{}" title="{}">{}{}</div>"#,
        art(&rodex("shadow_item_on"), x - 4, 319, 32, 18),
        at(x, 309, 24, 24),
        drag.map(dragged).unwrap_or_default(),
        item.id,
        if item.identified { "" } else { "?" },
        escape(&App::item_name(item)),
        App::icon(resource.as_deref()),
        count(item.amount)
    )
}

/// The frame both windows share: the picture, the round button of every window at the left of its bar, the
/// cross at the right.
fn frame(close: Press) -> String {
    format!(
        "{}{}{}",
        art(&rodex("bg_rodex_read"), 0, 0, 300, 400),
        art("basic_interface/sys_base_off.bmp", 3, 3, 11, 11),
        press_button(286, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", close)
    )
}

impl App {
    /// The letter being read (`UIRodexContentsWnd::OnCreate`, `SetMailContents`, `Paint`): "RODEX" in the bar;
    /// the sender in bold blue 8,29 and the title under it; the text from 15,81, 267 by 192; the things with
    /// the button that takes them at 252,312; the zeny with its button at 252,345; Delete at the bottom left,
    /// and Reply at the right for a letter a player sent. A letter of someone not known has the button that
    /// sends it back by the sender's name.
    ///
    /// Not here: the client reads tags in the title and the text (links to places and items, lines of its
    /// message table); they are shown as they are written.
    pub(in crate::web) fn mail_read_window(&self) -> String {
        let Some((kind, letter)) = self.mail.reading.and_then(|(kind, id)| Some((kind, self.mail.store.get(kind, id)?))) else { return String::new() };
        let mut window = frame(Press::ReadClose);
        window.push_str(&format!(r#"<div class="a t titled" style="left:17px;top:1px">{}</div>"#, escape(&Self::words(2727, "RODEX"))));
        window.push_str(&text_in(BOLD, 8, 29, "#0f3395", &letter.sender));
        window.push_str(&text(9, 52, "#313131", &letter.title));
        let from_player = kind == tab::NORMAL && letter.has & has::NPC == 0;
        if from_player && !letter.known {
            window.push_str(&basic_button(224, 29, 70, &Self::words(3548, "Return"), Some(Press::ReadGiveBack)));
        }
        window.push_str(&format!(
            r#"<div class="a t viewer" style="{};white-space:pre-wrap;overflow-wrap:anywhere;color:#313131;user-select:text">{}</div>"#,
            at(15, 81, 267, 192),
            escape(&letter.body)
        ));
        window.push_str(&places());
        for (place, item) in letter.items.iter().take(mail::MOST_ITEMS).enumerate() {
            window.push_str(&thing(place, item, None));
        }
        // A button that was pressed waits for the server's answer, dimmed; what was taken has no button.
        let take = |y: i32, waits: bool, press: Press| {
            if waits { format!(r#"<div class="a dim" style="{};background:url('{}') 0 0 no-repeat"></div>"#, at(252, y, 26, 20), picture(&rodex("btn_receive_out"))) } else { picture_button(252, y, (26, 20), "btn_receive", press, "") }
        };
        if !letter.items.is_empty() {
            window.push_str(&take(312, self.mail.taking.0, Press::TakeItems));
        }
        window.push_str(&art(&rodex("icon_zeny"), 19, 342, 28, 25));
        if letter.zeny > 0 {
            window.push_str(&text_in(Font(0, 11, true), 50, 350, "#737373", &format!("{} Zeny", grouped(letter.zeny))));
            window.push_str(&take(345, self.mail.taking.1, Press::TakeZeny));
        }
        window.push_str(&basic_button(6, 376, 50, &Self::words(3542, "Delete"), Some(Press::ReadDelete)));
        if from_player {
            window.push_str(&basic_button(224, 376, 70, &Self::words(3543, "Reply"), Some(Press::Reply)));
        }
        window
    }

    /// The letter being written (`UIRodexComposeWnd::OnCreate`, `Paint`, `SetRecipient`): the receiver's name
    /// 8,26 with the button that asks the server who that is, and once it has answered the name in bold blue
    /// with the receiver's level and job on a balloon beside it; the title 6,50; the text from 15,81, a
    /// thousand letters at most, counted under it; the weight of the things put in of the 2000 a letter
    /// takes; the things, dragged in from the inventory and out to it; the zeny to send beside what the
    /// character has; the fee at the bottom left and Send at the right.
    ///
    /// What is typed is taken from the fields on the page as the window is drawn anew (`mail_typed`). The count
    /// of letters and the fee follow the typing by a line of the page's own script each, since nothing else
    /// listens to these fields; the colours of the fee and the zeny (red when the character cannot pay) are
    /// those of the last time the window was drawn.
    ///
    /// Not here: the third line on the balloon, a code of the receiver's character whose making was not found
    /// (`CSession::ShortcutFilename`); the tags of items and skills the client lets be put into the text.
    pub(in crate::web) fn mail_write_window(&self) -> String {
        let Some(writing) = &self.mail.writing else { return String::new() };
        let typed = |id: &str, kept: &str| Self::mail_typed(id).unwrap_or_else(|| kept.to_owned());
        let mut window = frame(Press::WriteClose);
        let field = "border:1px solid #a6a6a6;background:#fff;box-sizing:border-box";
        match &writing.receiver {
            Some(receiver) => {
                window.push_str(&format!(
                    r#"<div class="a t" style="{};{field};padding:0 3px;line-height:16px;color:#0f3395;overflow:hidden;{}">{}</div>"#,
                    at(8, 26, 150, 18),
                    BOLD.style(),
                    escape(&receiver.name)
                ));
                window.push_str(&art(&rodex("bg_id_balloon"), 161, 21, 130, 42));
                window.push_str(&text_in(SMALL, 172, 25, "#c62828", &format!("Lv{}", receiver.level)));
                window.push_str(&text_in(SMALL, 172, 37, "#c62828", &data::job_name(receiver.job)));
            }
            None => {
                window.push_str(&format!(
                    r#"<input id="mail-to" class="a in" style="{};{field};color:#313131" type="text" maxlength="50" autocomplete="off" value="{}">"#,
                    at(8, 26, 150, 18),
                    escape(&typed("mail-to", &writing.to))
                ));
                window.push_str(&format!(
                    r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}');color:#313131;{}" {}>{}</button>"#,
                    at(162, 26, 70, 18),
                    picture(&rodex("btn_confirm_id_empty_out")),
                    picture(&rodex("btn_confirm_id_empty_over")),
                    picture(&rodex("btn_confirm_id_empty_press")),
                    Font(0, 12, false).style(),
                    pressed(Press::Validate),
                    escape(&Self::words(3545, "Validate"))
                ));
            }
        }
        window.push_str(&format!(
            r#"<input id="mail-title" class="a in" style="{}" type="text" maxlength="50" autocomplete="off" value="{}">"#,
            at(6, 50, 155, 14),
            escape(&typed("mail-title", &writing.title))
        ));
        let body = typed("mail-body", &writing.body);
        let pale = Font(0, 10, false);
        window.push_str(&format!(
            r#"<textarea id="mail-body" class="a in viewer" style="{};resize:none;color:#313131;line-height:15px;white-space:pre-wrap" maxlength="{}" oninput="document.getElementById('mail-letters').textContent=this.value.length">{}</textarea>"#,
            at(15, 81, 267, 192),
            mail::MOST_TEXT,
            escape(&body)
        ));
        window.push_str(&format!(r#"<div id="mail-letters" class="a t" style="right:42px;top:276px;color:#446ed6;{}">{}</div>"#, pale.style(), body.chars().count()));
        window.push_str(&text_in(pale, 261, 276, "#446ed6", &format!("/ {}", mail::MOST_TEXT)));
        window.push_str(&art(&rodex("bg_weight"), 199, 305, 82, 32));
        window.push_str(&text_ending_in(SMALL, 244, 300, 316, "#737373", &writing.weight.to_string()));
        window.push_str(&text_in(SMALL, 245, 316, "#737373", &Self::words(3035, "/2000")));
        window.push_str(&places());
        for (place, item) in writing.items.iter().take(mail::MOST_ITEMS).enumerate() {
            window.push_str(&thing(place, item, Some(Drag::Attached(item.index))));
        }
        window.push_str(&self.mail_write_zeny(writing.items.len(), &typed("mail-zeny", &writing.zeny)));
        window.push_str(&basic_button(226, 376, 70, &Self::words(3544, "Send"), Some(Press::Send)));
        window
    }

    /// The zeny of a letter being written (`UIRodexComposeWnd::Paint`): the white box 35,347 with the field
    /// in it, thirteen letters at most; after a stroke what the character has; and at the bottom left the fee
    /// for so many things and that much zeny (`mail::fee`).
    fn mail_write_zeny(&self, things: usize, typed: &str) -> String {
        let has = self.hero.params.get(&parameter::ZENY).copied().unwrap_or(0);
        let sent = mail::zeny_typed(typed);
        let fee = mail::fee(things, sent);
        let short = sent.saturating_add(fee) > has;
        let grey = Font(0, 11, false);
        let mut zeny = format!(r#"<div class="a" style="{};background:#fff;border:1px solid #a6a6a6;box-sizing:border-box"></div>"#, at(35, 347, 95, 16));
        zeny.push_str(&art(&rodex("icon_zeny"), 19, 342, 28, 25));
        // The fee follows the typing: 2500 a thing and two hundredths of the zeny, as `mail::fee` counts it.
        zeny.push_str(&format!(
            r#"<input id="mail-zeny" class="a in" style="{};color:{}" type="text" inputmode="numeric" maxlength="13" autocomplete="off" value="{}" oninput="document.getElementById('mail-fee').textContent=({things}*2500+Math.floor((parseInt(this.value.replace(/,/g,''))||0)*2/100)).toLocaleString('en-US')">"#,
            at(45, 348, 84, 14),
            if short && !typed.is_empty() { "#ff0000" } else { "#000" },
            escape(typed)
        ));
        zeny.push_str(&text_in(grey, 135, 350, "#737373", "/"));
        zeny.push_str(&text_in(Font(0, 11, true), 144, 350, "#737373", &format!("{} Zeny", grouped(has))));
        zeny.push_str(&art(&rodex("bg_tax"), 9, 377, 112, 18));
        zeny.push_str(&format!(r#"<div id="mail-fee" class="a t" style="right:196px;top:382px;color:{};{}">{}</div>"#, if short { "#cf714e" } else { "#000" }, grey.style(), grouped(fee)));
        zeny
    }
}
