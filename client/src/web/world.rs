//! The game world on the canvas: the map's ground in 3D, the player's character on it, the camera, and walking.
//!
//! The camera is the original client's: a 15 degree field of view, 300 units from the character, 45 degrees up,
//! looking north (values as rustnarok-client measured them). The right mouse button turns it, with Shift tilts it,
//! with Ctrl (or the wheel) moves it nearer and farther; a double right click puts it back.
//!
//! The map's models (buildings, trees) stand still, each face lit flatly by the map's sun, and the water is a flat
//! surface: no animated models, smoothed shading or waves yet.
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
use crate::map::{CELL, Gat, Gnd, Walk};
use crate::model::{self, Rsm, Rsw};
use crate::net::units::{Kind, Unit};

mod shaders;
mod math;
mod camera;
mod mover;
mod gpu;
mod loading;
mod effects;
mod session;
mod actors;
mod combat;
mod frame;
mod overlays;
mod pursuit;
mod pick;
mod input;

use shaders::*;
use math::*;
use mover::*;
use loading::*;
use effects::*;
pub use session::*;
pub use actors::*;
pub use combat::*;
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
/// How long one picture of a standing monster shows.
const IDLE_FRAME_MS: f64 = 150.0;
/// The sprite numbers of things that are there without being seen: portals' touch areas and invisible NPCs.
const UNSEEN: [u16; 3] = [111, 139, 32767];
/// The sprite number of a portal: drawn as rings of light, not as a sprite.
const PORTAL: u16 = 45;
/// The game's pointers, as actions of its pointer sprite: the plain one, over someone to talk to, over a monster,
/// over a portal, over a thing to pick up, while the camera is turned, while a skill is aimed, and while a skill
/// is aimed at someone it can be used on.
const POINTERS: [usize; 8] = [0, 1, 5, 7, 9, 4, 10, 11];
/// The walking speed until the server says otherwise, in milliseconds per cell.
const DEFAULT_SPEED: f64 = 150.0;
/// How long one picture of the walking animation shows at the default speed.
const WALK_FRAME_MS: f64 = 100.0;
const WALK_FRAMES: usize = 8;
/// The first of the walking actions; standing ones start at 0. Each has eight directions.
const WALKING: usize = 8;

/// How many pictures the water runs through, and how many world units one of them covers.
const WATER_FRAMES: usize = 32;
const WATER_TILE: f32 = 50.0;

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
}

/// The pictures of the effects drawn by code: the rings around a caster and the mark on the one aimed at.
const CAST_PICTURES: [&str; 14] = [
    "pikapika2.bmp",
    "ring_blue.tga",
    "ring_yellow.tga",
    "ring_white.tga",
    "magic_green.tga",
    "magic_violet.tga",
    "lockon128.tga",
    "icearrow.tga",
    "불화살1.tga",
    "불화살2.tga",
    "불화살3.tga",
    "불화살4.tga",
    "불화살5.tga",
    "불화살6.tga",
];
/// The pictures a bolt of fire runs through as it falls.
const FIRE_BOLT: [&str; 6] = ["불화살1.tga", "불화살2.tga", "불화살3.tga", "불화살4.tga", "불화살5.tga", "불화살6.tga"];

/// A bolt of fire or of ice falling on someone (the client's `FireArrow` and `IceArrow`): when it sets off, on
/// whom, from where beside and above them, and whether it is fire.
struct Bolt {
    leaves: f64,
    target: u32,
    from: Vector,
    fire: bool,
}
/// A frame of the client's coded effects (the rings of a cast last the cast's milliseconds over 16).
const CAST_FRAME_MS: f64 = 16.0;
/// The arrow's sprite, and how long it flies: 8 of the client's frames of 24 milliseconds (`CArrowEffect`).
const ARROW: &str = "몬스터/skel_archer_arrow";
const ARROW_MS: f64 = 192.0;

/// One of the client's time units for effects: 24 milliseconds (`CRenderObject::GetElapsedFrames`).
const EFFECT_FRAME_MS: f64 = 24.0;

/// A number (or "miss") flying off someone struck, as the original's `CMsgEffect` flies it.
struct Flying {
    began: f64,
    /// Where the one struck stood, on the ground.
    base: Vector,
    /// Which way it drifts, east and north.
    drift: (f32, f32),
    /// Its pictures, left to right: the digits, or the one word.
    pictures: Vec<SpriteKey>,
    miss: bool,
}

