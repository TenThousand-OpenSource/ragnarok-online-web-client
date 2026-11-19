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
//! shows its name. Talking to them and fighting them is not made yet.

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
use crate::net::units::{Kind, Unit, Walked};
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
mod session;
mod actors;
mod poses;
mod ai;
mod ailments;
mod players;
mod quests;
mod cart;
mod pvp;
mod guild;
mod combat;
mod earshot;
mod names;
mod written;
mod launch;
mod state_effects;
mod scopes;
mod skill_units;
mod signs;
mod scripted;
mod frame;
mod overlays;
mod pursuit;
mod pick;
mod input;

use shaders::*;
use math::*;
use mover::*;
use loading::*;
use land::{Models, WaterSurface};
pub use scenery::fog;
use effects::*;
use quads::*;
use state::*;
pub use session::*;
pub use actors::*;
pub use poses::{risen, turned};
pub use ai::{companions, sight, unit_job, walkable};
pub use ailments::ailing;
pub use players::*;
pub use quests::quest_mark;
pub use cart::carted;
pub use quests::visit;
pub use pvp::{sides, zone};
pub use guild::{emblem_known, guild_changed, own_guild, player_card, player_guild, player_position};
pub use combat::*;
pub use written::spot_says;
pub use names::{own_level, unit_titled};
pub use launch::*;
pub use state_effects::{own_states_again, state_effects, third_option_told};
pub use scopes::scope_shown;
pub use skill_units::{cell_retyped, ground_skill, skill_unit_becomes, skill_unit_gone, skill_unit_shown};
pub use signs::{Board, SIGN, sign};
pub use scripted::{gauge, sounded};
use frame::*;
use overlays::*;
use pursuit::*;
use pick::*;
use input::*;

const FIELD_OF_VIEW: f32 = 15.0;
const NEAR: f32 = 10.0;
const FAR: f32 = 5000.0;
const DISTANCE: f32 = 300.0;
const DISTANCE_RANGE: [f32; 2] = [150.0, 730.0];
/// How far the camera goes outdoors without the client's Zoom Out option (with it: `DISTANCE_RANGE`'s end).
const DISTANCE_NEAR_MOST: f32 = 400.0;
const PITCH: f32 = 45.0;
const PITCH_RANGE: [f32; 2] = [20.0, 70.0];
/// World units per sprite pixel: about 35 sprite pixels to a cell.
const SPRITE_SCALE: f32 = CELL / 35.0;
/// The sprite numbers of things that are there without being seen: portals' touch areas and invisible NPCs.
const UNSEEN: [u16; 3] = [111, 139, 32767];
/// The sprite number of a portal: drawn as rings of light, not as a sprite.
const PORTAL: u16 = 45;
/// The game's pointers, as actions of its pointer sprite: the plain one, over someone to talk to, over a monster,
/// over a portal, over a thing to pick up, while the camera is turned, while a skill is aimed, while a skill
/// is aimed at someone it can be used on, and over ground nobody can walk on.
const POINTERS: [usize; 9] = [0, 1, 5, 7, 9, 4, 10, 11, 13];
/// The walking speed until the server says otherwise, in milliseconds per cell.
const DEFAULT_SPEED: f64 = 150.0;
/// How long one picture of the walking animation shows at the default speed.
const WALK_FRAME_MS: f64 = 100.0;
const WALK_FRAMES: usize = 8;

/// How many pictures the water runs through, and how many world units one of them covers.
const WATER_FRAMES: usize = 32;

/// An effect file ready to play: its layers, and each layer's pictures on the GPU (for the see-through pipeline
/// and for the one that adds light).
struct Shown {
    effect: crate::effect::Effect,
    pictures: Vec<Vec<Option<Rc<[wgpu::BindGroup; 2]>>>>,
}

/// An effect playing on someone (`None`: the player's own character) since a moment.
struct Playing {
    shown: Rc<Shown>,
    on: Option<u32>,
    began: f64,
    /// How it is carried, for a file a class of the client's own moves, turns or begins late; none for a file
    /// that plays over the one it is on.
    carried: Option<Carrying>,
}

/// How a playing file is carried (`crate::effect::carried`), whose place it was told besides that of the one it
/// is on (none told: nobody; no one named: the player's own character), and what it measured as it began, as
/// the client's effect does in its `Open`: where the two stood and the angle it is turned by.
struct Carrying {
    how: crate::effect::carried::Carried,
    other: Option<Option<u32>>,
    set: Option<(Vector, Vector, f32)>,
}

