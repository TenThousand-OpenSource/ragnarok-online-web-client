//! What the windows are framed with, each piece written once: a piece of the client's art, the title bar, the
//! white ground under it, the bar the buttons stand on, the frame of the system boxes, the cross in the
//! corner, and the canvas a character is drawn into.


use super::*;

/// A piece of the window's art at its place and size (stretched to it).
pub(in crate::web) fn art(name: &str, x: i32, y: i32, width: i32, height: i32) -> String {
    format!(r#"<div class="a" style="{};background:url('{}') 0 0 / 100% 100% no-repeat"></div>"#, at(x, y, width, height), picture(name))
}

/// A rectangle of one colour.
pub(in crate::web) fn fill(x: i32, y: i32, width: i32, height: i32, colour: &str) -> String {
    format!(r#"<div class="a" style="{};background:{colour}"></div>"#, at(x, y, width, height))
}

/// The frame of the original's system boxes around a space `width` by `height`, `top` down: corners and edges
/// of 14-pixel pieces, white inside.
pub(in crate::web) fn system_box(top: i32, width: i32, height: i32) -> String {
    let piece = |name: &str, x: i32, y: i32, w: i32, h: i32, repeat: &str| {
        format!(r#"<div class="a" style="{};background:url('{}') 0 0 {repeat}"></div>"#, at(x, top + y, w, h), picture(&format!("sysbox_{name}.bmp")))
    };
    [
        // White under the pieces too, a little inside the frame: where the page is laid out between pixels (a
        // box centred in an odd width, a zoomed page) the pieces part by a hair, and the ground must not show.
        format!(r#"<div class="a" style="{};background:#fff;border-radius:2px"></div>"#, at(3, top + 3, width - 6, height - 6)),
        piece("mu", 14, 0, width - 28, 14, "repeat-x"),
        piece("md", 14, height - 14, width - 28, 14, "repeat-x"),
        piece("lm", 0, 14, 14, height - 28, "repeat-y"),
        piece("rm", width - 14, 14, 14, height - 28, "repeat-y"),
        piece("lu", 0, 0, 14, 14, "no-repeat"),
        piece("ru", width - 14, 0, 14, 14, "no-repeat"),
        piece("ld", 0, height - 14, 14, 14, "no-repeat"),
        piece("rd", width - 14, height - 14, 14, 14, "no-repeat"),
    ]
    .concat()
}

/// The standard title bar: caps that keep their size and a middle that stretches, the title in black over a
/// white copy of itself.
pub(in crate::web) fn title_bar(width: i32, title: &str) -> String {
    format!(
        "{}{}{}{}",
        art("basic_interface/titlebar_left.bmp", 0, 0, 12, 17),
        art("basic_interface/titlebar_mid.bmp", 12, 0, width - 24, 17),
        art("basic_interface/titlebar_right.bmp", width - 12, 0, 12, 17),
        titled(17, 1, tr(title))
    )
}

/// The cross in the corner of a window `width` wide (`sys_close_off.bmp`, 11 square, 14 from the right and 3
/// down), with what pressing it does.
pub(in crate::web) fn cross(width: i32, press: impl Into<crate::press::Press>) -> String {
    press_button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", press)
}

/// The small button of a title bar that does nothing here, drawn as it stands (`sys_base_off.bmp`, 11 square).
pub(in crate::web) fn base_button(x: i32, y: i32) -> String {
    art("basic_interface/sys_base_off.bmp", x, y, 11, 11)
}

/// The bar the buttons of a window `width` by `height` stand on, along its bottom (`UIWindow::DrawButtonBar`:
/// ends of 21 that keep their size and a middle that stretches, 21 high).
pub(in crate::web) fn button_bar(width: i32, height: i32) -> String {
    [
        art("basic_interface/btnbar_left.bmp", 0, height - 21, 21, 21),
        art("basic_interface/btnbar_mid.bmp", 21, height - 21, width - 42, 21),
        art("basic_interface/btnbar_right.bmp", width - 21, height - 21, 21, 21),
    ]
    .concat()
}

/// The white ground of a window under its title: a grey line at its sides and its bottom, none at its top.
pub(in crate::web) fn panel_ground(x: i32, y: i32, width: i32, height: i32) -> String {
    format!(r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0;box-sizing:border-box"></div>"#, at(x, y, width, height))
}

/// The same ground with its two lower corners rounded, for a window that has no bar of buttons under it.
pub(in crate::web) fn panel_ground_round(x: i32, y: i32, width: i32, height: i32) -> String {
    format!(r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0;border-radius:0 0 3px 3px;box-sizing:border-box"></div>"#, at(x, y, width, height))
}

/// A white strip with a grey line at its two sides only: what stands between a title and what is under it.
pub(in crate::web) fn side_rimmed(x: i32, y: i32, width: i32, height: i32) -> String {
    format!(r#"<div class="a" style="{};background:#fff;border-left:1px solid #c8c8c8;border-right:1px solid #c8c8c8;box-sizing:border-box"></div>"#, at(x, y, width, height))
}

/// The white panel every plain window has under its title (`UIWindow::DrawPanelBackground`), with the bar its
/// OK and Cancel stand on at the bottom (`DrawButtonBar`).
pub(in crate::web) fn panel(width: i32, height: i32) -> String {
    format!("{}{}", panel_ground(0, 17, width, height - 17), button_bar(width, height))
}

/// The canvas a character is drawn into (`crate::web::figure`, which finds it by its `id`), seen through a
/// space at that place and of that size: the canvas is as large as a figure is, moved by `offset` so that the
/// part wanted shows, and takes no press.
pub(in crate::web) fn figure_canvas(id: &str, (x, y, width, height): (i32, i32, i32, i32), (left, top): (i32, i32)) -> String {
    let px = |length: i32| if length == 0 { "0".to_owned() } else { format!("{length}px") };
    format!(
        r#"<div class="a" style="{};overflow:hidden;pointer-events:none"><canvas id="{id}" width="{}" height="{}" style="position:absolute;left:{};top:{}"></canvas></div>"#,
        at(x, y, width, height),
        crate::web::figure::WIDTH,
        crate::web::figure::HEIGHT,
        px(left),
        px(top)
    )
}
