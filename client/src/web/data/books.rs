//! The books (`crate::books`): which items are books, out of the client's `data\bookitemnametable.txt`, read
//! as it is once at the start; and the text of a book, a file of the client's `data\book` folder read when
//! the book is opened.

use super::*;

thread_local! {
    /// The items that are books (the client's `g_bookItemNames`).
    static BOOKS: RefCell<std::collections::HashSet<u32>> = RefCell::new(std::collections::HashSet::new());
}

/// A file of the client's texts as text: they are written in the Korean code page, whatever language they
/// are in.
fn korean(bytes: &[u8]) -> String {
    web_sys::TextDecoder::new_with_label("euc-kr").ok().and_then(|korean| korean.decode_with_u8_array(bytes).ok()).unwrap_or_else(|| String::from_utf8_lossy(bytes).into_owned())
}

/// Fetches the list of books; it is there some time after.
pub(super) fn fetch() {
    wasm_bindgen_futures::spawn_local(async {
        if let Ok(bytes) = crate::web::fetch_asset("data/bookitemnametable.txt").await {
            BOOKS.with_borrow_mut(|books| books.extend(crate::books::listed(&korean(&bytes))));
        }
    });
}

/// Whether an item is a book (`IsBookItem`).
pub fn is_book(item: u32) -> bool {
    BOOKS.with_borrow(|books| books.contains(&item))
}

/// The text of a book (`book\<number>.txt`), or nothing if the client has no such file.
pub async fn book_text(book: u32) -> Option<String> {
    crate::web::fetch_asset(&format!("data/book/{book}.txt")).await.ok().map(|bytes| korean(&bytes))
}
