//! The window of a talk with one player (the client's `UIWhisperWnd`, 280 by 120), and the menu the right
//! button opens on it; and the box that asks the away message (`UIAutoMessageWnd`). The rules of it that need
//! no page are `crate::whispers`.

use super::*;
use crate::press::whispers::Press;
use crate::whispers::SIZE;

impl App {
    /// Where the window of that place stands and how large it is, as a style: where it was opened
    /// (`crate::whispers::place`).
    pub(in crate::web) fn whisper_style(&self, place: usize) -> String {
        let (x, y) = self.whispers.talks.get(place).and_then(Option::as_ref).map(|talk| talk.at).unwrap_or_default();
        at(x, y, SIZE.0, SIZE.1)
    }

    /// The window (`UIWhisperWnd::OnCreate`, `Paint` and `DrawTransBox`): see-through black with a rounded
    /// grey rim, a strip of pale blue 17 high at the top with whom the talk is with in white 10,3 into it and
    /// the cross at its right; what was said from 17 down to 21 from the bottom; a grey line; and under it
    /// the line to type in, 3 in and 30 short of the window's width, of 70 letters at most. (The client's
    /// window can be made larger by its corner: this one is of the one size. The code of the other's
    /// character that the client writes into the title is left out: its making was not found.)
    pub(in crate::web) fn whisper_window(&self, place: usize) -> String {
        let Some(talk) = self.whispers.talks.get(place).and_then(Option::as_ref) else { return String::new() };
        let (width, height) = SIZE;
        let lines: String = talk.lines.iter().map(|(words, colour)| log_line(colour, &escape(words))).collect();
        [
            format!(
                r#"<div class="a" style="{};background:rgba(0,0,0,.4);border:1px solid #c0c0c0;border-radius:3px;box-sizing:border-box"></div>
               <div class="a" style="{};background:rgba(158,182,228,.27)"></div>
               {}{}
               "#,
                at(0, 0, width, height),
                at(1, 1, width - 2, 17),
                text(10, 3, "#fff", &talk.title),
                cross(width, Press::Close(place))
            ),
            format!(
                r#"<div class="a t" style="{};padding:0 3px;box-sizing:border-box;display:flex;flex-direction:column-reverse;"#,
                at(1, 17, width - 2, height - 38)
            ),
            format!(
                r#"overflow-x:hidden;overflow-y:auto;scrollbar-width:thin;white-space:normal;line-height:14px;user-select:text"><div>{lines}</div></div>
               "#
            ),
            format!(
                r#"<div class="a" style="{};background:#c0c0c0"></div>
               {}"#,
                at(1, height - 20, width - 2, 1),
                typed_line(
                    &sent(Form::Whisper(place)),
                    &at(0, height - 19, width, 19),
                    &format!("whisper-input-{place}"),
                    &at(3, 1, width - 30, 16),
                    "color:#fff"
                )
            )
        ].concat()
    }

    /// The menu the right button opens on the window (`UIWhisperWnd::SendMsg`, `UM_MAKEMENU`): asking the
    /// other to be a friend, when they are none. (The client's first row saves the talk as a file: not made.)
    pub(in crate::web) fn whisper_menu(&self) -> String {
        if self.whispers.menu.is_none() {
            return String::new();
        }
        menu_row(&Self::words(358, "Register as a Friend"), Press::Friend)
    }

    /// The box that asks the away message (`UIAutoMessageWnd::OnCreate` and `Paint`), 280 by 48 in the frame
    /// of the system boxes: the client's message 944 at 15,6; the line at 15,22, 77 short of the box's width
    /// and 16 high, on `232, 232, 232`, of a hundred letters, with the message as it stands in it; OK 55 from
    /// the right and 32 from the bottom. Enter is its OK (`m_message0`).
    pub(in crate::web) fn away_window(&self) -> String {
        if !self.whispers.away_asked {
            return String::new();
        }
        [
            system_box(0, 280, 48),
            format!(r#"<form {} class="a" style="{}">"#, sent(Form::AwayWords), at(0, 0, 280, 48)),
            text(15, 6, BLACK, &Self::words(944, "Please enter Away Message.")),
            field(&windows::AWAY.field(), (15, 22, 280 - 77, 16), &self.away_words(), crate::whispers::AWAY_LETTERS, "background:#e8e8e8"),
            picture_press(280 - 55, 48 - 32, "btn_ok", Press::AwayOk),
            r#"<button type="submit" hidden></button></form>"#.to_owned()
        ].concat()
    }

    /// The away message as it stands: what the player typed, or until then the client's line 940
    /// (`LoginMode`: `autoReplyText = LookupMessage(MSI_AUTO_MSG)`).
    pub(in crate::web) fn away_words(&self) -> String {
        self.whispers.away_words.clone().unwrap_or_else(|| Self::words(crate::whispers::AWAY_UNSET, "(Away)"))
    }
}