/// A frame of the client's coded effects (the rings of a cast last the cast's milliseconds over 16).
const CAST_FRAME_MS: f64 = 16.0;
/// The arrow's sprite, and how long it flies: 8 of the client's frames of 24 milliseconds (`CArrowEffect`).
const ARROW: &str = "몬스터/skel_archer_arrow";
const ARROW_MS: f64 = 192.0;

/// One of the client's time units for effects: 24 milliseconds (`CRenderObject::GetElapsedFrames`).
const EFFECT_FRAME_MS: f64 = 24.0;

/// A number or a word of a fight flying over someone, as the original's `CMsgEffect` flies it.
struct Flying {
    began: f64,
    /// Whom it is over (none named: the player's own character), and where that one stood on the ground when
    /// last seen: it stays over its owner while that one is in sight.
    on: Option<u32>,
    base: Vector,
    /// Which way it drifts, east and north.
    drift: (f32, f32),
    /// Its pictures, left to right: the digits, or the one word.
    pictures: Vec<SpriteKey>,
    flight: Flight,
}

use crate::blow::flight::Flight;

type Vector = [f32; 3];

/// A picture standing in the world this frame: which, where its feet are, how much of it is left to see (1
/// all of it), and how it is drawn (`crate::body::Quad`: its colour, its size, where its edges are; `PLAIN` as it
/// is). Pictures at one place are drawn in the order they were given.
type Stood = (SpriteKey, Vector, f32, crate::body::Quad);

/// The camera: where it looks, and from where.
#[derive(Clone, Copy)]
struct Camera {
    /// Degrees; 0 looks north, growing clockwise.
    yaw: f32,
    /// Degrees above the horizon.
    pitch: f32,
    distance: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self { yaw: 0.0, pitch: PITCH, distance: DISTANCE }
    }
}

/// Under which name the browser keeps the camera.
const CAMERA_KEPT: &str = "roweb.camera";

/// The camera at one moment: its place and axes, and the matrix that takes the world to the screen.
struct View {
    eye: Vector,
    forward: Vector,
    right: Vector,
    up: Vector,
    aspect: f32,
    /// Column by column, for the shaders.
    matrix: [f32; 16],
}

/// How someone on the map looks: a character put together from its parts, or a monster's or NPC's own sprite
/// (its path under `data/sprite/`).
#[derive(Clone)]
enum Look {
    Character(Figure),
    Creature(String),
}

/// A thing lying on the ground: its cell and where in the cell it lies (in cells), which item it is, whether it
/// is identified, and how many there are.
struct Lying {
    cell: (i32, i32),
    item: u32,
    at: (f32, f32),
    identified: bool,
    count: u16,
}

/// Someone on the map: the player's own character, another player, an NPC or a monster.
struct Mover {
    look: Look,
    /// A character's own job and the bits of the state it is in: the job its look is drawn as follows from the
    /// two (`crate::mounts`).
    job: u16,
    state: u32,
    name: String,
    /// Whether it moves while standing still, as monsters do.
    lively: bool,
    /// A pet (or a homunculus, a mercenary): someone's companion, neither fought nor talked to.
    companion: bool,
    /// A player's guild (0 for none) and the version of its emblem, as the server showed them.
    guild: (u32, u32),
    cell: (i32, i32),
    /// Where it is drawn, in cells: a walker along its way, one who stands where it stopped (the middle of
    /// its cell, or short of it when a blow stopped it).
    at: (f32, f32),
    facing: u8,
    /// The walk under way: its cells and the server's time at which each is reached (`crate::route`).
    walk: Option<Route>,
    /// When it began to walk from standing (milliseconds, as `now` counts): its legs go from then.
    strode: f64,
    speed: f64,
    /// The picture last drawn, kept on screen until the next one is ready.
    shown: Option<SpriteKey>,
    /// Sitting on the ground.
    sitting: bool,
    /// Lying dead (the player's own character, until it returns to its save point).
    dead: bool,
    /// A monster struck down: when its fall is over. From then it lies there and fades.
    dying: Option<f64>,
    /// What it said last, and when: shown over its head for a while.
    says: Option<(String, f64)>,
    /// The emotion it shows: the sprite it is in, which of its actions, and when it began.
    emotion: Option<(String, usize, f64)>,
    /// What is left of its life and how much it has, once the server has said (monsters being fought).
    life: Option<(i32, i32)>,
    /// The action and frame last drawn: a frame's sound plays when the frame comes, not while it stays.
    heard: Option<(usize, usize)>,
    /// A motion under way while standing (a blow, a flinch): its first action, when it began, how long it lasts.
    motion: Option<(usize, f64, f64)>,
    /// What its body state and health state make of it: its colour, the marks by it (`ailments.rs`).
    ail: ailments::Ail,
    /// A skill being cast: when it began, how long it takes, which of the client's effects shows around the
    /// caster, and on whom it is aimed.
    casting: Option<(f64, f64, u32, Option<u32>)>,
    /// How it is posed beyond that, and what else of it is drawn (`mover.rs`).
    pose: mover::Pose,
}

