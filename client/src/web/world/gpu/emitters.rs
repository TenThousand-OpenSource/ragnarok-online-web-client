//! The particle emitters on the GPU (`world/scenery/emitters.rs` works their corners out): a pipeline for each
//! pair of blends the emitters of the maps seen so far ask for, their pictures, and this frame's corners.

use super::*;
use crate::web::world::scenery::EmitterCorners as Corners;

/// What the GPU keeps for them; made when the first particle is drawn.
pub(in crate::web::world) struct Kept {
    /// By Direct3D's two blends: the pipeline that mixes so, and the camera for it.
    pipelines: HashMap<[i32; 2], (wgpu::RenderPipeline, wgpu::BindGroup)>,
    /// The pictures by name, and each for the pipeline of a pair of blends.
    textures: HashMap<String, wgpu::TextureView>,
    pictures: HashMap<(String, [i32; 2]), wgpu::BindGroup>,
    vertices: wgpu::Buffer,
    /// How many corners the buffer holds.
    room: usize,
    batches: Vec<(String, [i32; 2], std::ops::Range<u32>)>,
}

/// A corner is a place, where of the picture it is, and a colour: nine numbers.
const CORNER: u64 = 36;

/// One of Direct3D's blends (`D3DBLEND`) by its number, as a particle carries it (`sourceBlend`,
/// `destinationBlend`). What is told by the alpha of what is drawn onto (7, 8) is taken as all and none of it:
/// the client's screen keeps no alpha. A number that is no blend is "one".
fn factor(blend: i32) -> wgpu::BlendFactor {
    use wgpu::BlendFactor::*;
    match blend {
        1 | 8 => Zero,
        3 => Src,
        4 => OneMinusSrc,
        5 => SrcAlpha,
        6 => OneMinusSrcAlpha,
        9 => Dst,
        10 => OneMinusDst,
        11 => SrcAlphaSaturated,
        _ => One,
    }
}

impl Gpu {
    fn emitter_buffer(&self, room: usize) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: room as u64 * CORNER,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// The pipeline that mixes a particle into what is behind it by those two blends: the effects' program
    /// (a picture times a colour, nothing drawn where that has no alpha: the client's alpha test), tried
    /// against the depth and not written into it. The alpha of what is drawn onto is left as it is.
    fn emitter_pipeline(&self, [source, destination]: [i32; 2]) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
        let attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4];
        let blend = wgpu::BlendState {
            color: wgpu::BlendComponent { src_factor: factor(source), dst_factor: factor(destination), operation: wgpu::BlendOperation::Add },
            alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::Zero, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
        };
        let pipeline = pipeline(&self.device, self.config.format, EFFECT_SHADER, &attributes, CORNER, blend, false);
        let camera = camera_bind(&self.device, &self.camera_buffer, &pipeline);
        (pipeline, camera)
    }

    /// This frame's corners are put on the GPU, with the pipelines and the pictures they need that are not
    /// there yet; `read` has the pictures the map's emitters were read with.
    pub(in crate::web::world) fn emitters_loaded(&mut self, read: &HashMap<String, bmp::Image>, corners: Corners) {
        let count = corners.numbers.len() / 9;
        if count == 0 && self.emitters.is_none() {
            return;
        }
        let mut kept = self.emitters.take().unwrap_or_else(|| Kept {
            pipelines: HashMap::new(),
            textures: HashMap::new(),
            pictures: HashMap::new(),
            vertices: self.emitter_buffer(count.next_power_of_two().max(1024)),
            room: count.next_power_of_two().max(1024),
            batches: Vec::new(),
        });
        if count > kept.room {
            kept.room = count.next_power_of_two();
            kept.vertices = self.emitter_buffer(kept.room);
        }
        kept.batches.clear();
        for (emitter, range) in corners.batches {
            let key = (emitter.picture, emitter.blend);
            if !kept.pictures.contains_key(&key) {
                let Some(image) = read.get(&key.0) else { continue };
                let (pipeline, _) = kept.pipelines.entry(key.1).or_insert_with(|| self.emitter_pipeline(key.1));
                let texture = kept.textures.entry(key.0.clone()).or_insert_with(|| self.texture(image.width, image.height, &image.rgba));
                let bind = self.textured(&pipeline.get_bind_group_layout(1), texture, &self.sampler);
                kept.pictures.insert(key.clone(), bind);
            }
            kept.batches.push((key.0, key.1, range));
        }
        if count > 0 {
            self.queue.write_buffer(&kept.vertices, 0, bytemuck::cast_slice(&corners.numbers));
        }
        self.emitters = Some(kept);
    }

    /// The particles of this frame, each emitter's with its picture and its blends.
    pub(super) fn draw_emitters(&self, pass: &mut wgpu::RenderPass) {
        let Some(kept) = self.emitters.as_ref().filter(|kept| !kept.batches.is_empty()) else { return };
        pass.set_vertex_buffer(0, kept.vertices.slice(..));
        for (picture, blend, corners) in &kept.batches {
            let (Some((pipeline, camera)), Some(bind)) = (kept.pipelines.get(blend), kept.pictures.get(&(picture.clone(), *blend))) else { continue };
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, camera, &[]);
            pass.set_bind_group(1, bind, &[]);
            pass.draw(corners.clone(), 0..1);
        }
    }
}
