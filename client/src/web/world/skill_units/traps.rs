//! The traps standing on the map (`crate::skill_units::traps`, the client's `CSkill::ShowModel`): each a model
//! on its cell, read the first time its kind is seen, standing still until the trap springs and playing its
//! animation once from then on. The models themselves are the map's (`land.rs`, `Models::raise`).
//!
//! A skill for a trap (Remove Trap, Spring Trap) is pointed at a trap (`pick.rs trap_at`, `input/press.rs`).
//!
//! Not done: with no skill aimed a trap is not pointed at (the client's pointer becomes the sword over a Blast
//! Mine and a Claymore Trap, which it lets be struck, and over every trap in a siege); and the client shows a
//! trap that is not to be seen to the one who laid it and to those of their party, here to the one who laid
//! it alone.

use super::*;
use crate::skill_units::traps;

impl Grounds {
    /// The traps to be seen, each by its ID with its cell: what a skill for a trap can be pointed at.
    pub(in crate::web::world) fn traps(&self) -> impl Iterator<Item = (u32, (f32, f32))> + '_ {
        self.units.iter().filter(|(_, standing)| standing.trap_seen()).map(|(id, standing)| (*id, standing.cell))
    }

    /// Where the trap of an ID stands, while it is to be seen.
    pub(in crate::web::world) fn trap(&self, id: u32) -> Option<(f32, f32)> {
        self.units.get(&id).filter(|standing| standing.trap_seen()).map(|standing| standing.cell)
    }
}

impl Standing {
    /// Whether it is a trap that is to be seen: a unit shown, with a trap's model.
    fn trap_seen(&self) -> bool {
        self.seen && self.model.is_some()
    }

    /// The unit is of this kind from now on, seen or not: the model it shows follows, if the kind has one
    /// (`ShowModel` leaves the model there is for a kind that has none). Gives back whether it is another
    /// model than before, to be put on the map anew.
    pub(super) fn shows_as(&mut self, unit: u32, seen: bool) -> bool {
        (self.unit, self.seen) = (unit, seen);
        let Some(name) = traps::model(unit).filter(|_| seen) else { return false };
        (self.model, self.sprung) = (Some(name), None);
        true
    }
}

/// A unit strikes someone (ZC_NOTIFY_SKILL from it; the client's `AM_SKILL`): a trap's model plays from now on.
pub fn skill_unit_strikes(id: u32) {
    STATE.with_borrow_mut(|state| {
        if let Some(standing) = state.grounds.units.get_mut(&id).filter(|standing| standing.model.is_some()) {
            standing.sprung.get_or_insert_with(now);
        }
    });
}

/// A skill is used on a unit (ZC_USE_SKILL whose target is a unit's ID: Remove Trap, Spring Trap): the unit
/// launches on itself the effect the client has for that skill (`traps::used_with`). Nothing for one that
/// is no unit standing here, or any other skill.
pub fn skill_unit_used(id: u32, skill: u16) {
    let Some(effect) = traps::used_with(skill) else { return };
    STATE.with_borrow_mut(|state| {
        if state.grounds.units.contains_key(&id) {
            launch(state, effect, id, None, now());
        }
    });
}

/// A unit that was not to be seen is seen by everyone (ZC_SKILL_UPDATE; the client's `AM_SKILL_UPDATE`): its
/// model shows, if its kind has one.
pub fn skill_unit_seen(id: u32) {
    STATE.with_borrow_mut(|state| {
        if let Some(standing) = state.grounds.units.get_mut(&id).filter(|standing| !standing.seen) {
            standing.shows_as(standing.unit, true);
        }
    });
}

/// One frame of the traps: each stands on the map as its model, at the frame its animation has come to; a
/// model not read yet is asked for, once.
pub(super) fn step(state: &mut State, time: f64) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &mut state.scene) else { return };
    let mut wanted = Vec::new();
    for (id, standing) in &state.grounds.units {
        let Some(name) = standing.model else { continue };
        let frame = |length: i32| traps::frame(length, standing.sprung.map(|since| ((time - since) / scenery::CLIENT_FRAME_MS) as u64));
        if scene.models.raised(*id) {
            scene.models.raised_frame(gpu, *id, frame);
            continue;
        }
        match state.grounds.models.get(name) {
            Some(Some(read)) => scene.models.raise(gpu, *id, read, on_ground(&scene.gat, standing.cell), traps::light(), frame(read.length())),
            // Being read, or not to be read.
            Some(None) => {}
            None => wanted.push(name),
        }
    }
    for name in wanted {
        if state.grounds.models.insert(name, None).is_none() {
            read(name);
        }
    }
}

/// Reads a trap's model and its pictures, for every trap of its kind from then on.
fn read(name: &'static str) {
    wasm_bindgen_futures::spawn_local(async move {
        let Some((model, pictures)) = land::read_raisable(name).await else { return };
        STATE.with_borrow_mut(|state| {
            if let Some(gpu) = &state.gpu {
                state.grounds.models.insert(name, Some(Rc::new(land::Raisable::new(gpu, model, pictures))));
            }
        });
    });
}
