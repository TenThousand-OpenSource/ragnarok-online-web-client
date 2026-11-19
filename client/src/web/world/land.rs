//! The models and the water of a map on the GPU: read from the map's files, and what of them moves moved on
//! every frame: the models whose parts turn (`C3dActorV1::Update`), each lit anew as it stands, and the waves
//! of the water (`GroundGeometry::UpdateWater`, `DrawWaterCell`).

use super::*;
use crate::model::{Light, Placement};

/// How many pictures the water runs through, and how many world units one of them covers.
pub(super) const WATER_FRAMES: usize = 32;

/// The corners of the triangles drawn with one picture: so many corners, each its place, where of the picture
/// it shows, the light on it and the way its face looks out (`CORNER` numbers); and whether the picture has
/// an alpha of its own, to be mixed with what is behind it (a TGA: the client's `A4R4G4B4` textures; a BMP is
/// cut out by its magenta).
pub(super) struct Batch {
    pub(super) vertices: wgpu::Buffer,
    pub(super) count: u32,
    pub(super) bind: wgpu::BindGroup,
    pub(super) blends: bool,
}

/// How many numbers a corner of a model is.
const CORNER: usize = 11;
/// Within how many world units of where the camera looks a model's animation goes on: farther off, where it
/// is out of the view, it holds still.
const ANIMATED_WITHIN: f32 = 500.0;

/// A model standing on the map whose parts turn: the model, where it stands, the frame it was last drawn at,
/// and its triangles by texture.
struct Moving {
    model: Rc<Rsm>,
    placement: Placement,
    at: Vector,
    shown: i32,
    batches: Vec<(usize, Batch)>,
}

/// A model read to be put on the map while it is played on (a trap's: `CSkill::ShowModel`): the model and its
/// pictures on the GPU, in the order the model names them.
pub(super) struct Raisable {
    model: Rsm,
    pictures: Vec<(wgpu::TextureView, bool)>,
}

/// A model put on the map so: what it is, where and in which light it stands, the frame it shows, and its
/// triangles by texture.
struct Raised {
    of: Rc<Raisable>,
    placement: Placement,
    light: Light,
    shown: i32,
    batches: Vec<(usize, Batch)>,
}

/// The map's models: those that stand still, sorted by picture, those that move, and those put on it since it
/// was read, by whose they are (a unit of a skill).
pub(super) struct Models {
    pub(super) still: Vec<Batch>,
    moving: Vec<Moving>,
    raised: HashMap<u32, Raised>,
    light: Light,
    map_size: (f32, f32),
}

impl Models {
    /// A map with no models.
    pub(super) fn none() -> Self {
        Self { still: Vec::new(), moving: Vec::new(), raised: HashMap::new(), light: Light { direction: [0.0, 1.0, 0.0], diffuse: [1.0; 3], ambient: [0.3; 3] }, map_size: (0.0, 0.0) }
    }

    /// Every batch to draw, the still ones first.
    pub(super) fn batches(&self) -> impl Iterator<Item = &Batch> {
        let raised = self.raised.values().flat_map(|raised| raised.batches.iter().map(|(_, batch)| batch));
        self.still.iter().chain(self.moving.iter().flat_map(|moving| moving.batches.iter().map(|(_, batch)| batch))).chain(raised)
    }

    /// Whether a model stands on the map for `id`.
    pub(super) fn raised(&self, id: u32) -> bool {
        self.raised.contains_key(&id)
    }

