//! The Hotkey Guide (the client's `UIHotkeyGuideWnd`): a picture of the keyboard with what is written on each
//! key, and under it eighteen things of the game, each a button with its keys beside it; while one of the
//! buttons is held down its keys are lit on the keyboard. Its rules and its table of keys are
//! `crate::hotkeys::guide`.
//!
//! The client gives its texts' colours blue first: 0x2B2625 is written here as #25262b.

use super::*;
use crate::hotkeys::guide::{self, Align, Cap};

/// What is written on the keys and by the keyboard, the keys beside a thing of the list, and the words over
/// the keys of the hotkey bar (`UIHotkeyGuideWnd::Paint`: 0x2B2625, 0xA55139, 0x5B2C24).
const WORDS: &str = "#25262b";
const KEYS: &str = "#3951a5";
const BAR: &str = "#242c5b";

/// A key lit, at its place counted from `from` (`UIHotkeyGuideWnd::DrawKey`): a key 36 wide is one picture
/// (`bt_keyboard.bmp`, or `bt_enter2.bmp` for the tall ones, 75 high); a wider one has ends of 6
/// (`bt_left.bmp`, `bt_right.bmp`) and `bt_mid.bmp` laid every 6 between them.
fn lit(cap: &Cap, from: (i32, i32)) -> String {
    let place = at(cap.x - from.0, cap.y - from.1, cap.width, cap.height);
    let piece = |name: &str| format!("url('{}')", picture(&format!("hotkey_guide/{name}.bmp")));
    if cap.width == 36 {
        return format!(r#"<span class="a" style="{place};background:{} no-repeat"></span>"#, piece(if cap.height == 75 { "bt_enter2" } else { "bt_keyboard" }));
    }
    [
        format!(r#"<span class="a" style="{place};padding:0 6px;box-sizing:border-box;background:{} 0 0 no-repeat,"#, piece("bt_left")),
        format!(r#"{} 100% 0 no-repeat,{} 0 0 repeat-x;background-clip:border-box,border-box,content-box;"#, piece("bt_right"), piece("bt_mid")),
        r#"background-origin:border-box,border-box,content-box"></span>"#.to_owned()
    ].concat()
}

/// What is written on a key (`UIHotkeyGuideWnd::DrawKeyLabel`): from its left edge with its padding, ending
/// short of its right edge by the padding, or in its middle; down the middle of its height.
fn label(words: &str, cap: &Cap) -> String {
    let lies = match cap.align {
        Align::Left => format!("text-align:left;padding-left:{}px", cap.padding),
        Align::Right => format!("text-align:right;padding-right:{}px", cap.padding),
        Align::Middle => "text-align:center".to_owned(),
    };
    [
        format!(r#"<div class="a t pass" style="left:{}px;top:{}px;width:{}px;box-sizing:border-box;"#, cap.x, cap.words_top(), cap.width),
        format!(r#"{lies};color:{WORDS}">{}</div>"#, escape(words))
    ].concat()
}

/// Words in the middle of a width that begins at a place (the client's `DrawCenteredText`).
fn centred(font: Font, x: i32, y: i32, width: i32, colour: &str, words: &str) -> String {
    [
        format!(r#"<div class="a t pass" style="left:{x}px;top:{y}px;width:{width}px;text-align:center;color:{colour};"#),
        format!(r#"{}">{}</div>"#, font.style(), escape(words))
    ].concat()
}

impl App {
    /// What is written on a key of the guide's keyboard: the table's words, and for the one key whose words
    /// are a line of the client's message table, that line.
    fn guide_words(code: u16) -> String {
        match guide::key(code) {
            Some((guide::KEY_25, ..)) => Self::words(guide::WORDS_OF_25, "Ctrl"),
            Some((_, words, ..)) => (*words).to_owned(),
            None => String::new(),
        }
    }

    /// The window (`UIHotkeyGuideWnd::OnCreate` and `Paint`), 934 by 498: its picture
    /// (`hotkey_guide\bg_shortcuts.bmp`) under the title bar with the client's message 3128 and the cross;
    /// the things listed and the keyboard's words; the words by the keyboard and by the mouse (the lines 3129
    /// and 3045 over Escape, 3130 over the keys of the hotkey bar, 3131 over Print Screen, 3132 and 3133, 3139
    /// and 3140 by the mouse); and at 763,462 the button that opens the Shortcut Settings (`bt_keysettings`,
    /// 156 by 25, the line 3148 on it).
    pub(in crate::web) fn key_guide(&self) -> String {
        use crate::press::hotkeys::Press;
        let (width, height) = guide::SIZE;
        let both = |first: usize, second: usize| format!("{} / {}", Self::words(first, ""), Self::words(second, ""));
        let mut window = art("hotkey_guide/bg_shortcuts.bmp", 0, 0, width, height);
        window.push_str(&title_bar(width, &Self::words(3128, "Hotkey Guide")));
        window.push_str(&base_button(3, 3));
        window.push_str(&cross(width, Press::GuideClose));
        window.push_str(&self.guide_list());
        for (code, _, first, second) in &guide::KEYS {
            let words = Self::guide_words(*code);
            window.push_str(&label(&words, first));
            if let Some(second) = second {
                window.push_str(&label(&words, second));
            }
        }
        window.push_str(&centred(USUAL, 4, 32, 75, WORDS, &both(3129, 3045)));
        window.push_str(&centred(SMALL, 207, 34, 159, BAR, &Self::words(3130, "Hotkeys (F1 ~ F9)")));
        window.push_str(&centred(USUAL, 604, 32, 75, WORDS, &Self::words(3131, "Screenshot")));
        window.push_str(&centred(USUAL, 791, 313, 102, WORDS, &both(3132, 3133)));
        window.push_str(&centred(USUAL, 770, 346, 55, WORDS, &Self::words(3139, "Move")));
        window.push_str(&centred(USUAL, 859, 346, 55, WORDS, &Self::words(3140, "Rotate")));
        let [normal, over, down] = states("hotkey_guide/bt_keysettings", Convention::NormalOverPress).map(|name| picture(&name));
        window.push_str(&[
            format!(r#"<button type="button" class="a b" style="{};--n:url('{normal}');--o:url('{over}');"#, at(763, 462, 156, 25)),
            format!(r#"--p:url('{down}');text-align:center;color:#000;text-shadow:1px 1px 0 #d7d7d7;{}" "#, USUAL.style()),
            format!(r#"{}>{}</button>"#, pressed(Press::GuideSettings), escape(&Self::words(3148, "Set shortcut")))
        ].concat());
        window
    }

    /// The eighteen things listed, down three columns of six: each a button 118 wide with the thing's name,
    /// which lights the thing's keys on the keyboard while it is held down (`UIHotkeyGuideWnd::OnProcess`
    /// takes the button in its pressed state; a press does nothing more), and beside it the keys as the guide
    /// writes them, in the middle of 100 that begin 130 right of the button and 5 lower. The keys are the
    /// player's own (the client asks its table of keys, `GetHotKey`), but for the game's options, which are
    /// Escape always.
    fn guide_list(&self) -> String {
        let mut list = String::new();
        for (place, (line, keys)) in guide::LISTED.into_iter().enumerate() {
            let chord = keys.map_or(guide::ESCAPE, |keys| self.bound(crate::hotkeys::table::INTERFACE, keys));
            let (x, y) = guide::listed_place(place);
            let held: String = [chord.key, chord.with]
                .into_iter()
                .filter(|_| chord.key != 0)
                .filter_map(guide::key)
                .flat_map(|(_, _, first, second)| [Some(first), second.as_ref()])
                .flatten()
                .map(|cap| lit(cap, (x, y)))
                .collect();
            list.push_str(&basic_button_holding(x, y, 118, &Self::words(line, ""), &held));
            list.push_str(&centred(USUAL, x + 130, y + 5, 100, KEYS, &guide::written(chord, Self::guide_words)));
        }
        list
    }
}
