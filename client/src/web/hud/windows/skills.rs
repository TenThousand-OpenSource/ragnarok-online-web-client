//! The skills' entries of the list of windows: the hotkey bar, the list of skills, a skill's description.

use super::*;

pub(in crate::web) static HOTKEYS: Window = Window {
    id: "hotkeys",
    open: |app| !app.skills.hotkeys_hidden,
    frame: |app, _| framed(at(230, 0, 290, (i32::from(app.skills.hotkey_rows) + 1) * 33 + 1)),
    draw: App::hotkeys,
    order: None,
    layer: Layer::Usual,
    drawn: None,
};

pub(in crate::web) static SKILLS: Window = Window::usual("skills", |app| app.skills.open, |_, _| framed(at(230, 60, 560, 400)), App::skill_window, switched_off!(skills.open));

pub(in crate::web) static SKILL_INFO: Window = Window::usual(
    "skillinfo",
    |app| app.skills.info.is_some(),
    |_, _| framed("left:330px;top:110px;width:304px".to_owned()),
    App::skill_info_window,
    |app| app.pressed(crate::press::skills::Press::InfoClose),
);
