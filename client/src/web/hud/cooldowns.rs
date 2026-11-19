//! The shade the client lays over a picture while a time runs (`Source/UI/Cooldowns`): over a skill of the
//! hotkey bar until it may be used again, and over a state at the side of the screen as its time goes.
//!
//! The client draws it as a fan of triangles around the picture's middle, cut to the picture's square, one
//! corner of which goes round clockwise from the top (`CooldownDrawing::Draw`, `CooldownState::Update`). Here it
//! is a cone of colour the page turns by itself (the classes `cd` and `cd r` of `index.html`), started as far
//! through as the time is.

use super::*;

/// The shade over a skill that waits (`CooldownState`, not in reverse: `0xAA000000`): black, two thirds
/// opaque, over the part of the turn still to wait, 24 by 24 on the slot (`PaintCooldowns`: 12 either way from
/// the middle of the picture).
const WAITING: &str = "rgba(0,0,0,.667)";
/// The shades over a state (`CooldownState` in reverse): white (`0xAAFFFFFF`) over the time gone, and orange
/// (`0xAAFF6633`) through the last minute.
const GONE: &str = "rgba(255,255,255,.667)";
const LAST: &str = "rgba(255,102,51,.667)";

impl App {
    /// The shades over a skill on a slot of the bar: one for each wait that holds it back and is still running.
    pub(super) fn waiting(&self, skill: u16) -> String {
        let now = crate::web::app::now();
        self.cooldowns
            .waits
            .over(skill, now)
            .map(|wait| format!(
                r#"<i class="a cd" style="{};--c:{WAITING};animation:cd {}ms linear -{}ms both"></i>"#,
                at(0, 0, 24, 24),
                wait.duration,
                wait.progress(now)
            ))
            .collect()
    }
}

/// The shades over a state's picture, 32 by 32 (`CEffectPrim::Render2DPrimitive`: 16 either way), for a time
/// so long in all with so much left: the white one until a minute is left, then the orange one.
pub(super) fn passing(duration: u32, remaining: u32) -> String {
    let Some(((whole, gone), (last, through))) = crate::cooldowns::turns(duration, remaining) else { return String::new() };
    let (place, wait) = (at(0, 0, 32, 32), -through);
    [
        format!(r#"<i class="a cd r" style="{place};--c:{GONE};animation:cd {whole}ms linear -{gone}ms both,cdgone 1ms linear {wait}ms both"></i>"#),
        format!(r#"<i class="a cd r" style="{place};--c:{LAST};animation:cd {last}ms linear {wait}ms both"></i>"#)
    ].concat()
}
