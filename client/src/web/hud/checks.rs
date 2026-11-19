//! The client's small boxes that are ticked and its round buttons of which one is on, each written once.


use super::*;

/// One of the client's small boxes with its words beside it (`UICheckBox`: `checkbox_1.bmp` ticked,
/// `checkbox_0.bmp` not, the words 12 to the right of where the box begins), pressed to switch it.
pub(in crate::web) fn check_words(x: i32, y: i32, on: bool, words: &str, press: impl Into<crate::press::Press>) -> String {
    format!(
        r#"<div class="a t" style="left:{x}px;top:{y}px;height:12px;line-height:12px;padding-left:12px;white-space:nowrap;cursor:default;color:{BLACK};background:url('{}') 0 1px no-repeat" {}>{}</div>"#,
        picture(if on { "checkbox_1.bmp" } else { "checkbox_0.bmp" }),
        pressed(press),
        escape(words)
    )
}

/// One of the client's small boxes without words, 12 square, ticked or not (`checkbox_1.bmp`,
/// `checkbox_0.bmp`); `mark` is what marks it as pressable, written already (`pressed(..)`), or nothing for
/// one that only shows.
pub(in crate::web) fn check(x: i32, y: i32, on: bool, mark: &str) -> String {
    format!(r#"<div class="a" style="{};background:url('{}') 0 0 no-repeat" {mark}></div>"#, at(x, y, 12, 12), picture(if on { "checkbox_1.bmp" } else { "checkbox_0.bmp" }))
}

/// One of the client's round buttons with its words 13 to the right (`UIRadioBtn::Paint`): on or off, and grey
/// when it cannot be pressed; `press` is what pressing it does, if anything.
pub(in crate::web) fn radio(x: i32, y: i32, on: bool, enabled: bool, words: &str, press: Option<crate::press::Press>) -> String {
    radio_marked(x, y, on, enabled, words, &press.map(pressed).unwrap_or_default())
}

/// The same button with what marks it as pressable written already (`pressed(..)`), or nothing.
pub(in crate::web) fn radio_marked(x: i32, y: i32, on: bool, enabled: bool, words: &str, mark: &str) -> String {
    format!(
        r#"<div class="a t" style="left:{x}px;top:{y}px;height:12px;padding-left:13px;white-space:nowrap;cursor:default;color:{};background:url('{}') 0 0 no-repeat" {mark}>{}</div>"#,
        if enabled { BLACK } else { "#b5b5b5" },
        picture(if on { "radiobtn_on.bmp" } else { "radiobtn_off.bmp" }),
        escape(words)
    )
}
