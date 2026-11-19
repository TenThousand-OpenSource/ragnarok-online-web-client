//! RSM: a model of the classic maps (versions 1.1 to 1.5): its parts (nodes), each with its mesh and the keys of
//! its turning, put together as they stand at a frame of the model's animation.
//!
//! The layout and the way the parts are put together follow korangar (`ragnarok-formats`, `loaders/model`; MIT);
//! the turning of the parts (`LegacyRotationTrack::Sample`, `ModelRotationKey::Interpolate`), the frame an
//! animation is at (`C3dActorV1::Update`) and the normals (`ModelMesh::CalculateNormals`) are the client's,
//! rodecomp Source/Contents/Models and Source/Resource/Models.

use super::*;
use crate::vector::{cross, subtract};

#[derive(Debug)]
struct Face {
    vertices: [u16; 3],
    texture_vertices: [u16; 3],
    /// Which of the node's textures.
    texture: u16,
    /// It shows from both sides (the client's `twoSided`), and the group of faces it is smoothed with.
    two_sided: bool,
    smooth: i32,
}

#[derive(Debug)]
struct Node {
    name: Vec<u8>,
    parent: Vec<u8>,
    /// The node's textures, as indices into the model's.
    textures: Vec<i32>,
    /// From the mesh to the node: `offset` after `offset_matrix`.
    offset_matrix: [f32; 9],
    offset: Vector,
    /// From the node to its parent: moved to `position`, turned, stretched.
    position: Vector,
    rotation_angle: f32,
    rotation_axis: Vector,
    scale: Vector,
    /// The keys of the node's turning: a frame and a quaternion each.
    keys: Vec<(i32, [f32; 4])>,
    vertices: Vec<Vector>,
    texture_vertices: Vec<[f32; 2]>,
    faces: Vec<Face>,
}

/// One triangle of a model, in the model's own space.
#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    pub positions: [Vector; 3],
    pub uvs: [[f32; 2]; 3],
    /// Which of the model's textures.
    pub texture: usize,
    /// The way each corner looks out of the model for its light: the face's own way for a model shaded flat,
    /// that of the faces of its smoothing group about the corner for one shaded smooth.
    pub normals: [Vector; 3],
    /// The way the face looks out, and whether it shows from both sides.
    pub outward: Vector,
    pub two_sided: bool,
}

#[derive(Debug)]
pub struct Rsm {
    /// Texture files, by their path under `data\texture\`.
    pub textures: Vec<String>,
    /// The model as its animation starts: every part in its place, the model's base at the origin.
    pub triangles: Vec<Triangle>,
    /// How the client shades it: 0 not at all, 1 flat, 2 smooth.
    pub shade: i32,
    /// How many frames its animation has.
    pub length: i32,
    nodes: Vec<Node>,
    root: Vec<u8>,
}

impl Rsm {
    /// Reads a RSM of versions 1.1 to 1.5, the models of the classic maps. Names are EUC-KR and given to `name` to
    /// decode.
    pub fn parse(bytes: &[u8], name: impl Fn(&[u8]) -> String) -> Result<Self, String> {
        let mut reader = Reader::new(bytes);
        if reader.take(4)? != b"GRSM" {
            return Err("not a RSM file".to_owned());
        }
        reader.version = reader.u16()?.swap_bytes();
        let version = reader.version;
        if !(0x0101..=0x0105).contains(&version) {
            return Err(format!("RSM version {}.{} is not supported", version >> 8, version & 0xff));
        }
        let (length, shade) = (reader.i32()?, reader.i32()?);
        if version >= 0x0104 {
            // The model's own alpha: the client reads it and then draws every model at the alpha of its
            // actor, which is full (`C3dNodeV1::Draw`), so it shows nothing.
            reader.take(1)?;
        }
        reader.take(16)?;
        let texture_count = reader.count(40)?;
        let textures = (0..texture_count).map(|_| reader.name(40).map(&name)).collect::<Result<Vec<_>, _>>()?;
        let root = reader.name(40)?.to_vec();
        let node_count = reader.count(100)?;
        let nodes = (0..node_count).map(|_| node(&mut reader)).collect::<Result<Vec<_>, _>>()?;
        let triangles = assemble(&nodes, &root, 0);
        Ok(Self { textures, triangles, shade, length, nodes, root })
    }

    /// Whether any part of it turns as its animation goes.
    pub fn animated(&self) -> bool {
        self.length > 0 && self.nodes.iter().any(|node| node.keys.len() > 1)
    }

    /// The model at a frame of its animation.
    pub fn at(&self, frame: i32) -> Vec<Triangle> {
        assemble(&self.nodes, &self.root, frame)
    }
}

