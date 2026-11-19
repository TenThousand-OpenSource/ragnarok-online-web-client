//! The colours the windows share, as the page takes them. A colour only one window has is written in that
//! window's file; the colours of the chat's lines are `crate::said::colour`.


pub(in crate::web) const BLACK: &str = "#000";
pub(in crate::web) const WHITE: &str = "#fff";
/// What is too much or too little: a count that is full, a gauge under a quarter.
pub(in crate::web) const RED: &str = "#f00";
/// The grey the client writes in what is not to be had (120, 120, 120): a skill not learnt, a tab that is shut.
pub(in crate::web) const GREY: &str = "#787878";
/// The line around a window's white ground (200, 200, 200).
pub(in crate::web) const RIM: &str = "#c8c8c8";
/// The ground of a field that is typed into (232, 232, 232).
pub(in crate::web) const FIELD_GROUND: &str = "#e8e8e8";
