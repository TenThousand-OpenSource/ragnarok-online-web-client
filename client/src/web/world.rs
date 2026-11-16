//! The game world on the canvas: the map's ground in 3D, the player's character on it, the camera, and walking.
//!
//! The camera is the original client's: a 15 degree field of view, 300 units from the character, 45 degrees up,
//! looking north (values as rustnarok-client measured them). The right mouse button turns it, with Shift tilts it,
//! with Ctrl (or the wheel) moves it nearer and farther; a double right click puts it back.
//!
//! The map's models (buildings, trees) stand still, each face lit flatly by the map's sun, and the water is a flat
//! surface: no animated models, smoothed shading or waves yet. Nor is anyone else on the map drawn.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent, WheelEvent};
use wgpu::util::DeviceExt;

use super::figure::{self, Figure};
use super::{fetch_asset, image_url, js_error};
use crate::bmp;
use crate::map::{CELL, Gat, Gnd, Walk};
use crate::model::{self, Rsm, Rsw};

const FIELD_OF_VIEW: f32 = 15.0;
const NEAR: f32 = 10.0;
const FAR: f32 = 5000.0;
const DISTANCE: f32 = 300.0;
const DISTANCE_RANGE: [f32; 2] = [150.0, 500.0];
const PITCH: f32 = 45.0;
const PITCH_RANGE: [f32; 2] = [20.0, 70.0];
/// World units per sprite pixel: about 35 sprite pixels to a cell.
const SPRITE_SCALE: f32 = CELL / 35.0;
/// Where a figure's feet are in its picture (see `figure`).
const FEET: (f32, f32) = (80.0, 137.0);
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

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(picture, picture_sampler, in.uv);
    if (texel.a < 0.02) { discard; }
    return texel;
}
";

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

/// The player's character on the map.
struct Player {
    figure: Figure,
    cell: (i32, i32),
    facing: u8,
    /// The walk under way, and when it started (milliseconds, as `now` counts).
    walk: Option<(Walk, f64)>,
    speed: f64,
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
    sprite_texture: wgpu::Texture,
    sprite_bind: wgpu::BindGroup,
    sprite_vertices: wgpu::Buffer,
    /// The figure whose picture the sprite texture holds.
    sprite_shown: Option<Figure>,
}

struct Scene {
    gat: Gat,
    ground: Ground,
    models: Models,
    water: Option<WaterSurface>,
    player: Player,
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
    /// The map is drawn (or could not be: then with the reason).
    pub ready: fn(Result<(), String>),
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
    });
    if let Ok(canvas) = canvas() {
        let _ = canvas.style().set_property("display", "none");
    }
}

/// The server says the character walks from one cell to another.
pub fn walk(from: (u16, u16), to: (u16, u16)) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let (from, to) = ((i32::from(from.0), i32::from(from.1)), (i32::from(to.0), i32::from(to.1)));
        let path = scene.gat.path(from, to);
        // The server walks its own path. Should this map disagree with it, the character goes straight there.
        let path = if path.is_empty() && from != to { vec![to] } else { path };
        scene.player.walk = Some((Walk::new(from, &path, scene.player.speed), now()));
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

