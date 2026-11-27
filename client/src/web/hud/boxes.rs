//! The boxes every system asks with, each written once: the client's message box with a question and its two
//! buttons, the box that asks how many of a thing, and the rows of a menu opened with the right button. Their
//! places on the screen and how they lie over the windows are in the list of windows (`Window::question`,
//! `Window::count`, `Window::menu` of `windows.rs`).

use super::*;
use crate::press::Press;

/// A question someone else asks, to accept or refuse, in the client's box for those (`win_msgbox.bmp`, 280 by
/// 120; `UIExchangeAcceptWnd::OnCreate` lays out the boxes of this kind): the words 10,22 in 260 by 65, OK
/// 189,96 and Cancel 234,96. `words` are written as they are: what is not the client's own text is escaped
/// by whoever asks.
pub(super) fn message_box(words: &str, yes: impl Into<Press>, no: impl Into<Press>) -> String {
    format!(
        r#"{}<div class="a t" style="{};white-space:normal">{words}</div>{}{}"#,
        art("win_msgbox.bmp", 0, 0, 280, 120),
        at(10, 22, 260, 65),
        picture_press(189, 96, "btn_ok", yes),
        picture_press(234, 96, "btn_cancel", no)
    )
}

/// Whether the player means what was just pressed, in the client's message box (`UIWindowMgr::Confirm`, which
/// makes a `UIMessageBox` 280 by 120 with the caption "Message"; `UIMessageBox::Configure` and `Paint`): the
/// title bar with its caption and the plain panel under it, the words in a list of lines 10,22 in 260 by 65,
/// OK 189,96 and Cancel 234,96. The box does not grow with its words: they are cut into lines of 37 letters,
/// each 18 high from 3 in and 4 down, three show, and more are reached by the client's bar at the list's
/// right and by the wheel, three lines a notch (`crate::message_box`, `crate::scrolling`). `words` are the
/// client's own markup (`^RRGGBB`), written here.
pub(super) fn confirm_box(words: &str, yes: impl Into<Press>, no: impl Into<Press>) -> String {
    [message_frame("Message", words), picture_press(189, 96, "btn_ok", yes), picture_press(234, 96, "btn_cancel", no)].concat()
}

/// The same box telling something, with OK alone, 46 from its right edge (`UIMessageBox::Configure` with the
/// kind 0), under that caption ("Message", or "Error").
pub(super) fn told_box(caption: &str, words: &str, ok: impl Into<Press>) -> String {
    [message_frame(caption, words), picture_press(234, 96, "btn_ok", ok)].concat()
}

/// The client's message box without its buttons: the caption, the panel and the words in their list.
fn message_frame(caption: &str, words: &str) -> String {
    use crate::message_box::{LINE, SHOWN, letters, lines};
    let lines = lines(words, letters(280)).join("\n");
    // A line's letters stand 4 under its top; the line here is 18 with its 15 in the middle of it.
    let above = 4 - (LINE - i32::from(crate::font::cell(0, 12))) / 2;
    // What is left of the list's 65 under its three lines: no part of a fourth line is drawn there
    // (`UITextViewer::Paint` draws whole lines only), and the list ends on a whole line when scrolled.
    let under = 65 - above - SHOWN * LINE;
    [
        title_bar(280, caption),
        base_button(3, 3),
        panel_ground_round(0, 17, 280, 103),
        format!(
            r#"<div class="a t scroll" style="{};line-height:{LINE}px;white-space:pre" {}><div style="padding:{above}px 0 {under}px 3px">{}</div></div>{}"#,
            at(10, 22, 260, 65),
            lines_stepped(LINE),
            coloured(&lines),
            // The viewer's own bar, at its right edge (`UIListBox::OnSize`: `Move(width - 13, 0)`).
            scroll_bar(257, 22, 65, LINE)
        ),
        // (Begun a pixel higher, under the third line's letters: a page drawn larger than its measures lets
        // a hair of the fourth line show otherwise.)
        fill(10, 22 + 65 - under - 1, 260 - 13, under + 1, WHITE)
    ].concat()
}

/// A row of a menu opened with the right button, 16 high (`UM_MAKEMENU`).
pub(super) fn menu_row(words: &str, press: impl Into<Press>) -> String {
    format!(
        r#"<div class="t opt" style="height:16px;line-height:16px;padding:0 3px;cursor:default;white-space:nowrap" {}>{}</div>"#,
        pressed(press),
        escape(words)
    )
}

