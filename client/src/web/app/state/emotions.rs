//! What the app keeps of the window of emotions and of the Shortcut List.

use super::*;

/// The window of emotions (`UICashEmotionListWnd`) and the Shortcut List (`UIEmotionWnd`).
#[derive(Default)]
pub(in crate::web) struct Emotions {
    /// Each is open.
    pub open: bool,
    pub shortcuts: bool,
    /// The ten lines of the Shortcut List, said with Alt and 1 to 9, then 0 (the client's `EmotionHotkeys`):
    /// kept in the browser. An empty one is the line the client starts with.
    pub lines: [String; 10],
    /// The picture of each emotion for the grid, once read: a data URL and its size.
    pub pictures: HashMap<u16, (String, (usize, usize))>,
    /// The pictures were asked for.
    pub(in crate::web::app) asked: bool,
    /// When the last emotion was asked for (milliseconds): the client lets one be shown every 2500.
    pub(in crate::web::app) last: f64,
}
