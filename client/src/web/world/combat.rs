//! Blows, skills and their effects: what the app tells the world when someone strikes, casts or is hit.

use super::*;

/// Someone strikes someone: the one turns to the other and swings, the other flinches, and the damage shows as
/// a number over it. `own` is the player's own ID, who is not among the others. `lasts` is the speed of the swing
/// and `flinch` the time the one struck flinches, as the server sends them. Gives back how long from now the blow
/// lands (milliseconds). A critical blow lands with a flare and a mark of its own (see `frame/sprites.rs`).
pub fn struck(source: u32, target: u32, own: u32, (lasts, flinch): (f64, f64), (damage, critical): (i32, bool), sound: Option<String>) -> f64 {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return 0.0 };
        let time = now();
        let place = |scene: &Scene, id: u32| if id == own { Some(scene.player.cell) } else { scene.units.get(&id).map(|unit| unit.cell) };
        let (from, to) = (place(scene, source), place(scene, target));
        let mut lands = flinch.clamp(0.0, 1500.0);
        if source == own {
            state.own_struck = time;
            // The blow's own motion (the server's `amotion`, sent with the blow): until it is over the character
            // does not walk (see `striking`).
            state.own_striking_until = time + lasts.max(0.0);
        }
        if let Some(striker) = if source == own { Some(&mut scene.player) } else { scene.units.get_mut(&source) } {
            // Characters have two attacks, and which one shows depends on the job and on what is held (see
            // `attack_action`); monsters have one.
            let action = match &striker.look {
                Look::Character(figure) => attack_action(figure),
                Look::Creature(_) => 16,
            };
            striker.motion = Some((action, time, lasts.clamp(250.0, 2000.0)));
            lands = lands_after(striker, action, lasts, flinch);
            if let (Some(from), Some(to)) = (from, to) {
                if from != to {
                    striker.facing = crate::map::direction(to.0 - from.0, to.1 - from.1);
                }
            }
        }
        // The blow lands a moment into the swing: then the one struck flinches and stops, and the number rises.
        state.landing.push((time + lands, target, own, damage, sound, critical));
        lands
    })
}

/// How long into a swing its blow lands (milliseconds), as the client has it (`crate::blow`): by the frame of
/// the striker's sprite that the blow is on and the speed the server sent with the blow. Until the sprite is
/// read that frame is not known: then it is `otherwise`.
fn lands_after(striker: &Mover, action: usize, speed: f64, otherwise: f64) -> f64 {
    let frame = match &striker.look {
        Look::Character(figure) => figure::body_strike_frame(figure.job, figure.sex, action),
        Look::Creature(path) => figure::strike_frame(path, action),
    };
    frame.map_or(otherwise, |frame| crate::blow::lands_after(frame, speed, striker.job)).clamp(0.0, 1500.0)
}

/// The action of a character bending down for a thing on the ground (the client's `ST_PICKUP`).
const PICKING_UP: usize = 24;

/// Someone picks a thing up (the server's notice of it, `DMG_PICKUP_ITEM`): they turn to the thing and bend down
/// for it once, each picture for as long as the sprite says, and stand again (the client's `ST_PICKUP`). The
/// thing itself is gone from the ground when the server says so. `own` is the player's own ID.
pub fn picks_up(id: u32, own: u32, thing: u32) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let lies = scene.drops.get(&thing).map(|lying| lying.cell);
        let Some(mover) = (if id == own { Some(&mut scene.player) } else { scene.units.get_mut(&id) }) else { return };
        let Look::Character(figure) = &mover.look else { return };
        figure::prepare(figure, PICKING_UP);
        let lasts = figure::body_timing(figure.job, figure.sex, PICKING_UP).map_or(450.0, |(pictures, each)| pictures as f64 * each);
        if let Some(cell) = lies.filter(|cell| *cell != mover.cell) {
            mover.facing = crate::map::direction(cell.0 - mover.cell.0, cell.1 - mover.cell.1);
        }
        // The thing is asked for from a cell away, so the server may say it is taken while the character still
        // walks its last step (after a monster killed from afar, as with a spell): it bends down when it gets
        // there, not while walking, when the bending would not be seen at all.
        let time = now();
        let begins = mover.walk.as_ref().map_or(time, |(walk, started)| (started + walk.duration()).max(time));
        mover.motion = Some((PICKING_UP, begins, lasts.max(250.0)));
    });
}

