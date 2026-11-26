//! Who and what is drawn standing in the world this frame: everyone, what lies on the ground, the pointer's mark.

use super::*;

/// The hits that landed now, as `combat/landing.rs` tells them: the one struck stops and flinches, and a
/// skill's hit may have its sprite drawn in one colour for a moment.
pub(super) fn landings(state: &mut State, time: f64, landed: &[combat::Struck]) {
    let Some(scene) = &mut state.scene else { return };
    for &combat::Struck { target, own, flinches, flash } in landed {
        let Some(hurt) = scene.one_mut(target, own) else { continue };
        // (`render.started` and `isSingleColor` with the colour of `AM_ATTACKED_SKILL`: `Ail::flash`.)
        if let Some(colour) = flash {
            hurt.ail.flash = Some((time, colour));
        }
        if !flinches {
            continue;
        }
        // A blow that hurts stops the one struck where it walks, as it lands: the client puts it in its
        // struck state there and then (`ST_DAMAGE`, in `CGameActor::SendMsg`), and the server stops it at
        // that moment without a word, on the middle of the nearest cell (rAthena's `unit_set_walkdelay`,
        // with `synchronize_damage` on so that its moment is the client's). The player's own character
        // walks on only when asked again; a held button asks again a moment later.
        if target == own && hurt.walk.take().is_some() {
            state.held_asked = time;
        }
        // (One that runs through the frames of a state does not flinch: `ST_DAMAGE` is refused while a
        // sequence is under way. Nor does one whose motion is held: a blade caught, stone, ice.)
        if hurt.motion.is_none() && hurt.dying.is_none() && !hurt.dead && hurt.pose.sequence.is_none() && !hurt.pose.frozen {
            hurt.motion = Some((if matches!(hurt.look, Look::Character(_)) { 48 } else { 24 }, time, 280.0));
        }
    }
}

/// The pictures of one standing at `feet` and showing `shown`, as many times and in such ways as its states
/// and the effects on it have left its body (`crate::body`), `left` of each to see. A weapon that an effect
/// has drawn by itself (`Body::weapon`) is the picture of the weapon alone over the body, which is then the
/// character without its weapon if the effect says so and that picture is ready.
fn bodied(gpu: &mut Gpu, mover: &Mover, shown: &SpriteKey, feet: Vector, left: f32, time: f64, sprites: &mut Vec<Stood>) {
    // (`character` tells the client's two ways of drawing apart.)
    let (character, tall) = (matches!(mover.look, Look::Character(_)), gpu.sprite_textures.get(shown).map_or(0.0, |texture| texture.size.1));
    let armed = match shown {
        SpriteKey::Player(figure) if figure.weapon != 0 => mover.ail.body.weapon().map(|(quad, bare)| (figure, quad, bare)),
        _ => None,
    };
    // (Until the weapon's own picture is there, the body stays as it is.)
    let armed = armed.map(|(figure, quad, bare)| (SpriteKey::Weapon(figure.clone()), figure, quad, bare)).filter(|(weapon, ..)| gpu.has_sprite(weapon));
    let bare = armed
        .as_ref()
        .filter(|(.., bare)| *bare)
        .map(|(_, figure, ..)| SpriteKey::Player(Figure { weapon: 0, ..(*figure).clone() }))
        .filter(|bare| gpu.has_sprite(bare));
    // A baby job is drawn smaller, whatever else is done to it (`GetSkillFactor`, over `ScaleByPulse`).
    let size = if character { crate::pose::size(mover.job) } else { 1.0 };
    // One a skill's hit has flashed is drawn in that colour and no other for 100 milliseconds (the client's
    // `isSingleColor`: the colour takes the place of the sprite's own in `CRenderObject`'s quads, and its
    // `OnProcess` puts it back once `timeGetTime() > render.started + 100`).
    let flash = mover.ail.flash.filter(|(since, _)| time <= since + 100.0).map(|(_, [red, green, blue])| [red, green, blue, 255]);
    let sized = |quad: crate::body::Quad| crate::body::Quad { scale: quad.scale * size, colour: flash.filter(|_| !quad.adds).unwrap_or(quad.colour), ..quad };
    for quad in mover.ail.body.quads(character, tall, &mut lot) {
        sprites.push((bare.clone().unwrap_or_else(|| shown.clone()), feet, left, sized(quad)));
    }
    sprites.extend(armed.map(|(weapon, _, quad, _)| (weapon, feet, left, sized(quad))));
}

/// The shadow's sprite (`shadow.spr`, its one picture).
fn shadow() -> SpriteKey {
    SpriteKey::Creature("shadow".to_owned(), 0, 0)
}

