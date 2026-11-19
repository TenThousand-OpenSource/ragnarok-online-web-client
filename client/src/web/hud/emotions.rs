//! The window of emotions, and the Shortcut List: ten lines, each said with Alt and a digit.

use super::*;
use crate::press::emotions::Press;

/// The lines of the client's message table the ten shortcuts start as (`EmotionHotkeys`: `/!`, `/?`, `/ho`…).
pub(in crate::web) const SHORTCUTS_START: [usize; 10] = [544, 545, 546, 547, 549, 550, 551, 552, 553, 554];

impl App {
    /// A line of the Shortcut List as it is now: what the player typed, or the client's own.
    pub(in crate::web) fn shortcut_line(&self, place: usize) -> String {
        match self.emotions.lines.get(place) {
            Some(line) if !line.is_empty() => line.clone(),
            _ => SHORTCUTS_START.get(place).and_then(|line| data::message(*line)).unwrap_or_default(),
        }
    }

    /// The window of emotions (the client's `UICashEmotionListWnd`, 298 by 380): the title (line 542 of the
    /// message table); at the left edge, from 46 down, the tab of the basic pack; at (252, 22) the button that
    /// opens the Shortcut List; and at (28, 47) the list, 271 by 303: the emotions five to a row, each in a
    /// cell 40 by 40, the first at (15, 4) and the next 47 on, scrolled when they are more than fit. A press
    /// on one writes its name where the player types; a double click shows it. The tabs of emotions bought
    /// with cash, the search and Buy are not here. The client orders the cells by its `InsertOrderList`;
    /// here they stand in the order of their numbers.
    pub(in crate::web) fn emotion_window(&self) -> String {
        let mut window = title_bar(298, &Self::words(542, "Emoji List"));
        window.push_str(&cross(298, Press::Window));
        window.push_str(&panel_ground(0, 17, 298, 363));
        window.push_str(&art("cashemotion/tap_basic_press.bmp", 0, 46, 30, 66));
        window.push_str(&fill(29, 46, 268, 1, "#cecece"));
        window.push_str(&pictured_by(Convention::NormalOverPress, 252, 22, (37, 18), "cashemotion/btn_shortui", Press::Shortcuts, r#" data-tip="Alt+M""#));
        let cells: String = data::emotions()
            .into_iter()
            .enumerate()
            .map(|(place, emotion)| {
                let (x, y) = (15 + (place % 5) as i32 * 47, 4 + (place / 5) as i32 * 47);
                let inside = self.emotions.pictures.get(&emotion).map(|(picture, _)| format!(r#"<img src="{picture}" alt="" draggable="false" style="max-width:40px;max-height:40px">"#)).unwrap_or_default();
                format!(
                    r#"<div class="a" style="{};display:flex;align-items:center;justify-content:center;cursor:pointer" {} {} data-tip="{}">{inside}</div>"#,
                    at(x, y, 40, 40),
                    pressed(Press::Pick(emotion)),
                    used(Use::Emotion(emotion)),
                    escape(&data::emotion_name(emotion).unwrap_or_default())
                )
            })
            .collect();
        let rows = data::emotions().len().div_ceil(5) as i32;
        window.push_str(&format!(r#"<div class="a scroll" style="{};overflow-y:auto"><div style="position:relative;height:{}px">{cells}</div></div>"#, at(28, 47, 269, 303), 4 + rows * 47));
        window
    }

    /// The name of the field of one line of the Shortcut List.
    pub(in crate::web) fn shortcut_field(place: usize) -> String {
        format!("{}-{place}", windows::SHORTCUTS.field())
    }

    /// The Shortcut List (the client's `UIEmotionWnd`, 375 by 260): the title (line 574); ten rows, the first
    /// 23 down and the next 21 on: at 20 the keys that say the line (as the Shortcut Settings have them:
    /// "Alt + 1" to "Alt + 0"), and at 83, 3 higher, the line itself, 270 wide, of 50 letters at most, on a
    /// grey of 232; at the bottom right the button that shows the window of emotions. What is typed is kept
    /// when the window closes, and at Enter.
    pub(in crate::web) fn shortcut_window(&self) -> String {
        let mut window = title_bar(375, &Self::words(574, "Shortcut List"));
        window.push_str(&cross(375, Press::Shortcuts));
        window.push_str(&panel(375, 260));
        let rows: String = (0..10)
            .map(|place| {
                let y = 23 + 21 * place as i32;
                let keys = crate::hotkeys::written(self.bound(crate::hotkeys::table::MACROS, place), self.keys.mac, data::key_name);
                format!(
                    r#"{}<input class="a in" id="{}" maxlength="50" autocomplete="off" value="{}" style="{};background:#e8e8e8">"#,
                    text(20, y, BLACK, &keys),
                    Self::shortcut_field(place),
                    escape(&self.shortcut_line(place)),
                    at(83, y - 3, 270, 16)
                )
            })
            .collect();
        window.push_str(&format!(r#"<form {}>{rows}<button type="submit" hidden></button></form>"#, sent(Form::Shortcuts)));
        window.push_str(&picture_press(375 - 50, 260 - 25, "btn_view", Press::View));
        window
    }
}