/// The frame a model's animation is at after so many frames of the client's (`C3dActorV1::Update`, called once
/// a frame): each adds a third of the placement's speed times 100, cut to a whole number; an animation that
/// plays over and over (2) starts anew once it is past its length, one that plays once (1) stays on its last
/// frame, and any other does not play.
pub fn frame_at(elapsed: u64, speed: f32, length: i32, playback: i32) -> i32 {
    let step = (speed / 3.0 * 100.0) as i64;
    if step <= 0 || length <= 0 {
        return 0;
    }
    let length = i64::from(length);
    match playback {
        1 => (elapsed as i64).saturating_mul(step).min(length - 1) as i32,
        2 => ((elapsed as i64 % ((length + step - 1) / step)) * step) as i32,
        _ => 0,
    }
}

fn node(reader: &mut Reader) -> Result<Node, String> {
    let name = reader.name(40)?.to_vec();
    let parent = reader.name(40)?.to_vec();
    let texture_count = reader.count(4)?;
    let textures = (0..texture_count).map(|_| reader.i32()).collect::<Result<_, _>>()?;
    let mut offset_matrix = [0.0; 9];
    for part in &mut offset_matrix {
        *part = reader.f32()?;
    }
    let (offset, position, rotation_angle, rotation_axis, scale) = (reader.vector()?, reader.vector()?, reader.f32()?, reader.vector()?, reader.vector()?);

    let vertex_count = reader.count(12)?;
    let vertices = (0..vertex_count).map(|_| reader.vector()).collect::<Result<_, _>>()?;
    let texture_vertex_count = reader.count(8)?;
    let mut texture_vertices = Vec::with_capacity(texture_vertex_count);
    for _ in 0..texture_vertex_count {
        if reader.version >= 0x0102 {
            // A colour, not used.
            reader.take(4)?;
        }
        texture_vertices.push([reader.f32()?, reader.f32()?]);
    }
    let face_count = reader.count(24)?;
    let mut faces = Vec::with_capacity(face_count);
    for _ in 0..face_count {
        let vertices = [reader.u16()?, reader.u16()?, reader.u16()?];
        let texture_vertices = [reader.u16()?, reader.u16()?, reader.u16()?];
        let texture = reader.u16()?;
        // Padding, then whether the face shows from both sides, and its smoothing group (none in 1.1).
        reader.take(2)?;
        let (two_sided, smooth) = (reader.i32()? != 0, reader.i32()?);
        faces.push(Face { vertices, texture_vertices, texture, two_sided, smooth: if reader.version >= 0x0102 { smooth } else { 0 } });
    }
    // Key frames of the node's turning: a frame number and a quaternion each.
    let rotation_count = reader.count(20)?;
    let mut keys = Vec::with_capacity(rotation_count);
    for _ in 0..rotation_count {
        keys.push((reader.i32()?, [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?]));
    }
    Ok(Node { name, parent, textures, offset_matrix, offset, position, rotation_angle, rotation_axis, scale, keys, vertices, texture_vertices, faces })
}

/// A node's turn at a frame (`LegacyRotationTrack::Sample`): with no keys the turn of its file, with one that
/// key; else between the last key at or before the frame and the next, and none at all past the last key or
/// between two keys that are no turn.
fn turn(node: &Node, frame: i32) -> Matrix {
    match node.keys.as_slice() {
        [] => Matrix::rotation(node.rotation_axis, node.rotation_angle),
        [only] => Matrix::quaternion(only.1),
        keys => {
            let Some(next) = (1..keys.len()).find(|&next| keys[next].0 > frame) else { return Matrix::IDENTITY };
            let (from, to) = (keys[next - 1], keys[next]);
            if from.1[3] == 1.0 && to.1[3] == 1.0 {
                return Matrix::IDENTITY;
            }
            Matrix::quaternion(between(from.1, to.1, (frame - from.0) as f32 / (to.0 - from.0) as f32))
        }
    }
}

/// The turn so far of the way from one quaternion to another (`ModelRotationKey::Interpolate`): along the arc
/// between them, the shorter way round; straight between them when they are all but the same.
fn between(from: [f32; 4], to: [f32; 4], amount: f32) -> [f32; 4] {
    let dot: f32 = (0..4).map(|part| from[part] * to[part]).sum();
    let (flip, size) = (dot < 0.0, dot.abs());
    let (first, mut second) = if 1.0 - f64::from(size) >= 0.000_001 {
        let angle = size.acos();
        ((angle - angle * amount).sin() / angle.sin(), (angle * amount).sin() / angle.sin())
    } else {
        (1.0 - amount, amount)
    };
    if flip {
        second = -second;
    }
    std::array::from_fn(|part| from[part] * first + to[part] * second)
}

