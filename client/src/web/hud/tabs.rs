//! The tabs down a window's left side: the client's `UITabControl` in its layout 0, as the skill window and
//! the inventory make one (`UINewSkillListWnd::OnCreate`, `UIItemWnd::OnCreate`).
//!
//! A tab's words are written a letter under a letter (`UIWindow::DrawVerticalText`), each in its own line of
//! the usual text and in the middle of the widest. A tab is as tall as its letters and the room its window
//! asks round them (`AddTab`), and begins on the last line of the one above (`DrawVerticalTab`). The open tab
//! has a line over, left of and under it and none at its right, where it runs into the window; the others
//! stand one pixel further right and are closed by a line there. Both grounds are white unless the window
//! gives a tab a colour (`FillTab`), and both texts black: these are what `Configure` makes of the nothing the
//! two windows hand it.

use super::*;

/// How wide the strip of tabs is (`FitToTabs`: the 17 both windows ask for, and one).
pub(super) const SIDE_TABS: i32 = 18;

/// The colour of a tab's lines (`Configure`: `0xC3C3C3`).
const LINE: &str = "#c3c3c3";

/// How tall the letters of a tab's words are, one under another (`VerticalTextHeight`).
fn letters_tall(words: &str) -> i32 {
    words.chars().count() as i32 * i32::from(crate::font::cell(0, 12))
}

/// How tall each tab is: its letters and `padding`; every one as tall as the tallest when the window asks
/// for that (`RecalculateWidths`, `uniformWidths`).
fn side_tab_heights(words: &[String], padding: i32, uniform: bool) -> Vec<i32> {
    let heights: Vec<i32> = words.iter().map(|words| letters_tall(words) + padding).collect();
    let tallest = heights.iter().copied().max().unwrap_or(0);
    if uniform { vec![tallest; heights.len()] } else { heights }
}

/// How tall the whole strip is (`FitToTabs`).
pub(super) fn side_tabs_tall(words: &[String], padding: i32, uniform: bool) -> i32 {
    side_tab_heights(words, padding, uniform).iter().map(|height| height - 1).sum::<i32>() + 1
}

/// The strip with its top-left corner at a place of the window: each tab its words, the colour of its ground
/// if the window gives it one, and what makes it something to press (and to drop on).
pub(super) fn side_tabs(x: i32, y: i32, padding: i32, uniform: bool, open: usize, tabs: &[(String, Option<&str>, String)]) -> String {
    let words: Vec<String> = tabs.iter().map(|(words, ..)| words.clone()).collect();
    let font = Font(0, 12, false).style();
    let mut strip = String::new();
    let mut top = y;
    for (index, ((words, ground, more), height)) in tabs.iter().zip(side_tab_heights(&words, padding, uniform)).enumerate() {
        // The open tab from the strip's left edge, with no line at its right; the others one further right.
        let (shift, right) = if index == open { (0, "border-right:0;") } else { (1, "") };
        let room = height - letters_tall(words);
        let letters: String = words.chars().map(|letter| format!("<span>{}</span>", escape(&letter.to_string()))).collect();
        strip.push_str(&format!(
            r#"<button type="button" class="a" style="{};padding:0;border:1px solid {LINE};{right}border-radius:2px 0 0 2px;box-sizing:border-box;background:{};color:#000;cursor:pointer;{font}" {more}><span style="position:absolute;left:2px;top:{}px;display:flex;flex-direction:column;align-items:center">{letters}</span></button>"#,
            at(x + shift, top, SIDE_TABS - shift, height),
            ground.unwrap_or("#fff"),
            // (The letters begin half the room down, the odd pixel over them; the tab's own line is one of it.)
            room / 2 + room % 2 - 1,
        ));
        top += height - 1;
    }
    strip
}
