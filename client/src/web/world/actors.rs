//! What the server says of everyone else on the map and of the things lying on it.

use super::*;
use crate::creatures::Shape;

/// The sprite numbers of things that are there without being seen or pressed: the areas a script is touched
/// on (`JT_HIDDEN_WARP_NPC`, of which `CNpc::Render` makes nothing at all) and the server's own NPC of no
/// place. (The hidden NPC, 111, is not seen but is pressed: `crate::creatures::Shape::Hidden`.)
pub(super) const UNSEEN: [u16; 2] = [139, 32767];
/// The sprite number of a portal: drawn as rings of light, not as a sprite.
pub(super) const PORTAL: u16 = 45;

/// The character the packet that shows someone describes. `armed`: with what it holds, as a player is; an
/// NPC that wears a player's job is dressed by the packet's job, sex, hair, colours and headgear alone (the
/// client's `CCharacterNpc`, `FillCharacterNpcRenderPrimary`).
fn figure_of(unit: &Unit, armed: bool) -> Figure {
    Figure {
        job: unit.job,
        sex: unit.sex,
        hair_style: unit.hair_style,
        hair_colour: unit.hair_colour,
        headgear: unit.headgear,
        body_palette: unit.body_palette,
        weapon: if armed { unit.weapon } else { 0 },
        shield: if armed { unit.shield } else { 0 },
        action: 0,
        frame: 0,
    }
}