/// The ways a node's faces look out, in the space of its mesh: of each face, and of each corner of each face
/// with the faces of its smoothing group about that corner (`ModelMesh::CalculateNormals`; the client's own
/// point into the model, and it lights a face by the way the light goes: here they point out, to be lit by the
/// way to the sun).
fn normals(node: &Node, mesh: &[Vector]) -> (Vec<Vector>, Vec<[Vector; 3]>) {
    let corner = |face: &Face, index: usize| mesh.get(usize::from(face.vertices[index])).copied().unwrap_or_default();
    let flat: Vec<Vector> = node
        .faces
        .iter()
        .map(|face| {
            let (a, b, c) = (corner(face, 0), corner(face, 1), corner(face, 2));
            unit(cross(subtract(a, c), subtract(b, c)))
        })
        .collect();
    let mut about: std::collections::HashMap<u16, Vec<usize>> = std::collections::HashMap::new();
    for (index, face) in node.faces.iter().enumerate() {
        for vertex in face.vertices {
            about.entry(vertex).or_default().push(index);
        }
    }
    let smooth = node
        .faces
        .iter()
        .map(|face| {
            face.vertices.map(|vertex| {
                let mut sum = [0.0; 3];
                for &other in &about[&vertex] {
                    if node.faces[other].smooth == face.smooth {
                        sum = [sum[0] + flat[other][0], sum[1] + flat[other][1], sum[2] + flat[other][2]];
                    }
                }
                unit(sum)
            })
        })
        .collect();
    (flat, smooth)
}

/// Each node's place in the model at a frame: from the node to the model with every node at rest (by which
/// the model's size is measured), and with every node turned as the frame has it (as it is drawn).
fn placed(nodes: &[Node], root: usize, frame: i32) -> Vec<Option<(Matrix, Matrix)>> {
    let own = |node: &Node, turn: Matrix| Matrix::translation(node.position).after(&turn).after(&Matrix::scale(node.scale));
    let mut placed: Vec<Option<(Matrix, Matrix)>> = vec![None; nodes.len()];
    let mut queue = vec![(root, Matrix::IDENTITY, Matrix::IDENTITY)];
    while let Some((index, parent_box, parent)) = queue.pop() {
        if placed[index].is_some() {
            continue;
        }
        let node = &nodes[index];
        let (in_box, in_model) = (parent_box.after(&own(node, Matrix::rotation(node.rotation_axis, node.rotation_angle))), parent.after(&own(node, turn(node, frame))));
        placed[index] = Some((in_box, in_model));
        for (child, other) in nodes.iter().enumerate() {
            if child != index && other.parent == node.name && placed[child].is_none() {
                queue.push((child, in_box, in_model));
            }
        }
    }
    placed
}

/// Puts a model's nodes together as they stand at a frame: each node placed in its parent, and the whole moved
/// so that the middle of its base (its lowest point; y points down here) is the origin.
fn assemble(nodes: &[Node], root: &[u8], frame: i32) -> Vec<Triangle> {
    let Some(root) = nodes.iter().position(|node| node.name == root).or(if nodes.is_empty() { None } else { Some(0) }) else { return Vec::new() };
    // From a node's mesh to the node.
    let mesh = |node: &Node| Matrix::translation(node.offset).after(&Matrix::from_3x3(node.offset_matrix));
    let placed = placed(nodes, root, frame);

    let (mut low, mut high) = ([f32::MAX; 3], [f32::MIN; 3]);
    for (node, placed) in nodes.iter().zip(&placed) {
        let Some((in_box, _)) = placed else { continue };
        let to_box = in_box.after(&mesh(node));
        for &vertex in &node.vertices {
            let point = to_box.point(vertex);
            for axis in 0..3 {
                low[axis] = low[axis].min(point[axis]);
                high[axis] = high[axis].max(point[axis]);
            }
        }
    }
    if low[0] > high[0] {
        return Vec::new();
    }
    let to_base = Matrix::translation([-(low[0] + high[0]) / 2.0, -high[1], -(low[2] + high[2]) / 2.0]);

    let mut triangles = Vec::new();
    for (node, placed) in nodes.iter().zip(&placed) {
        let Some((_, in_model)) = placed else { continue };
        let in_mesh: Vec<Vector> = node.vertices.iter().map(|&vertex| mesh(node).point(vertex)).collect();
        let (flat, smooth) = normals(node, &in_mesh);
        let to_model = to_base.after(in_model);
        for (index, face) in node.faces.iter().enumerate() {
            let corner = |corner: usize| Some((to_model.point(*in_mesh.get(usize::from(face.vertices[corner]))?), *node.texture_vertices.get(usize::from(face.texture_vertices[corner]))?));
            let (Some(a), Some(b), Some(c)) = (corner(0), corner(1), corner(2)) else { continue };
            let Some(texture) = node.textures.get(usize::from(face.texture)).and_then(|&texture| usize::try_from(texture).ok()) else { continue };
            let outward = unit(to_model.direction(flat[index]));
            triangles.push(Triangle { positions: [a.0, b.0, c.0], uvs: [a.1, b.1, c.1], texture, normals: smooth[index].map(|normal| unit(to_model.direction(normal))), outward, two_sided: face.two_sided });
        }
    }
    triangles
}