/// The shadow under someone or something at `feet` (`CRenderObject::DrawShadow`): the one picture of
/// `shadow.spr` on the ground, `scale` times its size (the job's `shadowScale` and what shrinks or swells the
/// body; four tenths under a thing on the ground).
pub(in crate::web::world) fn shadowed(gpu: &mut Gpu, feet: Vector, scale: f32, sprites: &mut Vec<Stood>) {
    let key = shadow();
    if scale > 0.0 && gpu.has_sprite(&key) {
        sprites.push((key, feet, 1.0, crate::body::Quad { scale, under: true, ..crate::body::Quad::PLAIN }));
    }
}

/// How much of one is left to see: one struck down fades as it lies (`CCorpse::OnProcess`), one out of sight
/// fades over half a second (its alpha is half the milliseconds left: `CGameActor::ProcessState`), and
/// another player is see-through, 70 of 255, when the player has the others hidden (`translucentOther`).
pub(in crate::web::world) fn left_of(mover: &Mover, faint: bool, time: f64) -> f32 {
    let corpse = mover.dying.map_or(1.0, |fallen| 1.0 - ((time - fallen) / CORPSE_MS).clamp(0.0, 1.0) as f32);
    let going = mover.pose.vanishes.map_or(1.0, |gone| ((gone - time) / 2.0 / 255.0).clamp(0.0, 1.0) as f32);
    corpse * going * if faint { 70.0 / 255.0 } else { 1.0 }
}

/// What lies on the ground, once the item's picture is known, added to the pictures of this frame.
fn lying(gpu: &mut Gpu, scene: &Scene, time: f64, sprites: &mut Vec<Stood>) {
    let gat = &scene.gat;
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
            // Its shadow, four tenths of the size (`CItem::Render`); and every 92 of the client's frames it
            // glints yellow for two (`CItem::OnProcess`: `isSingleColor` with the colour 0xFFFFFF00).
            shadowed(gpu, on_ground(gat, *at), 0.4, sprites);
            let glints = ((time - scene.falls.get(id).copied().unwrap_or(0.0)) / EFFECT_FRAME_MS) as i64 % 92 >= 90;
            let quad = crate::body::Quad { colour: if glints { [255, 255, 0, 255] } else { [255; 4] }, ..crate::body::Quad::PLAIN };
            sprites.push((key, along(on_ground(gat, *at), [0.0, 1.0, 0.0], above), 1.0, quad));
        }
    }
}

/// The emotion someone shows, over where they stand (`feet`): the pictures of that action, each for as long as
/// the sprite says, once through. One nothing is drawn of shows it too (the client's `CMsgEffect` is a thing
/// of its own beside the actor).
fn emoting(gpu: &mut Gpu, mover: &mut Mover, feet: Vector, time: f64, sprites: &mut Vec<Stood>) {
    let Some((sprite, action, began)) = &mover.emotion else { return };
    let passed = time - began;
    // Until the sprite is read its length is not known: it is given four seconds to come.
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
                sprites.push((key, along(feet, [0.0, 1.0, 0.0], 20.0), 1.0, crate::body::Quad::PLAIN));
            }
        }
        None => mover.emotion = None,
    }
}