/// Someone comes into sight, or someone in sight starts to walk.
pub fn unit_shown(unit: &Unit) {
    STATE.with_borrow_mut(|state| {
        // The time a walker set out at corrects the clock, whether it is drawn or not (the client does so as
        // it reads each packet that shows someone walking).
        if let Some(walk) = &unit.walks {
            state.clock.sync(now(), walk.started);
        }
        // Its level and whether it is a boss: its name is coloured and marked by them (`names.rs`).
        names::ranked(state, unit.id, unit.level, unit.boss);
        let Some(scene) = &mut state.scene else {
            // The map is still being read: the server does not say it twice.
            state.early.retain(|early| early.id != unit.id);
            state.early.push(unit.clone());
            return;
        };
        if unit.job == PORTAL && unit.kind != Kind::Player {
            scene.portals.insert(unit.id, (i32::from(unit.cell.0), i32::from(unit.cell.1)));
            return;
        }
        // One that is there without being seen or pressed is a place effects play on and sounds are heard
        // from (what a script's `specialeffect` aims at it), and nothing else: not drawn, not picked, not
        // named.
        if UNSEEN.contains(&unit.job) && unit.kind != Kind::Player {
            scene.spots.insert(unit.id, (f32::from(unit.cell.0), f32::from(unit.cell.1)));
            return;
        }
        // A player's object is a character with what it holds. Anyone else is what its job makes it, whatever
        // kind of object the server calls it (`crate::creatures`): a character's body for a player's job, or
        // one sprite out of the folder of its job.
        let look = match (unit.kind, crate::creatures::shape(unit.job, data::sprite(unit.job).as_deref())) {
            (Kind::Player, _) => Look::Character(figure_of(unit, true)),
            (_, Some(Shape::Character)) => Look::Character(figure_of(unit, false)),
            (_, Some(Shape::Hidden)) => Look::Unseen,
            // A pet that comes into sight wearing its accessory (what would be a player's lower headgear) is
            // drawn with the actions for that, as when it puts it on (`dressed`).
            (kind, Some(Shape::Sprite(path))) => match Some(unit.headgear[0]).filter(|worn| kind == Kind::Companion && *worn != 0).and_then(data::pet_act) {
                Some(actions) => Look::Creature(format!("{path}@몬스터/{actions}")),
                None => Look::Creature(path),
            },
            // A job whose name is a model's file is drawn as that model (`web/world/granny.rs`).
            (_, Some(Shape::Model(name))) => granny::look(&mut state.granny, name),
            (_, None) => return,
        };
        let cell = (i32::from(unit.cell.0), i32::from(unit.cell.1));
        let mover = scene.units.entry(unit.id).or_insert_with(|| Mover::new(look.clone(), cell));
        mover.look = look;
        // One that went out of sight a moment ago and is back does not fade on.
        (mover.pose.vanishes, mover.pose.head) = (None, unit.head);
        // Who it is (a monster's kind too: `crate::blow` asks), and a player's state: what it rides shows at once.
        mover.job = unit.job;
        if unit.kind == Kind::Player {
            mover.state = unit.state;
            mover.dress();
        }
        // What follows a # in the name of an NPC or a monster tells the server's scripts apart, and is not shown.
        mover.name = if unit.kind == Kind::Player { unit.name.clone() } else { unit.name.split('#').next().unwrap_or_default().trim().to_owned() };
        mover.lively = unit.kind == Kind::Monster;
        // A monster's life as the server tells it with the monster: its bar shows from then, and goes when
        // the server tells none (the client's `AM_UPDATE_MONSTER_GAGE`, sent -1 for both).
        if mover.lively {
            mover.life = unit.life;
        }
        mover.companion = unit.kind == Kind::Companion;
        mover.guild = if unit.kind == Kind::Player || unit.job == granny::FLAG { (unit.guild, u32::from(unit.emblem)) } else { (0, 0) };
        // Its body and health states: one who comes into sight poisoned or frozen is seen so.
        ailments::seen(mover, unit.body, unit.health);
        mover.speed = if unit.speed > 0 { f64::from(unit.speed) } else { DEFAULT_SPEED };
        let lay = mover.dead;
        match &unit.walks {
            Some(walk) => {
                mover.dead = false;
                mover.walk(&scene.gat, &state.clock, walk);
            }
            None => {
                mover.walk = None;
                mover.put(cell);
                // The server counts directions from the north and turning left, the sprites from the south and
                // turning the other way.
                mover.facing = crate::pose::facing_of(unit.direction);
                // How it is there: sitting, or lying dead (the packet's `state`; the client's `AM_STANDENTRY`
                // puts it straight into that state, lying in the last picture of its fall).
                if unit.kind == Kind::Player {
                    (mover.sitting, mover.dead) = (unit.posture == crate::net::actors::Posture::Sits, unit.posture == crate::net::actors::Posture::Lies);
                }
            }
        }
        // One that lay dead and is shown alive again is up.
        if lay && !mover.dead {
            mover.rises();
        }
        // The states its third option tells show on it: one who walks into sight with Fury on is seen so.
        state_effects::third_option(state, unit.id, unit.third, unit.state, unit.kind == Kind::Player);
    });
}

/// Someone already in sight walks from one cell to another (ZC_NOTIFY_MOVE): the time it set out at corrects
/// the clock, and the walk is shown from that time (the client's `AM_MOVE`).
pub fn unit_walk(id: u32, told: &Walked) {
    STATE.with_borrow_mut(|state| {
        state.clock.sync(now(), told.started);
        let Some(scene) = &mut state.scene else { return };
        if let Some(unit) = scene.units.get_mut(&id) {
            unit.walk(&scene.gat, &state.clock, told);
        }
    });
}

/// Someone is on a cell at once (ZC_HIGHJUMP: a leap, a slide, being thrown back): none named is the player's
/// own character. The client writes the cell's middle as its place and nothing else.
pub fn slid(id: Option<u32>, cell: (u16, u16)) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let mover = scene.mover_mut(id);
        if let Some(mover) = mover {
            mover.put((i32::from(cell.0), i32::from(cell.1)));
        }
    });
}

/// How much life a monster has left, as the server tells while it is fought.
pub fn unit_life(id: u32, left: i32, most: i32) {
    STATE.with_borrow_mut(|state| {
        if let Some(unit) = state.scene.as_mut().and_then(|scene| scene.units.get_mut(&id)) {
            unit.life = Some((left, most));
        }
    });
}