/// A frame of a sprite as one picture on the GPU for the effects' pipelines: the picture for each of the two, and
/// its size.
#[derive(Clone)]
struct EffectSprite {
    binds: Rc<[wgpu::BindGroup; 2]>,
    size: (f32, f32),
}

/// A sprite's picture on the GPU: for the see-through pipeline, and for the one that adds its light to what is
/// behind it.
struct SpriteTexture {
    bind: wgpu::BindGroup,
    adding: wgpu::BindGroup,
    size: (f32, f32),
    feet: (f32, f32),
}

struct Ground {
    vertices: wgpu::Buffer,
    /// Per texture: its triangles, how many indices they are, and the texture with the baked light.
    batches: Vec<(wgpu::Buffer, u32, wgpu::BindGroup)>,
}

struct Gpu {
    canvas: HtmlCanvasElement,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth: wgpu::TextureView,
    camera_buffer: wgpu::Buffer,
    camera_bind: wgpu::BindGroup,
    sprite_camera_bind: wgpu::BindGroup,
    model_camera_binds: [wgpu::BindGroup; 2],
    water_camera_bind: wgpu::BindGroup,
    ground_pipeline: wgpu::RenderPipeline,
    /// The models' pipelines: for pictures cut out by their colour, and for those mixed in by their alpha.
    model_pipelines: [wgpu::RenderPipeline; 2],
    water_pipeline: wgpu::RenderPipeline,
    sprite_pipeline: wgpu::RenderPipeline,
    /// The sprites' pipeline that adds a picture's light to what is behind it (a body an effect has lit:
    /// `crate::body`), and the camera for it.
    sprite_adding_pipeline: wgpu::RenderPipeline,
    sprite_adding_camera_bind: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    /// For pictures laid side by side again and again: model textures and the water.
    repeating: wgpu::Sampler,
    /// Sprites are pixel art: shown pixel by pixel, not smoothed.
    pixel_sampler: wgpu::Sampler,
    /// The pictures drawn so far on this map.
    sprite_textures: HashMap<SpriteKey, SpriteTexture>,
    /// The corners of this frame's sprites, and how many sprites the buffer holds.
    sprite_vertices: wgpu::Buffer,
    sprite_room: usize,
    /// The picture of a portal's rings.
    ring: Option<wgpu::BindGroup>,
    /// The mark on the ground cell the pointer is over: green corners, as the original's.
    mark: Option<wgpu::BindGroup>,
    /// For the effects: the see-through pipeline and the one that adds light, the camera for each, and the
    /// corners of this frame's quads.
    effect_pipelines: [wgpu::RenderPipeline; 2],
    effect_camera_binds: [wgpu::BindGroup; 2],
    effect_vertices: wgpu::Buffer,
    effect_room: usize,
    /// The pictures of the effects drawn by code (the rings of a cast, the mark on its target, an arrow), by name.
    effect_pictures: HashMap<&'static str, Rc<[wgpu::BindGroup; 2]>>,
    /// The sprites' pictures the effects' pipelines draw, a frame to a picture (an arrow in flight), by picture.
    effect_sprites: HashMap<SpriteKey, EffectSprite>,
    /// The pictures of the sprites of the effects made of the client's primitives, each by itself: by the
    /// sprite's path, then by whether the picture is a truecolor one and its number (`gpu/layers.rs`).
    effect_layers: HashMap<String, HashMap<(bool, usize), Rc<[wgpu::BindGroup; 2]>>>,
}

struct Scene {
    /// Inside a building the camera is the original's: turned 45 degrees and not turning, never far away.
    indoor: bool,
    gat: Gat,
    ground: Ground,
    models: Models,
    water: Option<WaterSurface>,
    player: Mover,
    /// Everyone else in sight, by the server's ID.
    units: HashMap<u32, Mover>,
    /// The things lying on the ground, by the server's ID.
    drops: HashMap<u32, Lying>,
    /// When each thing that fell out of a monster began to fall.
    falls: HashMap<u32, f64>,
    /// Where the camera looks, and when it was last moved: it follows the character a tenth of the way each of
    /// the client's frames (`CameraState::Smooth`), so a step the server corrects does not jolt the view. None
    /// until the first frame, and after the character is put somewhere else: then it looks straight at it.
    aim: Option<(Vector, f64)>,
    /// Where the units of skills on the ground and the places skills were used on are, by ID, in cells
    /// (`skill_units.rs`): effects play on them as on someone.
    spots: HashMap<u32, (f32, f32)>,
    /// The portals in sight, by the server's ID: the cells they are on.
    portals: HashMap<u32, (i32, i32)>,
    camera: Camera,
    /// The camera as the player asked for it, and when `camera` last went towards it: a tenth of the way
    /// each of the client's frames (`CameraState::Smooth`, `camera.rs`).
    wanted: (Camera, f64),
    /// What surrounds the player here: the effects and sounds the map places, its fog, sky and weather
    /// (`scenery.rs`).
    scenery: scenery::Surroundings,
    /// The last cell told to the page, so it is told only when it changes.
    told: (i32, i32),
}