/// Everyone to draw, each with the picture wanted now if it is ready, else the one drawn last; with them what
/// lies on the ground and the pointer's mark in the world. `own` is where the player's own character is now and
/// the picture wanted of it. (One drawn as a model has no sprite and no shadow: `granny.rs`.)
pub(super) fn standing(state: &mut State, own: ((f32, f32), SpriteKey), time: f64, yaw: f32) -> Vec<Stood> {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &mut state.scene) else { return Vec::new() };
    let (position, key) = own;
    // Monsters struck down are gone once what is left of them has faded; so are those out of sight.
    scene.units.retain(|_, unit| unit.dying.is_none_or(|fallen| time - fallen < CORPSE_MS) && unit.pose.vanishes.is_none_or(|gone| time < gone));
    let mut sprites = Vec::with_capacity(scene.units.len() + 1);
    let (gat, feet_cell, hide_players) = (&scene.gat, position, state.options.hide_players);
    let mut own = true;
    let sprited = scene.units.values_mut().filter(|unit| !matches!(unit.look, Look::Model(_)));
    for (mover, position, wanted) in std::iter::once((&mut scene.player, position, key)).chain(sprited.map(|unit| {
        let (position, key) = unit.now(time, yaw);
        (unit, position, key)
    })) {
        // The first is the player's own character.
        let is_own = std::mem::replace(&mut own, false);
        let feet = on_ground(gat, position);
        // Of a hidden NPC nothing is drawn, no shadow either (`CNpc::Render`, its `JT_HIDDEN_NPC`).
        if matches!(mover.look, Look::Unseen) {
            emoting(gpu, mover, feet, time, &mut sprites);
            continue;
        }
        // What is left of one struck down has no shadow: the client's corpse (`CCorpse`, made when the death's
        // motion is over) is a render object whose `shadowVisible` nobody sets, so `DrawShadow` draws none
        // while it fades.
        let corpse = mover.dying.is_some_and(|fallen| time >= fallen);
        if mover.shadowed(is_own) && mover.job != 565 && !corpse {
            shadowed(gpu, feet, data::shadow(mover.job) * mover.ail.body.scale(), &mut sprites);
        }
        // A frame of a monster's sprite may start a sound: the nearer the monster, the louder.
        if let SpriteKey::Creature(path, action, frame) = &wanted {
            if mover.heard.replace((*action, *frame)) != Some((*action, *frame)) {
                if let Some(name) = figure::sound_of(path, *action, *frame) {
                    earshot::heard_from(&name, position, feet_cell);
                }
            }
        }
        // So may a frame of a character's body: the swing of its weapon, heard from where it is
        // (`CRenderObject::PlayFrameSounds`, `crate::sprite::frames_sounding`).
        if let SpriteKey::Player(figure) = &wanted {
            let (action, count) = (figure.action - figure.action % 8, figure::body_frames(figure.job, figure.sex, figure.action).unwrap_or(0));
            let frame = if count > 0 { figure.frame % count } else { figure.frame };
            for due in crate::sprite::frames_sounding(mover.heard.replace((action, frame)), action, frame, count.max(frame + 1)) {
                if let Some(name) = figure::body_sound_of(figure.job, figure.sex, figure.action, due) {
                    earshot::heard_from(&name, position, feet_cell);
                }
            }
        }
        // Which of the eight ways it is seen from: the marks of its states that turn with it need it.
        let turned = match &wanted {
            SpriteKey::Player(figure) | SpriteKey::Weapon(figure) => figure.action % 8,
            SpriteKey::Creature(_, action, _) => action % 8,
        };
        // One turned to stone or frozen, or whose blade is caught, stays in the picture it had (the client's
        // `isMotionFreezed`).
        if gpu.has_sprite(&wanted) && !((mover.ail.still() || mover.pose.frozen) && mover.shown.is_some()) {
            mover.shown = Some(wanted);
        }
        // Other players are see-through when the player has it so (the client's Hide other Players). One that
        // hides, cloaks or is invisible is not drawn, the player's own character neither.
        let faint = hide_players && !is_own && matches!(mover.look, Look::Character(_));
        if let Some(shown) = mover.shown.as_ref().filter(|_| !mover.hidden()) {
            bodied(gpu, mover, shown, feet, left_of(mover, faint, time), time, &mut sprites);
            // The marks of its body and health states stay by it.
            ailments::marks(gpu, gat, mover, feet, turned, time, &mut sprites);
        }
        // The puff of smoke of one that hid or came out, where it stood.
        if let Some((began, at)) = mover.pose.smoke {
            if !ailments::played_once(gpu, pose::SMOKE, (began, on_ground(gat, at)), time, &mut sprites) {
                mover.pose.smoke = None;
            }
        }
        emoting(gpu, mover, feet, time, &mut sprites);
    }
    roots(&mut state.roots, gpu, time, &mut sprites);
    // The copies of someone that effects left standing.
    ailments::copies(&mut state.copies, gpu, time, &mut sprites);
    lying(gpu, scene, time, &mut sprites);
    // Last frame's places of the marks in the world: their pictures turn at two of the client's frames each.
    for at in state.target_at.into_iter().flatten() {
        let pictures = figure::timing("cursors", 3).map_or(1, |(count, _)| count.max(1));
        let key = SpriteKey::Creature("cursors".to_owned(), 3, (time / EFFECT_FRAME_MS / 2.0) as usize % pictures);
        if gpu.has_sprite(&key) {
            sprites.push((key, at, 1.0, crate::body::Quad::PLAIN));
        }
    }
    sprites
}

/// The roots that come up under those a Mandragora or its like strikes (`CMandraAttackEffect`): each the first
/// action of its sprite played once where the one struck stood, drawn as any sprite standing there
/// (`CRenderObject::Render`, with no shadow). Until the sprite is read its length is not known: it is given
/// four seconds to come.
fn roots(roots: &mut Vec<(f64, Vector, u16, &'static str)>, gpu: &mut Gpu, time: f64, sprites: &mut Vec<Stood>) {
    roots.retain(|&(began, at, job, sprite)| {
        let Some((pictures, each)) = figure::timing(sprite, 0) else {
            // (Asking for its first picture has the sprite read.)
            gpu.has_sprite(&SpriteKey::Creature(sprite.to_owned(), 0, 0));
            return time - began <= 4000.0;
        };
        let Some(picture) = crate::blow::shots::root_picture(job, time - began, (each / figure::DELAY_MS) as f32, pictures) else { return false };
        let key = SpriteKey::Creature(sprite.to_owned(), 0, picture);
        if gpu.has_sprite(&key) {
            sprites.push((key, at, 1.0, crate::body::Quad::PLAIN));
        }
        true
    });
}
