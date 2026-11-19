//! The server's announcements across the top of the screen (the client's `UIBroadcastBalloon`): each in a
//! box of its own in the middle, one under the other.

use super::*;

impl App {
    /// The announcements shown now (`UIBroadcastBalloon::Paint` and `DrawTransBox`). A box is as wide as its
    /// longest line and 12 more, with the lines 6 in, 4 between them and 4 above and below; it is filled
    /// with black a quarter there (`0x40000000`) and has a rounded rim of `0x5C5C5C`; ten pixels lie between
    /// two boxes. The words are in the announcement's colour with a dark edge (`DrawShadowedText`; the
    /// edge's colour is not set by the client's code that makes the lines, and is taken for black here).
    pub(in crate::web) fn broadcast_window(&self) -> String {
        self.broadcast
            .shown
            .iter()
            .map(|announced| {
                let font = Font(0, announced.letters.min(64) as u8, false).style();
                let lines: String = announced.lines
                    .iter()
                    .map(|line| format!(r#"<div class="t" style="margin-top:4px;{font}">{}</div>"#, escape(line)))
                    .collect();
                [
                    format!(r#"<div style="display:table;margin:0 auto 10px;padding:0 6px 4px;box-sizing:border-box;"#),
                    format!(r#"background:rgba(0,0,0,0.25);border:1px solid #5c5c5c;border-radius:4px;color:{};text-shadow:1px 0 0 "#, announced.colour),
                    format!(r#"#000,-1px 0 0 #000,0 1px 0 #000,0 -1px 0 #000">{lines}</div>"#)
                ].concat()
            })
            .collect()
    }
}
