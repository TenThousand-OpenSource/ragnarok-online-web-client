//! What the app keeps of the books that are read (`crate::books`).

use std::collections::HashMap;

use crate::books::Book;

/// The colour of a line read aloud, in the chat and over the screen (the client's 0xE1E4FF, which it writes
/// blue first).
pub(in crate::web) const BOOK_SAID: &str = "#ffe4e1";

/// The pictures the buttons of a book are (the sprites of the client's `data\sprite\book`): the bookmark, the
/// two arrows and the cross of the book's window, and the open book that is both "Read" and "Auto Read" on an
/// item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::web) enum BookButton {
    Mark,
    Before,
    After,
    Close,
    Read,
}

impl BookButton {
    pub(in crate::web) const ALL: [BookButton; 5] = [BookButton::Mark, BookButton::Before, BookButton::After, BookButton::Close, BookButton::Read];

    /// Its sprite under `data\sprite` (`UIBookWnd::OnCreate`, `UIItemCollectionWnd::OnCreate`).
    pub(in crate::web) fn sprite(self) -> &'static str {
        match self {
            BookButton::Mark => "book/책갈피",
            BookButton::Before => "book/책왼쪽",
            BookButton::After => "book/책오른쪽",
            BookButton::Close => "book/책닫기",
            BookButton::Read => "book/책읽기",
        }
    }
}

/// How a book is asked for: to be shown in its window (`ID_BOOKITEM`, 287, which a script's book is too), or
/// to be read aloud (`ID_AUTOBOOKITEM`, 288).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::web) enum Reading {
    Shown,
    Aloud,
}

/// A book being read aloud (`m_autoRead`): the line it has come to (`m_autoLine`), the line over the screen
/// (the client's `m_balloon`), and which reading this is, so that a time set for one that was since ended is
/// known for what it is.
pub(in crate::web) struct Aloud {
    pub line: usize,
    pub said: Option<String>,
    pub turn: u32,
}

/// The book in the window (`UIBookWnd`): the item it is, its lines, the page open, and its being read aloud,
/// while the window does not show.
pub(in crate::web) struct OpenBook {
    pub id: u32,
    pub book: Book,
    pub page: usize,
    pub aloud: Option<Aloud>,
}

/// What comes back to the books from outside the app's own handling (`Action::Book`).
pub(in crate::web) enum BookNews {
    /// The text of a book's file was read, or there is no such file: the book asked for, how, and at which page.
    Text(u32, Reading, usize, Option<String>),
    /// A picture of a button's sprite was read: which picture of it, its address and its size.
    Picture(BookButton, usize, String, (usize, usize)),
    /// The name of the item a book is has come: it is the book's title.
    Named,
    /// The next line of a book read aloud is due, for that reading.
    Line(u32),
}

/// The books.
#[derive(Default)]
pub(in crate::web) struct Books {
    pub open: Option<OpenBook>,
    /// The pictures of the buttons' sprites read so far, by button and picture; whether they were asked for.
    pub pictures: HashMap<(BookButton, usize), (String, (usize, usize))>,
    pub pictures_asked: bool,
    /// How many readings aloud were begun.
    pub turns: u32,
}