/// The line the client draws between the groups of rows of a menu (`UM_ADDBOUNDARY`).
pub(super) const MENU_BOUNDARY: &str = r#"<div style="height:1px;margin:2px 0;background:#9a9a9a"></div>"#;

/// What a list to pick one thing from does: a press on a row, a double click on it, OK and Cancel.
pub(super) struct Picking {
    pub pick: fn(usize) -> Press,
    pub used: fn(usize) -> Use,
    pub ok: Press,
    pub cancel: Press,
}

/// The client's window that lists things to pick one of (`UIItemIdentifyWnd` and those made of it: the eggs
/// to hatch, what arrows are made of, the spells of Auto Spell, where a card goes), 200 by 200: rows 32 apart
/// from 34 down, each with a picture 8 in and a name 40 in, the picked one on a band 30 high that starts 5 in
/// and 3 above; OK and Cancel 92 and 46 from the right, 24 from the bottom. A row is its picture, as HTML,
/// and its name.
pub(super) fn pick_list(title: &str, rows: &[(String, String)], picked: Option<usize>, does: Picking) -> String {
    let mut window = title_bar(200, title);
    window.push_str(&side_rimmed(0, 17, 200, 162));
    let rows: String = rows
        .iter()
        .enumerate()
        .map(|(row, (icon, name))| {
            let band = if picked == Some(row) {
                r#"<div style="position:absolute;left:5px;top:0;width:176px;height:30px;background:#c6d6f7"></div>"#
            } else {
                ""
            };
            [
                format!(r#"<div style="position:relative;height:32px;cursor:default" {} {}>{band}"#, pressed((does.pick)(row)), used((does.used)(row))),
                format!(r#"<div style="position:absolute;left:8px;top:3px;width:24px;height:24px">{icon}</div>"#),
                format!(r#"<div class="t" style="position:absolute;left:40px;top:3px;white-space:nowrap">{}</div></div>"#, escape(name))
            ].concat()
        })
        .collect();
    window.push_str(&format!(r#"<div class="a scroll" style="{}" {}>{rows}</div>{}"#, at(1, 31, 198, 128), rows_stepped(32, 4), scroll_bar(185, 31, 128, 32)));
    window.push_str(&button_bar(200, 200));
    window.push_str(&picture_press(108, 176, "btn_ok", does.ok));
    window.push_str(&picture_press(154, 176, "btn_cancel", does.cancel));
    window
}

/// The box that asks how many of a thing (the client's `UIItemDropCntWnd`, 182 by 46, which asks this for
/// dropping, for a deal, for a letter, for a shop and for the storage): the thing's name, the number to type
/// over (a plain field of six characters, `shown` in it at first) and OK. `form` is what is sent when the
/// number is given.
pub(super) fn count_box_of(window: &windows::Window, name: &str, shown: u32, form: Form, ok: impl Into<Press>) -> String {
    format!(
        r#"{}<form {} class="a" style="{}">
             <div class="a t" style="left:15px;top:6px;width:104px;overflow:hidden;white-space:nowrap">{}</div>
             {}
             {}
           </form>"#,
        system_box(0, 182, 46),
        sent(form),
        at(0, 0, 182, 46),
        escape(name),
        numeric_field(&window.field(), (15, 22, 80, 16), &shown.to_string(), 6, &format!("background:{FIELD_GROUND}")),
        submit_press(127, 14, "btn_ok", ok)
    )
}

impl App {
    /// The rows of a list with pictures (`pick_list`) for things carried, by where each is: its picture and
    /// its name. A thing that is no longer carried keeps its row, empty, so that the rows stay those of the
    /// list.
    pub(super) fn carried_rows(&self, places: &[u16]) -> Vec<(String, String)> {
        places
            .iter()
            .map(|index| {
                let Some(item) = self.bag.items.iter().find(|item| item.index == *index) else { return Default::default() };
                (Self::icon_of(item), Self::item_name(item))
            })
            .collect()
    }

    /// The box that asks how many of a thing carried (`count_box_of`): all of them in it at first. `counting`
    /// is the thing's place in the inventory and how many there are; `form` is what is sent when the number is
    /// given, read back with `App::counted`.
    pub(super) fn count_box(&self, window: &windows::Window, counting: Option<(u16, u16)>, form: Form, ok: impl Into<Press>) -> String {
        let Some((index, most)) = counting else { return String::new() };
        let name = self.bag.items.iter().find(|item| item.index == index).map(Self::item_name).unwrap_or_default();
        count_box_of(window, &name, u32::from(most), form, ok)
    }
}
