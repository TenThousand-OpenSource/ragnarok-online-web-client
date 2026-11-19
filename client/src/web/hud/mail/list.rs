//! The mailbox's list (`UIRodexInboxWnd`): a row of it, and the search over it.

use super::*;

impl App {
    /// The search of the mailbox: the two round boxes that say where to look, the titles (the first, on at
    /// first) or the senders, 51 apart; the words to look for, sixteen letters at most; and its button. Enter
    /// in the field searches, as the button does.
    pub(super) fn mail_search(&self) -> String {
        let mut search = String::new();
        for (x, line, name, by_sender) in [(22, 3192, "Title", false), (73, 3193, "Sender", true)] {
            search.push_str(&[
                format!(r#"<div class="a t" style="left:{x}px;top:68px;height:12px;line-height:12px;padding-left:13px;"#),
                format!(
                    r#"cursor:default;color:#394965;background:url('{}') 0 1px no-repeat" {}>{}</div>"#,
                    picture(&rodex(if by_sender == self.mail.by_sender { "checkbox_search_on" } else { "checkbox_search_off" })),
                    pressed(Press::SearchIn(by_sender)),
                    escape(&Self::words(line, name))
                )
            ].concat());
        }
        search.push_str(&format!(
            r#"<form {} class="a" style="{}">{}{}</form>"#,
            sent(Form::MailSearch),
            at(133, 65, 163, 18),
            field(
                "mail-sought",
                (0, 0, 132, 18),
                &crate::web::app::written("mail-sought").unwrap_or_default(),
                16,
                "background:#fff;border:1px solid #adadad;box-sizing:border-box"
            ),
            picture_button_as(Kind::Submit, 131, 0, (32, 18), "btn_search", Press::Search, "")
        ));
        search
    }

    /// A row of the list, 44 high, the first 121 down (`Paint`, `DrawMailIcons`, `DrawExpireTime`,
    /// `SetMailRow`): the box to tick; the picture of a letter read or not (of one returned, among what was
    /// searched for); the title, and under it the sender, each pressed to read and to write back; the picture
    /// of what is in it; how long it still lies there; and for a letter of someone not known the button that
    /// sends it back. One the server has deleted says so in grey and is not pressed.
    pub(super) fn mail_row(&self, row: usize, kind: u8, letter: &Letter, now: f64) -> String {
        let y = 121 + 44 * row as i32;
        let returned = self.mail.searching && kind == tab::RETURNED;
        let status = if returned { "icon_status_mail_returned" } else if letter.read { "icon_status_mail_read" } else { "icon_status_mail_received" };
        // The client opens a letter by its title alone; here its picture opens it too, and the title is pressed
        // along its whole 140, not only where its letters are (this project's, for a hand that looks for it).
        let mut html = if letter.gone {
            art(&rodex(status), 36, y + 3, 24, 24)
        } else {
            format!(
                r#"<div class="a" style="{};background:url('{}') 0 0 no-repeat;cursor:pointer" {}></div>"#,
                at(36, y + 3, 24, 24),
                picture(&rodex(status)),
                pressed(Press::Title(row))
            )
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
        let (line, words, count) = crate::mail::time_left((letter.expires - now) as i64);
        let left = Self::words(line, words).replace("%d", &count.to_string());
        if returned {
            html.push_str(&text_centred(SMALL, 260, y + 4, "#736666", &Self::words(2853, "Returned")));
            html.push_str(&text_centred(SMALL, 260, y + 17, "#736666", &left));
        } else {
            html.push_str(&text_centred(SMALL_BOLD, 260, y + 9, "#e90a0a", &left));
        }
        html.push_str(&format!(
            r#"<div class="a" style="{};background:url('{}') 0 0 no-repeat" {}></div>"#,
            at(17, y + 7, 14, 14),
            picture(&rodex(if self.mail.checked[row] { "checkbox_on" } else { "checkbox_off" })),
            pressed(Press::Check(row))
        ));
        // The title and the sender are cut at 140 across (`ResizeToTextClipped`).
        let clipped = "max-width:140px;overflow:hidden;cursor:default";
        let colour = crate::mail::title_colour(kind, letter);
        if letter.gone {
            html.push_str(&format!(
                r#"<div class="a t" style="left:69px;top:{y}px;{clipped};color:{colour}">{}</div>"#,
                escape(&Self::words(2907, "The mail has been deleted."))
            ));
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
            SMALL_BOLD.style(),
            pressed(Press::Sender(row)),
            escape(&Self::words(2701, "Write reply")),
            escape(&letter.sender)
        ));
        if !letter.known {
            html.push_str(&picture_button(186, y + 10, (42, 20), "btn_return", Press::GiveBack(row), &Self::words(2848, "Return")));
        }
        html
    }
}
