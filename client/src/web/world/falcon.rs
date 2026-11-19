//! The falcons hunters keep: drawn by each owner as the client's effect that follows one (`CMsgEffect` of the
//! kind `MET_BIRD`, made by `AM_MAKE_BIRD`; the rules are `crate::falcon`).

use super::*;
use crate::falcon::Falcon;

/// The most ticks a frame moves the falcons on: a page that was out of sight does not make them all up.
const MOST_TICKS: u32 = 30;

/// The falcons in the world.
#[derive(Default)]
pub(super) struct Falcons {
    /// The falcon of each owner in sight who has one (none is the player's own character), of the visit to
    /// a map they were made on (`State::entered`).
    birds: HashMap<Option<u32>, Falcon>,
    visit: u32,
    /// Until when the falcons have been moved on, a tick of the client's at a time.
    ticked: f64,
}

/// What a skill sends a falcon at.
pub enum Prey {
    /// Someone (none named: the player's own character), or a unit of a skill on the ground: a trap.
    Someone(Option<u32>),
    /// A cell of the map.
    Cell((u16, u16)),
}

/// A skill sends the falcon of its user (none named: the player's own character) at something
/// (`AM_REQ_ATTACK` to the user's `statusEffects[0]`): nothing happens for one who has no falcon in sight, or
/// when what it is sent at is not in sight.
pub fn falcon_sent(owner: Option<u32>, prey: Prey) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &state.scene else { return };
        let Some(bird) = state.falcons.birds.get_mut(&owner).filter(|_| state.falcons.visit == state.entered) else { return };
        let place = match prey {
            Prey::Someone(who) => scene.place_of(who),
            Prey::Cell(cell) => Some((f32::from(cell.0), f32::from(cell.1))),
        };
        if let Some(place) = place {
            bird.send(place, on_ground(&scene.gat, place)[1]);
        }
    });
}

/// The falcons of this frame, each a picture standing in the world as the sprites do (`frame.rs` adds them to
/// those). Everyone drawn as a character whose state has the falcon's bit has one, the player's own character
/// and the others alike; it is made a cell behind its owner, moved on by the client's ticks, and shows the
/// picture it has come to. The client takes the falcon away while its owner hides and makes it anew as the
/// owner comes out, and so does this; one whose owner went out of sight, or lost the bit, goes with it.
pub(super) fn drawn(state: &mut State, time: f64, pictures: &mut Vec<Stood>) {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &state.scene) else { return };
    let falcons = &mut state.falcons;
    if falcons.visit != state.entered {
        (falcons.visit, falcons.ticked) = (state.entered, time);
        falcons.birds.clear();
    }
    let ticks = ((time - falcons.ticked) / crate::falcon::TICK_MS) as u32;
    falcons.ticked = if ticks > MOST_TICKS { time } else { falcons.ticked + f64::from(ticks) * crate::falcon::TICK_MS };
    let keeps = |mover: &Mover| matches!(mover.look, Look::Character(_)) && mover.state & crate::falcon::BIT != 0 && mover.state & crate::falcon::HIDDEN == 0;
    let owners: Vec<(Option<u32>, &Mover)> = std::iter::once((None, &scene.player)).chain(scene.units.iter().map(|(id, mover)| (Some(*id), mover))).filter(|(_, mover)| keeps(mover)).collect();
    falcons.birds.retain(|owner, _| owners.iter().any(|(keeper, _)| keeper == owner));
    let ground = |place: (f32, f32)| on_ground(&scene.gat, place)[1];
    let yaw = scene.camera.yaw;
    for (owner, mover) in owners {
        let sprite = crate::falcon::sprite(mover.job);
        let bird = falcons.birds.entry(owner).or_insert_with(|| Falcon::behind(mover.at, mover.facing, ground(mover.at)));
        // Until the sprite is read its actions are taken to have the eight pictures the client's files have.
        let count = figure::timing(sprite, 0).map_or(8, |(count, _)| count);
        for _ in 0..ticks.min(MOST_TICKS) {
            bird.tick(mover.at, ground, count);
        }
        let key = SpriteKey::Creature(sprite.to_owned(), bird.action(mover.at, yaw), bird.picture());
        if gpu.has_sprite(&key) {
            let under = on_ground(&scene.gat, bird.at);
            pictures.push((key, [under[0], bird.up, under[2]], 1.0, crate::body::Quad::PLAIN));
        }
    }
}
