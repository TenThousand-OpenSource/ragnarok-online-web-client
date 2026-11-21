//! What the server told of those in sight beside their names, for the name under whoever the pointer is on
//! (`crate::names`, written by `overlays.rs`): the level and whether one is a boss, the title and group of a
//! monster or an NPC, and the player's own level, by which a monster's name is coloured.

use super::*;
use crate::names::{Card, Who};

/// The player's own base level, as the app tells it.
pub fn own_level(level: u16) {
    STATE.with_borrow_mut(|state| state.own_level = level);
}

/// The title and the group the server names for a monster or an NPC with its name (ZC_ACK_REQNAMEALL_NPC).
pub fn unit_titled(id: u32, group: u32, title: &str) {
    STATE.with_borrow_mut(|state| {
        let card = state.cards.entry(id).or_default();
        (card.group, card.title) = (group, title.to_owned());
    });
}

/// The level of someone who came into sight, and whether it is a boss (the packet that shows a unit).
pub(super) fn ranked(state: &mut State, id: u32, level: u16, boss: u8) {
    let card = state.cards.entry(id).or_default();
    (card.level, card.boss) = (level, boss);
}

/// What is written under someone in sight, in which colour, and the picture that stands beside it, among the
/// client's pictures of its windows (the client's `UpdateNameBalloon`). `label` is the name as it is written
/// without what is told here (a player's with its party and guild).
pub(super) fn written(state: &State, id: u32, unit: &Mover, label: String) -> (String, &'static str, Option<String>) {
    let who = match &unit.look {
        _ if unit.lively => Who::Monster,
        look if look.creature() && !unit.companion => Who::Npc,
        _ => Who::Other,
    };
    let none = Card::default();
    let card = state.cards.get(&id).unwrap_or(&none);
    let colour = crate::names::colour(who, unit.job, card.level, state.own_level);
    // A title and a group are a monster's or an NPC's: a player's name is written by its own rules.
    let creature = unit.look.creature();
    // The name of a pet or a homunculus that is no monster by its job, which is a homunculus's, has stars for
    // a word that is not let be written (`UpdateNameBalloon`, `crate::manner`). (A mercenary's, which the
    // client does not go through, is not told apart here: it is of the server's own words.)
    let label = if unit.companion && !crate::jobs::monster(unit.job) { data::starred(&label) } else { label };
    let label = if creature { crate::names::written(&label, &card.title) } else { label };
    (label, colour, Some(card).filter(|_| creature).and_then(|card| crate::names::mark(card, unit.lively)))
}

/// Puts the picture beside the name the page shows now: 24 pixels each way at the label's left, the label so
/// much wider (`UINameBalloonText::SetName` widens the balloon by the picture's width).
pub(super) fn marked(picture: &str) {
    let Some(label) = crate::web::by_id("unit-name").and_then(|label| label.dyn_into::<web_sys::HtmlElement>().ok()) else { return };
    let style = label.style();
    let _ = style.set_property("background", &format!("url('{}') 0 50% / 24px 24px no-repeat", crate::web::parts::picture(picture)));
    let _ = style.set_property("padding-left", "25px");
    let _ = style.set_property("min-height", "24px");
}
