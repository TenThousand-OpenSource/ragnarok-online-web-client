//! The entries of the list of windows for the books: the window a book is read in, and the line of a book
//! read aloud.

use super::*;
use crate::web::hud::books::{HEIGHT, WIDTH};

/// The book's window (`UIBookWnd`): 600 by 420, at the left of the client's 640-wide stage and the top of the
/// screen (`UIWindowMgr::EnsureWindow`, `WID_BOOKWND`: `CenterOnViewportX(0)`, `ScaleByScreenHeight(0)`). It
/// does not show while its book is read aloud. Escape closes it as its cross does: that the client's Escape
/// does so was not found in its code and is taken from what its other framed windows do.
pub(in crate::web) static BOOK: Window = Window {
    id: "book",
    open: |app| app.books.open.as_ref().is_some_and(|open| open.aloud.is_none()),
    frame: |_, view| staged(view, 0, 0, WIDTH, HEIGHT),
    draw: App::book_window,
    order: ordered(|app| app.books.open.as_ref().is_some_and(|open| open.aloud.is_none()), |app| app.pressed(crate::press::books::Press::Close)),
    layer: Layer::Usual,
    drawn: None,
};

/// The line of a book read aloud (`UIBookWnd::OnProcess`): its middle on the middle of the client's stage, 81
/// down the screen, in front of everything (`AddRenderedFront`). That the pointer goes through it is this
/// client's own choice: the client's balloon was not looked at for it.
pub(in crate::web) static BOOK_LINE: Window = Window {
    id: "book-line",
    open: |app| app.books.open.as_ref().and_then(|open| open.aloud.as_ref()).is_some_and(|aloud| aloud.said.is_some()),
    frame: |_, _| r#"class="a" style="left:50%;top:81px;transform:translateX(-50%);pointer-events:none""#.to_owned(),
    draw: App::book_line,
    order: None,
    layer: Layer::Top,
    drawn: None,
};
