//! The mail's entries of the list of windows.

use super::*;

/// The notice of mail not read (`UIOpenMailBoxWnd`): 43 by 43, 234 from the right edge of the screen and 16
/// down. It is not dragged, and lies under the windows.
pub(in crate::web) static MAIL_NOTICE: Window = Window {
    id: "mail-notice",
    open: |app| app.mail.notice,
    frame: |_, view| format!(r#"class="a" style="{};pointer-events:auto""#, at(view.0 as i32 - 234, 16, 43, 43)),
    draw: App::mail_notice,
    order: None,
    layer: Layer::Under,
    drawn: None,
};

/// The mailbox, 309 by 416 (`UIWindowMgr::EnsureWindow`, `WID_RODEXINBOXWND`). Escape closes it with the
/// letter read or written beside it, as its icon does.
pub(in crate::web) static MAIL: Window = Window::usual(
    "mail",
    |app| app.mail.open,
    |_, view| staged(view, 100, 100, 309, 416),
    App::mail_window,
    |app| app.pressed(crate::press::mail::Press::Window),
);

/// The letter being read, 300 by 400 (`WID_RODEXCONTENTSWND`).
pub(in crate::web) static MAIL_READ: Window = Window::usual(
    "mail-read",
    |app| app.mail.reading.is_some(),
    |_, view| staged(view, 120, 120, 300, 400),
    App::mail_read_window,
    |app| app.pressed(crate::press::mail::Press::ReadClose),
);

/// The letter being written, in the same place (`WID_RODEXCOMPOSEWND`).
pub(in crate::web) static MAIL_WRITE: Window = Window::usual(
    "mail-write",
    |app| app.mail.writing.is_some(),
    |_, view| staged(view, 120, 120, 300, 400),
    App::mail_write_window,
    |app| app.pressed(crate::press::mail::Press::WriteClose),
);

/// What the mail asks, in the client's message box: 200 of 480 down the middle of the screen.
pub(in crate::web) static MAIL_ASKS: Window = Window::question("mail-asks", |app| app.mail.asks.is_some(), App::mail_question);

/// How many of a thing go into the letter.
pub(in crate::web) static MAIL_COUNT: Window = Window::count("mail-count", |app| app.mail.counting.is_some(), App::mail_count_window);
