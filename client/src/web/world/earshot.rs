//! The sounds of the world, each heard from where it starts: as loud as its distance from the player's own
//! character leaves it (`sound::at`, the client's `PlayWave(name, dx, dy, dz, 250, 40, …)`).

use super::*;

/// A sound of the world that starts on someone at a moment yet to come (none named: the player's own
/// character).
pub(super) type Due = (f64, String, Option<u32>);

/// Plays a sound that starts where someone is (none named: the player's own character). One who is not in
/// sight has no place: the sound is then played whole, as the client plays one it gives no place.
pub(super) fn heard(scene: &Scene, name: &str, on: Option<u32>) {
    match scene.place_of(on) {
        Some(at) => heard_at(scene, name, at),
        None => sound::effect(name, 1.0),
    }
}

/// Plays a sound that starts at a place of the map, in cells.
pub(super) fn heard_at(scene: &Scene, name: &str, at: (f32, f32)) {
    sound::at(name, (at.0 - scene.player.at.0) * CELL, (at.1 - scene.player.at.1) * CELL);
}

/// The sounds whose moment has come are played, each from where the one it starts on is now.
pub(super) fn due(state: &mut State, time: f64) {
    let now: Vec<Due> = state.heard.extract_if(.., |due| due.0 <= time).collect();
    for (_, name, on) in now {
        match &state.scene {
            Some(scene) => heard(scene, &name, on),
            None => sound::effect(&name, 1.0),
        }
    }
}
