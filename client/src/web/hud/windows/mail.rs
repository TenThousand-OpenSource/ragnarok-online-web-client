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
pub(in crate::web) static MAIL: Window = Window {
    id: "mail",
    open: |app| app.mail.open,
    frame: |_, view| staged(view, 100, 100, 309, 416),
    draw: App::mail_window,
    order: ordered(|app| app.mail.open, |app| app.pressed(crate::press::mail::Press::Window)),
    layer: Layer::Usual,
    drawn: None,
};

/// The letter being read, 300 by 400 (`WID_RODEXCONTENTSWND`).
pub(in crate::web) static MAIL_READ: Window = Window {
    id: "mail-read",
    open: |app| app.mail.reading.is_some(),
    frame: |_, view| staged(view, 120, 120, 300, 400),
    draw: App::mail_read_window,
    order: ordered(|app| app.mail.reading.is_some(), |app| app.pressed(crate::press::mail::Press::ReadClose)),
    layer: Layer::Usual,
    drawn: None,
};

/// The letter being written, in the same place (`WID_RODEXCOMPOSEWND`).
pub(in crate::web) static MAIL_WRITE: Window = Window {
    id: "mail-write",
    open: |app| app.mail.writing.is_some(),
    frame: |_, view| staged(view, 120, 120, 300, 400),
    draw: App::mail_write_window,
    order: ordered(|app| app.mail.writing.is_some(), |app| app.pressed(crate::press::mail::Press::WriteClose)),
    layer: Layer::Usual,
    drawn: None,
};

/// What the mail asks, in the client's message box: 200 of 480 down the middle of the screen.
pub(in crate::web) static MAIL_ASKS: Window = Window::question("mail-asks", |app| app.mail.asks.is_some(), App::mail_question);

/// How many of a thing go into the letter.
pub(in crate::web) static MAIL_COUNT: Window = Window::count("mail-count", |app| app.mail.counting.is_some(), App::mail_count_window);