/// The player's own character is about to pick something up: the pictures of its bending down are made ready
/// while it walks there.
pub fn ready_to_pick_up() {
    STATE.with_borrow(|state| {
        if let Some(Look::Character(figure)) = state.scene.as_ref().map(|scene| &scene.player.look) {
            figure::prepare(figure, PICKING_UP);
        }
    });
}

/// Someone starts to cast a skill (the client's `NotifySkillCasting`): they turn to the one aimed at and stand
/// casting, the bar over them fills over the cast's time, the rings of the cast's element turn around them and
/// the mark closes in on the one aimed at.
pub fn cast(source: u32, own: u32, target: u32, cell: (u16, u16), milliseconds: f64, effect: u32) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let place = |scene: &Scene, id: u32| if id == own { Some(scene.player.cell) } else { scene.units.get(&id).map(|unit| unit.cell) };
        let towards = if target != 0 { place(scene, target) } else { Some((i32::from(cell.0), i32::from(cell.1))) };
        let Some(caster) = (if source == own { Some(&mut scene.player) } else { scene.units.get_mut(&source) }) else { return };
        let time = now();
        if let Some(towards) = towards.filter(|towards| *towards != caster.cell) {
            caster.facing = crate::map::direction(towards.0 - caster.cell.0, towards.1 - caster.cell.1);
        }
        caster.casting = Some((time, milliseconds, effect, (target != 0 && target != source).then_some(target)));
        // Characters stand in their casting action for as long.
        if matches!(caster.look, Look::Character(_)) {
            caster.motion = Some((96, time, milliseconds.max(250.0)));
        }
    });
    sound::effect("effect/ef_beginspell.wav", 1.0);
}

/// A cast is over or broken off (the server's ZC_DISPEL, or the skill landing): the bar, the rings and the mark
/// go, and the caster stands as before.
pub fn cast_over(source: u32, own: u32, broken: bool) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let Some(caster) = (if source == own { Some(&mut scene.player) } else { scene.units.get_mut(&source) }) else { return };
        if caster.casting.take().is_some() && broken {
            caster.motion = None;
        }
    });
}

/// Another hit of a blow already struck (a skill that hits several times): it lands after `delay` milliseconds.
pub fn lands(target: u32, own: u32, delay: f64, damage: i32, sound: Option<String>) {
    STATE.with_borrow_mut(|state| state.landing.push((now() + delay.clamp(0.0, 5000.0), target, own, damage, sound, false)));
}

/// Bolts of fire or of ice fall on someone, one for each hit (the client's `FireArrow` and `IceArrow`): the
/// first 12 frames after the skill lands and one every 10 frames after it, each from 60 above and some 25 and 15
/// to the side, at 2.75 a frame. The sound is one of the client's three, with the first bolt.
pub fn bolts(target: u32, count: u32, fire: bool) {
    let random = |most: f64| (js_sys::Math::random() * most).floor() as f32;
    STATE.with_borrow_mut(|state| {
        let time = now();
        for bolt in 0..count.min(50) {
            let from = [25.0 + random(10.0), 60.0, 15.0 + random(10.0)];
            state.bolts.push(Bolt { leaves: time + f64::from(12 + bolt * 10) * CAST_FRAME_MS, target, from, fire });
        }
        let sound = format!("effect/{}{}.wav", if fire { "ef_firearrow" } else { "ef_icearrow" }, 1 + random(3.0) as u32);
        state.sounds_due.push((time + 12.0 * CAST_FRAME_MS, sound));
    });
}

/// One of the client's effect files plays on someone after a while, with a sound if it has one.
pub fn effect_later(name: &str, on: Option<u32>, delay: f64, sound: Option<&str>) {
    STATE.with_borrow_mut(|state| {
        let time = now() + delay.max(0.0);
        state.effects_due.push((time, name.to_owned(), on));
        if let Some(sound) = sound {
            state.sounds_due.push((time, sound.to_owned()));
        }
    });
}

/// One of the client's effects that it draws by code begins on someone (none named: the player's own character)
/// after `delay` milliseconds, with the sound it starts with. Gives back whether it is one drawn here yet.
pub fn burst(effect: u32, on: Option<u32>, delay: f64) -> bool {
    let begins = now() + delay.max(0.0);
    let Some((burst, sound)) = prims::burst(effect, on, begins) else { return false };
    STATE.with_borrow_mut(|state| {
        state.bursts.push(burst);
        if !sound.is_empty() {
            state.sounds_due.push((begins, sound.to_owned()));
        }
    });
    true
}

