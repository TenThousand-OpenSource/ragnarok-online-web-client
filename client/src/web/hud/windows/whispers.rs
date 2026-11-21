//! The entries of the list of windows for the talks with one player each: a window to a talk, as many as
//! there are places for them (`crate::whispers::WINDOWS`), the one menu of them, and the box that asks the away
//! message.

use super::*;
use crate::web::parts::menued;

/// The window of the talk in one place: where it was opened, with the right button's menu on all of it. It
/// lies in the order of windows as any other, and Escape closes it as its cross does.
macro_rules! talk_window {
    ($name:ident, $id:literal, $place:literal) => {
        pub(in crate::web) static $name: Window = Window::usual(
            $id,
            |app| app.whispers.talks[$place].is_some(),
            |app, _| format!("{} {}", framed(app.whisper_style($place)), menued(crate::press::things::Menu::Whisper($place))),
            |app| app.whisper_window($place),
            |app| app.pressed(crate::press::whispers::Press::Close($place)),
        );
    };
}

talk_window!(WHISPER_0, "whisper-0", 0);
talk_window!(WHISPER_1, "whisper-1", 1);
talk_window!(WHISPER_2, "whisper-2", 2);
talk_window!(WHISPER_3, "whisper-3", 3);
talk_window!(WHISPER_4, "whisper-4", 4);
talk_window!(WHISPER_5, "whisper-5", 5);
talk_window!(WHISPER_6, "whisper-6", 6);
talk_window!(WHISPER_7, "whisper-7", 7);

/// The windows of the talks, by their places.
pub(in crate::web) static WHISPERS: [&Window; crate::whispers::WINDOWS] =
    [&WHISPER_0, &WHISPER_1, &WHISPER_2, &WHISPER_3, &WHISPER_4, &WHISPER_5, &WHISPER_6, &WHISPER_7];

/// The menu of a talk's window, opened with the right button on it, where the pointer was.
pub(in crate::web) static WHISPER_MENU: Window =
    Window::menu("whisper-menu", |app| app.whispers.menu.is_some(), |app, _| menu_frame(app.whispers.menu.map(|(at, _)| at)), App::whisper_menu);

/// The box that asks the away message (`WID_AUTOMESSAGEWND`): 280 by 48, 185 across the client's stage and
/// 180 of 480 down (`CenterOnViewportX(185)`, `CenterOnViewportY(180)`); its line has the keyboard, what
/// stands in it ready to be typed over (`SelectAll`). It has no place in the order (`IsOrderFixed`).
pub(in crate::web) static AWAY: Window = Window {
    id: "away",
    open: |app| app.whispers.away_asked,
    frame: |_, view| framed(format!("left:{}px;top:{}px;width:280px;height:48px", (view.0 as i32 - 640) / 2 + 185, (view.1 as i32 - 480) / 2 + 180)),
    draw: App::away_window,
    order: None,
    layer: Layer::Asks,
    drawn: Some(typed_over),
};
