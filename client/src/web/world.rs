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

const FIELD_OF_VIEW: f32 = 15.0;
const NEAR: f32 = 10.0;
const FAR: f32 = 5000.0;
const DISTANCE: f32 = 300.0;
const DISTANCE_RANGE: [f32; 2] = [150.0, 500.0];
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
/// over a portal, over a thing to pick up, while the camera is turned, and while a skill is aimed.
const POINTERS: [usize; 7] = [0, 1, 5, 7, 9, 4, 10];
/// The walking speed until the server says otherwise, in milliseconds per cell.
const DEFAULT_SPEED: f64 = 150.0;
/// How long one picture of the walking animation shows at the default speed.
const WALK_FRAME_MS: f64 = 100.0;
const WALK_FRAMES: usize = 8;
/// The first of the walking actions; standing ones start at 0. Each has eight directions.
const WALKING: usize = 8;

const GROUND_SHADER: &str = "
struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var ground: texture_2d<f32>;
@group(1) @binding(1) var ground_sampler: sampler;
@group(1) @binding(2) var light: texture_2d<f32>;

struct In { @location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) light_uv: vec2<f32>, @location(3) colour: vec3<f32> }
struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) light_uv: vec2<f32>, @location(2) colour: vec3<f32> }

@vertex fn vs(in: In) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.light_uv = in.light_uv;
    out.colour = in.colour;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(ground, ground_sampler, in.uv);
    let baked = textureSample(light, ground_sampler, in.light_uv);
    if (texel.a < 0.5) { discard; }
    // The texture, tinted, darkened by the baked shadow, and lit by the baked light's colour.
    return vec4<f32>(texel.rgb * in.colour * baked.a + baked.rgb, 1.0);
}
";

const MODEL_SHADER: &str = "
struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) colour: vec3<f32> }

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) colour: vec3<f32>) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.colour = colour;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(picture, picture_sampler, in.uv);
    if (texel.a < 0.5) { discard; }
    return vec4<f32>(texel.rgb * in.colour, 1.0);
}
";

/// The water lets the ground under it show through.
const WATER_SHADER: &str = "
struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(picture, picture_sampler, in.uv).rgb, 0.56);
}
";

/// How many pictures the water runs through, and how many world units one of them covers.
const WATER_FRAMES: usize = 32;
const WATER_TILE: f32 = 50.0;

const SPRITE_SHADER: &str = "
struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    return out;
}

// How see-through a quad is rides in its v coordinate: every 2 added to it is a sixteenth less of the picture.
@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let faded = floor(in.uv.y / 2.0);
    var texel = textureSample(picture, picture_sampler, vec2<f32>(in.uv.x, in.uv.y - faded * 2.0));
    texel.a = texel.a * (1.0 - faded / 16.0);
    if (texel.a < 0.02) { discard; }
    return texel;
}
";

/// The client's effects (`.str`): textured quads, each tinted and as see-through as its corners say.
const EFFECT_SHADER: &str = "
struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) colour: vec4<f32> }

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) colour: vec4<f32>) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.colour = colour;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(picture, picture_sampler, in.uv) * in.colour;
    if (texel.a < 0.004) { discard; }
    return texel;
}
";

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
const CAST_PICTURES: [&str; 13] = [
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

/// How the rings around a caster look, by the client's effect (`EF_BEGINSPELL` and on): the picture, its tint,
/// and whether it adds light. Water, fire and wind are the client's own figures (`QuadRingStackSized` with
/// `RenderTeiRect`'s colours); the others are drawn the same way with their own picture.
fn cast_look(effect: u32) -> (&'static str, [f32; 3], bool) {
    match effect {
        54 => ("ring_blue.tga", [195.0 / 255.0, 195.0 / 255.0, 1.0], false),
        55 => ("ring_yellow.tga", [1.0, 175.0 / 255.0, 175.0 / 255.0], true),
        56 => ("magic_green.tga", [1.0, 1.0, 1.0], false),
        57 => ("ring_yellow.tga", [1.0, 1.0, 1.0], false),
        58 => ("ring_white.tga", [1.0, 1.0, 1.0], false),
        59 => ("magic_violet.tga", [1.0, 1.0, 1.0], false),
        _ => ("ring_yellow.tga", [1.0, 175.0 / 255.0, 175.0 / 255.0], true),
    }
}

/// The rings around a caster, `frame` frames into a cast of `frames` (the client's `PP_3DCASTING`: its
/// `UpdateStoneCurse` and `Render3DCasting`). Three arcs turn around the caster, each rising outwards to its own
/// height, inside a tall faint column; all come up over the first quarter turn of a sine and fade over the last
/// 40 frames. Each corner: place, texture coordinates, colour.
fn cast_rings(centre: Vector, frame: f32, frames: f32, tint: [f32; 3], corners: &mut Vec<f32>) -> usize {
    // Of each arc: how far round it goes, how high it rises, where it starts, how far out it stands, how steeply
    // it rises, all in degrees and the client's units.
    const ARCS: [(i32, f32, f32, f32, f32); 4] = [(315, 25.0, 0.0, 4.5, 70.0), (315, 22.0, 90.0, 5.0, 57.0), (315, 19.0, 180.0, 5.5, 45.0), (360, 250.0, 0.0, 4.0, 89.0)];
    let mut quads = 0;
    for (index, (round, tall, start, out, steep)) in ARCS.into_iter().enumerate() {
        let column = index == 3;
        let (most, comes, goes) = if column { (60.0, 8.0, 2.0) } else { (180.0, 10.0, 5.0) };
        let alpha = if frame < frames - 40.0 { (comes * frame).min(most) } else { most - goes * (frame - (frames - 40.0)) };
        if alpha <= 0.0 {
            continue;
        }
        let turned = if column { start } else { start + (index as f32 + 3.0) * frame.floor() };
        let risen = frame.min(90.0).to_radians().sin();
        let (step, steep) = ((round / 20) as f32, steep.to_radians());
        let corner = |order: usize, top: bool| {
            let angle = (turned + order as f32 * step).to_radians();
            let height = if top { tall * (order as f32 * 9.0).to_radians().sin() * risen } else { 0.0 };
            let radius = out + steep.cos() * height;
            [centre[0] + angle.cos() * radius, centre[1] + 0.3 + steep.sin() * height, centre[2] + angle.sin() * radius]
        };
        for order in 1..=20usize {
            let (left, right) = ((order - 1) as f32 / 21.0, order as f32 / 21.0);
            for (order, top, u) in [(order - 1, false, left), (order, false, right), (order - 1, true, left), (order - 1, true, left), (order, false, right), (order, true, right)] {
                corners.extend(corner(order, top));
                corners.extend([u, if top { 0.0 } else { 1.0 }]);
                corners.extend([tint[0], tint[1], tint[2], alpha / 255.0]);
            }
            quads += 1;
        }
    }
    quads
}

/// The mark on the one a cast is aimed at (the client's `LockOnMarker`): two copies of its picture lying on the
/// ground, one an eighth of a turn ahead of the other, turning 4.5 degrees a frame and closing in from 30 across
/// to 10 over the first 20 frames.
fn lock_on(centre: Vector, frame: f32, corners: &mut Vec<f32>) -> usize {
    let half = (30.0 - frame).max(10.0) / 2.0;
    for ahead in [0.0f32, 45.0] {
        let turned = (ahead + 4.5 * frame).to_radians();
        let corner = |x: f32, z: f32| [centre[0] + (x * turned.cos() - z * turned.sin()) * half, centre[1] + 0.4, centre[2] + (x * turned.sin() + z * turned.cos()) * half];
        for (x, z, u, v) in [(-1.0, -1.0, 0.0, 0.0), (1.0, -1.0, 1.0, 0.0), (-1.0, 1.0, 0.0, 1.0), (-1.0, 1.0, 0.0, 1.0), (1.0, -1.0, 1.0, 0.0), (1.0, 1.0, 1.0, 1.0)] {
            corners.extend(corner(x, z));
            corners.extend([u, v]);
            corners.extend([250.0 / 255.0, 150.0 / 255.0, 150.0 / 255.0, 1.0]);
        }
    }
    2
}

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

fn subtract(a: Vector, b: Vector) -> Vector {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: Vector, b: Vector) -> Vector {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: Vector, b: Vector) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(a: Vector) -> Vector {
    let length = dot(a, a).sqrt().max(f32::EPSILON);
    [a[0] / length, a[1] / length, a[2] / length]
}

fn along(origin: Vector, direction: Vector, distance: f32) -> Vector {
    [origin[0] + direction[0] * distance, origin[1] + direction[1] * distance, origin[2] + direction[2] * distance]
}

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

impl Camera {
    /// How far the camera may tilt and how far away it may be, outdoors or inside (the original's limits: 20
    /// degrees either way of 45 outdoors and 10 inside, and inside no farther than where it starts).
    fn limits(indoor: bool) -> ([f32; 2], [f32; 2]) {
        if indoor { ([35.0, 55.0], [DISTANCE_RANGE[0], DISTANCE]) } else { (PITCH_RANGE, DISTANCE_RANGE) }
    }

    /// The camera a map starts with: looking north outdoors, turned 45 degrees inside, as near as it was left.
    fn starting(indoor: bool) -> Self {
        let (_, distances) = Self::limits(indoor);
        Self { yaw: if indoor { 45.0 } else { 0.0 }, pitch: PITCH, distance: Self::remembered().distance.clamp(distances[0], distances[1]) }
    }

    /// The camera as the player last left it, on any map and in any earlier visit; the game's own if never moved.
    fn remembered() -> Self {
        let kept = web_sys::window().and_then(|window| window.local_storage().ok().flatten()).and_then(|storage| storage.get_item(CAMERA_KEPT).ok().flatten());
        let numbers: Vec<f32> = kept.unwrap_or_default().split(',').filter_map(|number| number.parse().ok()).filter(|number: &f32| number.is_finite()).collect();
        match numbers[..] {
            [yaw, pitch, distance] => Self { yaw: yaw.rem_euclid(360.0), pitch: pitch.clamp(PITCH_RANGE[0], PITCH_RANGE[1]), distance: distance.clamp(DISTANCE_RANGE[0], DISTANCE_RANGE[1]) },
            _ => Self::default(),
        }
    }

    /// Keeps the camera for the next map and the next visit.
    fn remember(&self) {
        if let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok().flatten()) {
            let _ = storage.set_item(CAMERA_KEPT, &format!("{},{},{}", self.yaw, self.pitch, self.distance));
        }
    }
}

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

impl Camera {
    /// The world is left-handed (x east, y up, z north), and so is this view: depth grows along `forward`.
    fn view(&self, target: Vector, aspect: f32) -> View {
        let (yaw, pitch) = (self.yaw.to_radians(), self.pitch.to_radians());
        let horizontal = self.distance * pitch.cos();
        let eye = [target[0] - horizontal * yaw.sin(), target[1] + self.distance * pitch.sin(), target[2] - horizontal * yaw.cos()];
        let forward = normalize(subtract(target, eye));
        let right = normalize(cross([0.0, 1.0, 0.0], forward));
        let up = cross(forward, right);
        let focal = 1.0 / (FIELD_OF_VIEW.to_radians() / 2.0).tan();
        let depth = FAR / (FAR - NEAR);
        // Projection times view, written out: each column is what a world axis (or the origin) becomes.
        let column = |axis: Vector, offset: f32| {
            let (x, y, z) = (dot(right, axis) + offset * -dot(right, eye), dot(up, axis) + offset * -dot(up, eye), dot(forward, axis) + offset * -dot(forward, eye));
            [x * focal / aspect, y * focal, z * depth - offset * NEAR * depth, z]
        };
        let columns = [column([1.0, 0.0, 0.0], 0.0), column([0.0, 1.0, 0.0], 0.0), column([0.0, 0.0, 1.0], 0.0), column([0.0, 0.0, 0.0], 1.0)];
        let mut matrix = [0.0; 16];
        for (index, column) in columns.iter().enumerate() {
            matrix[index * 4..index * 4 + 4].copy_from_slice(column);
        }
        View { eye, forward, right, up, aspect, matrix }
    }
}

impl View {
    /// The direction from the eye through a point of the screen (0 to 1 on each axis, y down).
    fn ray(&self, screen: (f32, f32)) -> Vector {
        let tangent = (FIELD_OF_VIEW.to_radians() / 2.0).tan();
        let across = (screen.0 * 2.0 - 1.0) * tangent * self.aspect;
        let upward = (1.0 - screen.1 * 2.0) * tangent;
        normalize(along(along(self.forward, self.right, across), self.up, upward))
    }

