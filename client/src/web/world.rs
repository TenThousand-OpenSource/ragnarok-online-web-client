//! The game world on the canvas: the map's ground in 3D, the player's character on it, the camera, and walking.
//!
//! The camera is the original client's: a 15 degree field of view, 300 units from the character, 45 degrees up,
//! looking north (values as rustnarok-client measured them). The right mouse button turns it, with Shift tilts it,
//! with Ctrl (or the wheel) moves it nearer and farther; a double right click puts it back.
//!
//! The map's models (buildings, trees) are lit by the map's sun as the client shades each (`crate::model`); the
//! parts of those that have an animation turn, and the water has its waves (`land.rs`). Around them are the
//! effects and sounds the map places, its fog, its sky and its weather (`scenery.rs`).
//!
//! Everyone else on the map (players, NPCs, monsters) stands and walks as the server tells; pointing at one
//! shows its name, pressing one talks to it or fights it (`input.rs`, `pursuit.rs`, `combat.rs`).
//!
//! This file has what the children share and the list of them: the types live with the child that owns them
//! (`Scene` in `scene.rs`, `State` in `state.rs`, `Camera` in `camera.rs`, `Mover` in `mover.rs`, `Gpu` in
//! `gpu.rs`, `Options` and `Hooks` in `hooks.rs`).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent, WheelEvent};
use wgpu::util::DeviceExt;

use super::figure::{self, Figure, SpriteKey};
use super::{data, fetch_asset, image_url, js_error, sound};
use crate::bmp;
use crate::map::{CELL, Gat, Gnd};
use crate::model::{self, Rsm, Rsw};
use crate::net::actors::{Kind, Unit, Walked};
use crate::route::Route;
use crate::server_time::Clock;

mod shaders;
mod math;
mod camera;
mod mover;
mod gpu;
mod loading;
mod land;
mod scenery;
mod effects;
mod prims;
mod quads;
mod state;
mod scene;
mod hooks;
mod session;
mod actors;
mod pose;
mod ai;
mod ailments;
mod aura;
mod players;
mod quests;
mod cart;
mod falcon;
mod pvp;
mod guild;
mod granny;
mod combat;
mod earshot;
mod names;
mod written;
mod launch;
mod state_effects;
mod scope;
mod skill_units;
mod signs;
mod scripted;
mod spirits;
mod frame;
mod overlays;
mod pursuit;
mod pick;
mod input;

use shaders::*;
use math::*;
use camera::{Camera, DISTANCE, NEAR, View};
use mover::*;
use gpu::{EffectSprite, Gpu, Ground, canvas};
use loading::*;
use land::{Models, WaterSurface};
pub use scenery::fog;
use effects::*;
use quads::*;
use state::*;
use scene::*;
pub use hooks::{Hooks, Options};
pub use session::*;
pub use actors::*;
pub use pose::{puffed, risen, turned};
pub use ai::{companions, sight, unit_job, walkable};
pub use ailments::ailing;
pub use players::*;
pub use quests::quest_mark;
pub use cart::carted;
pub use falcon::{Prey, falcon_sent};
pub use quests::visit;
pub use pvp::{sides, zone};
pub use guild::{emblem_known, emblem_pixels, guild_changed, guild_gone, own_guild, player_card, player_guild, player_position};
pub use combat::*;
pub use written::spot_says;
pub use names::{own_level, unit_titled};
pub use launch::*;
pub use state_effects::{own_states_again, state_effects, third_option_told};
pub use scope::{area_warned, scope_shown};
pub use skill_units::{cell_retyped, ground_skill, skill_unit_becomes, skill_unit_gone, skill_unit_seen, skill_unit_shown, skill_unit_stands, skill_unit_strikes, skill_unit_used};
pub use signs::{Board, SIGN, sign};
pub use scripted::{gauge, sounded};
pub use spirits::{spheres_gone, spheres_seen, spheres_told};
use frame::*;
use overlays::*;
use pursuit::*;
use pick::*;
use input::*;

/// World units per sprite pixel: about 35 sprite pixels to a cell.
const SPRITE_SCALE: f32 = CELL / 35.0;

/// A frame of the client's coded effects (the rings of a cast last the cast's milliseconds over 16).
const CAST_FRAME_MS: f64 = 16.0;
/// One of the client's time units for effects: 24 milliseconds (`CRenderObject::GetElapsedFrames`).
const EFFECT_FRAME_MS: f64 = 24.0;

use crate::blow::flight::Flight;
use crate::vector::Vector;

/// A picture standing in the world this frame: which, where its feet are, how much of it is left to see (1
/// all of it), and how it is drawn (`crate::body::Quad`: its colour, its size, where its edges are; `PLAIN` as it
/// is). Pictures at one place are drawn in the order they were given.
type Stood = (SpriteKey, Vector, f32, crate::body::Quad);

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

use super::now;