    /// Puts a model on the map for `id`, at a place of the world and a frame of its animation, lit by `light`
    /// (`CSkill::ShowModel`: not turned, of its own size). One that stood there for `id` is gone.
    pub(super) fn raise(&mut self, gpu: &Gpu, id: u32, of: &Rc<Raisable>, at: Vector, light: Light, frame: i32) {
        // A placement is in the files' space: from the map's middle, y down.
        let placement = Placement { model: String::new(), position: [at[0] - self.map_size.0 / 2.0, -at[1], at[2] - self.map_size.1 / 2.0], rotation: [0.0; 3], scale: [1.0; 3], playback: 0, speed: 1.0 };
        let placed = model::place(&of.model.at(frame), of.model.shade, &placement, self.map_size, &light);
        let layout = gpu.model_pipelines[0].get_bind_group_layout(1);
        let batches = of.pictures.iter().enumerate().map(|(texture, (view, blends))| (texture, corners(&placed, texture), view, *blends)).filter(|(_, floats, _, _)| !floats.is_empty());
        let batches = batches.map(|(texture, floats, view, blends)| (texture, batch(gpu, &layout, view, blends, &floats, wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST))).collect();
        self.raised.insert(id, Raised { of: of.clone(), placement, light, shown: frame, batches });
    }

    /// The model standing for `id` shows the frame of its animation that `frame` makes of how many frames the
    /// animation has. Nothing for an ID no model stands for.
    pub(super) fn raised_frame(&mut self, gpu: &Gpu, id: u32, frame: impl Fn(i32) -> i32) {
        let Some(raised) = self.raised.get_mut(&id) else { return };
        let frame = frame(raised.of.model.length);
        if frame == raised.shown {
            return;
        }
        raised.shown = frame;
        let placed = model::place(&raised.of.model.at(frame), raised.of.model.shade, &raised.placement, self.map_size, &raised.light);
        for (texture, batch) in &raised.batches {
            gpu.queue.write_buffer(&batch.vertices, 0, bytemuck::cast_slice(&corners(&placed, *texture)));
        }
    }

    /// The model standing for `id` is taken off the map.
    pub(super) fn lower(&mut self, id: u32) {
        self.raised.remove(&id);
    }

    /// Moves the models near `aim` on to the frame of their animation that `time` is at.
    pub(super) fn animate(&mut self, gpu: &Gpu, aim: Vector, time: f64) {
        let elapsed = (time / scenery::CLIENT_FRAME_MS) as u64;
        for moving in &mut self.moving {
            let frame = model::frame_at(elapsed, moving.placement.speed, moving.model.length, moving.placement.playback);
            if frame == moving.shown || f32::hypot(moving.at[0] - aim[0], moving.at[2] - aim[2]) > ANIMATED_WITHIN {
                continue;
            }
            moving.shown = frame;
            let placed = model::place(&moving.model.at(frame), moving.model.shade, &moving.placement, self.map_size, &self.light);
            for (texture, batch) in &moving.batches {
                gpu.queue.write_buffer(&batch.vertices, 0, bytemuck::cast_slice(&corners(&placed, *texture)));
            }
        }
    }
}

impl Raisable {
    /// How many frames the model's animation has.
    pub(super) fn length(&self) -> i32 {
        self.model.length
    }

    /// A model and the pictures read for it (`read_raisable`), put on the GPU; a picture that could not be read
    /// is grey.
    pub(super) fn new(gpu: &Gpu, model: Rsm, pictures: Vec<Option<bmp::Image>>) -> Self {
        let pictures = model.textures.iter().zip(pictures).map(|(name, picture)| (picture.map_or_else(|| gpu.texture(1, 1, &[160, 160, 160, 255]), |picture| gpu.texture(picture.width, picture.height, &picture.rgba)), blends(name))).collect();
        Self { model, pictures }
    }
}

/// Reads a model by its path under `data\model\`, and its pictures in the order it names them, for
/// `Raisable::new`; nothing for one that cannot be read.
pub(super) async fn read_raisable(name: &str) -> Option<(Rsm, Vec<Option<bmp::Image>>)> {
    let korean = web_sys::TextDecoder::new_with_label("euc-kr").ok()?;
    let decode = |raw: &[u8]| korean.decode_with_u8_array(raw).unwrap_or_else(|_| String::from_utf8_lossy(raw).into_owned());
    let model = fetched(&BTreeSet::from([name]), &decode).await.remove(name)?;
    let pictures = futures_util::future::join_all(model.textures.iter().map(|name| async move { picture(name).await.ok() })).await;
    Rc::try_unwrap(model).ok().map(|model| (model, pictures))
}