    /// Where on the screen a point of the world shows (0 to 1 on each axis, y down), if it is in front of the eye.
    fn project(&self, [x, y, z]: Vector) -> Option<(f32, f32)> {
        let matrix = &self.matrix;
        let clip = |row: usize| matrix[row] * x + matrix[4 + row] * y + matrix[8 + row] * z + matrix[12 + row];
        let depth = clip(3);
        (depth > 0.0).then(|| ((clip(0) / depth + 1.0) / 2.0, (1.0 - clip(1) / depth) / 2.0))
    }

    /// The cell of the ground a point of the screen shows.
    fn pick(&self, gat: &Gat, screen: (f32, f32)) -> Option<(i32, i32)> {
        let direction = self.ray(screen);
        let mut distance = NEAR;
        while distance < FAR {
            let point = along(self.eye, direction, distance);
            let (x, y) = (point[0] / CELL, point[2] / CELL);
            let inside = x >= 0.0 && y >= 0.0 && x < gat.width as f32 && y < gat.height as f32;
            if inside && point[1] <= gat.height_at(x, y) {
                return Some((x as i32, y as i32));
            }
            distance += 1.0;
        }
        None
    }
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
    /// A monster struck down: when. It falls, and is gone a moment later.
    dying: Option<f64>,
    /// What it said last, and when: shown over its head for a while.
    says: Option<(String, f64)>,
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

impl Mover {
    fn new(look: Look, cell: (i32, i32)) -> Self {
        Self { look, name: String::new(), lively: false, cell, at: (cell.0 as f32, cell.1 as f32), facing: 0, walk: None, speed: DEFAULT_SPEED, shown: None, sitting: false, dead: false, dying: None, says: None, life: None, heard: None, motion: None, casting: None }
    }

    /// Starts a walk from one cell to another, along the path this map gives.
    fn walk(&mut self, gat: &Gat, from: (i32, i32), to: (i32, i32)) {
        let path = gat.path(from, to);
        // The server walks its own path. Should this map disagree with it, the walker goes straight there.
        let path = if path.is_empty() && from != to { vec![to] } else { path };
        self.walk = Some((Walk::new(from, &path, self.speed), now()));
    }

    /// Where it is now, in cells, and which picture of it to show to a camera turned by `yaw` degrees.
    fn now(&mut self, now: f64, yaw: f32) -> ((f32, f32), SpriteKey) {
        let (position, walking, elapsed) = match &self.walk {
            Some((walk, started)) => {
                let elapsed = now - started;
                let (position, facing) = walk.at(elapsed);
                self.facing = facing;
                if elapsed >= walk.duration() {
                    self.cell = walk.goal();
                    self.walk = None;
                    (position, false, 0.0)
                } else {
                    self.cell = (position.0.round() as i32, position.1.round() as i32);
                    (position, true, elapsed)
                }
            }
            None => ((self.cell.0 as f32, self.cell.1 as f32), false, 0.0),
        };
        // The legs keep the pace of the walk: a slower walk shows each picture longer.
        let frame = if walking {
            (elapsed / (WALK_FRAME_MS * self.speed / DEFAULT_SPEED)) as usize % WALK_FRAMES
        } else if self.lively {
            (now / IDLE_FRAME_MS) as usize
        } else {
            0
        };
        // The sprite's directions are numbered as the game's, seen by a camera looking north: a camera turned
        // around it sees it from another side.
        let turned = (f32::from(self.facing) * 45.0 - yaw).rem_euclid(360.0);
        let motion = self.motion.filter(|(_, began, lasts)| !walking && now - began < *lasts);
        if motion.is_none() {
            self.motion = None;
        }
        // A motion plays its pictures once over the time it takes, and holds the last.
        let frame = match motion {
            Some((action, began, lasts)) => {
                let pictures = match &self.look {
                    Look::Character(figure) => figure::body_frames(figure.job, figure.sex, action),
                    Look::Creature(path) => figure::frames_of(path, action),
                }
                .unwrap_or(5)
                .max(1);
                (((now - began) / lasts * pictures as f64) as usize).min(pictures - 1)
            }
            None => frame,
        };
        if walking {
            self.sitting = false;
        }
        let resting = match (&self.look, self.dead, self.sitting) {
            (Look::Character(_), true, _) => 64,
            (Look::Character(_), _, true) => 16,
            _ => 0,
        };
        let action = motion.map_or(if walking { WALKING } else { resting }, |(action, ..)| action) + ((turned / 45.0).round() as usize % 8);
        self.at = position;
        let key = match &self.look {
            Look::Character(figure) => SpriteKey::Player(Figure { action, frame: if walking || motion.is_some() { frame } else { 0 }, ..figure.clone() }),
            Look::Creature(path) => SpriteKey::Creature(path.clone(), action, frame),
        };
        (position, key)
    }
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
    drops: HashMap<u32, ((i32, i32), u32, (f32, f32))>,
    /// When each thing that fell out of a monster began to fall.
    falls: HashMap<u32, f64>,
    /// The portals in sight, by the server's ID: the cells they are on.
    portals: HashMap<u32, (i32, i32)>,
    camera: Camera,
    /// The last cell told to the page, so it is told only when it changes.
    told: (i32, i32),
}

/// What the page is told, and what it is asked for.
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
    listening: bool,
    /// Those the server showed while the map was still being read: put on it once it is there.
    early: Vec<Unit>,
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
    strike_asked: f64,
    /// How near the character must stand to strike with what it holds, in cells, once the server has said so
    /// (it says when a blow was asked for from too far).
    strike_range: Option<i32>,
    /// The thing on the ground the character is walking to, to pick it up, and when it was last looked after.
    fetch: Option<(u32, f64)>,
    /// The flashes of blows that landed: where, and when.
    sparks: Vec<(Vector, f64)>,
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

/// Enters a map: loads its ground and puts the character on it. `hooks.ready` is called when it shows.
pub fn enter(map: String, cell: (u16, u16), figure: Figure, hooks: Hooks) {
    let entered = STATE.with_borrow_mut(|state| {
        state.scene = None;
        state.early.clear();
        state.hooks = Some(hooks);
        state.entered += 1;
        state.entered
    });
    if let Ok(canvas) = canvas() {
        let _ = canvas.style().set_property("display", "block");
    }
    wasm_bindgen_futures::spawn_local(async move {
        let loaded = load(&map, cell, figure).await;
        let ready = STATE.with_borrow_mut(|state| {
            if state.entered != entered {
                return None;
            }
            let ready = state.hooks.as_ref().map(|hooks| hooks.ready);
            match loaded {
                Ok((gpu, scene)) => {
                    state.gpu = Some(gpu);
                    state.scene = Some(scene);
                    ready.map(|ready| (ready, Ok(())))
                }
                Err(error) => ready.map(|ready| (ready, Err(error))),
            }
        });
        if let Some((ready, outcome)) = ready {
            let drawn = outcome.is_ok();
            ready(outcome);
            if drawn {
                for unit in STATE.with_borrow_mut(|state| std::mem::take(&mut state.early)) {
                    unit_shown(&unit);
                }
                listen();
                next_frame();
            }
        }
    });
}

/// Leaves the world: the canvas is hidden and nothing is drawn any more.
pub fn leave() {
    STATE.with_borrow_mut(|state| {
        state.scene = None;
        state.entered += 1;
        state.turning = None;
        state.own = [0; 4];
        state.bars = false;
        state.named = None;
        state.pointer = None;
        state.held = None;
        state.chase = None;
        state.fetch = None;
        state.hits.clear();
        state.flying.clear();
        state.arrows.clear();
        state.bolts.clear();
        state.effects_due.clear();
        state.sounds_due.clear();
        state.effects.clear();
        state.landing.clear();
        state.sparks.clear();
    });
    // What the game drew over the page goes with it: the bars, the names, the numbers of blows.
    if let Some(document) = web_sys::window().and_then(|window| window.document()) {
        if let Some(bars) = document.get_element_by_id("bars") {
            bars.set_inner_html("");
        }
        if let Ok(numbers) = document.query_selector_all(".hit") {
            for index in 0..numbers.length() {
                if let Some(number) = numbers.item(index).and_then(|number| number.dyn_into::<web_sys::Element>().ok()) {
                    number.remove();
                }
            }
        }
    }
    name_shown(None);
    if let Ok(canvas) = canvas() {
        let _ = canvas.style().set_property("display", "none");
    }
}

/// The server says the character walks from one cell to another.
pub fn walk(from: (u16, u16), to: (u16, u16)) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        scene.player.walk(&scene.gat, (i32::from(from.0), i32::from(from.1)), (i32::from(to.0), i32::from(to.1)));
    });
}

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
                Some(name) => Look::Creature(format!("{}/{name}", if kind == Kind::Npc { "npc" } else { "몬스터" })),
                None => return,
            },
        };
        let cell = (i32::from(unit.cell.0), i32::from(unit.cell.1));
        let mover = scene.units.entry(unit.id).or_insert_with(|| Mover::new(look.clone(), cell));
        mover.look = look;
        // What follows a # in the name of an NPC or a monster tells the server's scripts apart, and is not shown.
        mover.name = if unit.kind == Kind::Player { unit.name.clone() } else { unit.name.split('#').next().unwrap_or_default().trim().to_owned() };
        mover.lively = unit.kind == Kind::Monster;
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

/// Which of a character's two attacks shows when it strikes: the action at 88 for the weapons its job handles
/// in that way, the one at 80 for everything else (the client's `IsJobWeaponAllowed`). An Archer shoots with
/// the action at 80 and strikes with the other when it holds no bow; a Swordsman thrusts a spear with the one
/// at 88, a Mage and a Merchant stab with a dagger so.
fn attack_action(figure: &Figure) -> usize {
    let kind = data::about(figure.weapon).and_then(|(_, facts)| facts.into_iter().find(|(key, _)| key == "SubType")).map(|(_, kind)| kind).unwrap_or_default();
    let kind = kind.as_str();
    // The reborn and the baby jobs fight as the ones they come from.
    let job = match figure.job {
        4001..=4022 => figure.job - 4001,
        4023..=4045 => figure.job - 4023,
        job => job,
    };
    let male = figure.sex == 1;
    let second = match job {
        1 | 7 | 13 | 14 | 21 => matches!(kind, "1hSpear" | "2hSpear"),
        2 | 5 => kind == "Dagger",
        3 => kind != "Bow",
        6 | 11 | 17 | 19 | 20 => kind == "Bow",
        8 => kind == "Book",
        9 => (kind == "Dagger" && male) || (matches!(kind, "Staff" | "2hStaff") && !male),
        10 | 18 => matches!(kind, "1hSword" | "1hAxe" | "2hAxe" | "Mace"),
        12 => kind == "Katar",
        15 => matches!(kind, "" | "Knuckle"),
        16 => matches!(kind, "2hSpear" | "Staff" | "Book" | "2hStaff"),
        0 | 23 => (kind == "Dagger" && !male) || (male && matches!(kind, "1hSword" | "2hSword" | "1hAxe" | "2hAxe" | "Mace" | "2hMace" | "Staff" | "2hStaff")),
        _ => false,
    };
    if second { 88 } else { 80 }
}

/// Someone strikes someone: the one turns to the other and swings, the other flinches, and the damage shows as
/// a number over it. `own` is the player's own ID, who is not among the others.
pub fn struck(source: u32, target: u32, own: u32, lasts: f64, delay: f64, damage: i32, sound: Option<String>) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let time = now();
        let place = |scene: &Scene, id: u32| if id == own { Some(scene.player.cell) } else { scene.units.get(&id).map(|unit| unit.cell) };
        let (from, to) = (place(scene, source), place(scene, target));
        if source == own {
            state.own_struck = time;
        }
        if let Some(striker) = if source == own { Some(&mut scene.player) } else { scene.units.get_mut(&source) } {
            // Characters have two attacks, and which one shows depends on the job and on what is held (see
            // `attack_action`); monsters have one.
            let action = match &striker.look {
                Look::Character(figure) => attack_action(figure),
                Look::Creature(_) => 16,
            };
            striker.motion = Some((action, time, lasts.clamp(250.0, 2000.0)));
            if let (Some(from), Some(to)) = (from, to) {
                if from != to {
                    striker.facing = crate::map::direction(to.0 - from.0, to.1 - from.1);
                }
            }
        }
        // The blow lands a moment into the swing: then the one struck flinches, and the number rises.
        state.landing.push((time + delay.clamp(0.0, 1500.0), target, own, damage, sound));
    });
}