/// Something lies on the ground, or is gone from it.
pub fn drop_shown(id: u32, (item, identified, count): (u32, bool, u16), cell: (u16, u16), within: (u8, u8), falling: bool) {
    STATE.with_borrow_mut(|state| {
        if let Some(scene) = &mut state.scene {
            // The thing lies where the server put it within its cell, in sixteenths from the cell's corner (the
            // client's `SubcellPosition`): things on one cell do not lie on top of each other.
            let at = (f32::from(cell.0) + f32::from(within.0) / 16.0 - 0.5, f32::from(cell.1) + f32::from(within.1) / 16.0 - 0.5);
            scene.drops.insert(id, Lying { cell: (i32::from(cell.0), i32::from(cell.1)), item, at, identified, count });
            if falling {
                scene.falls.insert(id, now());
            }
        }
    });
}

pub fn drop_gone(id: u32) {
    STATE.with_borrow_mut(|state| {
        if let Some(scene) = &mut state.scene {
            scene.drops.remove(&id);
            scene.falls.remove(&id);
        }
    });
}

/// A monster or an NPC becomes another kind (the server's ZC_NPCSPRITE_CHANGE: a Fabre turns into a Pupa, a
/// Pupa into a Creamy; a script's `setnpcdisplay`): it is drawn as the new kind from now on, its sprite out of
/// the folder of the new job (the client's `CNpc::SetSprAct`, which ends in `SetSprJob`).
pub fn unit_becomes(id: u32, job: u16) {
    STATE.with_borrow_mut(|state| {
        if let Some(unit) = state.scene.as_mut().and_then(|scene| scene.units.get_mut(&id)) {
            reshaped(unit, job);
        }
    });
}

/// Gives one drawn as a sprite the sprite of another job (`CNpc::SetSprJob`: a `CNpc` stays one sprite, so a
/// job that has none in the client's table changes nothing). Says whether it did.
fn reshaped(unit: &mut Mover, job: u16) -> bool {
    if !matches!(unit.look, Look::Creature(_)) {
        return false;
    }
    let Some(name) = data::sprite(job).filter(|name| !name.contains(".gr2")) else { return false };
    (unit.job, unit.look) = (job, Look::Creature(crate::creatures::sprite(job, &name)));
    (unit.motion, unit.shown) = (None, None);
    true
}

/// The server tells what a monster or an NPC is called (after it became another kind, for one).
pub fn unit_named(id: u32, name: &str) {
    STATE.with_borrow_mut(|state| {
        if let Some(unit) = state.scene.as_mut().and_then(|scene| scene.units.get_mut(&id)).filter(|unit| unit.look.creature()) {
            unit.name = name.split('#').next().unwrap_or_default().trim().to_owned();
        }
    });
}

/// Someone's look changes (ZC_SPRITE_CHANGE): `None` is the player's own character. The kinds are the server's:
/// 1 the hair, 2 the weapon, 3 to 5 what is worn on the head (low, top, middle), 6 the hair's colour, 7 the
/// clothes' colour. `second` is the packet's second value: the shield, beside a weapon.
pub fn looked(id: Option<u32>, kind: u8, value: u32, second: u32) {
    STATE.with_borrow_mut(|state| {
        if id.is_none() && kind == 2 {
            (state.weapon, state.shield) = (value, second);
        }
        let Some(scene) = &mut state.scene else { return };
        let mover = scene.mover_mut(id);
        let Some(mover) = mover else { return };
        // The body itself: the job changed (the server's `LOOK_BASE`).
        if kind == 0 && !reshaped(mover, value as u16) {
            mover.job = value as u16;
            mover.dress();
        }
        let Look::Character(figure) = &mut mover.look else { return };
        match kind {
            1 => figure.hair_style = value as u16,
            // The weapon, and with it the shield (rAthena's `clif_changelook` sends both in one).
            2 => (figure.weapon, figure.shield) = (value, second),
            3 => figure.headgear[0] = value as u16,
            4 => figure.headgear[1] = value as u16,
            5 => figure.headgear[2] = value as u16,
            6 => figure.hair_colour = value as u16,
            7 => figure.body_palette = value as u16,
            _ => {}
        }
    });
}

