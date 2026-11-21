//! One of the client's effects begins on someone, or ends: what the app tells the world of an effect by its
//! number or by its file. An effect that is played from files (`crate::effect::played`: one file of the
//! client's table, one of several picked by lot, files a class of the client's carries) is played as its
//! files; any other is drawn by code, if it is ported (`prims`).

use super::*;

/// An effect file ready to play: its layers, and each layer's pictures on the GPU (for the see-through pipeline
/// and for the one that adds light).
pub(super) struct Shown {
    pub(super) effect: crate::effect::Effect,
    pub(super) pictures: Vec<Vec<Option<Rc<[wgpu::BindGroup; 2]>>>>,
}

/// An effect playing on someone (`None`: the player's own character) since a moment.
pub(super) struct Playing {
    pub(super) shown: Rc<Shown>,
    pub(super) on: Option<u32>,
    pub(super) began: f64,
    /// How it is carried, for a file a class of the client's own moves, turns or begins late; none for a file
    /// that plays over the one it is on.
    pub(super) carried: Option<Carrying>,
}

/// How a playing file is carried (`crate::effect::carried`), whose place it was told besides that of the one it
/// is on (none told: nobody; no one named: the player's own character), and what it measured as it began, as
/// the client's effect does in its `Open`: where the two stood and the angle it is turned by.
pub(super) struct Carrying {
    pub(super) how: crate::effect::carried::Carried,
    pub(super) other: Option<Option<u32>>,
    pub(super) set: Option<(Vector, Vector, f32)>,
}

/// One of the client's effect files plays on someone after a while, with a sound if it has one.
pub fn effect_later(name: &str, on: Option<u32>, delay: f64, sound: Option<&str>) {
    STATE.with_borrow_mut(|state| {
        let time = now() + delay.max(0.0);
        state.effects_due.push((time, name.to_owned(), on, None));
        if let Some(sound) = sound {
            state.heard.push((time, sound.to_owned(), on));
        }
    });
}

/// A sound starts where someone stands (none named: the player's own character), and is heard from there
/// (`earshot.rs`): a sound the client's handler of a skill starts beside the skill's effects.
pub fn sound_on(sound: &str, on: Option<u32>) {
    STATE.with_borrow_mut(|state| state.heard.push((now(), sound.to_owned(), on)));
}

/// The screen shakes (the client's `MM_QUAKE` with that number, `crate::quake`) as a handler of the client's
/// asks for it beside a skill's effects, not an effect itself; nothing with the shakes off (`/quake`).
pub fn quake(kind: u32) {
    STATE.with_borrow_mut(|state| {
        if state.options.quake {
            state.quake = Some(crate::quake::Quake::begun(kind, now()));
        }
    });
}

/// What an effect launched for a blow is told besides whom it is on: who strikes, whose place it is told (the
/// client's `AM_SETPOS`; for both, none named is the player's own character), and of how many hits the blow is
/// (`AM_SETCOUNT`).
#[derive(Clone, Copy)]
pub struct Blow {
    /// (For a skill that strikes nobody: the one the effect is on, and `place_of` the other of the two.)
    pub striker: Option<u32>,
    pub place_of: Option<u32>,
    pub hits: u32,
}

/// One of the client's effects begins on someone (none named: the player's own character) after `delay`
/// milliseconds, with the sounds it starts. One launched for a blow is told of the blow: an effect the client
/// has the striker launch towards a place is then on the striker, whoever `on` is. An effect that is played
/// from files is played so, and what of it the client draws by code besides is begun with it (as
/// `skill_units::launch` does for what stands on the ground). Gives back whether it is one shown here yet.
/// An effect the client launches only while another runs on that one, or does not, is left out when that is
/// not so (`crate::state_effects::begins`).
pub fn burst(effect: u32, on: Option<u32>, blow: Option<Blow>, delay: f64) -> bool {
    STATE.with_borrow_mut(|state| {
        if !crate::state_effects::begins(effect, |other| running(state, other, on)) {
            return true;
        }
        let begins = now() + delay.max(0.0);
        let played = files(state, effect, on, blow, begins);
        let before = state.sounds_due.len();
        // (What the client's function of an effect that is a file draws by code besides is begun with the
        // file: the coins of Mammonite, `CRagEffect::Coin`.)
        let Some(burst) = prims::begin(effect, on, blow, begins, &mut state.bursts, &mut state.sounds_due) else { return played };
        // The sound it starts with is heard from where the one it is on stands (`earshot.rs`).
        let whose = burst.on;
        state.heard.extend(state.sounds_due.drain(before..).map(|(at, sound)| (at, sound, whose)));
        // One that begins at once counts how far the one it is on goes from where that one stands now.
        let stands = state.scene.as_ref().and_then(|scene| scene.place_of(burst.on).map(|at| on_ground(&scene.gat, at)));
        if let Some(place) = stands.filter(|_| delay <= 0.0) {
            burst.stood(place);
        }
        true
    })
}