/// Someone does something that has a motion of its own (picking a thing up): they are seen doing it.
pub fn acts(id: u32, own: u32, action: usize, lasts: f64) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        if let Some(mover) = if id == own { Some(&mut scene.player) } else { scene.units.get_mut(&id) } {
            if matches!(mover.look, Look::Character(_)) {
                mover.motion = Some((action, now(), lasts));
            }
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
pub fn drop_shown(id: u32, item: u32, cell: (u16, u16), within: (u8, u8), falling: bool) {
    STATE.with_borrow_mut(|state| {
        if let Some(scene) = &mut state.scene {
            // The thing lies where the server put it within its cell, in sixteenths from the cell's corner (the
            // client's `SubcellPosition`): things on one cell do not lie on top of each other.
            let at = (f32::from(cell.0) + f32::from(within.0) / 16.0 - 0.5, f32::from(cell.1) + f32::from(within.1) / 16.0 - 0.5);
            scene.drops.insert(id, ((i32::from(cell.0), i32::from(cell.1)), item, at));
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

/// The weapon the player's own character holds, as its item ID (0 for none): it is drawn in hand.
pub fn own_weapon(weapon: u32) {
    STATE.with_borrow_mut(|state| {
        // Another weapon reaches another way: the server will say how far.
        if state.weapon != weapon {
            state.strike_range = None;
        }
        state.weapon = weapon;
        if let Some(Look::Character(figure)) = state.scene.as_mut().map(|scene| &mut scene.player.look) {
            figure.weapon = weapon;
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
    STATE.with_borrow_mut(|state| state.landing.push((now() + delay.clamp(0.0, 5000.0), target, own, damage, sound)));
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
}

/// Whether a skill is being aimed.
pub fn aiming() -> bool {
    STATE.with_borrow(|state| state.aiming.is_some())
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

/// The player's own character recovers life (or spirit): the amount rises over it, green (or blue).
pub fn recovered(amount: i32, spirit: bool) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &state.scene else { return };
        let sprite = if spirit { "이팩트/숫자#0000ff" } else { "이팩트/숫자#00ff00" };
        let pictures = amount.to_string().bytes().map(|digit| SpriteKey::Creature(sprite.to_owned(), 0, usize::from(digit - b'0'))).collect();
        state.flying.push(Flying { began: now(), base: on_ground(&scene.gat, scene.player.at), drift: (0.0, 0.0), pictures, miss: true });
    });
}

/// Plays one of the client's effects (a `.str` file of `data/texture/effect`) on someone: `None` is the player's
/// own character. The file and its pictures are read the first time it is wanted.
pub fn effect(name: &str, on: Option<u32>) {
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

/// The player's own life and spirit, left and most.
pub fn own_life(hp: i64, most_hp: i64, sp: i64, most_sp: i64) {
    STATE.with_borrow_mut(|state| state.own = [hp, most_hp, sp, most_sp]);
}

/// The player's own character dies, or is alive again.
pub fn own_dead(dead: bool) {
    STATE.with_borrow_mut(|state| {
        if let Some(scene) = &mut state.scene {
            scene.player.dead = dead;
            scene.player.walk = None;
            scene.player.motion = None;
        }
        state.chase = None;
        state.fetch = None;
        state.held = None;
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

/// The character is moved to another place of the map it is on (a Fly Wing, a warp within the map): the map
/// stays, and everyone seen at the old place is forgotten: the server shows who is at the new one.
pub fn jump(cell: (u16, u16)) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let cell = (i32::from(cell.0), i32::from(cell.1));
        scene.player.cell = cell;
        scene.player.at = (cell.0 as f32, cell.1 as f32);
        scene.player.walk = None;
        scene.player.motion = None;
        scene.player.dead = false;
        scene.units.clear();
        scene.drops.clear();
        scene.falls.clear();
        scene.portals.clear();
        state.chase = None;
        state.fetch = None;
        state.reach = None;
        state.held = None;
        state.target_at = None;
        state.flying.clear();
        state.arrows.clear();
        state.bolts.clear();
        state.effects_due.clear();
        state.sounds_due.clear();
        state.effects.clear();
        state.landing.clear();
        state.sparks.clear();
    });
}

/// The server says the character stands on a cell.
pub fn stand(cell: (u16, u16)) {
    STATE.with_borrow_mut(|state| {
        if let Some(scene) = &mut state.scene {
            scene.player.walk = None;
            scene.player.cell = (i32::from(cell.0), i32::from(cell.1));
        }
    });
}

/// The character's walking speed, in milliseconds per cell.
pub fn speed(speed: i32) {
    STATE.with_borrow_mut(|state| {
        if let Some(scene) = state.scene.as_mut().filter(|_| speed > 0) {
            scene.player.speed = f64::from(speed);
        }
    });
}

/// Tells the page how far the map is read: the bar of the loading screen fills, with the figure on it.
fn progress(percent: u32) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else { return };
    if let Some(fill) = document.get_element_by_id("loading-fill") {
        let _ = fill.set_attribute("style", &format!("width:{}px", 238 * percent / 100));
    }
    if let Some(figure) = document.get_element_by_id("loading-figure") {
        figure.set_text_content(Some(&format!("{percent:3}%")));
    }
}

async fn load(map: &str, cell: (u16, u16), figure: Figure) -> Result<(Gpu, Scene), String> {
    progress(0);
    let gpu = match STATE.with_borrow_mut(|state| state.gpu.take()) {
        Some(gpu) => gpu,
        None => Gpu::new().await?,
    };
    progress(10);
    let gat = Gat::parse(&fetch_asset(&format!("data/{map}.gat")).await?)?;
    // The client's own list of the maps that are inside buildings: a line `name.rsw#` each.
    let indoor = match figure::text_file("data/indoorrswtable.txt").await {
        Some(list) => {
            let wanted = format!("{}.rsw#", map.to_ascii_lowercase());
            list.lines().any(|line| line.trim().to_ascii_lowercase().starts_with(&wanted))
        }
        None => false,
    };
    let korean = web_sys::TextDecoder::new_with_label("euc-kr").map_err(js_error)?;
    let gnd = Gnd::parse(&fetch_asset(&format!("data/{map}.gnd")).await?, |name| korean.decode_with_u8_array(name).unwrap_or_else(|_| String::from_utf8_lossy(name).into_owned()))?;

    // Every texture of the ground, fetched side by side.
    let pictures = futures_util::future::join_all(gnd.textures.iter().map(|name| async move {
        let path = format!("data/texture/{}", name.replace('\\', "/"));
        let picture = fetch_asset(&path).await.and_then(|bytes| bmp::decode(&bytes));
        if let Err(error) = &picture {
            web_sys::console::warn_1(&format!("texture {path}: {error}").into());
        }
        picture.ok()
    }))
    .await;

    let mesh = gnd.mesh();
    let ground = gpu.ground(&mesh, &pictures);
    progress(50);

    // What stands on the ground, and the water. A map still shows without them, should they not read.
    let decode = |name: &[u8]| korean.decode_with_u8_array(name).unwrap_or_else(|_| String::from_utf8_lossy(name).into_owned());
    let described = match fetch_asset(&format!("data/{map}.rsw")).await {
        Ok(bytes) => Rsw::parse(&bytes, decode),
        Err(error) => Err(error),
    };
    let (models, water) = match described {
        Ok(rsw) => {
            let size = (gnd.width as f32 * gnd.zoom, gnd.height as f32 * gnd.zoom);
            (models(&gpu, &rsw, size, decode).await, water(&gpu, &rsw, size).await)
        }
        Err(error) => {
            web_sys::console::warn_1(&format!("{map}.rsw: {error}").into());
            (Vec::new(), None)
        }
    };
    let cell = (i32::from(cell.0), i32::from(cell.1));
    // The pictures of the last map's people are not needed here.
    let mut gpu = gpu;
    gpu.sprite_textures.clear();
    if gpu.mark.is_none() {
        // The client's own picture for the cell under the pointer, in the colour the client gives it there
        // (0x6600FFAA: green, two fifths there).
        if let Ok(mut image) = picture("grid.tga").await {
            for pixel in image.rgba.chunks_exact_mut(4) {
                pixel[0] = 0;
                pixel[2] = (u16::from(pixel[2]) * 0xaa / 255) as u8;
                pixel[3] = (u16::from(pixel[3]) * 0x66 / 255) as u8;
            }
            let view = gpu.texture(image.width, image.height, &image.rgba);
            gpu.mark = Some(gpu.textured(&gpu.sprite_pipeline.get_bind_group_layout(1), &view, &gpu.pixel_sampler));
        }
    }
    if gpu.ring.is_none() {
        if let Ok(image) = picture("effect\\ring_blue.tga").await {
            let view = gpu.texture(image.width, image.height, &image.rgba);
            gpu.ring = Some(gpu.textured(&gpu.sprite_pipeline.get_bind_group_layout(1), &view, &gpu.pixel_sampler));
        }
    }
    for name in CAST_PICTURES {
        if !gpu.effect_pictures.contains_key(name) {
            if let Ok(image) = picture(&format!("effect\\{name}")).await {
                let view = gpu.texture(image.width, image.height, &image.rgba);
                let binds = [0, 1].map(|which| gpu.textured(&gpu.effect_pipelines[which].get_bind_group_layout(1), &view, &gpu.sampler));
                gpu.effect_pictures.insert(name, Rc::new(binds));
            }
        }
    }
    pointers();
    progress(100);
    let weapon = STATE.with_borrow(|state| state.weapon);
    let player = Mover::new(Look::Character(Figure { weapon, ..figure }), cell);
    Ok((gpu, Scene { indoor, gat, ground, models, water, player, units: HashMap::new(), drops: HashMap::new(), falls: HashMap::new(), portals: HashMap::new(), // A new map is looked at from the south, as the original does: only how near the camera is stays.
        camera: Camera::starting(indoor), told: (-1, -1) }))
}

/// A picture of the game by its path under `data/texture/`: BMP files are read here, everything else (TGA with
/// its own transparency, JPEG) by the browser, from the gateway's PNG of it.
async fn picture(name: &str) -> Result<bmp::Image, String> {
    let path = format!("data/texture/{}", name.replace('\\', "/"));
    if path.to_ascii_lowercase().ends_with(".bmp") {
        return bmp::decode(&fetch_asset(&path).await?);
    }
    let image = web_sys::HtmlImageElement::new().map_err(js_error)?;
    image.set_src(&image_url(&path));
    wasm_bindgen_futures::JsFuture::from(image.decode()).await.map_err(|_| format!("{path} could not be read"))?;
    let (width, height) = (image.natural_width(), image.natural_height());
    let document = web_sys::window().and_then(|window| window.document()).ok_or("no document")?;
    let canvas: HtmlCanvasElement = document.create_element("canvas").map_err(js_error)?.dyn_into().map_err(js_error)?;
    canvas.set_width(width);
    canvas.set_height(height);
    let context: web_sys::CanvasRenderingContext2d = canvas.get_context("2d").map_err(js_error)?.ok_or("no 2D canvas")?.unchecked_into();
    context.draw_image_with_html_image_element(&image, 0.0, 0.0).map_err(js_error)?;
    let rgba = context.get_image_data(0.0, 0.0, f64::from(width), f64::from(height)).map_err(js_error)?.data().0;
    Ok(bmp::Image { width, height, rgba })
}

/// Every model of the map, standing where the map puts it: fetched, put together, lit, and sorted by texture.
async fn models(gpu: &Gpu, rsw: &Rsw, map_size: (f32, f32), decode: impl Fn(&[u8]) -> String + Copy) -> Models {
    let names: BTreeSet<&str> = rsw.placements.iter().map(|placement| placement.model.as_str()).collect();
    let fetched = futures_util::future::join_all(names.iter().map(|name| async move {
        let path = format!("data/model/{}", name.replace('\\', "/"));
        let model = match fetch_asset(&path).await {
            Ok(bytes) => Rsm::parse(&bytes, decode),
            Err(error) => Err(error),
        };
        (*name, model)
    }))
    .await;
    let mut read = BTreeMap::new();
    let mut failed: BTreeMap<String, usize> = BTreeMap::new();
    for (name, model) in fetched {
        match model {
            Ok(model) => {
                read.insert(name, model);
            }
            Err(error) => *failed.entry(error).or_default() += 1,
        }
    }
    for (error, count) in failed {
        web_sys::console::warn_1(&format!("{count} models left out: {error}").into());
    }

    // Per texture, the corners of all triangles drawn with it: three positions, two texture coordinates and a
    // colour each.
    let mut by_texture: BTreeMap<&str, Vec<f32>> = BTreeMap::new();
    for placement in &rsw.placements {
        let Some(model) = read.get(placement.model.as_str()) else { continue };
        for (texture, corners) in model::place(model, placement, map_size, &rsw.light) {
            let Some(name) = model.textures.get(texture) else { continue };
            let floats = by_texture.entry(name.as_str()).or_default();
            for corner in corners {
                floats.extend(corner.position);
                floats.extend(corner.uv);
                floats.extend(corner.colour);
            }
        }
    }
    let pictures = futures_util::future::join_all(by_texture.keys().map(|name| async move { (*name, picture(name).await) })).await;
    let layout = gpu.model_pipeline.get_bind_group_layout(1);
    let mut batches = Vec::new();
    for (name, loaded) in pictures {
        let view = match loaded {
            Ok(picture) => gpu.texture(picture.width, picture.height, &picture.rgba),
            Err(error) => {
                web_sys::console::warn_1(&format!("texture {name}: {error}").into());
                gpu.texture(1, 1, &[160, 160, 160, 255])
            }
        };
        let floats = &by_texture[name];
        let vertices = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(floats), usage: wgpu::BufferUsages::VERTEX });
        batches.push((vertices, (floats.len() / 8) as u32, gpu.textured(&layout, &view, &gpu.repeating)));
    }
    batches
}

/// The map's water: one flat surface over the whole map at the water's height. Where the ground is higher, the
/// ground hides it.
async fn water(gpu: &Gpu, rsw: &Rsw, map_size: (f32, f32)) -> Option<WaterSurface> {
    let kind = rsw.water.kind;
    let loaded = futures_util::future::join_all((0..WATER_FRAMES).map(|frame| async move { picture(&format!("워터/water{kind}{frame:02}.jpg")).await })).await;
    let layout = gpu.water_pipeline.get_bind_group_layout(1);
    let frames: Vec<wgpu::BindGroup> = loaded
        .into_iter()
        .filter_map(|picture| picture.ok())
        .map(|picture| gpu.textured(&layout, &gpu.texture(picture.width, picture.height, &picture.rgba), &gpu.repeating))
        .collect();
    if frames.is_empty() {
        web_sys::console::warn_1(&format!("no pictures for water of kind {kind}").into());
        return None;
    }
    let level = rsw.water.level;
    let mut floats = Vec::with_capacity(30);
    for (x, z) in [(0.0, 0.0), (map_size.0, 0.0), (0.0, map_size.1), (0.0, map_size.1), (map_size.0, 0.0), (map_size.0, map_size.1)] {
        floats.extend([x, level, z, x / WATER_TILE, z / WATER_TILE]);
    }
    let vertices = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&floats), usage: wgpu::BufferUsages::VERTEX });
    Some(WaterSurface { vertices, frames, frame_ms: f64::from(rsw.water.frame_ticks) * 1000.0 / 60.0 })
}

impl Gpu {
    async fn new() -> Result<Self, String> {
        let canvas = canvas()?;
        let (width, height) = pixel_size(&canvas);
        canvas.set_width(width);
        canvas.set_height(height);

        // WebGPU where the browser has it, WebGL2 otherwise.
        let instance = wgpu::util::new_instance_with_webgpu_detection(wgpu::InstanceDescriptor::new_without_display_handle()).await;
        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone())).map_err(|error| error.to_string())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..Default::default() })
            .await
            .map_err(|error| format!("no GPU adapter: {error}"))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await
            .map_err(|error| error.to_string())?;
        let mut config = surface.get_default_config(&adapter, width, height).ok_or("the canvas surface is not supported")?;
        // Colours are mixed as the original mixes them: as stored, without converting to linear light.
        config.format = config.format.remove_srgb_suffix();
        surface.configure(&device, &config);

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });

        let pipeline = |shader: &str, attributes: &[wgpu::VertexAttribute], stride: u64, blend: wgpu::BlendState, depth_write: bool| {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(shader.into()) });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout { array_stride: stride, step_mode: wgpu::VertexStepMode::Vertex, attributes }],
                },
                primitive: Default::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: Some(depth_write),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(blend), write_mask: wgpu::ColorWrites::ALL })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let ground_pipeline = pipeline(GROUND_SHADER, &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x2, 3 => Float32x3], 40, wgpu::BlendState::REPLACE, true);
        let model_pipeline = pipeline(MODEL_SHADER, &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3], 32, wgpu::BlendState::REPLACE, true);
        let water_pipeline = pipeline(WATER_SHADER, &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2], 20, wgpu::BlendState::ALPHA_BLENDING, false);
        let sprite_pipeline = pipeline(SPRITE_SHADER, &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2], 20, wgpu::BlendState::ALPHA_BLENDING, false);
        let effect_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4];
        let adding = wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add };
        let effect_pipelines = [
            pipeline(EFFECT_SHADER, &effect_attributes, 36, wgpu::BlendState::ALPHA_BLENDING, false),
            pipeline(EFFECT_SHADER, &effect_attributes, 36, wgpu::BlendState { color: adding, alpha: adding }, false),
        ];
        let repeating = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Both pipelines read the same camera, each through a bind group of its own layout.
        let camera = |pipeline: &wgpu::RenderPipeline| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() }],
            })
        };
        let (camera_bind, sprite_camera_bind) = (camera(&ground_pipeline), camera(&sprite_pipeline));
        let (model_camera_bind, water_camera_bind) = (camera(&model_pipeline), camera(&water_pipeline));

        let effect_camera_binds = [camera(&effect_pipelines[0]), camera(&effect_pipelines[1])];
        let pixel_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let sprite_room = 64;
        let sprite_vertices = sprite_buffer(&device, sprite_room);
        let depth = depth_view(&device, &config);
        let effect_vertices = effect_buffer(&device, 64);
        Ok(Self {
            canvas,
            surface,
            device,
            queue,
            config,
            depth,
            camera_buffer,
            camera_bind,
            sprite_camera_bind,
            model_camera_bind,
            water_camera_bind,
            ground_pipeline,
            model_pipeline,
            water_pipeline,
            sprite_pipeline,
            sampler,
            repeating,
            pixel_sampler,
            sprite_textures: HashMap::new(),
            sprite_vertices,
            sprite_room,
            ring: None,
            mark: None,
            effect_pipelines,
            effect_camera_binds,
            effect_vertices,
            effect_room: 64,
            effect_pictures: HashMap::new(),
            arrow: None,
        })
    }

    fn texture(&self, width: u32, height: u32, rgba: &[u8]) -> wgpu::TextureView {
        self.device
            .create_texture_with_data(
                &self.queue,
                &wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                rgba,
            )
            .create_view(&Default::default())
    }

    /// A bind group of one picture and how it is sampled.
    fn textured(&self, layout: &wgpu::BindGroupLayout, picture: &wgpu::TextureView, sampler: &wgpu::Sampler) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(picture) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) }],
        })
    }

    /// Puts the ground's mesh and textures on the GPU. A texture that could not be read is drawn plain grey.
    fn ground(&self, mesh: &crate::map::Mesh, pictures: &[Option<bmp::Image>]) -> Ground {
        let mut floats = Vec::with_capacity(mesh.vertices.len() * 10);
        for vertex in &mesh.vertices {
            floats.extend(vertex.position);
            floats.extend(vertex.uv);
            floats.extend(vertex.light_uv);
            floats.extend(vertex.colour);
        }
        let vertices = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&floats), usage: wgpu::BufferUsages::VERTEX });
        let light = self.texture(mesh.light_atlas_size.0.max(1), mesh.light_atlas_size.1.max(1), if mesh.light_atlas.is_empty() { &[0, 0, 0, 255] } else { &mesh.light_atlas });
        let layout = self.ground_pipeline.get_bind_group_layout(1);
        let mut batches = Vec::new();
        for (index, triangles) in mesh.by_texture.iter().enumerate() {
            if triangles.is_empty() {
                continue;
            }
            let picture = match pictures.get(index).and_then(Option::as_ref) {
                Some(picture) => self.texture(picture.width, picture.height, &picture.rgba),
                None => self.texture(1, 1, &[128, 128, 128, 255]),
            };
            let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&picture) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&light) },
                ],
            });
            let indices = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(triangles), usage: wgpu::BufferUsages::INDEX });
            batches.push((indices, triangles.len() as u32, bind));
        }
        Ground { vertices, batches }
    }

    /// Follows the canvas when the window changes size.
    fn fit(&mut self) {
        let (width, height) = pixel_size(&self.canvas);
        if (width, height) != (self.config.width, self.config.height) {
            self.canvas.set_width(width);
            self.canvas.set_height(height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.depth = depth_view(&self.device, &self.config);
        }
    }

    /// Whether a sprite's picture is on the GPU: put there now if it is ready.
    fn has_sprite(&mut self, key: &SpriteKey) -> bool {
        if self.sprite_textures.contains_key(key) {
            return true;
        }
        let Some(picture) = figure::picture(key) else { return false };
        let canvas = &picture.canvas;
        let view = self.texture(canvas.width as u32, canvas.height as u32, &canvas.rgba);
        let bind = self.textured(&self.sprite_pipeline.get_bind_group_layout(1), &view, &self.pixel_sampler);
        self.sprite_textures.insert(key.clone(), SpriteTexture { bind, size: (canvas.width as f32, canvas.height as f32), feet: picture.feet });
        true
    }

    /// Draws the map and, on it, each sprite standing on its feet's place.
    fn draw(&mut self, scene: &Scene, view: &View, sprites: &mut Vec<(SpriteKey, Vector)>, rings: &[f32], marks: &[f32], numbers: &[(SpriteKey, Vector, f32, f32, f32)], effects: &(Vec<f32>, Vec<(Rc<[wgpu::BindGroup; 2]>, bool)>)) {
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&view.matrix));
        if effects.1.len() > self.effect_room {
            self.effect_room = effects.1.len().next_power_of_two();
            self.effect_vertices = effect_buffer(&self.device, self.effect_room);
        }
        if !effects.1.is_empty() {
            self.queue.write_buffer(&self.effect_vertices, 0, bytemuck::cast_slice(&effects.0));
        }

        // The farthest first, so that nearer ones cover them where their see-through edges overlap.
        sprites.sort_by(|a, b| dot(subtract(b.1, view.eye), view.forward).total_cmp(&dot(subtract(a.1, view.eye), view.forward)));
        let rings = if self.ring.is_some() { rings } else { &[] };
        let marks = if self.mark.is_some() { marks } else { &[] };
        let mark_corners = (marks.len() / 5) as u32;
        let ring_corners = (rings.len() / 5) as u32;
        let numbers: Vec<_> = numbers.iter().filter(|number| self.sprite_textures.contains_key(&number.0)).collect();
        let room = sprites.len() + rings.len() / 30 + marks.len() / 30 + numbers.len() + 1;
        if room > self.sprite_room {
            self.sprite_room = room.next_power_of_two();
            self.sprite_vertices = sprite_buffer(&self.device, self.sprite_room);
        }
        let mut corners = Vec::with_capacity(sprites.len() * 30 + rings.len() + marks.len());
        corners.extend_from_slice(marks);
        corners.extend_from_slice(rings);
        for (key, feet) in sprites.iter() {
            let texture = &self.sprite_textures[key];
            // A picture standing on the feet, facing the camera, a touch nearer than the ground it stands on.
            // It is drawn as if two cells nearer, in the same place on the screen, as the original does: a ledge
            // or a doorway over the cell does not cut through the one standing there.
            let nearer = 1.0 - 10.0 / dot(subtract(*feet, view.eye), view.forward).max(20.0);
            let (width, height) = texture.size;
            for (x, y) in [(0.0, 0.0), (width, 0.0), (0.0, height), (0.0, height), (width, 0.0), (width, height)] {
                let corner = along(along(*feet, view.right, (x - texture.feet.0) * SPRITE_SCALE), view.up, (texture.feet.1 - y) * SPRITE_SCALE);
                corners.extend(along(view.eye, subtract(corner, view.eye), nearer));
                corners.extend([x / width, y / height]);
            }
        }
        // The numbers: each picture at its place, as large and as faded as it is now, over everything.
        for (key, at, scale, alpha, across) in numbers.iter().copied() {
            let texture = &self.sprite_textures[key];
            let nearer = 1.0 - 25.0 / dot(subtract(*at, view.eye), view.forward).max(40.0);
            let (width, height) = texture.size;
            let faded = ((1.0 - alpha.clamp(0.0, 1.0)) * 16.0).floor().min(15.0) * 2.0;
            for (x, y) in [(0.0, 0.0), (width, 0.0), (0.0, height), (0.0, height), (width, 0.0), (width, height)] {
                let corner = along(along(*at, view.right, (x - texture.feet.0 + across) * SPRITE_SCALE * scale), view.up, (texture.feet.1 - y) * SPRITE_SCALE * scale);
                corners.extend(along(view.eye, subtract(corner, view.eye), nearer));
                corners.extend([x / width, y / height + faded]);
            }
        }
        self.queue.write_buffer(&self.sprite_vertices, 0, bytemuck::cast_slice(&corners));

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => return,
        };
        let target = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&self.ground_pipeline);
            pass.set_bind_group(0, &self.camera_bind, &[]);
            pass.set_vertex_buffer(0, scene.ground.vertices.slice(..));
            for (indices, count, bind) in &scene.ground.batches {
                pass.set_bind_group(1, bind, &[]);
                pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..*count, 0, 0..1);
            }
            pass.set_pipeline(&self.model_pipeline);
            pass.set_bind_group(0, &self.model_camera_bind, &[]);
            for (vertices, count, bind) in &scene.models {
                pass.set_bind_group(1, bind, &[]);
                pass.set_vertex_buffer(0, vertices.slice(..));
                pass.draw(0..*count, 0..1);
            }
            if let Some(water) = &scene.water {
                pass.set_pipeline(&self.water_pipeline);
                pass.set_bind_group(0, &self.water_camera_bind, &[]);
                pass.set_bind_group(1, &water.frames[(now() / water.frame_ms) as usize % water.frames.len()], &[]);
                pass.set_vertex_buffer(0, water.vertices.slice(..));
                pass.draw(0..6, 0..1);
            }
            if !corners.is_empty() {
                pass.set_pipeline(&self.sprite_pipeline);
                pass.set_bind_group(0, &self.sprite_camera_bind, &[]);
                pass.set_vertex_buffer(0, self.sprite_vertices.slice(..));
                if let Some(mark) = self.mark.as_ref().filter(|_| mark_corners > 0) {
                    pass.set_bind_group(1, mark, &[]);
                    pass.draw(0..mark_corners, 0..1);
                }
                if let Some(ring) = self.ring.as_ref().filter(|_| ring_corners > 0) {
                    pass.set_bind_group(1, ring, &[]);
                    pass.draw(mark_corners..mark_corners + ring_corners, 0..1);
                }
                for (index, (key, _)) in sprites.iter().enumerate() {
                    pass.set_bind_group(1, &self.sprite_textures[key].bind, &[]);
                    let first = mark_corners + ring_corners + index as u32 * 6;
                    pass.draw(first..first + 6, 0..1);
                }
                for (index, number) in numbers.iter().enumerate() {
                    pass.set_bind_group(1, &self.sprite_textures[&number.0].bind, &[]);
                    let first = mark_corners + ring_corners + (sprites.len() + index) as u32 * 6;
                    pass.draw(first..first + 6, 0..1);
                }
            }
            if !effects.1.is_empty() {
                pass.set_vertex_buffer(0, self.effect_vertices.slice(..));
                for (index, (binds, adds)) in effects.1.iter().enumerate() {
                    let which = usize::from(*adds);
                    pass.set_pipeline(&self.effect_pipelines[which]);
                    pass.set_bind_group(0, &self.effect_camera_binds[which], &[]);
                    pass.set_bind_group(1, &binds[which], &[]);
                    let first = index as u32 * 6;
                    pass.draw(first..first + 6, 0..1);
                }
            }
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
    }
}