/// Whether a model's picture has an alpha of its own, to be mixed with what is behind it (a TGA; a BMP is cut
/// out by its magenta).
fn blends(name: &str) -> bool {
    !name.to_ascii_lowercase().ends_with(".bmp")
}

/// The triangles drawn with one picture, on the GPU.
fn batch(gpu: &Gpu, layout: &wgpu::BindGroupLayout, view: &wgpu::TextureView, blends: bool, floats: &[f32], usage: wgpu::BufferUsages) -> Batch {
    Batch {
        vertices: gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(floats), usage }),
        count: (floats.len() / CORNER) as u32,
        bind: gpu.textured(layout, view, &gpu.repeating),
        blends,
    }
}

/// The corners of the triangles of one texture, as numbers for the GPU.
fn corners(placed: &[(usize, [model::Vertex; 3])], texture: usize) -> Vec<f32> {
    let mut floats = Vec::new();
    for (_, triangle) in placed.iter().filter(|(of, _)| *of == texture) {
        for corner in triangle {
            floats.extend(corner.position);
            floats.extend(corner.uv);
            floats.extend(corner.colour);
            floats.extend(corner.outward);
        }
    }
    floats
}

/// The models of those names, fetched and read; the ones that could not be are left out, with a note.
async fn fetched<'a>(names: &BTreeSet<&'a str>, decode: impl Fn(&[u8]) -> String + Copy) -> BTreeMap<&'a str, Rc<Rsm>> {
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
                read.insert(name, Rc::new(model));
            }
            Err(error) => *failed.entry(error).or_default() += 1,
        }
    }
    for (error, count) in failed {
        web_sys::console::warn_1(&format!("{count} models left out: {error}").into());
    }
    read
}

/// Every model of the map, standing where the map puts it: fetched, put together, lit, and sorted by texture;
/// one whose animation plays (`Placement::playback`, at a speed that moves it at all) by itself.
pub(super) async fn models(gpu: &Gpu, rsw: &Rsw, map_size: (f32, f32), decode: impl Fn(&[u8]) -> String + Copy) -> Models {
    let names: BTreeSet<&str> = rsw.placements.iter().map(|placement| placement.model.as_str()).collect();
    let read = fetched(&names, decode).await;
    // Per texture, the corners of all triangles of the still models drawn with it.
    let mut by_texture: BTreeMap<&str, Vec<f32>> = BTreeMap::new();
    let mut turning = Vec::new();
    for placement in &rsw.placements {
        let Some(model) = read.get(placement.model.as_str()) else { continue };
        let placed = model::place(&model.triangles, model.shade, placement, map_size, &rsw.light);
        if model.animated() && model::frame_at(1, placement.speed, i32::MAX, placement.playback) > 0 {
            turning.push((placement, model.clone(), placed));
            continue;
        }
        for (texture, name) in model.textures.iter().enumerate() {
            by_texture.entry(name.as_str()).or_default().extend(corners(&placed, texture));
        }
    }
    let wanted: BTreeSet<String> = by_texture.keys().map(|name| (*name).to_owned()).chain(turning.iter().flat_map(|(_, model, _)| model.textures.iter().cloned())).collect();
    let pictures: HashMap<String, wgpu::TextureView> = futures_util::future::join_all(wanted.into_iter().map(|name| async move {
        let loaded = picture(&name).await;
        (name, loaded)
    }))
        .await
        .into_iter()
        .map(|(name, loaded)| {
            let view = loaded.map(|picture| gpu.texture(picture.width, picture.height, &picture.rgba)).unwrap_or_else(|error| {
                web_sys::console::warn_1(&format!("texture {name}: {error}").into());
                gpu.texture(1, 1, &[160, 160, 160, 255])
            });
            (name, view)
        })
        .collect();
    let layout = gpu.model_pipelines[0].get_bind_group_layout(1);
    let batch = |name: &str, floats: &[f32], usage: wgpu::BufferUsages| batch(gpu, &layout, &pictures[name], blends(name), floats, usage);
    let still = by_texture.iter().filter(|(_, floats)| !floats.is_empty()).map(|(name, floats)| batch(name, floats, wgpu::BufferUsages::VERTEX)).collect();
    let in_world = |placement: &Placement| [placement.position[0] + map_size.0 / 2.0, -placement.position[1], placement.position[2] + map_size.1 / 2.0];
    let moving = turning
        .into_iter()
        .map(|(placement, model, placed)| {
            let batches = model.textures.iter().enumerate().map(|(texture, name)| (texture, corners(&placed, texture), name)).filter(|(_, floats, _)| !floats.is_empty());
            let batches = batches.map(|(texture, floats, name)| (texture, batch(name, &floats, wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST))).collect();
            Moving { model, placement: placement.clone(), at: in_world(placement), shown: 0, batches }
        })
        .collect();
    Models { still, moving, raised: HashMap::new(), light: rsw.light.clone(), map_size }
}