async fn load(map: &str, cell: (u16, u16), figure: Figure) -> Result<(Gpu, Scene), String> {
    let gpu = match STATE.with_borrow_mut(|state| state.gpu.take()) {
        Some(gpu) => gpu,
        None => Gpu::new().await?,
    };
    let gat = Gat::parse(&fetch_asset(&format!("data/{map}.gat")).await?)?;
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
    let player = Player { figure, cell, facing: 0, walk: None, speed: DEFAULT_SPEED };
    Ok((gpu, Scene { gat, ground, models, water, player, camera: Camera::default(), told: (-1, -1) }))
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

        let sprite_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: figure::WIDTH as u32, height: figure::HEIGHT as u32, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        // Sprites are pixel art: shown pixel by pixel, not smoothed.
        let pixel_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let sprite_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &sprite_pipeline.get_bind_group_layout(1),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&sprite_texture.create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&pixel_sampler) },
            ],
        });
        let sprite_vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 6 * 20,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let depth = depth_view(&device, &config);
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
            sprite_texture,
            sprite_bind,
            sprite_vertices,
            sprite_shown: None,
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

    fn draw(&mut self, scene: &Scene, view: &View, feet: Vector, figure: &Figure) {
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&view.matrix));

        // The character's picture, when it has changed and is ready; until then the last one stays.
        if self.sprite_shown.as_ref() != Some(figure) {
            if let Some(picture) = figure::painted(figure) {
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo { texture: &self.sprite_texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                    &picture.rgba,
                    wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * figure::WIDTH as u32), rows_per_image: None },
                    wgpu::Extent3d { width: figure::WIDTH as u32, height: figure::HEIGHT as u32, depth_or_array_layers: 1 },
                );
                self.sprite_shown = Some(figure.clone());
            }
        }
        // A picture standing on the feet, facing the camera, a touch nearer than the ground it stands on.
        let feet = along(feet, view.forward, -2.0);
        let corner = |x: f32, y: f32| along(along(feet, view.right, (x - FEET.0) * SPRITE_SCALE), view.up, (FEET.1 - y) * SPRITE_SCALE);
        let (width, height) = (figure::WIDTH as f32, figure::HEIGHT as f32);
        let mut quad = Vec::with_capacity(30);
        for (x, y) in [(0.0, 0.0), (width, 0.0), (0.0, height), (0.0, height), (width, 0.0), (width, height)] {
            quad.extend(corner(x, y));
            quad.extend([x / width, y / height]);
        }
        self.queue.write_buffer(&self.sprite_vertices, 0, bytemuck::cast_slice(&quad));

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
            if self.sprite_shown.is_some() {
                pass.set_pipeline(&self.sprite_pipeline);
                pass.set_bind_group(0, &self.sprite_camera_bind, &[]);
                pass.set_bind_group(1, &self.sprite_bind, &[]);
                pass.set_vertex_buffer(0, self.sprite_vertices.slice(..));
                pass.draw(0..6, 0..1);
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

impl Scene {
    /// Where the character is now, in cells, which way it faces, and which picture of it to show.
    fn player_now(&mut self, now: f64) -> ((f32, f32), Figure) {
        let player = &mut self.player;
        let (position, action, frame) = match &player.walk {
            Some((walk, started)) => {
                let elapsed = now - started;
                let (position, facing) = walk.at(elapsed);
                player.facing = facing;
                if elapsed >= walk.duration() {
                    player.cell = walk.goal();
                    player.walk = None;
                    (position, 0, 0)
                } else {
                    player.cell = (position.0.round() as i32, position.1.round() as i32);
                    // The legs keep the pace of the walk: a slower walk shows each picture longer.
                    let frame = (elapsed / (WALK_FRAME_MS * player.speed / DEFAULT_SPEED)) as usize % WALK_FRAMES;
                    (position, WALKING, frame)
                }
            }
            None => ((player.cell.0 as f32, player.cell.1 as f32), 0, 0),
        };
        let mut figure = player.figure.clone();
        // The sprite's directions are numbered as the game's, seen by a camera looking north: a camera turned
        // around the character sees it from another side.
        let turned = (f32::from(player.facing) * 45.0 - self.camera.yaw).rem_euclid(360.0);
        figure.action = action + ((turned / 45.0).round() as usize % 8);
        figure.frame = frame;
        (position, figure)
    }
}

/// Draws one frame, and asks for the next while there is a world to draw.
fn frame() {
    let told = STATE.with_borrow_mut(|state| {
        let (Some(gpu), Some(scene)) = (&mut state.gpu, &mut state.scene) else { return None };
        gpu.fit();
        let ((x, y), figure) = scene.player_now(now());
        // Cell centres: a character on cell 3 stands at 3.5 cells.
        let feet = [(x + 0.5) * CELL, scene.gat.height_at(x + 0.5, y + 0.5), (y + 0.5) * CELL];
        let view = scene.camera.view(feet, gpu.config.width as f32 / gpu.config.height as f32);
        gpu.draw(scene, &view, feet, &figure);

        let cell = scene.player.cell;
        let changed = cell != scene.told;
        scene.told = cell;
        state.hooks.as_ref().filter(|_| changed).map(|hooks| (hooks.moved, cell))
    });
    if let Some((moved, cell)) = told {
        moved(cell.0.max(0) as u16, cell.1.max(0) as u16);
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
            match event.button() {
                0 => {
                    let request = STATE.with_borrow(|state| {
                        let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
                        let player = &scene.player;
                        let feet = [(player.cell.0 as f32 + 0.5) * CELL, scene.gat.height_at(player.cell.0 as f32 + 0.5, player.cell.1 as f32 + 0.5), (player.cell.1 as f32 + 0.5) * CELL];
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
                            scene.camera = Camera::default();
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
        STATE.with_borrow_mut(|state| {
            let (Some(last), Some(scene)) = (state.turning, &mut state.scene) else { return };
            // The right button was let go outside the page.
            if event.buttons() & 2 == 0 {
                state.turning = None;
                return;
            }
            let (x, y) = (event.client_x() as f32, event.client_y() as f32);
            let camera = &mut scene.camera;
            if event.shift_key() {
                camera.pitch = (camera.pitch + (y - last.1) * 0.3).clamp(PITCH_RANGE[0], PITCH_RANGE[1]);
            } else if event.ctrl_key() {
                camera.distance = (camera.distance + (y - last.1) * 1.5).clamp(DISTANCE_RANGE[0], DISTANCE_RANGE[1]);
            } else {
                camera.yaw = (camera.yaw + (x - last.0) * 0.4).rem_euclid(360.0);
            }
            state.turning = Some((x, y));
        });
    });
    let released = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| {
        if event.button() == 2 {
            STATE.with_borrow_mut(|state| state.turning = None);
        }
    });
    let wheel = Closure::<dyn FnMut(WheelEvent)>::new(|event: WheelEvent| {
        event.prevent_default();
        STATE.with_borrow_mut(|state| {
            if let Some(scene) = &mut state.scene {
                scene.camera.distance = (scene.camera.distance + event.delta_y().signum() as f32 * 15.0).clamp(DISTANCE_RANGE[0], DISTANCE_RANGE[1]);
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
    }
    // These live as long as the page.
    pressed.forget();
    moved.forget();
    released.forget();
    wheel.forget();
    menu.forget();
}

/// The size, in cells, of the map shown.
pub fn map_size() -> Option<(usize, usize)> {
    STATE.with_borrow(|state| state.scene.as_ref().map(|scene| (scene.gat.width, scene.gat.height)))
}