/// Someone's state changed (none named: the player's own character): the bits say what they ride, and they are
/// drawn so from now on. With a `mask`, only those bits are set or cleared and the rest stay as they were.
pub fn stated(id: Option<u32>, bits: u32, mask: Option<u32>) {
    let bash = STATE.with_borrow_mut(|state| {
        let merged = |was: u32| mask.map_or(bits, |mask| was & !mask | bits & mask);
        if id.is_none() {
            state.own_state = merged(state.own_state);
        }
        let Some(scene) = &mut state.scene else { return false };
        let mover = scene.mover_mut(id);
        let Some(mover) = mover else { return false };
        let now_in = merged(mover.state);
        let was = std::mem::replace(&mut mover.state, now_in);
        mover.dress();
        // One that hides, or comes out, puffs smoke (`poses.rs`).
        mover.hides(was, now())
    });
    pose::hid(id, bash);
}

/// Someone says something: it shows over their head for a while. `own` is the player's own ID.
pub fn said(id: u32, own: u32, words: &str) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        if let Some(mover) = scene.one_mut(id, own) {
            mover.says = Some((words.to_owned(), now()));
        }
    });
}

/// What a monster, an NPC or a pet on the map is called, if the server has said.
pub fn unit_name(id: u32) -> Option<String> {
    STATE.with_borrow(|state| state.scene.as_ref()?.units.get(&id).map(|unit| unit.name.clone()).filter(|name| !name.is_empty()))
}

/// Someone shows an emotion (the client's `AM_CASH_EMOTION`): its picture plays once at them, where the sprite
/// itself puts it, over the head. An emotion the client's list does not have shows nothing. `own` is the
/// player's own ID.
pub fn emotes(id: u32, own: u32, emotion: u16) {
    let Some((sprite, action)) = data::emotion(emotion) else { return };
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        if let Some(mover) = scene.one_mut(id, own) {
            mover.emotion = Some((sprite, action, now()));
        }
    });
}

/// Someone sits down or stands up. `own` is the player's own ID.
pub fn sat(id: u32, own: u32, down: bool) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        if let Some(mover) = scene.one_mut(id, own) {
            mover.sitting = down;
            mover.pose.at_ease();
        }
    });
}

/// Someone plays dead, or stops (the client's `EFST_TRICKDEAD` in `ApplyStatusChange`: the last picture of
/// the dying action, held; then standing again). The client treats one who plays dead as one dead wherever it
/// asks (`isTrickDead` beside `ST_DEAD`), and so it is here. `own` is the player's own ID.
pub fn plays_dead(id: u32, own: u32, down: bool) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        if let Some(mover) = scene.one_mut(id, own) {
            mover.dead = down;
            if down {
                mover.walk = None;
            }
        }
    });
}

/// How long one that goes out of sight takes to fade (`AM_OUTOFSIGHT`).
pub(super) const VANISH_MS: f64 = 510.0;

