//! The books that are read (`crate::books`, the client's `UIBookWnd`): one asked for by an NPC's script or
//! from the window of an item, its pages turned, its bookmark kept, its lines said one by one when it is read
//! aloud.

use super::*;
use crate::books::Book;
use crate::i18n::tr;
use wasm_bindgen::JsCast;

/// How wide a line of a book may be: the viewer's width, the window's 600 less 60, less 10
/// (`UIBookWnd::OnCreate`, `UM_GETBOOKCONTENTS`).
const LINE_WIDTH: f64 = 530.0;
/// How long a line read aloud stays before the next (`UIBookWnd::OnProcess`: 4000 milliseconds).
const LINE_LASTS: i32 = 4000;

/// What tells how wide a piece of text is as the page writes the client's usual text (`crate::font`: family
/// 0, height 12, in the page's own face). The client adds up the widths its own font has; the page's letters
/// are not to the pixel as wide, so a long line may be cut a letter sooner or later than in the client.
fn widths() -> impl FnMut(&str) -> f64 {
    let context = crate::web::document()
        .and_then(|document| document.create_element("canvas").ok())
        .and_then(|canvas| canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .and_then(|canvas| canvas.get_context("2d").ok().flatten())
        .map(|context| context.unchecked_into::<web_sys::CanvasRenderingContext2d>());
    let face = crate::web::document()
        .and_then(|document| document.document_element())
        .and_then(|root| web_sys::window()?.get_computed_style(&root).ok().flatten())
        .and_then(|style| style.get_property_value("--face").ok())
        .filter(|face| !face.trim().is_empty())
        .unwrap_or_else(|| "Arial, sans-serif".to_owned());
    if let Some(context) = &context {
        context.set_font(&format!("{}px {face}", crate::font::pixels(0, 12)));
    }
    let mut known: HashMap<String, f64> = HashMap::new();
    move |text: &str| {
        if let Some(width) = known.get(text) {
            return *width;
        }
        // Where the page cannot measure, a letter is taken as half as wide as it is tall.
        let width = context.as_ref().and_then(|context| context.measure_text(text).ok()).map_or(text.chars().count() as f64 * 6.5, |measured| measured.width());
        known.insert(text.to_owned(), width);
        width
    }
}

impl App {
    /// A press of this system (`crate::press::books`).
    pub(super) fn books_pressed(&mut self, press: crate::press::books::Press) {
        use crate::press::books::Press;
        match press {
            Press::Mark => self.book_marked(),
            // The arrows do nothing while the book is read aloud (`ID_PREVIEW`, `ID_NEXTVIEW`).
            Press::Before | Press::After => {
                if let Some(page) = self.books.open.as_ref().filter(|open| open.aloud.is_none()).map(
                    |open| open.book.turned(open.page, press == Press::After)
                ) {
                    self.book_shown(page);
                }
            }
            // The cross keeps the page as the bookmark does, and the window closes (`ID_CLOSE`).
            Press::Close => {
                self.book_marked();
                self.book_closed();
            }
            Press::Read | Press::Aloud => {
                if let Some((item, _)) = self.bag.info {
                    self.book_asked(item, if press == Press::Read { Reading::Shown } else { Reading::Aloud }, 0);
                }
            }
        }
    }

    /// The left and right arrows turn the pages of a book that is open (`UIWindowMgr`, `VK_LEFT` and
    /// `VK_RIGHT`: `UM_LEFTPUSH` and `UM_RIGHTPUSH` go to the book's window before any other). Gives back
    /// whether the key was the book's.
    pub(in crate::web::app) fn book_key(&mut self, key: &str) -> bool {
        if !self.books.open.as_ref().is_some_and(|open| open.aloud.is_none()) || !matches!(key, "ArrowLeft" | "ArrowRight") {
            return false;
        }
        self.books_pressed(if key == "ArrowLeft" { crate::press::books::Press::Before } else { crate::press::books::Press::After });
        true
    }

    /// A book is asked for (`CGameMode::ReceiveBookOpen`, and `ID_BOOKITEM` and `ID_AUTOBOOKITEM` of the
    /// window of an item): its file is read, and so are the item's name, which is the book's title, and the
    /// pictures of the buttons.
    pub(in crate::web::app) fn book_asked(&mut self, book: u32, how: Reading, page: usize) {
        data::want_items(std::iter::once(book), || dispatch(Action::Book(BookNews::Named)));
        self.book_pictures_wanted();
        wasm_bindgen_futures::spawn_local(async move {
            let text = data::book_text(book).await;
            dispatch(Action::Book(BookNews::Text(book, how, page, text)));
        });
    }

    /// The pictures of the buttons' sprites are read, once: the first of each, and all nine of the open book
    /// that turns its pages on "Auto Read".
    pub(in crate::web::app) fn book_pictures_wanted(&mut self) {
        if std::mem::replace(&mut self.books.pictures_asked, true) {
            return;
        }
        for button in BookButton::ALL {
            wasm_bindgen_futures::spawn_local(async move {
                for picture in 0..if button == BookButton::Read { 9 } else { 1 } {
                    if let Ok((address, size, _)) = figure::frame_picture(button.sprite(), 0, picture).await {
                        dispatch(Action::Book(BookNews::Picture(button, picture, address, size)));
                    }
                }
            });
        }
    }

    /// Something of the books has come back (`Action::Book`).
    pub(in crate::web::app) fn book_news(&mut self, news: BookNews) {
        match news {
            BookNews::Text(book, how, page, text) => self.book_read(book, how, page, text),
            BookNews::Picture(button, picture, address, size) => {
                self.books.pictures.insert((button, picture), (address, size));
                self.redraw(&[&wnd::BOOK, &wnd::ITEM_INFO]);
            }
            BookNews::Named => self.redraw(&[&wnd::BOOK]),
            BookNews::Line(turn) => self.book_line_due(turn),
        }
    }

    /// The file of a book asked for was read (`UM_GETBOOKCONTENTS`). One the client has no file of is told in
    /// the chat in red; the client's words for it are Korean and in no table ("책이 없습니다"), here they are
    /// this client's own. The book that is already open is left as it is when it is asked to be shown again;
    /// asked for otherwise it keeps its page, and a book that was not open opens at the page asked for. Read
    /// aloud it is said from that page on; while a book is being read aloud, "Auto Read" ends the reading
    /// and closes the window, whichever book it is pressed on. (The client knows the open book by its title;
    /// here it is known by its number.)
    fn book_read(&mut self, id: u32, how: Reading, asked: usize, text: Option<String>) {
        let Some(text) = text else { return self.chat_line(tr("There is no such book.").to_owned(), RED) };
        let aloud = self.books.open.as_ref().is_some_and(|open| open.aloud.is_some());
        let page = match self.books.open.as_ref().filter(|open| open.id == id) {
            Some(_) if !aloud && how == Reading::Shown => return,
            Some(open) => open.page,
            None => asked,
        };
        self.books.open = Some(OpenBook { id, book: Book::bound(&text, LINE_WIDTH, &mut widths()), page, aloud: None });
        match how {
            Reading::Shown => self.book_shown(page),
            Reading::Aloud if aloud => self.book_closed(),
            Reading::Aloud => self.book_aloud(page),
        }
    }

    /// The book shows a page (`UM_READBOOK`): the one asked for, or for 0 the one its bookmark was left at.
    /// Reading aloud ends, and the window comes to the front.
    fn book_shown(&mut self, page: usize) {
        let marked = self.books.open.as_ref().and_then(|open| self.bookmark(open.id));
        let Some(open) = &mut self.books.open else { return };
        open.aloud = None;
        open.page = open.book.opened_at(page, marked);
        self.redraw(&[&wnd::BOOK, &wnd::BOOK_LINE]);
    }

    /// The book is read aloud from a page on (`UM_AUTOREAD`): its window does not show, and its first line is
    /// said at once.
    fn book_aloud(&mut self, page: usize) {
        let marked = self.books.open.as_ref().and_then(|open| self.bookmark(open.id));
        self.books.turns += 1;
        let turn = self.books.turns;
        let Some(open) = &mut self.books.open else { return };
        open.page = open.book.opened_at(page, marked);
        open.aloud = Some(BookAloud { line: open.book.aloud_from(open.page), said: None, turn });
        self.redraw(&[&wnd::BOOK, &wnd::BOOK_LINE]);
        self.book_line_due(turn);
    }

    /// The next line of the book read aloud is due (`UIBookWnd::OnProcess`): it is written into the chat and
    /// over the screen without its colours, and the one after is due 4 seconds later; when the lines are at
    /// an end the window closes. A line that is empty shows nothing over the screen.
    fn book_line_due(&mut self, turn: u32) {
        let Some(open) = self.books.open.as_mut().filter(|open| open.aloud.as_ref().is_some_and(|aloud| aloud.turn == turn)) else { return };
        let line = open.aloud.as_ref().map_or(0, |aloud| aloud.line);
        let Some((said, next)) = open.book.aloud(line).map(|(said, next)| (crate::markup::uncoloured(said), next)) else { return self.book_closed() };
        open.aloud = Some(BookAloud { line: next, said: (!said.trim().is_empty()).then(|| said.clone()), turn });
        self.chat_line(said, BOOK_SAID);
        self.redraw(&[&wnd::BOOK_LINE]);
        after(LINE_LASTS, Action::Book(BookNews::Line(turn)));
    }

    /// The window closes, and a reading aloud ends with it (`UM_ENDPUSH`, `CloseWindow(WID_BOOKWND)`).
    fn book_closed(&mut self) {
        self.books.open = None;
        self.redraw(&[&wnd::BOOK, &wnd::BOOK_LINE]);
    }

    /// Under which name the browser keeps the page a book's bookmark was left at. (The client keeps it in
    /// the registry, under the server's and the character's names, by the book's title: `ID_BOOKMARK`,
    /// `UIBookWnd::LoadBookmark`. Here it is kept by the character's name and the book's number.)
    fn bookmark_name(&self, book: u32) -> String {
        format!("roweb.bookmark.{}.{book}", self.game.as_ref().map(|game| game.name.as_str()).unwrap_or_default())
    }

    /// The page the bookmark of a book was left at, if it has one.
    fn bookmark(&self, book: u32) -> Option<usize> {
        stored(&self.bookmark_name(book))?.parse().ok()
    }

    /// The page open is kept as the book's bookmark (`ID_BOOKMARK`).
    fn book_marked(&mut self) {
        if let Some(open) = &self.books.open {
            store(&self.bookmark_name(open.id), Some(&open.page.to_string()));
        }
    }
}
