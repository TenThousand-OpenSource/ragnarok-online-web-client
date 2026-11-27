//! The skills' entries of the list of windows: the hotkey bar, the list of skills, a skill's description, the
//! button that tells of a job level gained, and the box that asks the words of a Talkie Box or of Graffiti.

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

pub(in crate::web) static SKILLS: Window =
    Window::usual(
    "skills",
    |app| app.skills.open,
    // The tree is 560 wide and the list of rows 330 (`UINewSkillListWnd::SendMsg`, `UM_RESIZE` from `ID_MINI`).
    |app, _| framed(at(230, 60, if app.skills.mini { 330 } else { 560 }, 400)),
    App::skill_window,
    switched_off!(skills.open),
);

/// What Apply of the skill window asks before the levels planned are sent.
pub(in crate::web) static SKILL_ASKS: Window = Window::question("skill-asks", |app| app.skills.asks, App::skill_question);

pub(in crate::web) static SKILL_INFO: Window = Window::usual(
    "skillinfo",
    |app| app.skills.info.is_some(),
    |_, _| framed("left:330px;top:110px;width:304px".to_owned()),
    App::skill_info_window,
    |app| app.pressed(crate::press::skills::Press::InfoClose),
);

/// The button that tells of a job level gained (`WID_NOTIFYJOBLEVELUPWND`): 43 by 43 in the lower left corner
/// of the screen, with no place in the order.
pub(in crate::web) static JOB_LEVEL_TOLD: Window = Window {
    id: "job-level-told",
    open: |app| app.skills.level_told,
    frame: |_, view| crate::web::hud::status::told_place(0, view),
    draw: App::job_level_told,
    order: None,
    layer: Layer::Usual,
    drawn: None,
};

/// The box that asks the words of a Talkie Box or of Graffiti (`WID_TALKBOXTRAPINPUTWND`,
/// `WID_GRAFFISTRBOXWND`): 280 by 120, 185 across the client's stage and 60 above 300 of 480 down; its line
/// has the keyboard. It has no place in the order: Escape does not close it (`UIWindowMgr::IsOrderFixed` names
/// the box of Graffiti; the Talkie Box's is not named there, but `CloseWindow` has no arm for it either).
pub(in crate::web) static SKILL_WORDS: Window = Window {
    id: "skill-words",
    open: |app| app.skills.writing.is_some(),
    frame: |_, view| framed(format!("left:{}px;top:{}px;width:280px;height:120px", (view.0 as i32 - 640) / 2 + 185, view.1 as i32 * 300 / 480 - 60)),
    draw: App::skill_words_window,
    order: None,
    layer: Layer::Asks,
    drawn: Some(typed_over),
};
