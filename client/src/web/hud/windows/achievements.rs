//! The achievements' entries of the list of windows.

use super::*;

/// The achievements' window (`UIAchievementWnd`): 750 by 468 in the middle of the screen
/// (`UIWindowMgr::EnsureWindow`, `WID_ACHIEVEMENTWND`).
pub(in crate::web) static ACHIEVEMENTS: Window = Window::usual(
    "achievements",
    |app| app.achievements.open,
    |_, view| framed(format!("{};height:468px", centred(view, 750, 468))),
    App::achievements_window,
    |app| app.pressed(crate::press::achievements::Press::Window),
);

/// The achievements kept in sight (`UIAchTracingWnd`): 300 by 150, 350 from the screen's right and in the
/// middle of its height (`WID_ACHTRACINGWND`), with no ground: what is under it shows and is pressed through
/// it (`IsTransmitMouseInput`). It lies under the windows.
pub(in crate::web) static ACHIEVEMENTS_TRACED: Window = Window {
    id: "achievements-traced",
    open: |app| !app.achievements.traced.is_empty(),
    frame: |_, view| format!(r#"class="a" style="left:{}px;top:{}px;width:300px;height:150px""#, view.0 as i32 - 350, (view.1 as i32 - 150) / 2),
    draw: App::achievements_traced,
    order: None,
    layer: Layer::Under,
    drawn: None,
};

/// The line that tells of an achievement completed (`UIAchBalloonText`): across the screen, 50 high and 50
/// down it, in front of everything (`AddRenderedFront`), and the pointer goes through it.
pub(in crate::web) static ACHIEVEMENT_TOLD: Window = Window {
    id: "achievement-told",
    open: |app| app.achievements.balloon.is_some(),
    frame: |_, _| r#"class="a" style="left:0;top:50px;width:100%;height:50px;pointer-events:none""#.to_owned(),
    draw: App::achievement_told_line,
    order: None,
    layer: Layer::Top,
    drawn: None,
};