fn depth_view(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: config.width, height: config.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

/// A buffer for the corners of `room` quads of effects: six corners each, of three coordinates, two texture ones
/// and four of colour.
fn effect_buffer(device: &wgpu::Device, room: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (room * 6 * 36) as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })
}

/// A buffer for the corners of `room` sprites: six corners each, of three coordinates and two texture ones.
fn sprite_buffer(device: &wgpu::Device, room: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (room * 6 * 20) as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })
}

/// The rings of light of a portal standing at a place, at a moment: two sets of four bands opening upwards,
/// each turning its own way. (The original's `AlphaDownRipples` with `ring_blue.tga`; its sparks are not made.)
fn rings(centre: Vector, time: f64, corners: &mut Vec<f32>) {
    const SIDES: usize = 20;
    for (size, turn, growth) in [(11.0f32, 0.0f32, 0.0f32), (4.0, 1.0, 0.2)] {
        for (slot, (start, distance, rise)) in [(270.0f32, 2.5f32, 53.0f32), (0.0, 5.0, 60.0), (90.0, 7.5, 55.0), (180.0, 10.0, 50.0)].into_iter().enumerate() {
            let rise = (rise - turn).to_radians();
            let (low, high, height) = (distance + growth, distance + growth + size * rise.cos(), size * rise.sin());
            let way = if slot % 2 == 0 { 1.0 } else { -1.0 };
            let spin = (start + turn).to_radians() + way * (time / 1000.0) as f32 * 1.2;
            let point = |side: usize, radius: f32, up: f32| {
                let angle = spin + side as f32 / SIDES as f32 * std::f32::consts::TAU;
                [centre[0] + radius * angle.cos(), centre[1] + 0.3 + up, centre[2] + radius * angle.sin()]
            };
            for side in 0..SIDES {
                // The picture goes around five times.
                let (left, right) = ((side % 4) as f32 / 4.0, (side % 4 + 1) as f32 / 4.0);
                let quad = [(side, low, 0.0, left, 1.0), (side + 1, low, 0.0, right, 1.0), (side, high, height, left, 0.0), (side, high, height, left, 0.0), (side + 1, low, 0.0, right, 1.0), (side + 1, high, height, right, 0.0)];
                for (side, radius, up, u, v) in quad {
                    corners.extend(point(side, radius, up));
                    corners.extend([u, v]);
                }
            }
        }
    }
}