type Vector = [f32; 3];

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

/// Someone on the map: the player's own character, another player, an NPC or a monster.
struct Mover {
    look: Look,
    name: String,
    /// Whether it moves while standing still, as monsters do.
    lively: bool,
    /// A pet (or a homunculus, a mercenary): someone's companion, neither fought nor talked to.
    companion: bool,
    cell: (i32, i32),
    /// Where it was drawn last, in cells.
    at: (f32, f32),
    facing: u8,
    /// The walk under way, and when it started (milliseconds, as `now` counts).
    walk: Option<(Walk, f64)>,
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
    /// A skill being cast: when it began, how long it takes, which of the client's effects shows around the
    /// caster, and on whom it is aimed.
    casting: Option<(f64, f64, u32, Option<u32>)>,
}

/// A sprite's picture on the GPU.
struct SpriteTexture {
    bind: wgpu::BindGroup,
    size: (f32, f32),
    feet: (f32, f32),
}

struct Ground {
    vertices: wgpu::Buffer,
    /// Per texture: its triangles, how many indices they are, and the texture with the baked light.
    batches: Vec<(wgpu::Buffer, u32, wgpu::BindGroup)>,
}

/// The map's models: per texture, the corners of every triangle with it, how many, and the texture.
type Models = Vec<(wgpu::Buffer, u32, wgpu::BindGroup)>;

struct WaterSurface {
    vertices: wgpu::Buffer,
    /// The pictures the surface runs through.
    frames: Vec<wgpu::BindGroup>,
    /// How long each picture shows, in milliseconds.
    frame_ms: f64,
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
    model_camera_bind: wgpu::BindGroup,
    water_camera_bind: wgpu::BindGroup,
    ground_pipeline: wgpu::RenderPipeline,
    model_pipeline: wgpu::RenderPipeline,
    water_pipeline: wgpu::RenderPipeline,
    sprite_pipeline: wgpu::RenderPipeline,
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
    /// The arrow in flight: its picture and that picture's size.
    arrow: Option<(Rc<[wgpu::BindGroup; 2]>, (f32, f32))>,
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
    /// The things lying on the ground, by the server's ID: the cell, and which item.
    drops: HashMap<u32, ((i32, i32), u32, (f32, f32), bool)>,
    /// When each thing that fell out of a monster began to fall.
    falls: HashMap<u32, f64>,
    /// Where the camera looks, and when it was last moved: it follows the character a tenth of the way each of
    /// the client's frames (`CameraState::Smooth`), so a step the server corrects does not jolt the view. None
    /// until the first frame, and after the character is put somewhere else: then it looks straight at it.
    aim: Option<(Vector, f64)>,
    /// The portals in sight, by the server's ID: the cells they are on.
    portals: HashMap<u32, (i32, i32)>,
    camera: Camera,
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
}

impl Default for Options {
    fn default() -> Self {
        Self { no_ctrl: true, quick_right: false, quick_wheel: false, effects: true, misses: true, monster_life: true, hide_players: false, zoom_out: true }
    }
}

pub struct Hooks {
    /// The player pressed a cell to walk to.
    pub walk_to: fn(u16, u16),
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
}

