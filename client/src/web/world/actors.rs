//! What the server says of everyone else on the map and of the things lying on it.

use super::*;

/// Someone comes into sight, or someone in sight starts to walk.
pub fn unit_shown(unit: &Unit) {
    STATE.with_borrow_mut(|state| {
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
        if UNSEEN.contains(&unit.job) && unit.kind != Kind::Player {
            return;
        }
        let look = match unit.kind {
            Kind::Player => Look::Character(Figure {
                job: unit.job,
                sex: unit.sex,
                hair_style: unit.hair_style,
                hair_colour: unit.hair_colour,
                headgear: unit.headgear,
                body_palette: unit.body_palette,
                weapon: 0,
                action: 0,
                frame: 0,
            }),
            // Monsters and NPCs are one sprite each, in folders of their own.
            kind => match data::sprite(unit.job) {
                // A pet that comes into sight wearing its accessory (what would be a player's lower headgear) is
                // drawn with the actions for that, as when it puts it on (`dressed`).
                Some(name) => match Some(unit.headgear[0]).filter(|worn| kind == Kind::Companion && *worn != 0).and_then(data::pet_act) {
                    Some(actions) => Look::Creature(format!("몬스터/{name}@몬스터/{actions}")),
                    None => Look::Creature(format!("{}/{name}", if kind == Kind::Npc { "npc" } else { "몬스터" })),
                },
                None => return,
            },
        };
        let cell = (i32::from(unit.cell.0), i32::from(unit.cell.1));
        let mover = scene.units.entry(unit.id).or_insert_with(|| Mover::new(look.clone(), cell));
        mover.look = look;
        // What follows a # in the name of an NPC or a monster tells the server's scripts apart, and is not shown.
        mover.name = if unit.kind == Kind::Player { unit.name.clone() } else { unit.name.split('#').next().unwrap_or_default().trim().to_owned() };
        mover.lively = unit.kind == Kind::Monster;
        mover.companion = unit.kind == Kind::Companion;
        mover.speed = if unit.speed > 0 { f64::from(unit.speed) } else { DEFAULT_SPEED };
        match unit.walks_to {
            Some(to) => mover.walk(&scene.gat, cell, (i32::from(to.0), i32::from(to.1))),
            None => {
                mover.walk = None;
                mover.cell = cell;
                // The server counts directions from the north and turning left, the sprites from the south and
                // turning the other way.
                mover.facing = (12 - unit.direction % 8) % 8;
            }
        }
    });
}

/// Someone already in sight walks from one cell to another.
pub fn unit_walk(id: u32, from: (u16, u16), to: (u16, u16)) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        if let Some(unit) = scene.units.get_mut(&id) {
            unit.walk(&scene.gat, (i32::from(from.0), i32::from(from.1)), (i32::from(to.0), i32::from(to.1)));
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
pub fn drop_shown(id: u32, item: u32, identified: bool, cell: (u16, u16), within: (u8, u8), falling: bool) {
    STATE.with_borrow_mut(|state| {
        if let Some(scene) = &mut state.scene {
            // The thing lies where the server put it within its cell, in sixteenths from the cell's corner (the
            // client's `SubcellPosition`): things on one cell do not lie on top of each other.
            let at = (f32::from(cell.0) + f32::from(within.0) / 16.0 - 0.5, f32::from(cell.1) + f32::from(within.1) / 16.0 - 0.5);
            scene.drops.insert(id, ((i32::from(cell.0), i32::from(cell.1)), item, at, identified));
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
/// Pupa into a Creamy): it is drawn as the new kind from now on.
pub fn unit_becomes(id: u32, job: u16) {
    STATE.with_borrow_mut(|state| {
        let Some(unit) = state.scene.as_mut().and_then(|scene| scene.units.get_mut(&id)) else { return };
        let Look::Creature(path) = &unit.look else { return };
        let Some(name) = data::sprite(job) else { return };
        let folder = path.split('/').next().unwrap_or("몬스터").to_owned();
        unit.look = Look::Creature(format!("{folder}/{name}"));
        unit.motion = None;
        unit.shown = None;
    });
}

/// The server tells what a monster or an NPC is called (after it became another kind, for one).
pub fn unit_named(id: u32, name: &str) {
    STATE.with_borrow_mut(|state| {
        if let Some(unit) = state.scene.as_mut().and_then(|scene| scene.units.get_mut(&id)).filter(|unit| matches!(unit.look, Look::Creature(_))) {
            unit.name = name.split('#').next().unwrap_or_default().trim().to_owned();
        }
    });
}

/// Someone's look changes (ZC_SPRITE_CHANGE): `None` is the player's own character. The kinds are the server's:
/// 1 the hair, 2 the weapon, 3 to 5 what is worn on the head (low, top, middle), 6 the hair's colour, 7 the
/// clothes' colour.
pub fn looked(id: Option<u32>, kind: u8, value: u32) {
    STATE.with_borrow_mut(|state| {
        if id.is_none() && kind == 2 {
            state.weapon = value;
        }
        let Some(scene) = &mut state.scene else { return };
        let mover = match id {
            None => Some(&mut scene.player),
            Some(id) => scene.units.get_mut(&id),
        };
        let Some(Look::Character(figure)) = mover.map(|mover| &mut mover.look) else { return };
        match kind {
            1 => figure.hair_style = value as u16,
            2 => figure.weapon = value,
            3 => figure.headgear[0] = value as u16,
            4 => figure.headgear[1] = value as u16,
            5 => figure.headgear[2] = value as u16,
            6 => figure.hair_colour = value as u16,
            7 => figure.body_palette = value as u16,
            _ => {}
        }
    });
}

/// Someone says something: it shows over their head for a while. `own` is the player's own ID.
pub fn said(id: u32, own: u32, words: &str) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        if let Some(mover) = if id == own { Some(&mut scene.player) } else { scene.units.get_mut(&id) } {
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
        if let Some(mover) = if id == own { Some(&mut scene.player) } else { scene.units.get_mut(&id) } {
            mover.emotion = Some((sprite, action, now()));
        }
    });
}

/// Someone sits down or stands up. `own` is the player's own ID.
pub fn sat(id: u32, own: u32, down: bool) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        if let Some(mover) = if id == own { Some(&mut scene.player) } else { scene.units.get_mut(&id) } {
            mover.sitting = down;
        }
    });
}

/// Someone is out of sight, or gone.
pub fn unit_gone(id: u32, died: bool) {
    STATE.with_borrow_mut(|state| {
        state.early.retain(|early| early.id != id);
        if let Some(scene) = &mut state.scene {
            // A monster struck down is seen falling before it is gone.
            match scene.units.get_mut(&id).filter(|unit| died && unit.lively) {
                Some(unit) => {
                    let time = now();
                    unit.dying = Some(time);
                    unit.walk = None;
                    unit.life = None;
                    unit.motion = Some((32, time + 250.0, 700.0));
                }
                None => {
                    scene.units.remove(&id);
                }
            }
            scene.portals.remove(&id);
        }
    });
}

/// The server says someone stands on a cell.
pub fn unit_stand(id: u32, cell: (u16, u16)) {
    STATE.with_borrow_mut(|state| {
        if let Some(unit) = state.scene.as_mut().and_then(|scene| scene.units.get_mut(&id)) {
            unit.walk = None;
            unit.cell = (i32::from(cell.0), i32::from(cell.1));
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
