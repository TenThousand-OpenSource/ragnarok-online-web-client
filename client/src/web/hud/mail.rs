//! The mail (the client's RODEX): the notice of mail not read (`UIOpenMailBoxWnd`), the mailbox with its tabs,
//! its list of six letters a page and its search (`UIRodexInboxWnd`), what the mail asks before it acts, and
//! how many of a thing go into a letter. The letter being read and the one being written are in `mail/letter.rs`.
//!
//! Places, sizes, pictures and colours are those the client's windows are made with (`OnCreate`, `Paint`); its
//! colours are written blue first there and are turned round here. The words are lines of its message table.

use super::*;
use crate::net::mail::{self, Letter, has, tab};
use crate::press::Step;
use crate::press::mail::Press;
use crate::web::app::{MailAsk, epoch};

mod letter;

/// The colour of the numbers by the mailbox's logo (the client's 0xC89482).
const PALE: &str = "#8294c8";

/// One of the mail's pictures.
fn rodex(name: &str) -> String {
    format!("basic_interface/rodexsystem/renewal/{name}.bmp")
}

/// One of the mail's picture buttons (`UIBitmapButton`): `name_out.bmp`, `name_over.bmp` under the pointer,
/// `name_press.bmp` pressed; with what it says of itself when pointed at.
fn picture_button(x: i32, y: i32, size: (i32, i32), name: &str, press: Press, tip: &str) -> String {
    let file = |state: &str| rodex(&format!("{name}_{state}"));
    pictured(x, y, size, [&file("out"), &file("over"), &file("press")], press, &format!(r#" title="{}""#, escape(tip)))
}

/// The client's button with words on it (`UIBasicButton`), 20 high and as wide as asked: ends of 6 that keep
/// their size and a middle that repeats between them, each in three pictures (`btn_out_*`, `btn_over_*`,
/// `btn_press_*`); the words in the middle, black over a grey copy of themselves. Without a press it is the
/// one that cannot be pressed (`btn_disable_*`, its words grey).
fn basic_button(x: i32, y: i32, width: i32, words: &str, press: Option<Press>) -> String {
    let skin = |state: &str| ["left", "right", "mid"].map(|piece| format!("url('{}')", picture(&format!("basic_interface/btn_{state}_{piece}.bmp")))).join(",");
    let (states, colours, press) = match press {
        Some(press) => ([skin("out"), skin("over"), skin("press")], "color:#000;text-shadow:1px 1px 0 #d7d7d7", pressed(press)),
        None => ([skin("disable"), skin("disable"), skin("disable")], "color:#6f6f6f;text-shadow:-1px -1px 0 #e2e2e2", "disabled".to_owned()),
    };
    format!(
        r#"<button type="button" class="a b" style="{};--n:{};--o:{};--p:{};padding:0 6px;background-position:0 0,100% 0,0 0;background-repeat:no-repeat,no-repeat,repeat-x;background-clip:border-box,border-box,content-box;background-origin:border-box,border-box,content-box;white-space:nowrap;{colours};{}" {press}>{}</button>"#,
        at(x, y, width, 20),
        states[0],
        states[1],
        states[2],
        Font(0, 12, false).style(),
        escape(words)
    )
}

/// A line of text with its middle at `x`, in the font the client asks for there (`DrawCenteredText`).
fn text_about(font: Font, x: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="left:{}px;top:{y}px;width:96px;text-align:center;pointer-events:none;color:{colour};{}">{}</div>"#, x - 48, font.style(), escape(words))
}

impl App {
    /// What a field of the page has in it now, if the field is there. What is typed into the mail's windows
    /// stays in their fields between presses; a window drawn anew takes it from the page it replaces, and so
    /// loses none of it when the whole screen is written again.
    pub(in crate::web) fn mail_typed(id: &str) -> Option<String> {
        let field = crate::web::by_id(id)?;
        js_sys::Reflect::get(field.as_ref(), &"value".into()).ok()?.as_string()
    }

    /// The notice of mail not read, 43 by 43 (`UIOpenMailBoxWnd`): one button of one picture, which opens the
    /// mailbox.
    pub(in crate::web) fn mail_notice(&self) -> String {
        let icon = rodex("icon_noti_gotmail");
        pictured(0, 0, (43, 43), [&icon, &icon, &icon], Press::Window, "")
    }

    /// The name of the tab shown, as the questions about all its letters have it.
    fn mail_tab_name(&self) -> String {
        match (self.mail.searching, self.mail.tab) {
            (true, _) => Self::words(3549, "Search"),
            (_, tab::ACCOUNT) => Self::words(3546, "Notice"),
            (_, tab::RETURNED) => Self::words(3548, "Return"),
            _ => Self::words(3547, "General"),
        }
    }

    /// The mailbox, 309 by 416 (`UIRodexInboxWnd`): its picture with the logo; how many letters were sent
    /// today of the hundred a day; the buttons that ask for the mailbox anew and write a letter; the search;
    /// the four tabs; six letters; under them the two buttons for all of the tab or for the rows ticked, and
    /// the pages. Until the server has listed the mailbox it shows the client's picture of waiting instead of
    /// the letters.
    pub(in crate::web) fn mail_window(&self) -> String {
        let mail = &self.mail;
        let mut window = art(&rodex("bg_rodex_list"), 0, 0, 309, 416);
        window.push_str(&art("basic_interface/sys_base_off.bmp", 3, 3, 11, 11));
        window.push_str(&press_button(295, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::Window));
        window.push_str(&picture_button(248, 20, (24, 26), "btn_refresh", Press::Refresh, &Self::words(2849, "Refresh")));
        window.push_str(&picture_button(277, 18, (24, 26), "btn_write", Press::Write, &Self::words(1026, "Write Mail")));
        window.push_str(&self.mail_search());
        for (x, line, name, shown) in [(19, 3546, "Notice", tab::ACCOUNT), (87, 3547, "General", tab::NORMAL), (155, 3548, "Return", tab::RETURNED), (223, 3549, "Search", 3)] {
            let open = if shown == 3 { mail.searching } else { !mail.searching && mail.tab == shown };
            let (out, over) = if open { ("press", "press") } else { ("out", "over") };
            // The tab's name stands three pixels above the middle (`AddText`), pale until the tab is open.
            window.push_str(&format!(
                r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}');line-height:25px;color:{};{}" {}>{}</button>"#,
                at(x, 86, 68, 28),
                picture(&rodex(&format!("btn_listtab_empty_{out}"))),
                picture(&rodex(&format!("btn_listtab_empty_{over}"))),
                picture(&rodex("btn_listtab_empty_press")),
                if open { "#1f3e9c" } else { "#adb5cc" },
                BOLD.style(),
                pressed(Press::Tab(shown)),
                escape(&Self::words(line, name))
            ));
        }
        let now = epoch();
        if mail.loaded {
            for (row, (kind, id)) in mail.rows(now).into_iter().enumerate() {
                if let Some(letter) = mail.store.get(kind, id) {
                    window.push_str(&self.mail_row(row, kind, letter, now));
                }
            }
        } else {
            window.push_str(&art(&rodex("img_notice"), 31, 154, 248, 162));
            window.push_str(&format!(
                r#"<div class="a t" style="{};white-space:normal;text-align:center">{}</div>"#,
                at(31, 229, 248, 40),
                coloured(&Self::words(3190, "Loading mailbox, Please don't delete the^c92114 message^000000~!!"))
            ));
            window.push_str(&format!(
                r#"<div class="a t" style="left:31px;top:270px;width:248px;text-align:center;color:{PALE};{}">{}</div>"#,
                Font(0, 18, true).style(),
                escape(&Self::words(3191, "NOW LOADING.."))
            ));
        }
        // The two buttons are for the rows ticked while any is, and for the whole tab otherwise; they and the
        // pages wait while the mailbox is asked for anew.
        let ticked = mail.checked.contains(&true);
        let (delete, claim) = if ticked { (Self::words(3588, "Delete"), Self::words(3591, "Claim")) } else { (Self::words(3589, "Delete All"), Self::words(3592, "Claim All")) };
        window.push_str(&basic_button(4, 392, 70, &delete, (!mail.waiting).then_some(Press::Delete)));
        window.push_str(&basic_button(84, 392, 70, &claim, (!mail.waiting).then_some(Press::Claim)));
        window.push_str(&picture_button(255, 390, (24, 24), "btn_prev", Press::Page(Step::Less), &Self::words(1297, "Previous")));
        window.push_str(&picture_button(279, 390, (24, 24), "btn_next", Press::Page(Step::More), &Self::words(1298, "Next")));
        let small = Font(0, 11, true);
        window.push_str(&text_ending_in(small, 273, 309, 46, PALE, &mail.sent_today.to_string()));
        window.push_str(&text_in(small, 273, 46, PALE, &Self::words(3034, "/100")));
        window
    }

    /// The search of the mailbox: the two round boxes that say where to look, the titles (the first, on at
    /// first) or the senders, 51 apart; the words to look for, sixteen letters at most; and its button. Enter
    /// in the field searches, as the button does.
    fn mail_search(&self) -> String {
        let mut search = String::new();
        for (x, line, name, by_sender) in [(22, 3192, "Title", false), (73, 3193, "Sender", true)] {
            search.push_str(&format!(
                r#"<div class="a t" style="left:{x}px;top:68px;height:12px;line-height:12px;padding-left:13px;cursor:default;color:#394965;background:url('{}') 0 1px no-repeat" {}>{}</div>"#,
                picture(&rodex(if by_sender == self.mail.by_sender { "checkbox_search_on" } else { "checkbox_search_off" })),
                pressed(Press::SearchIn(by_sender)),
                escape(&Self::words(line, name))
            ));
        }
        let button = picture_button(131, 0, (32, 18), "btn_search", Press::Search, "").replace(r#"type="button""#, r#"type="submit""#);
        search.push_str(&format!(
            r#"<form {} class="a" style="{}"><input id="mail-sought" class="a in" style="{};background:#fff;border:1px solid #adadad;box-sizing:border-box" type="text" maxlength="16" autocomplete="off" value="{}">{button}</form>"#,
            sent(Form::MailSearch),
            at(133, 65, 163, 18),
            at(0, 0, 132, 18),
            escape(&Self::mail_typed("mail-sought").unwrap_or_default())
        ));
        search
    }

    /// A row of the list, 44 high, the first 121 down (`Paint`, `DrawMailIcons`, `DrawExpireTime`,
    /// `SetMailRow`): the box to tick; the picture of a letter read or not (of one returned, among what was
    /// searched for); the title, and under it the sender, each pressed to read and to write back; the picture
    /// of what is in it; how long it still lies there; and for a letter of someone not known the button that
    /// sends it back. One the server has deleted says so in grey and is not pressed.
    fn mail_row(&self, row: usize, kind: u8, letter: &Letter, now: f64) -> String {
        let y = 121 + 44 * row as i32;
        let returned = self.mail.searching && kind == tab::RETURNED;
        let status = if returned { "icon_status_mail_returned" } else if letter.read { "icon_status_mail_read" } else { "icon_status_mail_received" };
        // The client opens a letter by its title alone; here its picture opens it too, and the title is pressed
        // along its whole 140, not only where its letters are (this project's, for a hand that looks for it).
        let mut html = if letter.gone {
            art(&rodex(status), 36, y + 3, 24, 24)
        } else {
            format!(r#"<div class="a" style="{};background:url('{}') 0 0 no-repeat;cursor:pointer" {}></div>"#, at(36, y + 3, 24, 24), picture(&rodex(status)), pressed(Press::Title(row)))
        };
        let inside = match letter.has & (has::ZENY | has::ITEMS) {
            0 => None,
            has::ZENY => Some("icon_zeny"),
            has::ITEMS => Some("icon_item"),
            _ => Some("icon_zeny_n_item"),
        };
        if let Some(inside) = inside {
            html.push_str(&art(&rodex("shadow_item_on"), 216, y + 18, 32, 18));
            html.push_str(&art(&rodex(inside), 219, y + 6, 28, 25));
        }
        let (line, words, count) = mail::time_left((letter.expires - now) as i64);
        let left = Self::words(line, words).replace("%d", &count.to_string());
        if returned {
            html.push_str(&text_about(SMALL, 260, y + 4, "#736666", &Self::words(2853, "Returned")));
            html.push_str(&text_about(SMALL, 260, y + 17, "#736666", &left));
        } else {
            html.push_str(&text_about(Font(0, 11, true), 260, y + 9, "#e90a0a", &left));
        }
        html.push_str(&format!(
            r#"<div class="a" style="{};background:url('{}') 0 0 no-repeat" {}></div>"#,
            at(17, y + 7, 14, 14),
            picture(&rodex(if self.mail.checked[row] { "checkbox_on" } else { "checkbox_off" })),
            pressed(Press::Check(row))
        ));
        // The title and the sender are cut at 140 across (`ResizeToTextClipped`).
        let clipped = "max-width:140px;overflow:hidden;cursor:default";
        let colour = mail::title_colour(kind, letter);
        if letter.gone {
            html.push_str(&format!(r#"<div class="a t" style="left:69px;top:{y}px;{clipped};color:{colour}">{}</div>"#, escape(&Self::words(2907, "The mail has been deleted."))));
            return html;
        }
        html.push_str(&format!(
            r#"<div class="a t" style="left:69px;top:{y}px;width:140px;{clipped};cursor:pointer;color:{colour}" {} title="{}">{}</div>"#,
            pressed(Press::Title(row)),
            escape(&Self::words(2702, "Read mail")),
            escape(&letter.title)
        ));
        html.push_str(&format!(
            r#"<div class="a t" style="left:71px;top:{}px;{clipped};color:#424a84;{}" {} title="{}">{}</div>"#,
            y + 20,
            Font(0, 11, true).style(),
            pressed(Press::Sender(row)),
            escape(&Self::words(2701, "Write reply")),
            escape(&letter.sender)
        ));
        if !letter.known {
            html.push_str(&picture_button(186, y + 10, (42, 20), "btn_return", Press::GiveBack(row), &Self::words(2848, "Return")));
        }
        html
    }

    /// What the mail asks before it acts, in the client's message box (280 by 120, `UIWindowMgr::Confirm`):
    /// OK and Cancel.
    pub(in crate::web) fn mail_question(&self) -> String {
        let words = match &self.mail.asks {
            Some(MailAsk::DeleteAll) => Self::words(3590, "Are you sure you want to delete all mails in the [%s] mailbox?").replace("%s", &self.mail_tab_name()),
            Some(MailAsk::ClaimAll) => Self::words(3594, "Would you like to claim all attachments in the [%s] mailbox?").replace("%s", &self.mail_tab_name()),
            Some(MailAsk::Delete(_)) => Self::words(2854, "Do you want to delete the message?"),
            Some(MailAsk::Claim(_)) => Self::words(3593, "Would you like to claim the attachment of the selected email?"),
            Some(MailAsk::DeleteRead) => Self::words(356, "Are you sure that you want to delete it?"),
            Some(MailAsk::Stranger(..)) => Self::words(3867, "The sender of this RODEX is not a friend or guild member. Please be careful."),
            None => return String::new(),
        };
        message_box(Asked::Sure, &escape(&words), Press::Yes, Press::No)
    }

    /// How many of a thing go into the letter: the window that asks how many to drop, asking this (the
    /// client's `UIItemDropCntWnd` in its thirteenth state), with all of them ready to be typed over.
    pub(in crate::web) fn mail_count_window(&self) -> String {
        self.count_box(&windows::MAIL_COUNT, self.mail.counting, Form::Attach, Press::CountOk)
    }
}
