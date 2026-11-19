//! The texts of the windows: how the client asks for a font, a line at its place in that font (from its left,
//! ending at its right, about its middle, cut to a width), a window's title, a line with the colours written
//! into it (`^RRGGBB`), and how many of a thing there are in the corner of its cell.

// Not every one is taken up by the windows yet (they are being moved onto these).
#![allow(dead_code)]

use super::*;

/// How the client asks for a text: the family, the height and whether it is bold, as `UIWindow::DrawTextAt`
/// and the other drawing functions take them, and `UIStaticText::SetFont` and `CUIStaticText::SetFont` with
/// `SetFontHeight` and `SetBold` (whose "Gulim" is family 0: the first of `FontCache`'s names). A window that
/// writes a text in anything but the usual (0, 12) says so with one of these, never with a size in pixels.
#[derive(Clone, Copy)]
pub(in crate::web) struct Font(pub u8, pub u8, pub bool);

/// The usual text.
pub(in crate::web) const USUAL: Font = Font(0, 12, false);
/// The usual text in bold.
pub(in crate::web) const BOLD: Font = Font(0, 12, true);
/// The smaller text: how many of an item there are (`DrawItemWithQuantity`), the counts under the lists.
pub(in crate::web) const SMALL: Font = Font(0, 11, false);
/// The smaller text in bold.
pub(in crate::web) const SMALL_BOLD: Font = Font(0, 11, true);
/// What Basic Information and Status write their words and numbers in (`UIBasicInfoWnd`, `UIStatusWnd::Paint`).
pub(in crate::web) const FIGURES: Font = Font(1, 14, false);

impl Font {
    /// The style of a text asked for so: its size and the line it stands in (`crate::font::style`), and its
    /// weight. The face is the page's, also in a button, which would otherwise take the browser's own.
    pub(in crate::web) fn style(self) -> String {
        format!("font-family:inherit;{};font-weight:{}", crate::font::style(self.0, self.1), if self.2 { "bold" } else { "normal" })
    }
}

/// A line of text with its top-left corner at a place, in the usual font.
pub(in crate::web) fn text(x: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="left:{x}px;top:{y}px;color:{colour}">{}</div>"#, escape(words))
}

/// A line of text with its top-left corner at a place, in the font the client asks for there.
pub(in crate::web) fn text_in(font: Font, x: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="left:{x}px;top:{y}px;color:{colour};{}">{}</div>"#, font.style(), escape(words))
}

/// A line of text ending at `right`, in a window `width` wide, in the font the client asks for there.
pub(in crate::web) fn text_ending_in(font: Font, right: i32, width: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="right:{}px;top:{y}px;color:{colour};{}">{}</div>"#, width - right, font.style(), escape(words))
}

/// A line of text ending at `right`, in a window `width` wide.
pub(in crate::web) fn text_ending(right: i32, width: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="right:{}px;top:{y}px;color:{colour}">{}</div>"#, width - right, escape(words))
}

/// A line of text with its middle at `x`, in the font the client asks for there (`DrawCenteredText`): it has
/// 96 to stand in, and takes no press.
pub(in crate::web) fn text_centred(font: Font, x: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="left:{}px;top:{y}px;width:96px;text-align:center;pointer-events:none;color:{colour};{}">{}</div>"#, x - 48, font.style(), escape(words))
}

/// A text cut to the width it has, three dots where it was cut, as the client's fitted labels are
/// (`UIStaticText::SetTextFitted`); what was cut is told when it is pointed at, as the client's tooltip tells it.
pub(in crate::web) fn fitted(x: i32, y: i32, width: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="left:{x}px;top:{y}px;width:{width}px;overflow:hidden;text-overflow:ellipsis;color:{colour}" title="{0}">{0}</div>"#, escape(words))
}

/// A window's title at its place in the title bar: black over a white copy of itself (the page's `titled`).
pub(in crate::web) fn titled(x: i32, y: i32, words: &str) -> String {
    format!(r#"<div class="a t titled" style="left:{x}px;top:{y}px">{}</div>"#, escape(words))
}

/// What a style says to rim a text's letters in a colour, a pixel on each of the four sides (the client's
/// `DrawAmount` and its other outlined texts): `text-shadow:…`, without a `;` around it.
pub(in crate::web) fn outlined(rim: &str) -> String {
    format!("text-shadow:-1px 0 {rim},1px 0 {rim},0 -1px {rim},0 1px {rim}")
}

/// A line an NPC says, as HTML: `^RRGGBB` in it changes the colour of what follows (`crate::markup`). A line
/// of nothing is a blank that keeps its height.
pub(in crate::web) fn coloured(line: &str) -> String {
    let mut html = String::new();
    let mut open = false;
    for piece in crate::markup::pieces(line, &[]) {
        match piece {
            crate::markup::Piece::Colour(code) => {
                if std::mem::replace(&mut open, true) {
                    html.push_str("</span>");
                }
                html.push_str(&format!(r#"<span style="color:#{code}">"#));
            }
            crate::markup::Piece::Text(words) => html.push_str(&escape(words)),
            crate::markup::Piece::Tag { .. } => {}
        }
    }
    if open {
        html.push_str("</span>");
    }
    if html.is_empty() { "&nbsp;".to_owned() } else { html }
}

/// How many of a thing there are, written in the corner of its cell: the client's smaller text
/// (`DrawItemWithQuantity`, `CUIGameItemWithNum::Paint` and the hotkey bar's `DrawRightOutlinedText`: (0, 11)).
pub(in crate::web) fn count(amount: impl std::fmt::Display) -> String {
    format!(r#"<span class="n" style="{}">{amount}</span>"#, SMALL.style())
}