/// The map's water: its surface over every cube of the ground that has a corner under it, the pictures it
/// runs through, and its waves.
pub(super) struct WaterSurface {
    pub(super) vertices: wgpu::Buffer,
    pub(super) count: u32,
    /// The pictures the surface runs through, one every so many of the client's frames.
    pub(super) frames: Vec<wgpu::BindGroup>,
    pub(super) frame_ticks: u32,
    /// Its waves: their height, the degrees their turn goes on a frame, and the degrees between a cube and
    /// the next. (How high the still surface is, is in its corners.)
    pub(super) wave: (f32, i32, i32),
    /// The colour the pictures are drawn in, and how much of the surface hides what is under it.
    pub(super) colour: [f32; 4],
}

/// The water of a map over its ground (`GroundGeometry::DrawWaterCell`): a square a cube, a picture laid over
/// four cubes each way. Each corner is told the number its wave goes by: the east and the north of it, less one.
pub(super) async fn water(gpu: &Gpu, rsw: &Rsw, gnd: &Gnd) -> Option<WaterSurface> {
    let water = &rsw.water;
    let cubes = gnd.under(water.level + water.wave_height.abs());
    if cubes.is_empty() {
        return None;
    }
    let kind = water.kind;
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
    let mut floats = Vec::with_capacity(cubes.len() * 36);
    for (east, north) in &cubes {
        for (x, z) in [(0, 0), (1, 0), (0, 1), (0, 1), (1, 0), (1, 1)] {
            let (column, row) = (east + x, north + z);
            // The picture's quarter this cube shows: by the cube's own number, as the client counts it.
            let (u, v) = (((east & 3) + x) as f32 * 0.25, ((north & 3) + z) as f32 * 0.25);
            floats.extend([column as f32 * gnd.zoom, water.level, row as f32 * gnd.zoom, u, v, (column + row) as f32 - 1.0]);
        }
    }
    let vertices = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&floats), usage: wgpu::BufferUsages::VERTEX });
    // The water of kind 4 hides the ground whole and is in the colour of the map's ambient light; any other
    // lets the ground show through (the client's 0x90 of 255).
    let colour = if kind == 4 { [rsw.light.ambient[0], rsw.light.ambient[1], rsw.light.ambient[2], 1.0] } else { [1.0, 1.0, 1.0, 144.0 / 255.0] };
    Some(WaterSurface { vertices, count: (floats.len() / 6) as u32, frames, frame_ticks: water.frame_ticks, wave: (water.wave_height, water.wave_speed, water.wave_pitch), colour })
}