/// What the page is told, and what it is asked for.
/// The options of the client's Game Settings window the world goes by.
#[derive(Clone, Copy)]
pub struct Options {
    /// A press on a monster fights it until it is down (`/noctrl`); off, that takes Ctrl, and a plain press
    /// fights only while the button is held.
    pub no_ctrl: bool,
    /// The right button uses the bar's ninth slot (`/q1`); the wheel its eighth and seventh (`/q2`).
    pub quick_right: bool,
    pub quick_wheel: bool,
    /// Effects of skills and items are drawn (`/effect`); a miss is shown (`/miss`); a monster being fought
    /// shows its life (`/monsterhp`).
    pub effects: bool,
    pub misses: bool,
    pub monster_life: bool,
    /// Other players are not drawn.
    pub hide_players: bool,
    /// The camera may go out to 730 instead of 400 (`/zoom`).
    pub zoom_out: bool,
    /// This project's own, off as in the client: a press farther than the server walks in one go sends the
    /// character there in stretches instead of doing nothing (`pursuit::journey`).
    pub far_walk: bool,
    /// The screen shakes when an effect asks for it (`/quake`: the client's `MM_QUAKE` does nothing with it off).
    pub quake: bool,
    /// The name of a character the pointer is on stands over its head, not under its feet (`/font`).
    pub name_above: bool,
    /// The emblem of a player's guild is drawn by the name (`/emblem`).
    pub emblems: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { no_ctrl: true, quick_right: false, quick_wheel: false, effects: true, misses: true, monster_life: true, hide_players: false, zoom_out: true, far_walk: false, quake: true, name_above: false, emblems: true }
    }
}

pub struct Hooks {
    /// The player pressed a cell to walk to.
    pub walk_to: fn(u16, u16),
    /// The character turned, or turned its head, towards a place pressed: how the head is turned and the
    /// direction, as the server is told them.
    pub face: fn(u8, u8),
    /// The character stands on, or passes, another cell.
    pub moved: fn(u16, u16),
    /// The player pressed someone: their ID, and whether it is a monster.
    pub press: fn(u32, bool),
    /// The character stands by a thing on the ground the player pressed: pick it up.
    pub take: fn(u32),
    /// The map is drawn (or could not be: then with the reason).
    pub ready: fn(Result<(), String>),
    /// The skill being aimed was pointed at someone (their ID) or at a cell.
    pub aimed: fn(Option<u32>, (u16, u16)),
    /// The right button was pressed and let go on someone without moving: their ID, and where on the page.
    pub menu: fn(u32, (f32, f32)),
    /// The wheel was turned while a skill is aimed: up (1) or down (-1) a level.
    pub aim_level: fn(i32),
    /// One blow at a monster, not a fight until it is down (the client without Ctrl, when its No Ctrl option
    /// is off).
    pub strike_once: fn(u32),
    /// The fight is called off: the server is told to stop striking.
    pub stop: fn(),
    /// A slot of the hotkey bar is used by the mouse (the client's Quick Skill options).
    pub quick: fn(usize),
    /// The mouse commands the homunculus, or the mercenary (true): to fight someone (their ID) or to walk to
    /// a cell, at once or, with Shift (true), after its other commands (`ai.rs`).
    pub commanded: fn(bool, Option<u32>, (u16, u16), bool),
}


thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn now() -> f64 {
    web_sys::window().and_then(|window| window.performance()).map_or(0.0, |performance| performance.now())
}

fn canvas() -> Result<HtmlCanvasElement, String> {
    crate::web::by_id("game")
        .ok_or("no #game canvas")?
        .dyn_into()
        .map_err(js_error)
}

/// The canvas's size in real pixels, for a sharp picture on dense screens.
fn pixel_size(canvas: &HtmlCanvasElement) -> (u32, u32) {
    let ratio = web_sys::window().map_or(1.0, |window| window.device_pixel_ratio());
    (((f64::from(canvas.client_width()) * ratio) as u32).max(1), ((f64::from(canvas.client_height()) * ratio) as u32).max(1))
}

