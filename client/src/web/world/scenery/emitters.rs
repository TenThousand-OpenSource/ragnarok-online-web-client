//! The particle emitters a map places (`crate::emitters` has the rules, the client's `CEmitterEffect`): made
//! as the map is read, each at its place and set as the map's script sets it, moved on with the surroundings,
//! and drawn through pipelines of their own (`gpu/emitters.rs`).

use super::*;
use crate::emitters::{Emitter, Emitting};

/// An emitter where the map places it, in the world.
struct Standing {
    at: Vector,
    emitting: Emitting,
}

/// The emitters of the map shown, and their pictures as they were read, by name.
#[derive(Default)]
pub(in crate::web::world) struct Emitters {
    standing: Vec<Standing>,
    pictures: HashMap<String, bmp::Image>,
}

impl Emitters {
    /// The emitters a map places at these places, in the order of its `.rsw`: the client makes one at each
    /// (`CWorld::LoadMapEffects`) and the map's script then sets them by their numbers
    /// (`EffectLists::LoadMapEffectScript`); one the script says nothing of stays as it was made.
    ///
    /// Left out, for no map of the classic game has them: an emitter whose particles are turned by its own
    /// angles and not to the camera (`billboard_off`), and one that nothing standing before it hides
    /// (`zenable` of none).
    pub(super) async fn read(map: &str, places: Vec<Vector>) -> Self {
        if places.is_empty() {
            return Self::default();
        }
        let Some(set) = data::emitters(map).await else { return Self::default() };
        let (mut set, mut emitters) = (set.into_iter(), Self::default());
        for at in places {
            let emitter = set.next().unwrap_or_default();
            if emitter.flat || !emitter.depth {
                continue;
            }
            if !emitters.pictures.contains_key(&emitter.picture) {
                match picture(&emitter.picture).await {
                    Ok(image) => drop(emitters.pictures.insert(emitter.picture.clone(), image)),
                    Err(error) => web_sys::console::warn_1(&format!("emitter picture {}: {error}", emitter.picture).into()),
                }
            }
            emitters.standing.push(Standing { at, emitting: Emitting::new(emitter, &mut lot) });
        }
        emitters
    }

    /// So many frames of the client's go by, the player's character standing there (`CEffect::Update`: an
    /// emitter measures how far the character is in all three directions).
    pub(super) fn step(&mut self, player: Vector, frames: u32) {
        for Standing { at, emitting } in &mut self.standing {
            let away = (0..3).map(|axis| (player[axis] - at[axis]).powi(2)).sum::<f32>().sqrt();
            for _ in 0..frames {
                emitting.frame(away, &mut lot);
            }
        }
    }
}

/// What the GPU is given of the emitters each frame: the corners of their particles' quads (a place, where of
/// the picture, a colour: six corners to a quad), and which corners are drawn with which picture and blends.
#[derive(Default)]
pub(in crate::web::world) struct Corners {
    pub(in crate::web::world) numbers: Vec<f32>,
    pub(in crate::web::world) batches: Vec<(Emitter, std::ops::Range<u32>)>,
}

/// The emitters' particles as they are now, put on the GPU for this frame's drawing; none with the effects
/// off (the client's `/effect`: `CEffect::UpdateVisibility`).
///
/// A particle is a quad of one to each side at one before its place, made as large as the particle is, tipped
/// by the camera's pitch and turned by its yaw (`CEmitterEffect::UpdateParticle`, `CParticle::Render`): its x
/// lies along the screen's right, its y down the screen and its z away from the eye. The picture's top is on
/// the edge of the higher y (`CParticle::InitParticleQuad`, `SetQuad`), which is the lower one on the screen.
pub(in crate::web::world) fn drawn(state: &mut State, view: &View) {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &state.scene) else { return };
    let emitters = &scene.scenery.emitters;
    let mut corners = Corners::default();
    for Standing { at, emitting } in emitters.standing.iter().filter(|_| state.options.effects) {
        let (first, [red, green, blue]) = ((corners.numbers.len() / 9) as u32, emitting.colour());
        for particle in emitting.shown() {
            // (The client's y points down.)
            let place = along([at[0] + particle.at[0], at[1] - particle.at[1], at[2] + particle.at[2]], view.forward, 1.0);
            let colour = [red, green, blue, particle.alpha.min(255) as f32 / 255.0];
            for (x, y) in [(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (-1.0, -1.0), (1.0, 1.0), (1.0, -1.0)] {
                corners.numbers.extend(along(along(place, view.right, x * particle.half[0]), view.up, -y * particle.half[1]));
                corners.numbers.extend([(x + 1.0) / 2.0, (1.0 - y) / 2.0]);
                corners.numbers.extend(colour);
            }
        }
        let last = (corners.numbers.len() / 9) as u32;
        if last > first {
            corners.batches.push((emitting.emitter.clone(), first..last));
        }
    }
    gpu.emitters_loaded(&emitters.pictures, corners);
}
