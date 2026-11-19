//! The gauges of the windows, their pieces written once: the fill of a gauge made of the client's pictures
//! (`gzeblue_*`, `gzered_*`: life, spirit, Activity Points, a homunculus's and a mercenary's), the two numbers
//! over it, and the bar that is a plain colour in a rim (experience, hunger, intimacy).
//!
//! A window draws the back of its gauge itself (the backs differ), and decides itself when a gauge shows
//! nothing: these write only what they are asked for.

// Not taken up by the windows yet (they are being moved onto these).
#![allow(dead_code)]

use super::*;

/// When a gauge is filled in red and not in blue.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum Red {
    /// Under a quarter of its most: life and spirit.
    UnderQuarter,
    /// Once it is full: Activity Points (`UIBarGraphPlayerAP`).
    Full,
    Never,
}

/// The fill of a gauge at `x`, `y`: its left end 4 wide, its middle as much of `room` as `now` is of `most`
/// (cut to a whole pixel), and its right end 4 wide after it, all 8 high. `most` must be more than nought.
pub(in crate::web) fn filling(x: i32, y: i32, room: i32, now: i64, most: i64, red: Red) -> String {
    let colour = match red {
        Red::UnderQuarter if now * 4 < most => "red",
        Red::Full if now == most => "red",
        _ => "blue",
    };
    let fill = (i64::from(room) * now.max(0) / most) as i32;
    [
        art(&format!("basic_interface/gze{colour}_left.bmp"), x, y, 4, 8),
        art(&format!("basic_interface/gze{colour}_mid.bmp"), x + 4, y, fill, 8),
        art(&format!("basic_interface/gze{colour}_right.bmp"), x + 4 + fill, y, 4, 8),
    ]
    .concat()
}

/// The two numbers over a gauge `width` wide at `x`, `y` (`UIBarGraphPlayer::Paint`): in its middle, 2 above
/// it, as (1, 13), each number three wide at least.
pub(in crate::web) fn numbers_over(x: i32, y: i32, width: i32, now: i64, most: i64) -> String {
    format!(r#"<div class="a t centre" style="{};{};white-space:pre">{now:3}  /  {most:3}</div>"#, at(x, y - 2, width, 13), Font(1, 13, false).style())
}

/// A bar that is a plain colour in a rim, `width` by 6 (the client's `UIINT64BarGraph` and `UIBarGraph4`):
/// `now` of `most` filled, in red under a quarter when `warns`; nothing at all when there is more than the
/// most. The rim, the ground and the fill are colours of the client's skin (its `colorchip.bmp`, 26,2, 30,2
/// and 2,6).
pub(in crate::web) fn plain_bar(x: i32, y: i32, width: i32, now: i64, most: i64, warns: bool) -> String {
    if most < now {
        return String::new();
    }
    let ratio = if most > 0 { now.max(0) as f64 / most as f64 } else { 0.0 };
    let colour = if warns && ratio * 100.0 < 25.0 { "#f00" } else { "#567fcf" };
    format!(
        r#"<div class="a" style="{};background:#fff;border:1px solid #afafaf;box-sizing:border-box"><div style="width:{}px;height:{}px;background:{colour}"></div></div>"#,
        at(x, y, width, 6),
        ((width - 2) as f64 * ratio) as i32,
        4
    )
}
