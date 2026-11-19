//! The words of units on the ground: what a talkie box says when someone steps on it, and what Graffiti
//! writes (the client's `AM_CHAT` to a `CSkill`, and the words of ZC_SKILL_ENTRY2). They are laid over the
//! picture where the unit stands, with the bars and what was said over heads (`overlays/bars.rs`).
//!
//! The client writes Graffiti's words on the ground itself; here they stand over the unit as what is said
//! does, for as long as the unit is there.

use super::*;

/// How long what a talkie box says stays, as what is said over a head does (milliseconds).
const SAID_FOR: f64 = 5000.0;

/// The words of a unit on the ground: what they are, and until when they show (none: while the unit stands).
pub(super) type Words = HashMap<u32, (String, Option<f64>)>;

/// A unit on the ground says something, or carries words for as long as it stands (`lasting`).
pub fn spot_says(unit: u32, words: &str, lasting: bool) {
    STATE.with_borrow_mut(|state| {
        state.written.insert(unit, (words.to_owned(), (!lasting).then(|| now() + SAID_FOR)));
    });
}

/// The words to show now, each with where its unit stands on the ground; those whose time is over, or whose
/// unit is gone, are forgotten.
pub(super) fn shown(written: &mut Words, scene: &Scene, time: f64) -> Vec<(Vector, String)> {
    written.retain(|unit, (_, until)| scene.spots.contains_key(unit) && until.is_none_or(|until| time <= until));
    written.iter().filter_map(|(unit, (words, _))| Some((on_ground(&scene.gat, *scene.spots.get(unit)?), words.clone()))).collect()
}
