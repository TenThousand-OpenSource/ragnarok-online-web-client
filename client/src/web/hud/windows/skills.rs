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

pub(in crate::web) static SKILLS: Window = Window {
    id: "skills",
    open: |app| app.skills.open,
    frame: |_, _| framed(at(230, 60, 560, 400)),
    draw: App::skill_window,
    order: ordered(|app| app.skills.open, switched_off!(skills.open)),
    layer: Layer::Usual,
    drawn: None,
};

pub(in crate::web) static SKILL_INFO: Window = Window {
    id: "skillinfo",
    open: |app| app.skills.info.is_some(),
    frame: |_, _| framed("left:330px;top:110px;width:304px".to_owned()),
    draw: App::skill_info_window,
    order: ordered(|app| app.skills.info.is_some(), |app| app.pressed(crate::press::skills::Press::InfoClose)),
    layer: Layer::Usual,
    drawn: None,
};
