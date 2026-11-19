//! The list of the packets of the map server that this client does not read yet: a tool of this project's own
//! for whoever builds it, opened with the chat command `/packets`. The original client has no such window;
//! its size and what it shows were decided here.

use super::*;
use crate::net::unread;
use crate::press::unread::Press;

/// The window's size, and how high a row of the list is.
pub(super) const SIZE: (i32, i32) = (330, 240);
const ROW: i32 = 14;

impl App {
    /// The window: how many packets were not read in its title, then a row to each number, the one that came
    /// last first: the number in hexadecimal, what rAthena calls it, how many came and how long the last was.
    pub(in crate::web) fn unread_window(&self) -> String {
        let tally = &self.unread.tally;
        let mut window = title_bar(SIZE.0, &format!("{}: {}", tr("Packets not read"), tally.total()));
        window.push_str(&press_button(SIZE.0 - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::Close));
        window.push_str(&panel(SIZE.0, SIZE.1));
        let rows: String = tally
            .rows()
            .iter()
            .enumerate()
            .map(|(row, packet)| {
                let y = row as i32 * ROW;
                format!(
                    "{}{}{}{}",
                    text(4, y, BLACK, &format!("{:#06x}", packet.id)),
                    text(50, y, BLACK, unread::name(packet.id).unwrap_or("?")),
                    text_ending(SIZE.0 - 60, SIZE.0 - 12, y, BLACK, &format!("×{}", packet.count)),
                    text_ending(SIZE.0 - 16, SIZE.0 - 12, y, "#666", &packet.length.to_string()),
                )
            })
            .collect();
        window.push_str(&format!(
            r#"<div class="a scroll" style="{}"><div style="position:relative;height:{}px">{rows}</div></div>"#,
            at(6, 20, SIZE.0 - 12, SIZE.1 - 44),
            tally.rows().len() as i32 * ROW
        ));
        window.push_str(&format!(r#"<div class="a t" style="left:8px;top:{}px;color:#00f;cursor:pointer" {}>{}</div>"#, SIZE.1 - 18, pressed(Press::Clear), escape(tr("Clear"))));
        window
    }
}