/// The flash of a blow at a place, `through` of its moment (0 to 1) gone: a low ring opening outwards.
fn spark(centre: Vector, through: f32, corners: &mut Vec<f32>) {
    const SIDES: usize = 12;
    let (low, high, height) = (2.0 + through * 5.0, 5.0 + through * 8.0, 2.5);
    let point = |side: usize, radius: f32, up: f32| {
        let angle = side as f32 / SIDES as f32 * std::f32::consts::TAU;
        [centre[0] + radius * angle.cos(), centre[1] + up, centre[2] + radius * angle.sin()]
    };
    for side in 0..SIDES {
        let (left, right) = ((side % 4) as f32 / 4.0, (side % 4 + 1) as f32 / 4.0);
        for (side, radius, up, u, v) in [(side, low, 0.0, left, 1.0), (side + 1, low, 0.0, right, 1.0), (side, high, height, left, 0.0), (side, high, height, left, 0.0), (side + 1, low, 0.0, right, 1.0), (side + 1, high, height, right, 0.0)] {
            corners.extend(point(side, radius, up));
            corners.extend([u, v]);
        }
    }
}

/// Reads the game's own mouse pointers, once.
fn pointers() {
    if STATE.with_borrow(|state| !state.pointers.is_empty()) {
        return;
    }
    wasm_bindgen_futures::spawn_local(async {
        let mut read = Vec::new();
        for action in POINTERS {
            // The first picture tells how many there are.
            let Ok(first) = figure::cursor(action, 0).await else { return };
            let (count, shows) = figure::timing("cursors", action).unwrap_or((1, 100.0));
            let mut pictures = vec![first];
            for frame in 1..count.min(32) {
                match figure::cursor(action, frame).await {
                    Ok(picture) => pictures.push(picture),
                    Err(_) => break,
                }
            }
            read.push((pictures, shows.max(40.0)));
        }
        // Everything pressable on the page has the game's pointers too: the plain one, and the pressing one.
        if let (Some(root), Some(plain), Some(press)) = (
            web_sys::window().and_then(|window| window.document()).and_then(|document| document.document_element()).and_then(|root| root.dyn_into::<web_sys::HtmlElement>().ok()),
            read.first().and_then(|(pictures, _)| pictures.first()),
            figure::cursor(2, 0).await.ok(),
        ) {
            let _ = root.style().set_property("--point", plain);
            let _ = root.style().set_property("--press", &press);
        }
        // Every picture of every pointer is read by the browser ahead of its use and kept on the page, out of
        // sight: a pointer whose picture the browser has yet to read shows as the plain arrow for a moment, which
        // made the pointer flicker the first time it ran over a monster.
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            if let (Ok(keep), Some(body)) = (document.create_element("div"), document.body()) {
                let _ = keep.set_attribute("style", "position:fixed;left:-99px;top:-99px;width:1px;height:1px;overflow:hidden;pointer-events:none");
                for picture in read.iter().flat_map(|(pictures, _)| pictures) {
                    let Some(url) = picture.strip_prefix("url(").and_then(|rest| rest.split(')').next()) else { continue };
                    if let Ok(image) = web_sys::HtmlImageElement::new() {
                        image.set_src(url);
                        let _ = wasm_bindgen_futures::JsFuture::from(image.decode()).await;
                        let _ = keep.append_child(&image);
                    }
                }
                let _ = body.append_child(&keep);
            }
        }
        STATE.with_borrow_mut(|state| {
            state.pointers = read;
            // Shown with the next frame.
            state.pointing = usize::MAX;
        });
    });
}

/// Where a point given in cells is in the world: cell centres, on the ground.
fn on_ground(gat: &Gat, (x, y): (f32, f32)) -> Vector {
    // A character on cell 3 stands at 3.5 cells.
    [(x + 0.5) * CELL, gat.height_at(x + 0.5, y + 0.5), (y + 0.5) * CELL]
}

