//! The client's box that drops a list to pick one line of (`UIComboBoxDefWnd`), written once: the box with
//! what is picked and its button, and the list it drops.

use super::*;

/// The box's ground and its rim, as the windows that have one give them (`SetColor(0xE8, 0xE8, 0xE8)`,
/// `SetBorderColor(0x64, 0x64, 0x64)`).
const GROUND: &str = "#e8e8e8";
const RIM: &str = "#646464";
/// How high the box is (its button's picture, `txtbox_btn_a.bmp`, 18 square) and a line of its list. (The
/// height of a line of the client's `UIComboBoxEx` was not read: a line of its plain lists.)
pub(in crate::web) const COMBO: i32 = 18;
const LINE: i32 = 16;

/// The box at its place, so wide (`UIComboBoxDefWnd::OnCreate` and `Paint`): a rim and a ground, what is
/// picked 2 in and 3 down, and at its right the button that drops the list; a press on either drops it, or
/// puts it away.
pub(in crate::web) fn combo((x, y, width): (i32, i32, i32), words: &str, opens: impl Into<crate::press::Press>) -> String {
    let opens = opens.into();
    [
        format!(r#"<div class="a t" style="{};box-sizing:border-box;border:1px solid {RIM};background:{GROUND};"#, at(x, y, width, COMBO)),
        format!(r#"padding:2px 18px 0 1px;overflow:hidden;cursor:default" {}>{}</div>"#, pressed(opens.clone()), escape(words)),
        pictured_by(Convention::ABC, x + width - 18, y, (18, 18), "basic_interface/txtbox_btn", opens, ""),
    ]
    .concat()
}

/// The list a box at that place drops, so high at most (the box's `dropdownHeight`): its lines, each with the
/// colour it is written in, the one picked marked; `pick` is what pressing a line does. Written after
/// everything it lies over.
pub(in crate::web) fn combo_dropped<P: Into<crate::press::Press>>(
    (x, y, width): (i32, i32, i32),
    height: i32,
    lines: &[(String, String)],
    picked: usize,
    pick: impl Fn(usize) -> P
) -> String {
    let lines: String = lines
        .iter()
        .enumerate()
        .map(|(place, (words, colour))| {
            let ground = if place == picked { "background:#c6d6f7;" } else { "" };
            [
                format!(r#"<div class="t opt" style="height:{LINE}px;line-height:{LINE}px;padding:0 2px;overflow:hidden;cursor:default;"#),
                format!(r#"color:{colour};{ground}" {}>{}</div>"#, pressed(pick(place)), escape(words)),
            ]
            .concat()
        })
        .collect();
    format!(
        r#"<div class="a bar" style="{};box-sizing:border-box;border:1px solid {RIM};background:{GROUND}" {}>{lines}</div>"#,
        at(x, y + COMBO, width, height),
        lines_stepped(LINE)
    )
}
