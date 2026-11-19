//! The boxes every system asks with, each written once: the client's message box with a question and its two
//! buttons, the box that asks how many of a thing, and the rows of a menu opened with the right button. Their
//! places on the screen and how they lie over the windows are in the list of windows (`Window::question`,
//! `Window::count`, `Window::menu` of `windows.rs`).

use super::*;
use crate::press::Press;

/// How the client lays out its message box (`win_msgbox.bmp`, 280 by 120) for a question.
#[derive(Clone, Copy)]
pub(super) enum Asked {
    /// A question someone else asks, to accept or refuse (`UIExchangeAcceptWnd::OnCreate` lays out the boxes
    /// of this kind): the words 10,22 in 260 by 65, OK 189,96 and Cancel 234,96.
    Offer,
    /// Whether the player means what was just pressed (`UIWindowMgr::Confirm`): the words 12,30 in 256 by 56,
    /// OK 189,96 and Cancel 235,96.
    Sure,
}

/// A question in the client's message box. `words` are written as they are: what is not the client's own
/// text is escaped by whoever asks.
pub(super) fn message_box(asked: Asked, words: &str, yes: impl Into<Press>, no: impl Into<Press>) -> String {
    let (place, cancel) = match asked {
        Asked::Offer => (at(10, 22, 260, 65), 234),
        Asked::Sure => (at(12, 30, 256, 56), 235),
    };
    format!(
        r#"{}<div class="a t" style="{place};white-space:normal">{words}</div>{}{}"#,
        art("win_msgbox.bmp", 0, 0, 280, 120),
        picture_press(189, 96, "btn_ok", yes),
        picture_press(cancel, 96, "btn_cancel", no)
    )
}

/// A row of a menu opened with the right button, 16 high (`UM_MAKEMENU`).
pub(super) fn menu_row(words: &str, press: impl Into<Press>) -> String {
    format!(r#"<div class="t opt" style="height:16px;line-height:16px;padding:0 3px;cursor:default;white-space:nowrap" {}>{}</div>"#, pressed(press), escape(words))
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
    window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border-left:1px solid #c8c8c8;border-right:1px solid #c8c8c8;box-sizing:border-box"></div>"#, at(0, 17, 200, 162)));
    let rows: String = rows
        .iter()
        .enumerate()
        .map(|(row, (icon, name))| {
            let band = if picked == Some(row) { r#"<div style="position:absolute;left:5px;top:0;width:176px;height:30px;background:#c6d6f7"></div>"# } else { "" };
            format!(
                r#"<div style="position:relative;height:32px;cursor:default" {} {}>{band}<div style="position:absolute;left:8px;top:3px;width:24px;height:24px">{icon}</div><div class="t" style="position:absolute;left:40px;top:3px;white-space:nowrap">{}</div></div>"#,
                pressed((does.pick)(row)),
                used((does.used)(row)),
                escape(name)
            )
        })
        .collect();
    window.push_str(&format!(r#"<div class="a scroll" style="{}">{rows}</div>"#, at(1, 31, 198, 128)));
    window.push_str(&art("basic_interface/btnbar_left.bmp", 0, 179, 21, 21));
    window.push_str(&art("basic_interface/btnbar_mid.bmp", 21, 179, 158, 21));
    window.push_str(&art("basic_interface/btnbar_right.bmp", 179, 179, 21, 21));
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
             <input id="{}" class="a in" style="{};background:#e8e8e8" type="text" inputmode="numeric" maxlength="6" autocomplete="off" value="{shown}">
             {}
           </form>"#,
        system_box(0, 182, 46),
        sent(form),
        at(0, 0, 182, 46),
        escape(name),
        window.field(),
        at(15, 22, 80, 16),
        picture_press(127, 14, "btn_ok", ok).replace(r#"type="button""#, r#"type="submit""#)
    )
}

impl App {
    /// The box that asks how many of a thing carried (`count_box_of`): all of them in it at first. `counting`
    /// is the thing's place in the inventory and how many there are; `form` is what is sent when the number is
    /// given, read back with `App::counted`.
    pub(super) fn count_box(&self, window: &windows::Window, counting: Option<(u16, u16)>, form: Form, ok: impl Into<Press>) -> String {
        let Some((index, most)) = counting else { return String::new() };
        let name = self.bag.items.iter().find(|item| item.index == index).map(Self::item_name).unwrap_or_default();
        count_box_of(window, &name, u32::from(most), form, ok)
    }
}