/// Draws one frame, and asks for the next while there is a world to draw.
fn frame() {
    // The effects and sounds whose moment has come.
    let (effects, sounds) = STATE.with_borrow_mut(|state| {
        let time = now();
        let effects: Vec<_> = state.effects_due.extract_if(.., |due| due.0 <= time).collect();
        let sounds: Vec<_> = state.sounds_due.extract_if(.., |due| due.0 <= time).collect();
        (effects, sounds)
    });
    for (_, name, on) in effects {
        effect(&name, on);
    }
    for (_, name) in sounds {
        sound::effect(&name, 1.0);
    }
    let told = STATE.with_borrow_mut(|state| {
        let (Some(gpu), Some(scene)) = (&mut state.gpu, &mut state.scene) else { return None };
        gpu.fit();
        let (time, yaw) = (now(), scene.camera.yaw);
        let (position, key) = scene.player.now(time, yaw);
        let feet = on_ground(&scene.gat, position);
        let view = scene.camera.view(feet, gpu.config.width as f32 / gpu.config.height as f32);

        // Everyone to draw: each with the picture wanted now if it is ready, else the one drawn last.
        // The blows that land now: the one struck flinches, the flash and the number show, the blow sounds.
        let mut landed = Vec::new();
        state.landing.retain(|blow| {
            if blow.0 > time {
                return true;
            }
            landed.push((blow.1, blow.2, blow.3, blow.4.clone()));
            false
        });
        for (target, own, damage, heard) in landed {
            let Some(hurt) = (if target == own { Some(&mut scene.player) } else { scene.units.get_mut(&target) }) else { continue };
            if damage > 0 {
                if hurt.motion.is_none() && hurt.dying.is_none() && !hurt.dead {
                    hurt.motion = Some((if matches!(hurt.look, Look::Character(_)) { 48 } else { 24 }, time, 280.0));
                }
                state.sparks.push((along(on_ground(&scene.gat, hurt.at), [0.0, 1.0, 0.0], 7.0), time));
                if let Some(heard) = heard {
                    sound::effect(&heard, 1.0);
                }
            }
            state.hits.push((target, damage));
        }
        // Monsters struck down are gone once they have fallen.
        scene.units.retain(|_, unit| unit.dying.is_none_or(|fell| time - fell < 1000.0));
        let mut sprites = Vec::with_capacity(scene.units.len() + 1);
        let gat = &scene.gat;
        let feet_cell = position;
        for (mover, position, wanted) in std::iter::once((&mut scene.player, position, key)).chain(scene.units.values_mut().map(|unit| {
            let (position, key) = unit.now(time, yaw);
            (unit, position, key)
        })) {
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
            if let Some(shown) = &mover.shown {
                sprites.push((shown.clone(), on_ground(gat, position)));
            }
        }
        // What lies on the ground, once the item's picture is known.
        for (id, (_, item, at)) in &scene.drops {
            let Some((_, resource)) = data::item(*item) else { continue };
            let key = SpriteKey::Creature(format!("아이템/{}", if resource.is_empty() { "사과" } else { resource.as_str() }), 0, 0);
            if gpu.has_sprite(&key) {
                // A thing that fell out of a monster starts 15 above the ground, is thrown up a little and
                // falls (the client's `CItem::OnProcess`, in its frames of 24 milliseconds).
                let above = scene.falls.get(id).map_or(0.0, |began| {
                    let elapsed = ((time - began) / EFFECT_FRAME_MS) as f32;
                    (15.0 + 0.6 * elapsed - 0.083 * elapsed * elapsed).max(0.0)
                });
                sprites.push((key, along(on_ground(gat, *at), [0.0, 1.0, 0.0], above)));
            }
        }
        // Last frame's place of the pointer in the world: its pictures turn at two of the client's frames each.
        if let Some(at) = state.target_at {
            let pictures = figure::timing("cursors", 3).map_or(1, |(count, _)| count.max(1));
            let key = SpriteKey::Creature("cursors".to_owned(), 3, (time / EFFECT_FRAME_MS / 2.0) as usize % pictures);
            if gpu.has_sprite(&key) {
                sprites.push((key, at));
            }
        }
        let mut ring_corners = Vec::new();
        for cell in scene.portals.values() {
            rings(on_ground(gat, (cell.0 as f32, cell.1 as f32)), time, &mut ring_corners);
        }
        // A blow that lands flashes: a ring of light opening for a moment where it struck.
        state.sparks.retain(|(_, struck)| time - struck < 180.0);
        for (at, struck) in &state.sparks {
            spark(*at, ((time - struck) / 180.0) as f32, &mut ring_corners);
        }
        // The ground cell under the pointer is marked, as the original marks where a press would walk to.
        let mut marks = Vec::new();
        let (page_width, page_height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
        // Only a cell that can be walked on is marked (the client skips walls and cliffs).
        if let Some(cell) = state.pointer.and_then(|pointer| view.pick(&scene.gat, (pointer.0 / page_width.max(1.0), pointer.1 / page_height.max(1.0)))).filter(|cell| scene.gat.walkable(cell.0, cell.1)) {
            let corner = |x: i32, y: i32| [x as f32 * CELL, scene.gat.height_at(x as f32, y as f32) + 0.4, y as f32 * CELL];
            let (x, y) = cell;
            for (corner_x, corner_y, u, v) in [(x, y + 1, 0.0, 0.0), (x + 1, y + 1, 1.0, 0.0), (x, y, 0.0, 1.0), (x, y, 0.0, 1.0), (x + 1, y + 1, 1.0, 0.0), (x + 1, y, 1.0, 1.0)] {
                marks.extend(corner(corner_x, corner_y));
                marks.extend([u, v]);
            }
        }
        // The numbers in the air, by the client's own figures. A number starts five times its size and shrinks
        // to a little over it, rises and falls back as it drifts, and fades: all over 70 of the client's frames.
        // A miss only rises, and fades over 80.
        state.flying.retain(|flying| (time - flying.began) / EFFECT_FRAME_MS <= if flying.miss { 80.0 } else { 70.0 });
        let mut numbers = Vec::new();
        for flying in &state.flying {
            let elapsed = ((time - flying.began) / EFFECT_FRAME_MS) as f32;
            let (at, scale, alpha) = if flying.miss {
                (along(flying.base, [0.0, 1.0, 0.0], 12.0 + elapsed * 0.54), 1.0, (250.0 - elapsed * 3.0) / 255.0)
            } else {
                let aside = elapsed / 3.0 + 3.0;
                let up = 12.0 - 8.0 + (2.0 - elapsed / 30.0) * elapsed;
                ([flying.base[0] + flying.drift.0 * aside, flying.base[1] + up, flying.base[2] + flying.drift.1 * aside], (5.0 - elapsed * 0.24).max(1.2), (250.0 - elapsed * 3.4) / 255.0)
            };
            let mut across = 0.0;
            for key in &flying.pictures {
                if !gpu.has_sprite(key) {
                    continue;
                }
                let wide = gpu.sprite_textures[key].size.0;
                numbers.push((key.clone(), at, scale, alpha, across - (flying.pictures.len() as f32 - 1.0) * wide * 0.4));
                across += wide * 0.8;
            }
        }
        // The effects playing: every layer's quad as it is at this frame of the effect, around the one it plays
        // on, by the client's own figures (a pixel of the effect is a pixel of a sprite).
        state.effects.retain(|playing| ((time - playing.began) / 1000.0 * f64::from(playing.shown.effect.frames_per_second)) as i32 <= playing.shown.effect.last_frame);
        let mut effects = (Vec::new(), Vec::new());
        for playing in &state.effects {
            let place = match playing.on {
                None => Some(position),
                Some(id) => scene.units.get_mut(&id).map(|unit| unit.now(time, yaw).0),
            };
            let Some(place) = place else { continue };
            // The effect's middle stands 80 pixels over the place of the one it plays on (the client's
            // `CEZ2STRScreenEffect::OnProcess` moves it up the screen by that much).
            let middle = along(on_ground(&scene.gat, place), view.up, 80.0 * SPRITE_SCALE);
            let nearer = 1.0 - 25.0 / dot(subtract(middle, view.eye), view.forward).max(40.0);
            let frame = ((time - playing.began) / 1000.0 * f64::from(playing.shown.effect.frames_per_second)) as i32;
            for (layer, pictures) in playing.shown.effect.layers.iter().zip(&playing.shown.pictures) {
                let Some(transform) = layer.at(frame) else { continue };
                let Some(Some(binds)) = pictures.get(transform.aniframe.max(0.0) as usize) else { continue };
                let colour = transform.colour.map(|part| (part / 255.0).clamp(0.0, 1.0));
                if colour[3] <= 0.0 {
                    continue;
                }
                let corners = transform.corners();
                for corner in [0, 1, 3, 3, 1, 2] {
                    let ([x, y], uv) = corners[corner];
                    let at = along(along(middle, view.right, x * SPRITE_SCALE), view.up, -y * SPRITE_SCALE);
                    effects.0.extend(along(view.eye, subtract(at, view.eye), nearer));
                    effects.0.extend(uv);
                    effects.0.extend(colour);
                }
                // Direct3D's "one" as the destination (2) adds the quad's light to what is there; so does "as
                // much as the destination's alpha" (7), the screen having none to speak of.
                effects.1.push((binds.clone(), matches!(transform.destination_blend, 2 | 7)));
            }
        }
        // The casts under way: the rings around each caster and the mark on the one aimed at.
        let mut casts = Vec::new();
        for mover in std::iter::once(&mut scene.player).chain(scene.units.values_mut()) {
            if mover.casting.is_some_and(|(began, lasts, ..)| time > began + lasts) {
                mover.casting = None;
            }
            if let Some((began, lasts, effect, target)) = mover.casting {
                casts.push((mover.at, began, lasts, effect, target));
            }
        }
        for (at, began, lasts, effect, target) in &casts {
            let frame = ((time - began) / CAST_FRAME_MS) as f32;
            let (name, tint, adds) = cast_look(*effect);
            if let Some(binds) = gpu.effect_pictures.get(name) {
                let quads = cast_rings(on_ground(&scene.gat, *at), frame, ((lasts / CAST_FRAME_MS) as f32).max(70.0), tint, &mut effects.0);
                effects.1.extend(std::iter::repeat_n((binds.clone(), adds), quads));
            }
            let aimed = target.and_then(|id| scene.units.get(&id)).map(|unit| unit.at);
            if let (Some(aimed), Some(binds)) = (aimed, gpu.effect_pictures.get("lockon128.tga")) {
                let quads = lock_on(on_ground(&scene.gat, aimed), frame, &mut effects.0);
                effects.1.extend(std::iter::repeat_n((binds.clone(), false), quads));
            }
        }
        // The bolts falling: each a picture turned along its fall, its tip ahead, until it reaches the ground.
        const BOLT_SPEED: f32 = 2.75;
        let mut falling = Vec::new();
        state.bolts.retain(|bolt| {
            if time < bolt.leaves {
                return true;
            }
            let Some(on) = scene.units.get(&bolt.target).map(|unit| unit.at).or((scene.units.is_empty() || bolt.target == 0).then_some(scene.player.at)) else { return false };
            let to = on_ground(&scene.gat, on);
            let from = [to[0] + bolt.from[0], to[1] + bolt.from[1], to[2] + bolt.from[2]];
            let way = subtract(to, from);
            let length = dot(way, way).sqrt().max(0.001);
            let gone = ((time - bolt.leaves) / CAST_FRAME_MS) as f32 * BOLT_SPEED;
            if gone >= length {
                return false;
            }
            falling.push((along(from, way, gone / length), way, bolt.fire, ((time - bolt.leaves) / CAST_FRAME_MS / 2.0) as usize));
            true
        });
        for (at, way, fire, frame) in falling {
            let (name, long, wide) = if fire { (FIRE_BOLT[frame % FIRE_BOLT.len()], 14.0, 3.5) } else { ("icearrow.tga", 11.5, 3.8) };
            let Some(binds) = gpu.effect_pictures.get(name) else { continue };
            let (across, up) = (dot(way, view.right), dot(way, view.up));
            let length = (across * across + up * up).sqrt().max(0.001);
            // The picture's tip is at its right: that end leads.
            let (ahead, aside) = ([across / length, up / length], [-up / length, across / length]);
            let nearer = 1.0 - 10.0 / dot(subtract(at, view.eye), view.forward).max(20.0);
            for (x, y, u, v) in [(-1.0, 0.5, 0.0, 0.0), (0.0, 0.5, 1.0, 0.0), (-1.0, -0.5, 0.0, 1.0), (-1.0, -0.5, 0.0, 1.0), (0.0, 0.5, 1.0, 0.0), (0.0, -0.5, 1.0, 1.0)] {
                let (x, y) = (x * long, y * wide);
                let corner = along(along(at, view.right, x * ahead[0] + y * aside[0]), view.up, x * ahead[1] + y * aside[1]);
                effects.0.extend(along(view.eye, subtract(corner, view.eye), nearer));
                effects.0.extend([u, v]);
                effects.0.extend([1.0, 1.0, 1.0, 1.0]);
            }
            effects.1.push((binds.clone(), fire));
        }
        // The arrows in flight: each a picture turned along its way on the screen, from the archer to its target.
        state.arrows.retain(|(leaves, ..)| time - leaves <= ARROW_MS);
        if !state.arrows.is_empty() && gpu.arrow.is_none() {
            let key = SpriteKey::Creature(ARROW.to_owned(), 0, 0);
            if let Some(picture) = figure::picture(&key) {
                let canvas = &picture.canvas;
                let texture = gpu.texture(canvas.width as u32, canvas.height as u32, &canvas.rgba);
                let binds = [0, 1].map(|which| gpu.textured(&gpu.effect_pipelines[which].get_bind_group_layout(1), &texture, &gpu.pixel_sampler));
                gpu.arrow = Some((Rc::new(binds), (canvas.width as f32, canvas.height as f32)));
            }
        }
        if let Some((binds, (wide, tall))) = &gpu.arrow {
            for (leaves, from, target) in &state.arrows {
                if time < *leaves {
                    continue;
                }
                let Some(to) = scene.units.get(target).map(|unit| unit.at).or((*target != 0).then_some(scene.player.at)) else { continue };
                let to = along(on_ground(&scene.gat, to), [0.0, 1.0, 0.0], 10.0);
                let at = along(*from, subtract(to, *from), ((time - leaves) / ARROW_MS) as f32);
                // The way it flies as the screen shows it: the picture's head is at its bottom, and is turned to
                // lead.
                let way = subtract(*from, to);
                let (across, up) = (dot(way, view.right), dot(way, view.up));
                let length = (across * across + up * up).sqrt().max(0.001);
                let (cosine, sine) = (up / length, across / length);
                let nearer = 1.0 - 10.0 / dot(subtract(at, view.eye), view.forward).max(20.0);
                for (x, y, u, v) in [(-0.5, 0.5, 0.0, 0.0), (0.5, 0.5, 1.0, 0.0), (-0.5, -0.5, 0.0, 1.0), (-0.5, -0.5, 0.0, 1.0), (0.5, 0.5, 1.0, 0.0), (0.5, -0.5, 1.0, 1.0)] {
                    let (x, y) = (x * wide * SPRITE_SCALE, y * tall * SPRITE_SCALE);
                    let corner = along(along(at, view.right, x * cosine + y * sine), view.up, y * cosine - x * sine);
                    effects.0.extend(along(view.eye, subtract(corner, view.eye), nearer));
                    effects.0.extend([u, v]);
                    effects.0.extend([1.0, 1.0, 1.0, 1.0]);
                }
                effects.1.push((binds.clone(), false));
            }
        }
        gpu.draw(scene, &view, &mut sprites, &ring_corners, &marks, &numbers, &effects);

        // Whoever the pointer is on: the nearest of those whose picture it touches. The name goes under the feet,
        // as the original puts it, and the pointer becomes the one for talking or for fighting.
        let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
        let over = state.pointer.and_then(|pointer| pointed_at(gpu, scene, &view, pointer)).and_then(|(id, feet)| scene.units.get(&id).map(|unit| (0.0, id, unit, feet)));
        // The client's pointer in the world (the third of its pointers, turning): at the feet of whoever the
        // pointer is on if it is a monster, or else of the monster being fought. Never at an NPC's.
        let marked = over.as_ref().filter(|(_, _, unit, _)| unit.lively).map(|(_, id, _, _)| *id).or(state.chase.map(|(id, ..)| id)).and_then(|id| scene.units.get(&id)).filter(|unit| unit.dying.is_none());
        state.target_at = marked.map(|unit| on_ground(&scene.gat, unit.at));
        state.over = over.as_ref().filter(|(_, _, unit, _)| matches!(unit.look, Look::Creature(_))).map(|(_, id, unit, _)| (*id, unit.lively));
        // The numbers of the hits rise from the heads of those hit, and are gone in a moment.
        for (target, damage) in std::mem::take(&mut state.hits) {
            let (hurt, own) = match scene.units.get(&target) {
                Some(unit) => (unit, false),
                None => (&scene.player, true),
            };
            // The client sends the number off by the quarter the one struck faces (`MsgEffectDrift`).
            let drift = match hurt.facing {
                0 | 1 => (-0.8, 0.8),
                2 | 3 => (-0.8, -0.8),
                4 | 5 => (0.8, -0.8),
                _ => (0.8, 0.8),
            };
            // Blows taken by the player's own character are red.
            let sprite = if own { "이팩트/숫자#ff0000" } else { "이팩트/숫자" };
            let pictures = if damage > 0 {
                damage.to_string().bytes().map(|digit| SpriteKey::Creature(sprite.to_owned(), 0, usize::from(digit - b'0'))).collect()
            } else {
                vec![SpriteKey::Creature("이팩트/msg".to_owned(), 0, 0)]
            };
            state.flying.push(Flying { began: time, base: on_ground(&scene.gat, hurt.at), drift, pictures, miss: damage <= 0 });
        }
        // The life of the monsters being fought, as a bar under each.
        let bars: String = scene
            .units
            .values()
            .filter_map(|unit| {
                let (left, most) = unit.life.filter(|(left, most)| *most > 0 && left < most)?;
                let under = view.project(on_ground(&scene.gat, unit.at))?;
                let full = (left.max(0) as f32 / most as f32 * 58.0).round();
                // 12 pixels of a screen 480 high under the feet, nearer when the camera is (the client's place for
                // a monster's gauge; its name stands at 20).
                let below = height * 12.0 / 480.0 * DISTANCE / scene.camera.distance;
                Some(format!(r#"<div class="life" style="left:{:.0}px;top:{:.0}px"><i style="width:{full}px"></i></div>"#, under.0 * width - 30.0, under.1 * height + below))
            })
            .collect();
        // Under the player's own character: life in green over spirit in blue, as the original shows them.
        let mut bars = bars;
        if let Some(under) = view.project(feet).filter(|_| state.own[1] > 0) {
            let part = |left: i64, most: i64| if most > 0 { (left.clamp(0, most) as f32 / most as f32 * 58.0).round() } else { 0.0 };
            let low = if state.own[0] * 4 < state.own[1] { " low" } else { "" };
            bars.push_str(&format!(
                r#"<div class="life own" style="left:{:.0}px;top:{:.0}px"><i class="hp{low}" style="width:{}px"></i><i class="sp" style="width:{}px"></i></div>"#,
                under.0 * width - 30.0,
                under.1 * height + height * 12.0 / 480.0 * DISTANCE / scene.camera.distance,
                part(state.own[0], state.own[1]),
                part(state.own[2], state.own[3]),
            ));
        }
        // The bar of a cast, over the caster (the client's `UIRechargeGage`): 60 by 6, 81 pixels over the feet,
        // filling over the cast's time.
        for (at, began, lasts, ..) in &casts {
            if let Some(under) = view.project(on_ground(&scene.gat, *at)) {
                let full = ((60.0 * ((time - began) / lasts.max(1.0)).clamp(0.0, 1.0) as f32) - 2.0).max(0.0).floor();
                let over = 81.0 * DISTANCE / scene.camera.distance * height / 480.0;
                bars.push_str(&format!(r#"<div class="cast" style="left:{:.0}px;top:{:.0}px"><i style="width:{full}px"></i></div>"#, under.0 * width - 30.0, under.1 * height - over));
            }
        }
        // What was said hangs over the head of whoever said it for a few seconds.
        for mover in std::iter::once(&scene.player).chain(scene.units.values()) {
            let Some((words, _)) = mover.says.as_ref().filter(|(_, when)| time - when <= 5000.0) else { continue };
            if let Some(over) = view.project(along(on_ground(&scene.gat, mover.at), [0.0, 1.0, 0.0], 21.0)) {
                let safe = words.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
                bars.push_str(&format!(r#"<div class="say" style="left:{:.0}px;top:{:.0}px">{safe}</div>"#, over.0 * width, over.1 * height));
            }
        }
        if !bars.is_empty() || state.bars {
            state.bars = !bars.is_empty();
            if let Some(element) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("bars")) {
                element.set_inner_html(&bars);
            }
        }
        // A portal is pointed at anywhere over its rings.
        let near = DISTANCE / scene.camera.distance * height / 480.0;
        let on_portal = state.pointer.is_some_and(|pointer| {
            scene.portals.values().any(|cell| {
                view.project(on_ground(&scene.gat, (cell.0 as f32, cell.1 as f32))).is_some_and(|middle| {
                    let (across, up) = ((pointer.0 - middle.0 * width).abs(), middle.1 * height - pointer.1);
                    across < 45.0 * near && (-25.0 * near..45.0 * near).contains(&up)
                })
            })
        });
        let pointing = if state.turning.is_some() { 5 } else { usize::MAX };
        let pointing = match &over {
            _ if pointing == 5 => 5,
            _ if state.aiming.is_some() => 6,
            Some((_, _, unit, _)) if unit.lively => 2,
            // A thing lying at an NPC's feet is still picked up.
            Some(_) if state.pointer.is_some_and(|pointer| lying_at(gpu, scene, &view, pointer).is_some()) => 4,
            Some((_, _, unit, _)) if matches!(unit.look, Look::Creature(_)) => 1,
            None if state.pointer.is_some_and(|pointer| lying_at(gpu, scene, &view, pointer).is_some()) => 4,
            None if on_portal => 3,
            _ => 0,
        };
        // The pointer's pictures run in turn, as the original's do (the hand over a thing to pick up grabs).
        let pointer_frame = state.pointers.get(pointing).map_or(0, |(pictures, shows)| (time / shows) as usize % pictures.len().max(1));
        if pointing != state.pointing || pointer_frame != state.pointer_frame {
            state.pointing = pointing;
            state.pointer_frame = pointer_frame;
            if let Some(pointer) = state.pointers.get(pointing).and_then(|(pictures, _)| pictures.get(pointer_frame)) {
                let _ = gpu.canvas.style().set_property("cursor", pointer);
            }
        }
        match &over {
            Some((_, id, unit, feet)) if !unit.name.is_empty() => {
                state.named = Some(*id);
                // 20 pixels of a screen 480 high under the feet, nearer when the camera is.
                let below = height * 20.0 / 480.0 * DISTANCE / scene.camera.distance;
                name_shown(Some((&unit.name, unit.lively, (feet.0, feet.1 + below))));
            }
            // With nothing pointed at, the name of the monster being fought stays (the client's lock-on).
            _ if state.chase.and_then(|(id, ..)| scene.units.get(&id)).is_some_and(|unit| !unit.name.is_empty() && unit.dying.is_none()) => {
                if let Some(unit) = state.chase.and_then(|(id, ..)| scene.units.get(&id)) {
                    if let Some(feet) = view.project(on_ground(&scene.gat, unit.at)) {
                        state.named = state.chase.map(|(id, ..)| id);
                        let below = height * 20.0 / 480.0 * DISTANCE / scene.camera.distance;
                        name_shown(Some((&unit.name, unit.lively, (feet.0 * width, feet.1 * height + below))));
                    }
                }
            }
            _ if state.named.take().is_some() => name_shown(None),
            _ => {}
        }

        // While the left button is held on the ground, the character keeps walking to where the pointer is.
        let follow = state.held.filter(|_| time - state.held_asked > 250.0).and_then(|held| {
            let picked = view.pick(&scene.gat, (held.0 / width.max(1.0), held.1 / height.max(1.0)))?;
            let cell = scene.player.cell;
            let goal = if scene.gat.path(cell, picked).is_empty() { scene.gat.reachable_towards(cell, picked)? } else { picked };
            (goal != cell).then_some(goal)
        });
        if follow.is_some() {
            state.held_asked = time;
        }
        let mut follow = follow.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));

        // A thing to pick up is walked to, and picked up from beside it.
        let mut take = None;
        if let Some((id, looked)) = state.fetch {
            match scene.drops.get(&id) {
                None => state.fetch = None,
                Some((cell, ..)) if time - looked > 350.0 => {
                    let own = scene.player.cell;
                    if (cell.0 - own.0).abs().max((cell.1 - own.1).abs()) <= 1 {
                        take = state.hooks.as_ref().map(|hooks| (hooks.take, id));
                    } else if scene.player.walk.is_none() {
                        let goal = if scene.gat.path(own, *cell).is_empty() { scene.gat.reachable_towards(own, *cell) } else { Some(*cell) };
                        follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
                    }
                    state.fetch = Some((id, time));
                }
                Some(_) => {}
            }
        }
        // The monster being fought is run after when it is out of reach, and struck again once it is near.
        let mut strike = None;
        if let Some((id, looked, told)) = state.chase {
            match scene.units.get(&id) {
                None => state.chase = None,
                Some(unit) if time - looked > 400.0 => {
                    let cell = scene.player.cell;
                    // Near enough is what the server said the weapon reaches. Until it has said (it does when a
                    // blow is asked for from too far), the character stays where it is: a bow reaches far, and
                    // its archer is not to run up to the monster.
                    let range = state.strike_range.unwrap_or(30).max(1);
                    let far = (unit.cell.0 - cell.0).abs().max((unit.cell.1 - cell.1).abs()) > range;
                    if far {
                        // The way to the monster, as far as the cell from which the weapon reaches it.
                        let way = scene.gat.path(cell, unit.cell);
                        let goal = if way.is_empty() { scene.gat.reachable_towards(cell, unit.cell) } else { way.get(way.len().saturating_sub(1 + range as usize).min(way.len() - 1)).copied().filter(|goal| *goal != cell).or(Some(unit.cell)) };
                        follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
                    } else if !told || (scene.player.walk.is_none() && time - state.own_struck > 1500.0 && time - state.strike_asked > 1500.0) {
                        // Near enough: the server is told to strike. It is told again when nothing came of it
                        // (it drops a request that reaches it while the character still ends something else,
                        // as after the player changed targets quickly), so the character does not stand idle.
                        strike = state.hooks.as_ref().map(|hooks| (hooks.press, id));
                        state.strike_asked = time;
                    }
                    state.chase = Some((id, time, !far));
                }
                Some(_) => {}
            }
        }
        // A skill aimed out of its reach: the character walks towards whom or where it was aimed, and uses it
        // as soon as it is near enough. The client's rule (`CPlayer::UpdatePendingAction`): the skill's reach in
        // cells of 5, and 5.5 more, is at least the distance.
        let mut used = None;
        if let Some((whom, cell, range, looked)) = state.reach {
            let aimed = match whom {
                Some(id) => scene.units.get(&id).filter(|unit| unit.dying.is_none()).map(|unit| (unit.cell, unit.at)),
                None => Some((cell, (cell.0 as f32, cell.1 as f32))),
            };
            match aimed {
                None => state.reach = None,
                Some((cell, at)) => {
                    let (across, along) = (at.0 - scene.player.at.0, at.1 - scene.player.at.1);
                    if (across * across + along * along).sqrt() * 5.0 <= range * 5.0 + 5.5 {
                        used = state.hooks.as_ref().map(|hooks| (hooks.aimed, whom, (cell.0.max(0) as u16, cell.1.max(0) as u16)));
                        state.reach = None;
                    } else if time - looked > 350.0 {
                        if scene.player.walk.is_none() {
                            let own = scene.player.cell;
                            let goal = if scene.gat.path(own, cell).is_empty() { scene.gat.reachable_towards(own, cell) } else { Some(cell) };
                            follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
                        }
                        state.reach = Some((whom, cell, range, time));
                    }
                }
            }
        }

        let cell = scene.player.cell;
        let changed = cell != scene.told;
        scene.told = cell;
        Some((state.hooks.as_ref().filter(|_| changed).map(|hooks| (hooks.moved, cell)), follow, strike, take, used))
    });
    if let Some((moved, follow, strike, take, used)) = told {
        if let Some((tell, whom, cell)) = used {
            tell(whom, cell);
        }
        if let Some((take, id)) = take {
            take(id);
        }
        if let Some((press, id)) = strike {
            press(id, true);
        }
        if let Some((moved, cell)) = moved {
            moved(cell.0.max(0) as u16, cell.1.max(0) as u16);
        }
        if let Some((walk_to, goal)) = follow {
            walk_to(goal.0 as u16, goal.1 as u16);
        }
    }
    if STATE.with_borrow(|state| state.scene.is_some()) {
        next_frame();
    }
}

fn next_frame() {
    STATE.with_borrow_mut(|state| {
        let frame = state.frame.get_or_insert_with(|| Closure::<dyn FnMut()>::new(frame));
        if let Some(window) = web_sys::window() {
            let _ = window.request_animation_frame(frame.as_ref().unchecked_ref());
        }
    });
}

/// The mouse on the canvas: a left press walks, the right button turns the camera, the wheel moves it.
fn listen() {
    if STATE.with_borrow_mut(|state| std::mem::replace(&mut state.listening, true)) {
        return;
    }
    let Ok(canvas) = canvas() else { return };
    let position = |event: &MouseEvent, canvas: &HtmlCanvasElement| {
        (event.offset_x() as f32 / canvas.client_width().max(1) as f32, event.offset_y() as f32 / canvas.client_height().max(1) as f32)
    };

    let pressed = {
        let canvas = canvas.clone();
        Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
            let at = position(&event, &canvas);
            // The browser does nothing of its own with a press on the world: no text marked, no scrolling by the
            // middle button, no page left by the side buttons.
            event.prevent_default();
            match event.button() {
                0 if STATE.with_borrow(|state| state.aiming.is_some()) => {
                    // A skill is being aimed: the press picks whom, or where, it is used on. Whatever is
                    // pressed, the aiming is over with it, as in the client (a skill for someone pressed on the
                    // bare ground is not used).
                    let aimed = STATE.with_borrow_mut(|state| {
                        let ground = state.aiming.take() == Some(true);
                        let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
                        let view = scene.camera.view(on_ground(&scene.gat, scene.player.at), gpu.config.width as f32 / gpu.config.height as f32);
                        let pointer = (event.client_x() as f32, event.client_y() as f32);
                        let aimed = if ground {
                            let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
                            let cell = view.pick(&scene.gat, (pointer.0 / width.max(1.0), pointer.1 / height.max(1.0)))?;
                            (None, (cell.0.max(0) as u16, cell.1.max(0) as u16))
                        } else {
                            let (id, _) = pointed_at(gpu, scene, &view, pointer)?;
                            let unit = scene.units.get(&id)?;
                            (Some(id), (unit.cell.0.max(0) as u16, unit.cell.1.max(0) as u16))
                        };
                        // The skill is used from within its reach: the character walks there first if it must
                        // (see the frame's code).
                        state.chase = None;
                        state.fetch = None;
                        state.reach = Some((aimed.0, (i32::from(aimed.1.0), i32::from(aimed.1.1)), state.aim_range, 0.0));
                        Some(())
                    });
                    let _ = aimed;
                }
                // The right button lets go of a skill being aimed.
                2 if STATE.with_borrow(|state| state.aiming.is_some()) => STATE.with_borrow_mut(|state| state.aiming = None),
                0 => {
                    // A press on someone talks to them or fights them; a press on the ground walks there.
                    let pressed = STATE.with_borrow(|state| {
                        let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
                        let view = scene.camera.view(on_ground(&scene.gat, scene.player.at), gpu.config.width as f32 / gpu.config.height as f32);
                        let (id, _) = pointed_at(gpu, scene, &view, (event.client_x() as f32, event.client_y() as f32))?;
                        let unit = scene.units.get(&id).filter(|unit| matches!(unit.look, Look::Creature(_)))?;
                        // A thing lying at an NPC's feet is picked up, not the NPC talked to.
                        if !unit.lively && lying_at(gpu, scene, &view, (event.client_x() as f32, event.client_y() as f32)).is_some() {
                            return None;
                        }
                        state.hooks.as_ref().map(|hooks| (hooks.press, id, unit.lively))
                    });
                    if let Some((press, id, monster)) = pressed {
                        // A monster is run after until it is down or the player presses somewhere else.
                        STATE.with_borrow_mut(|state| {
                            state.chase = monster.then(|| (id, now(), true));
                            state.strike_asked = now();
                            state.reach = None;
                        });
                        press(id, monster);
                        return;
                    }
                    // A press on a thing lying on the ground goes to it and picks it up.
                    let lying = STATE.with_borrow(|state| {
                        let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
                        let view = scene.camera.view(on_ground(&scene.gat, scene.player.at), gpu.config.width as f32 / gpu.config.height as f32);
                        lying_at(gpu, scene, &view, (event.client_x() as f32, event.client_y() as f32))
                    });
                    if let Some(id) = lying {
                        STATE.with_borrow_mut(|state| {
                            state.chase = None;
                            state.fetch = Some((id, 0.0));
                        });
                        return;
                    }
                    STATE.with_borrow_mut(|state| {
                        state.fetch = None;
                        state.chase = None;
                        state.reach = None;
                        state.held = Some((event.client_x() as f32, event.client_y() as f32));
                        state.held_asked = now();
                    });
                    let request = STATE.with_borrow(|state| {
                        let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
                        let player = &scene.player;
                        let feet = on_ground(&scene.gat, (player.cell.0 as f32, player.cell.1 as f32));
                        let view = scene.camera.view(feet, gpu.config.width as f32 / gpu.config.height as f32);
                        let picked = view.pick(&scene.gat, at)?;
                        // A press on a wall, or too far away, walks as far towards it as the way is open.
                        let goal = if scene.gat.path(player.cell, picked).is_empty() { scene.gat.reachable_towards(player.cell, picked)? } else { picked };
                        state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal))
                    });
                    if let Some((walk_to, goal)) = request {
                        walk_to(goal.0 as u16, goal.1 as u16);
                    }
                }
                2 => STATE.with_borrow_mut(|state| {
                    let pressed = now();
                    if pressed - state.right_pressed < 350.0 {
                        if let Some(scene) = &mut state.scene {
                            scene.camera = Camera { distance: scene.camera.distance, ..Camera::starting(scene.indoor) };
                            if !scene.indoor {
                        scene.camera.remember();
                    }
                        }
                    }
                    state.right_pressed = pressed;
                    state.turning = Some((event.client_x() as f32, event.client_y() as f32));
                }),
                _ => {}
            }
        })
    };
    let moved = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| {
        // With no button held, the pointer shows the name of whoever it is on.
        if event.buttons() == 0 {
            let over_world = event.target().and_then(|target| target.dyn_into::<web_sys::Element>().ok()).is_some_and(|target| target.id() == "game");
            STATE.with_borrow_mut(|state| state.pointer = over_world.then(|| (event.client_x() as f32, event.client_y() as f32)));
            return;
        }
        STATE.with_borrow_mut(|state| {
            state.pointer = None;
            if state.held.is_some() {
                state.held = if event.buttons() & 1 != 0 { Some((event.client_x() as f32, event.client_y() as f32)) } else { None };
            }
        });
        STATE.with_borrow_mut(|state| {
            let (Some(last), Some(scene)) = (state.turning, &mut state.scene) else { return };
            // The right button was let go outside the page.
            if event.buttons() & 2 == 0 {
                state.turning = None;
                return;
            }
            let (x, y) = (event.client_x() as f32, event.client_y() as f32);
            let (pitches, distances) = Camera::limits(scene.indoor);
            let indoor = scene.indoor;
            let camera = &mut scene.camera;
            if event.shift_key() {
                camera.pitch = (camera.pitch + (y - last.1) * 0.3).clamp(pitches[0], pitches[1]);
            } else if event.ctrl_key() {
                camera.distance = (camera.distance + (y - last.1) * 1.5).clamp(distances[0], distances[1]);
            } else if !indoor {
                camera.yaw = (camera.yaw + (x - last.0) * 0.4).rem_euclid(360.0);
            }
            state.turning = Some((x, y));
        });
    });
    let released = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| {
        if event.button() == 0 {
            STATE.with_borrow_mut(|state| state.held = None);
        }
        if event.button() == 2 {
            STATE.with_borrow_mut(|state| {
                state.turning = None;
                if let Some(scene) = &state.scene {
                    if !scene.indoor {
                        scene.camera.remember();
                    }
                }
            });
        }
        // The side buttons would leave the page for the one before or after it.
        if event.button() > 2 && STATE.with_borrow(|state| state.scene.is_some()) {
            event.prevent_default();
        }
    });
    let side = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| {
        if event.button() != 0 && event.button() != 2 && STATE.with_borrow(|state| state.scene.is_some()) {
            event.prevent_default();
        }
    });
    // Holding Ctrl, the wheel (and a pinch on a touchpad) would make the whole page larger or smaller.
    let page_zoom = Closure::<dyn FnMut(WheelEvent)>::new(|event: WheelEvent| {
        if event.ctrl_key() && STATE.with_borrow(|state| state.scene.is_some()) {
            event.prevent_default();
        }
    });
    let wheel = Closure::<dyn FnMut(WheelEvent)>::new(|event: WheelEvent| {
        event.prevent_default();
        STATE.with_borrow_mut(|state| {
            if let Some(scene) = &mut state.scene {
                // A notch of a mouse's wheel is about a hundred pixels, and one step of the camera. A touchpad sends
                // many small moves, a finger resting on it during a click among them: those move it as little.
                let pixels = event.delta_y() * [1.0, 16.0, 400.0][event.delta_mode().min(2) as usize];
                let step = (pixels * 0.15).clamp(-15.0, 15.0) as f32;
                if pixels.abs() >= 4.0 {
                    let (_, distances) = Camera::limits(scene.indoor);
                    scene.camera.distance = (scene.camera.distance + step).clamp(distances[0], distances[1]);
                    if !scene.indoor {
                        scene.camera.remember();
                    }
                }
            }
        });
    });
    // The right button is the camera's: the browser's menu stays away.
    let menu = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| event.prevent_default());

    let window = web_sys::window();
    let _ = canvas.add_event_listener_with_callback("mousedown", pressed.as_ref().unchecked_ref());
    let _ = canvas.add_event_listener_with_callback("contextmenu", menu.as_ref().unchecked_ref());
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(false);
    let _ = canvas.add_event_listener_with_callback_and_add_event_listener_options("wheel", wheel.as_ref().unchecked_ref(), &options);
    if let Some(window) = &window {
        let _ = window.add_event_listener_with_callback("mousemove", moved.as_ref().unchecked_ref());
        let _ = window.add_event_listener_with_callback("mouseup", released.as_ref().unchecked_ref());
        let _ = window.add_event_listener_with_callback("mousedown", side.as_ref().unchecked_ref());
        let _ = window.add_event_listener_with_callback("auxclick", side.as_ref().unchecked_ref());
        let _ = window.add_event_listener_with_callback_and_add_event_listener_options("wheel", page_zoom.as_ref().unchecked_ref(), &options);
    }
    // These live as long as the page.
    pressed.forget();
    side.forget();
    page_zoom.forget();
    moved.forget();
    released.forget();
    wheel.forget();
    menu.forget();
}

