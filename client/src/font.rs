//! The size the client writes a text in, from what its windows ask for.
//!
//! A window asks for a text by a font family and a height (`UIWindow::DrawTextAt(x, y, text, bytes, family,
//! height, colour, bold…)`). `TextContext::SetFont` turns that into a font: family 0 is "the service's own",
//! which for the English and the Russian service is Arial with the height made three more; any other family is
//! taken as named, with the height as asked (family 1 is Arial too). `FontEntry::CreateSystemFont` then makes a
//! font whose cell is that tall, without smoothing. A cell holds the letters and a little room over them: for
//! Arial at these sizes, two pixels.
//!
//! So most of the client's text, asked for as (0, 12), is Arial 13 pixels in a line 15 tall; the labels of
//! Basic Information, asked for as (1, 14), are Arial 12 in a line 14 tall.

/// How tall the cell of a text is, in pixels: the line it stands in. A height of 0 is the client's usual 12.
pub fn cell(family: u8, height: u8) -> u8 {
    let height = if height == 0 { 12 } else { height };
    if family == 0 { height + 3 } else { height }
}

/// How large the letters of a text are, in the pixels a page counts a font's size in.
pub fn pixels(family: u8, height: u8) -> u8 {
    cell(family, height).saturating_sub(2)
}

/// The style of a text asked for by that family and height: its size and the line it stands in. The face and
/// the lack of smoothing are the page's own (`--face`).
pub fn style(family: u8, height: u8) -> String {
    format!("font-size:{}px;line-height:{}px", pixels(family, height), cell(family, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_usual_text_is_thirteen_pixels_in_a_line_of_fifteen() {
        assert_eq!((pixels(0, 12), cell(0, 12)), (13, 15));
        // Asked for with no height at all, it is the same.
        assert_eq!((pixels(0, 0), cell(0, 0)), (13, 15));
    }

    #[test]
    fn a_named_family_is_as_tall_as_asked() {
        // The labels of Basic Information.
        assert_eq!((pixels(1, 14), cell(1, 14)), (12, 14));
        assert_eq!(style(1, 14), "font-size:12px;line-height:14px");
    }

    #[test]
    fn the_smaller_and_the_larger_of_the_services_own() {
        assert_eq!(pixels(0, 11), 12);
        assert_eq!(pixels(0, 13), 14);
    }
}
