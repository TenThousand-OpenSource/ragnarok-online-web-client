//! The window a book is read in (the client's `UIBookWnd`), the line of a book read aloud over the screen
//! (its `UITransBalloonText`), and the two buttons the window of an item has for a book that is carried
//! (`UIItemCollectionWnd`).

use super::*;
use crate::press::books::Press;
use crate::web::app::{BOOK_SAID, BookButton};

/// The book's window is 600 by 420 (`UIWindowMgr::EnsureWindow`, `WID_BOOKWND`).
pub(in crate::web) const WIDTH: i32 = 600;
pub(in crate::web) const HEIGHT: i32 = 420;

impl App {
    /// A button that is the first picture of a sprite (`UISpriteButton`, as large as that picture and the
    /// same under the pointer and pressed), with what it tells when pointed at. Nothing until the picture
    /// was read.
    fn book_button(&self, button: BookButton, x: i32, y: i32, tip: &str, press: Press) -> String {
        let Some((address, (width, height))) = self.books.pictures.get(&(button, 0)) else { return String::new() };
        [
            format!(
                r#"<button type="button" class="a" style="{};border:0;padding:0;background:none" {} data-tip="{}">"#,
                at(x, y, *width as i32, *height as i32),
                pressed(press),
                escape(tip)
            ),
            format!(r#"<img src="{address}" alt="" draggable="false" style="display:block"></button>"#)
        ].concat()
    }

    /// The book's window (`UIBookWnd::OnCreate`, `Paint`): a title bar with no words of its own, in which
    /// the book's title (the name of the item it is), the page and the number of pages are written from 72
    /// left of the middle, 2 down; under it the paper in the book's colour (the skin's white for a book that
    /// names none), a pixel in from the sides and the bottom; the page's lines in a viewer at 30, 45, each 3
    /// in and 18 under the last from 4 down, in the client's usual text; the bookmark at 10, 25, the arrows
    /// at 57 and 35 from the right edge, 25 down, and the cross 25 from the right and the bottom. The arrows
    /// on the keyboard turn the pages too. (The viewer cuts a line longer than 100 letters once more; no line
    /// 530 wide is that long, and it is not done here.)
    pub(in crate::web) fn book_window(&self) -> String {
        let Some(open) = &self.books.open else { return String::new() };
        let paper = crate::said::colour::css(open.book.paper.unwrap_or([255, 255, 255]));
        let lines: String = crate::books::carried(open.book.page(open.page))
            .iter()
            .enumerate()
            .map(|(row, line)| format!(
                r#"<div class="a t" style="left:33px;top:{}px;white-space:pre;color:{BLACK}">{}</div>"#,
                49 + row as i32 * 18,
                coloured(line)
            ))
            .collect();
        let title = format!("{} ( {} / {} )", data::item(open.id).map(|(name, _)| name).unwrap_or_default(), open.page, open.book.pages());
        format!(
            "{}{}{}{}{lines}{}{}{}{}",
            title_bar(WIDTH, ""),
            panel_ground(0, 17, WIDTH, HEIGHT - 17),
            fill(1, 17, WIDTH - 2, HEIGHT - 18, &paper),
            text(WIDTH / 2 - 72, 2, BLACK, &title),
            self.book_button(BookButton::Mark, 10, 25, &Self::words(1296, "Bookmark"), Press::Mark),
            self.book_button(BookButton::Before, WIDTH - 57, 25, &Self::words(1297, "Previous"), Press::Before),
            self.book_button(BookButton::After, WIDTH - 35, 25, &Self::words(1298, "Next"), Press::After),
            self.book_button(BookButton::Close, WIDTH - 25, HEIGHT - 25, &Self::words(1299, "Close"), Press::Close),
        )
    }

    /// The line of a book read aloud (`UIBookWnd::OnProcess`): the client's see-through black box with a
    /// rounded grey outline (`UITransBalloonText`: `0x66000000`, the outline `0x5C5C5C`), the line 6 in and 4
    /// down in pale pink with a dark edge.
    pub(in crate::web) fn book_line(&self) -> String {
        let Some(said) = self.books.open.as_ref().and_then(|open| open.aloud.as_ref()).and_then(|aloud| aloud.said.as_ref()) else { return String::new() };
        [
            format!(r#"<div class="t" style="padding:4px 6px;background:rgba(0,0,0,.4);border:1px solid #5c5c5c;"#),
            format!(r#"border-radius:3px;color:{BOOK_SAID};text-shadow:1px 1px 0 #000;white-space:pre">{}</div>"#, escape(said))
        ].concat()
    }

    /// "Read" and "Auto Read" in the window of an item (`UIItemCollectionWnd::SendMsg`, `UM_SETINFO`): on a
    /// book of the client's list (`IsBookItem`) of which the character carries one, two buttons side by side
    /// from 6, 6, both the open book of `책읽기.spr`, each as large as its first picture. The second turns
    /// its pages, a picture every tenth of a second (`UIItemCollectionWnd::OnProcess`): the pictures of the
    /// sprite's one action in turn, each in the middle of the button and cut to it (`UISpriteButton::Paint`).
    pub(in crate::web) fn book_buttons(&self, item: u32) -> String {
        if !data::is_book(item) || !self.bag.items.iter().any(|carried| carried.id == item) {
            return String::new();
        }
        let Some((_, (width, height))) = self.books.pictures.get(&(BookButton::Read, 0)) else { return String::new() };
        let (width, height) = (*width as i32, *height as i32);
        let pictures: Vec<&(String, (usize, usize))> = (0..).map_while(|picture| self.books.pictures.get(&(BookButton::Read, picture))).collect();
        let count = pictures.len();
        let turning: String = pictures
            .iter()
            .enumerate()
            .map(|(picture, (address, (wide, tall)))| {
                // Each picture shows for its tenth of a second of the round and is hidden for the rest.
                let (from, to) = (picture as f32 / count as f32, (picture + 1) as f32 / count as f32);
                let (values, times) = match (picture == 0, picture + 1 == count) {
                    (true, true) => ("visible;visible".to_owned(), "0;1".to_owned()),
                    (true, false) => ("visible;hidden;hidden".to_owned(), format!("0;{to:.4};1")),
                    (false, true) => ("hidden;visible;visible".to_owned(), format!("0;{from:.4};1")),
                    (false, false) => ("hidden;visible;hidden;hidden".to_owned(), format!("0;{from:.4};{to:.4};1")),
                };
                [
                    format!(
                        r#"<image href="{address}" x="{}" y="{}" width="{wide}" height="{tall}" visibility="hidden">"#,
                        width / 2 - *wide as i32 / 2,
                        height / 2 - *tall as i32 / 2
                    ),
                    format!(r#"<animate attributeName="visibility" calcMode="discrete" values="{values}" keyTimes="{times}" "#),
                    format!(r#"dur="{}ms" repeatCount="indefinite"/></image>"#, count * 100)
                ].concat()
            })
            .collect();
        [
            format!(
                r#"{}<button type="button" class="a" style="{};border:0;padding:0;background:none;overflow:hidden" {} "#,
                self.book_button(BookButton::Read, 6, 6, &Self::words(1294, "Read"), Press::Read),
                at(6 + width, 6, width, height),
                pressed(Press::Aloud)
            ),
            format!(r#"data-tip="{}"><svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" "#, escape(&Self::words(1295, "Auto Read"))),
            format!(r#"style="display:block">{turning}</svg></button>"#)
        ].concat()
    }
}