/// An arrow is shot at someone, to arrive after `delay` milliseconds (the client's `CArrowEffect`): it leaves
/// 10 above the archer's feet and flies for 8 of the client's frames.
pub fn arrow(source: u32, target: u32, own: u32, delay: f64) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &state.scene else { return };
        let Some(archer) = (if source == own { Some(&scene.player) } else { scene.units.get(&source) }) else { return };
        // Only one who holds a bow shoots (the client asks its weapon table whether the weapon uses arrows).
        let Look::Character(figure) = &archer.look else { return };
        if !data::about(figure.weapon).is_some_and(|(_, facts)| facts.iter().any(|(key, kind)| key == "SubType" && kind == "Bow")) {
            return;
        }
        let from = along(on_ground(&scene.gat, archer.at), [0.0, 1.0, 0.0], 10.0);
        state.arrows.push((now() + (delay - ARROW_MS).max(0.0), from, target));
    });
}

/// The server says how far the character's weapon reaches, in cells (ZC_ATTACK_RANGE): it does on entering the
/// map and whenever that changes (another weapon, a skill that lengthens a bow's reach).
pub fn weapon_reaches(range: i32) {
    STATE.with_borrow_mut(|state| state.strike_range = Some(range.max(1)));
}

/// The server says a blow was asked for from too far (ZC_ATTACK_FAILURE_FOR_DISTANCE), and how far the weapon
/// reaches: the character walks up to the monster it fights and strikes from there, as the client does.
pub fn too_far(target: u32, range: i32) {
    STATE.with_borrow_mut(|state| {
        state.strike_range = Some(range.max(1));
        if let Some((id, _, told)) = &mut state.chase {
            if *id == target {
                // Looked after at once, and the blow asked for again once it stands near enough.
                *told = false;
            }
        }
        if let Some(chase) = &mut state.chase {
            chase.1 = 0.0;
        }
    });
}

/// The character stops going after anything: the monster it fought, the thing it meant to pick up, the place
/// it meant to use a skill from.
pub fn stand_down() {
    STATE.with_borrow_mut(|state| {
        state.chase = None;
        state.fetch = None;
        state.reach = None;
    });
}

/// A skill starts or stops being aimed: at a place (`Some(true)`), at someone (`Some(false)`), or not at all.
pub fn aim(at: Option<bool>, range: u16) {
    STATE.with_borrow_mut(|state| {
        state.aiming = at;
        state.aim_range = f32::from(range);
        state.reach = None;
    });
    aim_shown();
}

/// Whether the skill being aimed needs Shift held to go on a monster (a skill of help, with the client's No
/// Shift option off).
pub fn aim_guarded(guarded: bool) {
    STATE.with_borrow_mut(|state| state.aim_guarded = guarded);
}

/// What is written by the pointer while a skill is aimed: its name and the level it will be used at.
pub fn aim_words(words: &str) {
    STATE.with_borrow_mut(|state| state.aim_words = words.to_owned());
    aim_shown();
}

/// Whether a skill is being aimed.
/// A character uses a skill that strikes nobody (the server's ZC_USE_SKILL): it goes through the motion of using
/// a skill once and stands again (the client's `CPlayer::SetState(ST_SKILL)`: action 96, the one it casts in; for
/// the bards, dancers and crusaders and what they grow into, action 32, their stance). One who sits stays
/// seated, and a monster has motions of its own, which are not shown here.
pub fn uses_skill(who: u32, own: u32, body: u8) {
    // Bard, Dancer, Crusader; their reborn and baby selves; Minstrel, Wanderer and Royal Guard with their reborn
    // selves; and the two the client's code lists after them.
    const STANCE: [u16; 17] = [19, 20, 14, 4020, 4021, 4015, 4042, 4043, 4037, 4068, 4075, 4069, 4076, 4073, 4066, 4263, 4264];
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let Some(mover) = (if who == own { Some(&mut scene.player) } else { scene.units.get_mut(&who) }) else { return };
        let Look::Character(figure) = &mover.look else { return };
        if mover.sitting {
            return;
        }
        // By the state the client's table gives the skill: the blow of an attack (2, 9), the stance of one (11),
        // or else the motion of using a skill.
        let action = match body {
            2 | 9 => attack_action(figure),
            11 => 32,
            _ if STANCE.contains(&figure.job) => 32,
            _ => 96,
        };
        figure::prepare(figure, action);
        let lasts = figure::body_timing(figure.job, figure.sex, action).map_or(600.0, |(pictures, each)| pictures as f64 * each);
        mover.motion = Some((action, now(), lasts.max(250.0)));
    });
}

