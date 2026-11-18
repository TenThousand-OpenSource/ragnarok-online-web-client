//! Who and what is drawn standing in the world this frame: everyone, what lies on the ground, the pointer's mark.

use super::*;

/// The blows that land now: the one struck flinches, the ring and the number show, the blow sounds.
pub(super) fn landings(state: &mut State, time: f64) {
    let Some(scene) = &mut state.scene else { return };
    let mut landed = Vec::new();
    state.landing.retain(|blow| {
        if blow.0 > time {
            return true;
        }
        landed.push((blow.1, blow.2, blow.3, blow.4.clone(), blow.5));
        false
    });
    for (target, own, damage, heard, critical) in landed {
        let Some(hurt) = (if target == own { Some(&mut scene.player) } else { scene.units.get_mut(&target) }) else { continue };
        if damage > 0 {
            // A blow that hurts stops the one struck where it walks, as it lands: the client puts it in its
            // struck state there and then (`ST_DAMAGE`, in `CGameActor::SendMsg`), and the server stops it at
            // that moment without a word, on the middle of the nearest cell (rAthena's `unit_set_walkdelay`,
            // with `synchronize_damage` on so that its moment is the client's). The player's own character
            // walks on only when asked again; a held button asks again a moment later.
            if target == own && hurt.walk.take().is_some() {
                state.held_asked = time;
            }
            if hurt.motion.is_none() && hurt.dying.is_none() && !hurt.dead {
                hurt.motion = Some((if matches!(hurt.look, Look::Character(_)) { 48 } else { 24 }, time, 280.0));
            }
            // The ring of a blow that lands (the client's `EF_HIT1`).
            state.bursts.extend(prims::burst(0, (target != own).then_some(target), time).map(|(burst, _)| burst));
            // A critical blow lands with the flare of a skill's hit over that (the client's `EF_HIT2`, what its
            // `ProcessHitMessage` gives a `DMG_CRITICAL`), and with that effect's own sound.
            if critical {
                if let Some((burst, sound)) = prims::burst(1, (target != own).then_some(target), time) {
                    state.bursts.push(burst);
                    sound::effect(sound, 1.0);
                }
            }
            if let Some(heard) = heard {
                sound::effect(&heard, 1.0);
            }
        }
        state.hits.push((target, damage, critical && damage > 0));
    }
}

/// Everyone to draw, each with the picture wanted now if it is ready, else the one drawn last; with them what
/// lies on the ground and the pointer's mark in the world. `own` is where the player's own character is now and
/// the picture wanted of it.
pub(super) fn standing(state: &mut State, own: ((f32, f32), SpriteKey), time: f64, yaw: f32) -> Vec<(SpriteKey, Vector, f32)> {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &mut state.scene) else { return Vec::new() };
    let (position, key) = own;
    // Monsters struck down are gone once what is left of them has faded.
    scene.units.retain(|_, unit| unit.dying.is_none_or(|fallen| time - fallen < CORPSE_MS));
    let mut sprites = Vec::with_capacity(scene.units.len() + 1);
    let gat = &scene.gat;
    let feet_cell = position;
    let hide_players = state.options.hide_players;
    let mut own = true;
    for (mover, position, wanted) in std::iter::once((&mut scene.player, position, key)).chain(scene.units.values_mut().map(|unit| {
        let (position, key) = unit.now(time, yaw);
        (unit, position, key)
    })) {
        // The first is the player's own character.
        let is_own = std::mem::replace(&mut own, false);
        // A frame of a monster's sprite may start a sound: the nearer the monster, the louder.
        if let SpriteKey::Creature(path, action, frame) = &wanted {
            if mover.heard.replace((*action, *frame)) != Some((*action, *frame)) {
                if let Some(name) = figure::sound_of(path, *action, *frame) {
                    let away = ((position.0 - feet_cell.0).powi(2) + (position.1 - feet_cell.1).powi(2)).sqrt();
                    sound::effect(&name, f64::from(1.0 - away / 18.0));
                }
            }
        }
        if gpu.has_sprite(&wanted) {
            mover.shown = Some(wanted);
        }
        // Other players are left out when the player has it so (the client's Hide other Players).
        let hidden = hide_players && !is_own && matches!(mover.look, Look::Character(_));
        if let Some(shown) = mover.shown.as_ref().filter(|_| !hidden) {
            // One struck down lies as it fell and fades away (the client's `CCorpse::OnProcess`).
            let left = mover.dying.map_or(1.0, |fallen| 1.0 - ((time - fallen) / CORPSE_MS).clamp(0.0, 1.0) as f32);
            sprites.push((shown.clone(), on_ground(gat, position), left));
        }
        // The emotion it shows: the pictures of that action, each for as long as the sprite says, once through.
        // Until the sprite is read its length is not known: it is given four seconds to come.
        if let Some((sprite, action, began)) = &mover.emotion {
            let passed = time - began;
            let picture = match figure::timing(sprite, *action) {
                Some((pictures, each)) => Some(((passed / each.max(1.0)) as usize, pictures)).filter(|(at, pictures)| at < pictures).map(|(at, _)| at),
                None => Some(0).filter(|_| passed < 4000.0),
            };
            match picture {
                Some(picture) => {
                    let key = SpriteKey::Creature(sprite.clone(), *action, picture);
                    // It follows the one who shows it, 20 above their feet (the client's `CMsgEffect`, for
                    // `MET_EMOTION`); the rest of the way up is in the sprite's own pictures.
                    if gpu.has_sprite(&key) {
                        sprites.push((key, along(on_ground(gat, position), [0.0, 1.0, 0.0], 20.0), 1.0));
                    }
                }
                None => mover.emotion = None,
            }
        }
    }
    // What lies on the ground, once the item's picture is known.
    for (id, Lying { item, at, identified, .. }) in &scene.drops {
        // One not identified lies there as what the client's table shows it as until it is.
        let Some((_, resource)) = data::item_as(*item, *identified) else { continue };
        let key = SpriteKey::Creature(format!("아이템/{}", if resource.is_empty() { "사과" } else { resource.as_str() }), 0, 0);
        if gpu.has_sprite(&key) {
            // A thing that fell out of a monster starts 15 above the ground, is thrown up a little and
            // falls (the client's `CItem::OnProcess`, in its frames of 24 milliseconds).
            let above = scene.falls.get(id).map_or(0.0, |began| {
                let elapsed = ((time - began) / EFFECT_FRAME_MS) as f32;
                (15.0 + 0.6 * elapsed - 0.083 * elapsed * elapsed).max(0.0)
            });
            sprites.push((key, along(on_ground(gat, *at), [0.0, 1.0, 0.0], above), 1.0));
        }
    }
    // Last frame's places of the marks in the world: their pictures turn at two of the client's frames each.
    for at in state.target_at.into_iter().flatten() {
        let pictures = figure::timing("cursors", 3).map_or(1, |(count, _)| count.max(1));
        let key = SpriteKey::Creature("cursors".to_owned(), 3, (time / EFFECT_FRAME_MS / 2.0) as usize % pictures);
        if gpu.has_sprite(&key) {
            sprites.push((key, at, 1.0));
        }
    }
    sprites
}
