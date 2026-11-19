//! The mail (the client's RODEX): the notice of mail not read (`UIOpenMailBoxWnd`), the mailbox with its tabs
//! and its list of six letters a page (`UIRodexInboxWnd`), what the mail asks before it acts, and
//! how many of a thing go into a letter. The rows of the list and the search are in `mail/list.rs`, the letter
//! being read and the one being written in `mail/letter.rs`.
//!
//! Places, sizes, pictures and colours are those the client's windows are made with (`OnCreate`, `Paint`); its
//! colours are written blue first there and are turned round here. The words are lines of its message table.

use super::*;
use crate::mail::{has, tab};
use crate::mail::Letter;
use crate::press::Step;
use crate::press::mail::Press;
use crate::web::app::{MailAsk, epoch};

mod letter;
mod list;

/// The colour of the numbers by the mailbox's logo (the client's 0xC89482).
const PALE: &str = "#8294c8";

/// One of the mail's pictures.
fn rodex(name: &str) -> String {
    format!("basic_interface/rodexsystem/renewal/{name}.bmp")
}

/// One of the mail's picture buttons (`UIBitmapButton`): `name_out.bmp`, `name_over.bmp` under the pointer,
/// `name_press.bmp` pressed; with what it says of itself when pointed at.
fn picture_button(x: i32, y: i32, size: (i32, i32), name: &str, press: Press, tip: &str) -> String {
    picture_button_as(Kind::Press, x, y, size, name, press, tip)
}

/// The same button of either kind: the one that sends the form it stands in, for the search.
fn picture_button_as(kind: Kind, x: i32, y: i32, size: (i32, i32), name: &str, press: Press, tip: &str) -> String {
    let [off, over, down] = states(&format!("basic_interface/rodexsystem/renewal/{name}"), Convention::OutOverPress);
    pictured_as(kind, x, y, size, [&off, &over, &down], press, &format!(r#" title="{}""#, escape(tip)))
}

impl App {
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
        window.push_str(&base_button(3, 3));
        window.push_str(&cross(309, Press::Window));
        window.push_str(&picture_button(248, 20, (24, 26), "btn_refresh", Press::Refresh, &Self::words(2849, "Refresh")));
        window.push_str(&picture_button(277, 18, (24, 26), "btn_write", Press::Write, &Self::words(1026, "Write Mail")));
        window.push_str(&self.mail_search());
        for (x, line, name, shown) in [
            (19, 3546, "Notice", tab::ACCOUNT),
            (87, 3547, "General", tab::NORMAL),
            (155, 3548, "Return", tab::RETURNED),
            (223, 3549, "Search", 3)
        ] {
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
        let (delete, claim) = if ticked { (Self::words(3588, "Delete"), Self::words(3591, "Claim")) } else { (
            Self::words(3589, "Delete All"),
            Self::words(3592, "Claim All")
        ) };
        window.push_str(&basic_button(4, 392, 70, &delete, (!mail.waiting).then_some(Press::Delete.into())));
        window.push_str(&basic_button(84, 392, 70, &claim, (!mail.waiting).then_some(Press::Claim.into())));
        window.push_str(&picture_button(255, 390, (24, 24), "btn_prev", Press::Page(Step::Less), &Self::words(1297, "Previous")));
        window.push_str(&picture_button(279, 390, (24, 24), "btn_next", Press::Page(Step::More), &Self::words(1298, "Next")));
        window.push_str(&text_ending_in(SMALL_BOLD, 273, 309, 46, PALE, &mail.sent_today.to_string()));
        window.push_str(&text_in(SMALL_BOLD, 273, 46, PALE, &Self::words(3034, "/100")));
        window
    }

    /// What the mail asks before it acts, in the client's message box (280 by 120, `UIWindowMgr::Confirm`):
    /// OK and Cancel.
    pub(in crate::web) fn mail_question(&self) -> String {
        let words = match &self.mail.asks {
            Some(MailAsk::DeleteAll) => Self::words(3590, "Are you sure you want to delete all mails in the [%s] mailbox?")
                .replace("%s", &self.mail_tab_name()),
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