/// Who a point of the page is on: the nearest of those whose picture it touches, and where on the page their
/// feet are.
fn pointed_at(gpu: &Gpu, scene: &Scene, view: &View, pointer: (f32, f32)) -> Option<(u32, (f32, f32))> {
    let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
    scene
        .units
        .iter()
        .filter_map(|(id, unit)| {
            if unit.dying.is_some() {
                return None;
            }
            let texture = gpu.sprite_textures.get(unit.shown.as_ref()?)?;
            let feet = on_ground(&scene.gat, unit.at);
            let corner = |x: f32, y: f32| view.project(along(along(feet, view.right, (x - texture.feet.0) * SPRITE_SCALE), view.up, (texture.feet.1 - y) * SPRITE_SCALE));
            let (low, high) = (corner(0.0, 0.0)?, corner(texture.size.0, texture.size.1)?);
            let inside = (low.0 * width..=high.0 * width).contains(&pointer.0) && (low.1 * height..=high.1 * height).contains(&pointer.1);
            let under = view.project(feet)?;
            inside.then(|| (dot(subtract(feet, view.eye), view.forward), *id, (under.0 * width, under.1 * height)))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id, feet)| (id, feet))
}

/// The thing on the ground a point of the page is on.
fn lying_at(gpu: &Gpu, scene: &Scene, view: &View, pointer: (f32, f32)) -> Option<u32> {
    let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
    scene.drops.iter().find_map(|(id, (_, _, at))| {
        let place = view.project(on_ground(&scene.gat, *at))?;
        let (across, up) = ((pointer.0 - place.0 * width).abs(), place.1 * height - pointer.1);
        (across < 16.0 && (-8.0..26.0).contains(&up)).then_some(*id)
    })
}

/// Shows a name on the page, its middle at a place, or takes it away. Monsters' names are pink, as the original's.
fn name_shown(named: Option<(&str, bool, (f32, f32))>) {
    let Some(label) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("unit-name")) else { return };
    match named {
        Some((name, monster, (x, y))) => {
            if label.text_content().as_deref() != Some(name) {
                label.set_text_content(Some(name));
            }
            let colour = if monster { "#ffc3c3" } else { "#fff" };
            let _ = label.set_attribute("style", &format!("display:block;left:{x:.0}px;top:{y:.0}px;color:{colour}"));
        }
        None => {
            let _ = label.set_attribute("style", "display:none");
        }
    }
}

/// The size, in cells, of the map shown.
pub fn map_size() -> Option<(usize, usize)> {
    STATE.with_borrow(|state| state.scene.as_ref().map(|scene| (scene.gat.width, scene.gat.height)))
}