/// Someone is out of sight, or gone.
pub fn unit_gone(id: u32, died: bool) {
    STATE.with_borrow_mut(|state| {
        state.early.retain(|early| early.id != id);
        if let Some(scene) = &mut state.scene {
            // A monster struck down is seen falling before it is gone. It falls once the blows on their way to
            // it have landed (the client's `AM_WILL_BE_DEAD`: it dies when its list of blows to come is empty),
            // through the pictures of its dying, each for as long as its sprite says (`ST_DEAD`, action 32), and
            // then lies as its last picture and fades (`MakeCorpse`).
            let time = now();
            let falls = state.blows.iter().filter(|blow| blow.1 == id).map(|blow| blow.0).fold(time, f64::max);
            // A player struck down lies where it fell (the client's `AM_DEAD`): it is gone when it leaves,
            // and stands up when it is raised (`pose::risen`).
            if let Some(player) = scene.units.get_mut(&id).filter(|unit| died && matches!(unit.look, Look::Character(_))) {
                if !player.dead {
                    player.falls(falls);
                }
                return;
            }
            match scene.units.get_mut(&id).filter(|unit| died && unit.lively) {
                Some(unit) => {
                    let lasts = match &unit.look {
                        Look::Creature(path) => figure::timing(path, 32).map(|(pictures, each)| pictures as f64 * each),
                        Look::Model(name) => state.granny.lasts(name, crate::model::granny::shown::Action::Dead),
                        Look::Character(_) | Look::Unseen => None,
                    }
                    .unwrap_or(700.0);
                    unit.dying = Some(falls + lasts);
                    unit.walk = None;
                    unit.life = None;
                    unit.motion = Some((32, falls, lasts));
                }
                // One out of sight fades away over half a second (`AM_OUTOFSIGHT`: its `vanishTime` is 510
                // milliseconds on, its alpha half the milliseconds left); its shadow goes at once.
                None => {
                    if let Some(unit) = scene.units.get_mut(&id).filter(|unit| unit.dying.is_none()) {
                        unit.pose.vanishes.get_or_insert(time + VANISH_MS);
                    }
                }
            }
            scene.portals.remove(&id);
            // (One that was only a place for effects is no place any more; its effects end with it.)
            if !scene.units.contains_key(&id) && scene.spots.remove(&id).is_some() {
                state.bursts.retain(|burst| burst.on != Some(id));
                state.effects.retain(|playing| playing.on != Some(id));
            }
        }
    });
}

/// The server says someone stands on a cell (ZC_STOPMOVE).
pub fn unit_stand(id: u32, cell: (u16, u16)) {
    STATE.with_borrow_mut(|state| {
        if let Some(unit) = state.scene.as_mut().and_then(|scene| scene.units.get_mut(&id)) {
            unit.stand((i32::from(cell.0), i32::from(cell.1)));
        }
    });
}

/// A pet does a trick (the client's `CHANGESTATEPET_PERFORMANCE`, played when the pet next stands): the first
/// trick is the sprite's actions from 40, the second from 48, the third from 56 and the special one from 64; it
/// plays once, each picture for as long as the sprite says, and the pet stands again. A sprite without those
/// actions shows nothing.
pub fn performs(id: u32, trick: i32) {
    STATE.with_borrow_mut(|state| {
        let Some(unit) = state.scene.as_mut().and_then(|scene| scene.units.get_mut(&id)) else { return };
        let Look::Creature(path) = &unit.look else { return };
        let action = match trick {
            2 => 48,
            3 => 56,
            4 => 64,
            _ => 40,
        };
        if figure::actions_of(path).is_some_and(|actions| action >= actions) {
            return;
        }
        let lasts = figure::timing(path, action).map_or(1000.0, |(pictures, each)| pictures as f64 * each);
        unit.motion = Some((action, now(), lasts.max(250.0)));
    });
}

/// A pet puts its accessory on or takes it off (0): it is drawn with the actions the client has for it wearing
/// that (`GetPetAccActName`), or with its own again. An accessory the client has no actions for changes nothing.
pub fn dressed(id: u32, accessory: u16) {
    STATE.with_borrow_mut(|state| {
        let Some(unit) = state.scene.as_mut().and_then(|scene| scene.units.get_mut(&id)) else { return };
        let Look::Creature(path) = &unit.look else { return };
        let own = path.split('@').next().unwrap_or(path).to_owned();
        let look = match (accessory, data::pet_act(accessory)) {
            (0, _) => own,
            (_, Some(actions)) => format!("{own}@몬스터/{actions}"),
            (_, None) => return,
        };
        if *path != look {
            unit.look = Look::Creature(look);
            unit.shown = None;
        }
    });
}