#[derive(Default)]
struct State {
    gpu: Option<Gpu>,
    scene: Option<Scene>,
    hooks: Option<Hooks>,
    /// Counts the maps entered: a map that finishes loading after the next was asked for is dropped.
    entered: u32,
    /// The right button is held: where the pointer was last.
    turning: Option<(f32, f32)>,
    /// When the right button was last pressed: twice in a moment puts the camera back.
    right_pressed: f64,
    /// The player's options the world goes by.
    options: Options,
    /// The skill being aimed is one of help, which goes on a monster only with Shift held (unless the client's
    /// No Shift option is on).
    aim_guarded: bool,
    /// How the monster being fought is fought: 0 until it is down, 1 while the left button is held, 2 with one
    /// blow as soon as it is near enough (the button was let go before any blow was asked for).
    fight: u8,
    /// What is written by the pointer while a skill is aimed: the skill and the level it will be used at.
    aim_words: String,
    /// The camera stays on the character without following it gradually (the client's `/camera` option).
    camera_fixed: bool,
    /// Where the right button went down, until it comes up.
    right_at: Option<(f32, f32)>,
    listening: bool,
    /// Those the server showed while the map was still being read: put on it once it is there.
    early: Vec<Unit>,
    /// The map being read, until it shows; and where the character is to stand on it when it does, if the
    /// server named another place meanwhile.
    reading: Option<String>,
    start: Option<(i32, i32)>,
    /// Where on the page the pointer is, while it is over the world with no button held.
    pointer: Option<(f32, f32)>,
    /// Who the pointer is on: their ID, and whether it is a monster.
    over: Option<(u32, bool)>,
    /// The left button is held on the ground: where the pointer is, and when a walk was last asked for.
    held: Option<(f32, f32)>,
    held_asked: f64,
    /// The monster being fought: its ID, when it was last looked after, and whether the server was told to strike
    /// since it came near.
    chase: Option<(u32, f64, bool)>,
    /// When the player's own character last struck, and when the server was last asked to strike: a fight that
    /// does not start is asked for again.
    own_struck: f64,
    /// Until when the player's own character is in the motion of a blow it deals: it does not walk meanwhile.
    own_striking_until: f64,
    /// Until when the player's own character cannot walk for a blow it took (the server's walk delay).
    own_stopped_until: f64,
    strike_asked: f64,
    /// How near the character must stand to strike with what it holds, in cells, once the server has said so
    /// (it says when a blow was asked for from too far).
    strike_range: Option<i32>,
    /// The thing on the ground the character is walking to, to pick it up, and when it was last looked after.
    fetch: Option<(u32, f64)>,
    /// The flashes of blows that landed: where, and when.
    sparks: Vec<(Vector, f64)>,
    /// The glitter of First Aid playing: on whom (none: the player's own character), since when, and the two
    /// angles it starts at (how it is turned, and where its slow pulse is), which the client draws by lot.
    aids: Vec<(Option<u32>, f64, f32, f32)>,
    /// The weapon the player's own character holds (kept across maps).
    weapon: u32,
    /// The player's own life and spirit, left and most: the bars under the character.
    own: [i64; 4],
    /// Whether life bars are on the page.
    bars: bool,
    /// Blows on their way: when each lands, on whom, the player's own ID, how much, and what it sounds like.
    landing: Vec<(f64, u32, u32, i32, Option<String>)>,
    /// Where the pointer in the world stands this frame (see the frame's code).
    target_at: Option<Vector>,
    /// The numbers in the air.
    flying: Vec<Flying>,
    /// Bolts falling, and the effects and sounds that are to start at a moment yet to come.
    bolts: Vec<Bolt>,
    effects_due: Vec<(f64, String, Option<u32>)>,
    sounds_due: Vec<(f64, String)>,
    /// Arrows in flight: when each leaves, from where, and at whom.
    arrows: Vec<(f64, Vector, u32)>,
    /// The effects playing, and the effect files read so far, by name.
    effects: Vec<Playing>,
    effect_files: HashMap<String, Rc<Shown>>,
    /// Hits to show as numbers: on whom, and how much.
    hits: Vec<(u32, i32)>,
    /// A skill is being aimed: at a place (`true`) or at someone.
    aiming: Option<bool>,
    /// How far the skill being aimed reaches, in cells.
    aim_range: f32,
    /// The skill was aimed at someone or somewhere out of its reach: the character walks there, and the skill is
    /// used once it is near enough. On whom (or else at which cell), the skill's reach, and when a walk was last
    /// asked for.
    reach: Option<(Option<u32>, (i32, i32), f32, f64)>,
    /// Which pointer and whose name the page shows now.
    pointing: usize,
    named: Option<u32>,
    /// The game's pointers as CSS, once read (see `POINTERS`): the pictures of each, in turn, and how long each
    /// picture shows.
    pointers: Vec<(Vec<String>, f64)>,
    /// Which picture of the pointer the page shows now.
    pointer_frame: usize,
    frame: Option<Closure<dyn FnMut()>>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn now() -> f64 {
    web_sys::window().and_then(|window| window.performance()).map_or(0.0, |performance| performance.now())
}

fn canvas() -> Result<HtmlCanvasElement, String> {
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("game"))
        .ok_or("no #game canvas")?
        .dyn_into()
        .map_err(js_error)
}

/// The canvas's size in real pixels, for a sharp picture on dense screens.
fn pixel_size(canvas: &HtmlCanvasElement) -> (u32, u32) {
    let ratio = web_sys::window().map_or(1.0, |window| window.device_pixel_ratio());
    (((f64::from(canvas.client_width()) * ratio) as u32).max(1), ((f64::from(canvas.client_height()) * ratio) as u32).max(1))
}