/// Someone used First Aid: its glitter shows over them (`effects::aid_glitter`). `on` is none for the player's own
/// character.
pub fn first_aid(on: Option<u32>) {
    if !STATE.with_borrow(|state| state.options.effects) {
        return;
    }
    let lot = |most: f64| (js_sys::Math::random() * most).floor() as f32;
    STATE.with_borrow_mut(|state| state.aids.push((on, now(), lot(360.0), lot(360.0))));
}

/// Whether the player's own character is in the motion of a blow it deals. The original client asks for no walk
/// during it (rAthena's `attack_walk_delay` is off for players because, as its comment says, "the client already
/// blocks sending movement requests during this time"): one cannot strike and walk on at once.
pub fn striking() -> bool {
    STATE.with_borrow(|state| now() < state.own_striking_until)
}

/// Whether the player's own character is walking now.
pub fn walking() -> bool {
    STATE.with_borrow(|state| state.scene.as_ref().is_some_and(|scene| scene.player.walk.is_some()))
}

pub fn aiming() -> bool {
    STATE.with_borrow(|state| state.aiming.is_some())
}

/// The player's own character recovers life (or spirit): the amount rises over it, green (or blue).
pub fn recovered(amount: i32, spirit: bool) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &state.scene else { return };
        let sprite = if spirit { "이팩트/숫자#0000ff" } else { "이팩트/숫자#00ff00" };
        let pictures = amount.to_string().bytes().map(|digit| SpriteKey::Creature(sprite.to_owned(), 0, usize::from(digit - b'0'))).collect();
        state.flying.push(Flying { began: now(), base: on_ground(&scene.gat, scene.player.at), drift: (0.0, 0.0), pictures, flight: Flight::Miss });
    });
}

/// Plays one of the client's effects (a `.str` file of `data/texture/effect`) on someone: `None` is the player's
/// own character. The file and its pictures are read the first time it is wanted.
pub fn effect(name: &str, on: Option<u32>) {
    // Effects are not shown when the player has them off (the client's `/effect`).
    if !STATE.with_borrow(|state| state.options.effects) {
        return;
    }
    if let Some(shown) = STATE.with_borrow(|state| state.effect_files.get(name).cloned()) {
        STATE.with_borrow_mut(|state| state.effects.push(Playing { shown, on, began: now() }));
        return;
    }
    let name = name.to_owned();
    wasm_bindgen_futures::spawn_local(async move {
        let Ok(bytes) = fetch_asset(&format!("data/texture/effect/{name}")).await else { return };
        let Ok(korean) = web_sys::TextDecoder::new_with_label("euc-kr") else { return };
        let Ok(effect) = crate::effect::parse(&bytes, |raw| korean.decode_with_u8_array(raw).unwrap_or_else(|_| String::from_utf8_lossy(raw).into_owned())) else { return };
        // Each picture once, however many layers show it.
        let names: BTreeSet<&str> = effect.layers.iter().flat_map(|layer| layer.textures.iter().map(String::as_str)).collect();
        let read: HashMap<&str, Option<bmp::Image>> = futures_util::future::join_all(names.iter().map(|name| async move { (*name, picture(&format!("effect\\{name}")).await.ok()) })).await.into_iter().collect();
        STATE.with_borrow_mut(|state| {
            let Some(gpu) = &state.gpu else { return };
            let on_gpu: HashMap<&str, Rc<[wgpu::BindGroup; 2]>> = read
                .iter()
                .filter_map(|(name, image)| {
                    let image = image.as_ref()?;
                    let view = gpu.texture(image.width, image.height, &image.rgba);
                    Some((*name, Rc::new([0, 1].map(|which| gpu.textured(&gpu.effect_pipelines[which].get_bind_group_layout(1), &view, &gpu.sampler)))))
                })
                .collect();
            let pictures = effect.layers.iter().map(|layer| layer.textures.iter().map(|name| on_gpu.get(name.as_str()).cloned()).collect()).collect();
            let shown = Rc::new(Shown { effect: effect.clone(), pictures });
            state.effect_files.insert(name.clone(), shown.clone());
            state.effects.push(Playing { shown, on, began: now() });
        });
    });
}
