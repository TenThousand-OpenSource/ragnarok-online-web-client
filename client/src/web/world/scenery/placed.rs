//! One frame of what surrounds the player: the makers of the effects a map places launch them
//! (`CEffectMaker::OnProcess`), its sounds are played (`CSoundMaker::OnProcess`), the weather is kept going,
//! and the night darkens or lifts.

use super::*;

/// How many of the makers' frames are lived through at once at most: after a pause (the page out of sight) the
/// rest is let go.
const FRAMES_AT_ONCE: u32 = 8;

/// Moves the surroundings on to `time`.
pub(in crate::web::world) fn step(state: &mut State, time: f64) {
    let Some(scene) = &mut state.scene else { return };
    let since = time - scene.scenery.stepped.unwrap_or(time);
    // The makers count frames as the effects they launch count ticks.
    let frames = ((since / CAST_FRAME_MS) as u32).min(FRAMES_AT_ONCE);
    // (What is left of a frame is kept for the next; after a pause it is let go.)
    scene.scenery.stepped = Some(match scene.scenery.stepped {
        Some(stepped) if frames < FRAMES_AT_ONCE => stepped + f64::from(frames) * CAST_FRAME_MS,
        _ => time,
    });
    state.scenery.dark.frames((f64::from(frames) * CAST_FRAME_MS / CLIENT_FRAME_MS).round() as u32);
    scene.scenery.light = state.scenery.dark.light;
    scene.scenery.fogged = state.scenery.fog && scene.scenery.fog.is_some();
    let player = on_ground(&scene.gat, scene.player.at);
    sounds(scene, player, time);
    // Effects are not shown when the player has them off (the client's `/effect`).
    if !state.options.effects {
        return;
    }
    let mut launched = Vec::new();
    for placed in &mut scene.scenery.effects {
        let away = f32::hypot(placed.at[0] - player[0], placed.at[2] - player[2]);
        for _ in 0..frames {
            if placed.maker.frame(away) {
                launched.push((placed.effect, placed.spot, placed.figures));
            }
        }
    }
    let weather = scene.scenery.weather.clone();
    for (effect, spot, figures) in launched {
        launch(state, effect, Some(spot), figures, time);
    }
    for effect in weather {
        if !launch::running(state, effect, None) {
            launch(state, effect, None, [0.0; 4], time);
        }
    }
}

/// One of the client's effects begins at a place of the map, told the map's figures (`AM_SETPARAM`), or on
/// the player's own character: from its files if it is played from files, else drawn by code if it is ported.
fn launch(state: &mut State, effect: u32, on: Option<u32>, figures: [f32; 4], time: f64) {
    if launch::files(state, effect, on, None, time) {
        return;
    }
    if let Some(burst) = prims::begin(effect, on, None, time, &mut state.bursts, &mut state.sounds_due) {
        burst.figured(figures);
    }
}

/// The sounds the map places, each played anew once its cycle is over while the player's character is in
/// its earshot, as loud as `crate::scenery::makers::heard` says.
fn sounds(scene: &mut Scene, player: Vector, time: f64) {
    for (sound, at, last) in &mut scene.scenery.sounds {
        let Some(loud) = crate::scenery::makers::heard(player[0] - at[0], player[2] - at[2], sound.size, sound.range, sound.volume) else { continue };
        if time > *last + f64::from(sound.cycle) * 1000.0 {
            *last = time;
            sound::effect(&sound.file, loud.min(1.0));
        }
    }
}
