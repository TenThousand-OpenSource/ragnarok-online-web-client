//! The buttons of the windows, each kind written once: one that is three of the client's pictures (as it
//! stands, under the pointer, pressed), by whichever way its files are named, and the client's button with
//! words on it.

// Not every one is taken up by the windows yet (they are being moved onto these).
#![allow(dead_code)]

use super::*;

/// What a button of the page is to its form: one that is pressed, or the one that sends the form it stands in.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum Kind {
    Press,
    Submit,
}

/// A button that is one of three of the client's pictures: as it stands, under the pointer, pressed. Every
/// picture button of the windows is written by this. `more` is anything else its tag says (what it tells when
/// pointed at), with the space before it.
pub(in crate::web) fn pictured(x: i32, y: i32, size: (i32, i32), pictures: [&str; 3], press: impl Into<crate::press::Press>, more: &str) -> String {
    pictured_as(Kind::Press, x, y, size, pictures, press, more)
}

/// The same button, of either kind: `Kind::Submit` for the one that sends its form (the OK of a box that
/// asks), which is pressed like any other besides.
pub(in crate::web) fn pictured_as(kind: Kind, x: i32, y: i32, (width, height): (i32, i32), [off, over, down]: [&str; 3], press: impl Into<crate::press::Press>, more: &str) -> String {
    format!(
        r#"<button type="{}" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" {}{more}></button>"#,
        if kind == Kind::Submit { "submit" } else { "button" },
        at(x, y, width, height),
        picture(off),
        picture(over),
        picture(down),
        pressed(press)
    )
}

/// How the files of a button's three pictures are named after the button.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum Convention {
    /// `name.bmp`, `name_a.bmp` under the pointer, `name_b.bmp` pressed: most of the client's small buttons.
    AB,
    /// `name.bmp`, `name_b.bmp` under the pointer, `name_a.bmp` pressed: the deal's.
    BA,
    /// `name_a.bmp` as it stands, `name_b.bmp` under the pointer and pressed: the chat's.
    AThenB,
    /// `name_out.bmp`, `name_over.bmp`, `name_press.bmp`: the mail's, the party's.
    OutOverPress,
    /// `name_normal.bmp`, `name_over.bmp`, `name_press.bmp`: the screens before the game.
    NormalOverPress,
}

/// The three pictures of a button, as it stands, under the pointer and pressed, by its name (with its folder,
/// without `.bmp`) and the way its files are named.
pub(in crate::web) fn states(name: &str, convention: Convention) -> [String; 3] {
    let endings = match convention {
        Convention::AB => ["", "_a", "_b"],
        Convention::BA => ["", "_b", "_a"],
        Convention::AThenB => ["_a", "_b", "_b"],
        Convention::OutOverPress => ["_out", "_over", "_press"],
        Convention::NormalOverPress => ["_normal", "_over", "_press"],
    };
    endings.map(|ending| format!("{name}{ending}.bmp"))
}

/// One of the original's small buttons with three pictures: `name.bmp`, `name_a.bmp` under the pointer,
/// `name_b.bmp` pressed.
pub(in crate::web) fn picture_press(x: i32, y: i32, name: &str, press: impl Into<crate::press::Press>) -> String {
    picture_press_sized(x, y, (42, 20), name, press)
}

/// The same button of another size than the usual 42 by 20.
pub(in crate::web) fn picture_press_sized(x: i32, y: i32, size: (i32, i32), name: &str, press: impl Into<crate::press::Press>) -> String {
    let [off, over, down] = states(name, Convention::AB);
    pictured(x, y, size, [&off, &over, &down], press, "")
}

/// A small picture button whose picture changes under the pointer.
pub(in crate::web) fn press_button(x: i32, y: i32, size: i32, off: &str, on: &str, press: impl Into<crate::press::Press>) -> String {
    pictured(x, y, (size, size), [off, on, on], press, "")
}

/// The client's button with words on it (`UIBasicButton`), 20 high and as wide as asked: ends of 6 that keep
/// their size and a middle that repeats between them, each in three pictures (`btn_out_*`, `btn_over_*`,
/// `btn_press_*`); the words in the middle, black over a grey copy of themselves. Without a press it is the
/// one that cannot be pressed (`btn_disable_*`, its words grey).
pub(in crate::web) fn basic_button(x: i32, y: i32, width: i32, words: &str, press: Option<crate::press::Press>) -> String {
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
        USUAL.style(),
        escape(words)
    )
}
