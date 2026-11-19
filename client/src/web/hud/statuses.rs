//! The states the character is in (buffs and debuffs), listed down the right side of the screen as the client
//! lists them, each with its time going and what is said of it when it is pointed at.

use super::*;
use crate::statuses::{listed, place, time_left};

/// Where the client's message table has the words the time left is written with: hours, minutes and seconds
/// (`MSI_TIME`, `MSI_MINUTE`, `MSI_SECOND`).
const TIME_WORDS: [usize; 3] = [1697, 1807, 1808];

impl App {
    /// The list of states (`CRagEffect::StatusIconColumn`): the picture of each, 32 by 32, down from 169 at 16
    /// from the right edge, a new column to the left when one is full; over each the shade of its time
    /// (`hud/cooldowns.rs`), and beside it, while it is pointed at, what the client says of it.
    pub(in crate::web) fn statuses_column(&self) -> String {
        let taekwon = crate::jobs::taekwon(self.own_figure().job);
        let height = (crate::web::page_size().1 / crate::web::stage_scale()) as i32;
        let now = crate::web::app::now();
        let active = self.statuses.on.iter().map(|active| active.state);
        listed(active, |state, kind| data::status_icon(state, kind, taekwon).is_some())
            .into_iter()
            .enumerate()
            .filter_map(|(index, listed)| {
                let file = data::status_icon(listed.pictured, listed.kind, taekwon)?;
                // The time is that of the state itself, whichever number it is listed by.
                let active = self.statuses.on.iter().find(|active| active.state == listed.pictured)?;
                let left = active.end.map(|end| end.saturating_sub(now));
                let shade = left.map(|left| cooldowns::passing(active.total, left)).unwrap_or_default();
                let (right, top) = place(index, height);
                Some([
                    format!(
                        r#"<div class="a sti" style="right:{right}px;top:{top}px;width:32px;height:32px;background:url('{}') 0 "#,
                        crate::web::image_url(&format!("data/texture/{file}"))
                    ),
                    format!(r#"0 / 100% 100% no-repeat">{shade}{}</div>"#, Self::status_told(listed.pictured, left))
                ].concat())
            })
            .collect()
    }

    /// What the client says of a state pointed at (`CGameMode::ProcessStatusIconTooltip`): a see-through black
    /// box with a grey outline (`UITransBalloonText`: `0x66000000`, the outline `0xC0C0C0`), its right edge at
    /// the picture's left and its top 2 under the picture's; the lines 6 in, the first 4 down and each 4 under
    /// the last, in the small text (11) with a dark edge, each in its own colour. The line that has the time
    /// has it written in, hours, minutes and seconds; nothing for a state without a time.
    fn status_told(state: u32, left: Option<u32>) -> String {
        let Some(told) = data::status_text(state) else { return String::new() };
        let words = TIME_WORDS.map(|line| data::message(line).unwrap_or_default());
        let time = left.map(|left| time_left(left / 1000, [&words[0], &words[1], &words[2]])).unwrap_or_default();
        let lines: String = told
            .lines
            .iter()
            .enumerate()
            .map(|(index, (line, colour))| {
                let line = if index + 1 == told.time { line.replacen("%s", &time, 1) } else { line.clone() };
                format!(
                    r#"<div style="color:{};min-height:11px;margin-bottom:4px;white-space:nowrap">{}</div>"#,
                    crate::said::colour::css(*colour),
                    escape(&line)
                )
            })
            .collect();
        [
            format!(r#"<div class="a tip" style="right:32px;top:2px;padding:4px 6px 0;background:rgba(0,0,0,.4);border:1px "#),
            format!(r#"solid #c0c0c0;border-radius:3px;{};text-shadow:1px 1px 0 #000">{lines}</div>"#, SMALL.style())
        ].concat()
    }
}