/// The files of an effect that is played from files (`crate::effect::played`) begin on someone at `time`: each
/// is told how it is carried, if a class of the client's carries it, and whose place the effect was told
/// besides; its sounds start at their ticks. Gives back whether the effect is such a one.
pub(super) fn files(state: &mut State, effect: u32, on: Option<u32>, blow: Option<Blow>, time: f64) -> bool {
    let Some(launch) = crate::effect::played::played(effect, &mut lot, &mut state.hit_turn) else { return false };
    for (file, how) in launch.files {
        let carried = how.map(|how| Carrying { how, other: blow.map(|blow| blow.place_of), set: None });
        state.effects_due.push((time, file, on, carried));
    }
    for (tick, sound) in launch.sounds {
        state.heard.push((time + f64::from(tick * crate::effect::carried::TICK_MS), sound, on));
    }
    true
}

/// Whether one of the client's effects runs on someone (its `HasId`).
pub(super) fn running(state: &State, effect: u32, on: Option<u32>) -> bool {
    state.bursts.iter().any(|burst| burst.is(effect, on))
}

/// One of the client's effects under way on someone is ended (none named: the player's own character): the
/// client's `AM_DELETE_EFFECT`, which it sends an actor for an effect that lives until it is told to go. One
/// that is played from a file of the client's ends too.
pub fn effect_ends(effect: u32, on: Option<u32>) {
    STATE.with_borrow_mut(|state| {
        state.bursts.retain(|burst| !burst.is(effect, on));
        let Some(file) = crate::effect_tables::effect_file(effect).map(str::to_lowercase) else { return };
        state.effects_due.retain(|due| due.1 != file || due.2 != on);
        if let Some(shown) = state.effect_files.get(&file).cloned() {
            state.effects.retain(|playing| playing.on != on || !Rc::ptr_eq(&playing.shown, &shown));
        }
    });
}

/// Plays one of the client's effects (a `.str` file of `data/texture/effect`) on someone: `None` is the player's
/// own character. The file and its pictures are read the first time it is wanted.
pub fn effect(name: &str, on: Option<u32>) {
    effect_carried(name, on, None);
}

/// Plays an effect file on someone, carried as it is told (`Carrying`; none: over the one it is on).
pub(super) fn effect_carried(name: &str, on: Option<u32>, carried: Option<Carrying>) {
    // Effects are not shown when the player has them off (the client's `/effect`).
    if !STATE.with_borrow(|state| state.options.effects) {
        return;
    }
    if let Some(shown) = STATE.with_borrow(|state| state.effect_files.get(name).cloned()) {
        STATE.with_borrow_mut(|state| state.effects.push(Playing { shown, on, began: now(), carried }));
        return;
    }
    let name = name.to_owned();
    wasm_bindgen_futures::spawn_local(async move {
        let Ok(bytes) = fetch_asset(&format!("data/texture/effect/{name}")).await else { return };
        let Ok(korean) = web_sys::TextDecoder::new_with_label("euc-kr") else { return };
        let Ok(effect) = crate::effect::parse(
            &bytes,
            |raw| korean.decode_with_u8_array(raw).unwrap_or_else(|_| String::from_utf8_lossy(raw).into_owned())
        ) else { return };
        // Each picture once, however many layers show it.
        let names: BTreeSet<&str> = effect.layers.iter().flat_map(|layer| layer.textures.iter().map(String::as_str)).collect();
        // (Each is in the folder of the effect's file: `crate::effect::texture_path`.)
        let folder = name.as_str();
        let read: HashMap<&str, Option<bmp::Image>> = futures_util::future::join_all(names.iter().map(
            |name| async move { (*name, picture(&format!("effect\\{}", crate::effect::texture_path(folder, name))).await.ok()) }
        )).await
            .into_iter()
            .collect();
        STATE.with_borrow_mut(|state| {
            let Some(gpu) = &state.gpu else { return };
            let on_gpu: HashMap<&str, Rc<[wgpu::BindGroup; 2]>> = read
                .iter()
                .filter_map(|(name, image)| {
                    let image = image.as_ref()?;
                    Some((*name, gpu.effect_binds(image.width, image.height, &image.rgba, &gpu.sampler)))
                })
                .collect();
            let pictures = effect.layers.iter().map(|layer| layer.textures.iter().map(|name| on_gpu.get(name.as_str()).cloned()).collect()).collect();
            let shown = Rc::new(Shown { effect: effect.clone(), pictures });
            state.effect_files.insert(name.clone(), shown.clone());
            state.effects.push(Playing { shown, on, began: now(), carried });
        });
    });
}
