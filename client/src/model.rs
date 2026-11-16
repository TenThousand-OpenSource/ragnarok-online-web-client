//! What stands on a map's ground: RSW (the map's water, its sunlight, and where each model stands) and RSM (the
//! models: buildings, trees, lamp posts), and the triangles they come to in the world.
//!
//! Format layouts and the way a model's parts are put together follow korangar (`ragnarok-formats`, and its
//! `loaders/model`; MIT).
//!
//! The files' own space has y pointing down and positions measured from the map's middle. Everything is worked
//! out there and turned into the world's space (y up, measured from the map's south-west corner) at the end.

type Vector = [f32; 3];

/// A 4 by 4 matrix, column by column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix([f32; 16]);

impl Matrix {
    pub const IDENTITY: Matrix = Matrix([1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);

    pub fn translation(by: Vector) -> Self {
        let mut matrix = Self::IDENTITY;
        matrix.0[12..15].copy_from_slice(&by);
        matrix
    }

    pub fn scale(by: Vector) -> Self {
        let mut matrix = Self::IDENTITY;
        (matrix.0[0], matrix.0[5], matrix.0[10]) = (by[0], by[1], by[2]);
        matrix
    }

    /// A turn by `angle` radians around an axis (of any length; none for an axis of no length).
    pub fn rotation(axis: Vector, angle: f32) -> Self {
        let length = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if length < 1e-6 {
            return Self::IDENTITY;
        }
        let [x, y, z] = axis.map(|part| part / length);
        let (sin, cos) = angle.sin_cos();
        let rest = 1.0 - cos;
        Matrix([
            cos + x * x * rest,
            y * x * rest + z * sin,
            z * x * rest - y * sin,
            0.0,
            x * y * rest - z * sin,
            cos + y * y * rest,
            z * y * rest + x * sin,
            0.0,
            x * z * rest + y * sin,
            y * z * rest - x * sin,
            cos + z * z * rest,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    /// The turn a quaternion (x, y, z, w) stands for.
    pub fn quaternion([x, y, z, w]: [f32; 4]) -> Self {
        Matrix([
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y + z * w),
            2.0 * (x * z - y * w),
            0.0,
            2.0 * (x * y - z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z + x * w),
            0.0,
            2.0 * (x * z + y * w),
            2.0 * (y * z - x * w),
            1.0 - 2.0 * (x * x + y * y),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    /// A 3 by 3 matrix, column by column, as the turn-and-stretch part.
    fn from_3x3(parts: [f32; 9]) -> Self {
        let mut matrix = Self::IDENTITY;
        for column in 0..3 {
            matrix.0[column * 4..column * 4 + 3].copy_from_slice(&parts[column * 3..column * 3 + 3]);
        }
        matrix
    }

    /// `self` after `first`: a point goes through `first`, then through `self`.
    pub fn after(&self, first: &Matrix) -> Matrix {
        let mut product = [0.0; 16];
        for column in 0..4 {
            for row in 0..4 {
                product[column * 4 + row] = (0..4).map(|index| self.0[index * 4 + row] * first.0[column * 4 + index]).sum();
            }
        }
        Matrix(product)
    }

    pub fn point(&self, [x, y, z]: Vector) -> Vector {
        let matrix = &self.0;
        [
            matrix[0] * x + matrix[4] * y + matrix[8] * z + matrix[12],
            matrix[1] * x + matrix[5] * y + matrix[9] * z + matrix[13],
            matrix[2] * x + matrix[6] * y + matrix[10] * z + matrix[14],
        ]
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    /// The file's version, as major times 256 plus minor.
    version: u16,
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let slice = self.bytes.get(self.at..self.at.checked_add(count).ok_or("a size too large")?).ok_or("the file ends too early")?;
        self.at += count;
        Ok(slice)
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn vector(&mut self) -> Result<Vector, String> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }

    /// A count that the rest of the file could hold.
    fn count(&mut self, each: usize) -> Result<usize, String> {
        let count = usize::try_from(self.i32()?).map_err(|_| "a negative count")?;
        if count.saturating_mul(each) > self.bytes.len() - self.at {
            return Err("a count larger than the file".to_owned());
        }
        Ok(count)
    }

    /// A zero-terminated name in a field of `size` bytes, as its bytes.
    fn name(&mut self, size: usize) -> Result<&'a [u8], String> {
        let field = self.take(size)?;
        Ok(&field[..field.iter().position(|&byte| byte == 0).unwrap_or(size)])
    }
}

// ---------------------------------------------------------------------------------------------------------------
// RSW

#[derive(Debug, Clone, PartialEq)]
pub struct Water {
    /// The height of the surface, in the world's space (pointing up).
    pub level: f32,
    /// Which of the water textures.
    pub kind: i32,
    /// How many sixtieths of a second each picture of the water shows.
    pub frame_ticks: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Light {
    /// The direction to the sun, in the world's space.
    pub direction: Vector,
    pub diffuse: Vector,
    pub ambient: Vector,
}

/// A model standing on the map.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    /// The model's file, by its path under `data\model\`.
    pub model: String,
    /// In the file's space: from the map's middle, y down.
    pub position: Vector,
    /// Degrees around each axis.
    pub rotation: Vector,
    pub scale: Vector,
}

pub struct Rsw {
    pub water: Water,
    pub light: Light,
    pub placements: Vec<Placement>,
}

impl Rsw {
    /// Reads a RSW of versions 1.2 to 2.5. Names are EUC-KR and given to `name` to decode.
    pub fn parse(bytes: &[u8], name: impl Fn(&[u8]) -> String) -> Result<Self, String> {
        let mut reader = Reader { bytes, at: 0, version: 0 };
        if reader.take(4)? != b"GRSW" {
            return Err("not a RSW file".to_owned());
        }
        reader.version = reader.u16()?.swap_bytes();
        let version = reader.version;
        if !(0x0102..=0x0205).contains(&version) {
            return Err(format!("RSW version {}.{} is not supported", version >> 8, version & 0xff));
        }
        if version >= 0x0205 {
            reader.take(4)?;
        }
        if version >= 0x0202 {
            reader.take(1)?;
        }
        // The names of the map's other files: its settings, ground, walkability and, later, source.
        reader.take(if version >= 0x0104 { 160 } else { 120 })?;

        let mut water = Water { level: 0.0, kind: 0, frame_ticks: 3 };
        if version >= 0x0103 {
            // Heights point down in the file.
            water.level = -reader.f32()?;
        }
        if version >= 0x0108 {
            water.kind = reader.i32()?;
            // The waves' height, speed and pitch: the surface is drawn flat for now.
            reader.take(12)?;
        }
        if version >= 0x0109 {
            water.frame_ticks = (reader.i32()?.max(1)) as u32;
        }

        let mut light = Light { direction: sun(45, 45), diffuse: [1.0; 3], ambient: [0.3; 3] };
        if version >= 0x0105 {
            let (longitude, latitude) = (reader.i32()?, reader.i32()?);
            light = Light { direction: sun(longitude, latitude), diffuse: reader.vector()?, ambient: reader.vector()? };
        }
        if version >= 0x0107 {
            reader.f32()?;
        }
        if version >= 0x0106 {
            reader.take(16)?;
        }

        let count = reader.count(4)?;
        let mut placements = Vec::new();
        for _ in 0..count {
            match reader.i32()? {
                1 => {
                    if version >= 0x0103 {
                        // The object's own name, how it animates, and how it blocks.
                        reader.take(40 + 12)?;
                    }
                    let model = name(reader.name(80)?);
                    reader.take(80)?;
                    placements.push(Placement { model, position: reader.vector()?, rotation: reader.vector()?, scale: reader.vector()? });
                }
                // A light: name, position, colour, range.
                2 => {
                    reader.take(80 + 12 + 12 + 4)?;
                }
                // A sound: name, file, position, volume, size, range and, later, how often it plays.
                3 => {
                    reader.take(80 + 80 + 12 + 4 + 8 + 4 + if version >= 0x0200 { 4 } else { 0 })?;
                }
                // An effect: name, position, kind, speed and four settings.
                4 => {
                    reader.take(80 + 12 + 4 + 4 + 16)?;
                }
                other => return Err(format!("an object of unknown kind {other}")),
            }
        }
        Ok(Self { water, light, placements })
    }
}

/// The direction to the sun from its longitude and latitude in degrees, in the world's space.
fn sun(longitude: i32, latitude: i32) -> Vector {
    let (longitude, latitude) = ((longitude as f32).to_radians(), (latitude as f32).to_radians());
    // In the file's space the light comes from (-cos lon sin lat, -cos lat, -sin lon sin lat), with y down.
    [-longitude.cos() * latitude.sin(), latitude.cos(), -longitude.sin() * latitude.sin()]
}

// ---------------------------------------------------------------------------------------------------------------
// RSM

struct Face {
    vertices: [u16; 3],
    texture_vertices: [u16; 3],
    /// Which of the node's textures.
    texture: u16,
}

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
    /// The node's turn at the start of its animation, where it has one.
    first_rotation: Option<[f32; 4]>,
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
}

#[derive(Debug)]
pub struct Rsm {
    /// Texture files, by their path under `data\texture\`.
    pub textures: Vec<String>,
    /// The model standing still: every part in its place, the model's base at the origin.
    pub triangles: Vec<Triangle>,
}

impl Rsm {
    /// Reads a RSM of versions 1.1 to 1.5, the models of the classic maps. Names are EUC-KR and given to `name` to
    /// decode.
    pub fn parse(bytes: &[u8], name: impl Fn(&[u8]) -> String) -> Result<Self, String> {
        let mut reader = Reader { bytes, at: 0, version: 0 };
        if reader.take(4)? != b"GRSM" {
            return Err("not a RSM file".to_owned());
        }
        reader.version = reader.u16()?.swap_bytes();
        let version = reader.version;
        if !(0x0101..=0x0105).contains(&version) {
            return Err(format!("RSM version {}.{} is not supported", version >> 8, version & 0xff));
        }
        // How long the animation is, and how the model is shaded.
        reader.take(8)?;
        if version >= 0x0104 {
            reader.take(1)?;
        }
        reader.take(16)?;
        let texture_count = reader.count(40)?;
        let textures = (0..texture_count).map(|_| reader.name(40).map(&name)).collect::<Result<Vec<_>, _>>()?;
        let root = reader.name(40)?.to_vec();
        let node_count = reader.count(100)?;
        let nodes = (0..node_count).map(|_| node(&mut reader)).collect::<Result<Vec<_>, _>>()?;
        Ok(Self { textures, triangles: assemble(&nodes, &root) })
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
        // Padding, whether the face shows from both sides, and its smoothing group.
        reader.take(2 + 4 + 4)?;
        faces.push(Face { vertices, texture_vertices, texture });
    }
    // Key frames of the node's turning: a frame number and a quaternion each.
    let rotation_count = reader.count(20)?;
    let mut first_rotation = None;
    for index in 0..rotation_count {
        reader.i32()?;
        let quaternion = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];
        if index == 0 {
            first_rotation = Some(quaternion);
        }
    }
    Ok(Node { name, parent, textures, offset_matrix, offset, position, rotation_angle, rotation_axis, scale, first_rotation, vertices, texture_vertices, faces })
}

/// Puts a model's nodes together as they stand: each node placed in its parent, and the whole moved so that the
/// middle of its base (its lowest point; y points down here) is the origin.
fn assemble(nodes: &[Node], root: &[u8]) -> Vec<Triangle> {
    let Some(root) = nodes.iter().position(|node| node.name == root).or(if nodes.is_empty() { None } else { Some(0) }) else { return Vec::new() };
    // From a node's mesh to the node, and from each node to the model (through its parents).
    let mesh = |node: &Node| Matrix::translation(node.offset).after(&Matrix::from_3x3(node.offset_matrix));
    let own = |node: &Node, animated: bool| {
        let turn = match (animated, node.first_rotation) {
            (true, Some(quaternion)) => Matrix::quaternion(quaternion),
            _ => Matrix::rotation(node.rotation_axis, node.rotation_angle),
        };
        Matrix::translation(node.position).after(&turn).after(&Matrix::scale(node.scale))
    };
    let mut placed: Vec<Option<(Matrix, Matrix)>> = vec![None; nodes.len()];
    let mut queue = vec![(root, Matrix::IDENTITY, Matrix::IDENTITY)];
    while let Some((index, parent_box, parent)) = queue.pop() {
        if placed[index].is_some() {
            continue;
        }
        let node = &nodes[index];
        // The size of the model is measured with every node at rest; it is drawn as its animation starts.
        let (in_box, in_model) = (parent_box.after(&own(node, false)), parent.after(&own(node, true)));
        placed[index] = Some((in_box, in_model));
        for (child, other) in nodes.iter().enumerate() {
            if child != index && other.parent == node.name && placed[child].is_none() {
                queue.push((child, in_box, in_model));
            }
        }
    }

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
        let to_model = to_base.after(in_model).after(&mesh(node));
        for face in &node.faces {
            let corner = |index: usize| Some((to_model.point(*node.vertices.get(usize::from(face.vertices[index]))?), *node.texture_vertices.get(usize::from(face.texture_vertices[index]))?));
            let (Some(a), Some(b), Some(c)) = (corner(0), corner(1), corner(2)) else { continue };
            let Some(texture) = node.textures.get(usize::from(face.texture)).and_then(|&texture| usize::try_from(texture).ok()) else { continue };
            triangles.push(Triangle { positions: [a.0, b.0, c.0], uvs: [a.1, b.1, c.1], texture });
        }
    }
    triangles
}

/// One corner of a model's triangle in the world: its place, texture coordinates, and the light on it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vector,
    pub uv: [f32; 2],
    pub colour: Vector,
}

/// A model's triangles as it stands at a placement, in the world's space, lit by the map's sun. `map_size` is
/// the ground's size in world units. Gives each triangle's three corners with the model's texture index.
pub fn place(model: &Rsm, placement: &Placement, map_size: (f32, f32), light: &Light) -> Vec<(usize, [Vertex; 3])> {
    let turn = |axis: Vector, degrees: f32| Matrix::rotation(axis, degrees.to_radians());
    let to_map = Matrix::translation([placement.position[0] + map_size.0 / 2.0, placement.position[1], placement.position[2] + map_size.1 / 2.0])
        .after(&turn([0.0, 0.0, 1.0], placement.rotation[2]))
        .after(&turn([1.0, 0.0, 0.0], placement.rotation[0]))
        .after(&turn([0.0, 1.0, 0.0], placement.rotation[1]))
        .after(&Matrix::scale(placement.scale));
    model
        .triangles
        .iter()
        .map(|triangle| {
            // The file's space has y down; the world's has it up.
            let positions = triangle.positions.map(|position| {
                let [x, y, z] = to_map.point(position);
                [x, -y, z]
            });
            let colour = shade(positions, light);
            let corners = [0, 1, 2].map(|corner| Vertex { position: positions[corner], uv: triangle.uvs[corner], colour });
            (triangle.texture, corners)
        })
        .collect()
}

/// The light on a flat triangle: the map's ambient light, and its sunlight by how squarely the face meets it.
/// A face shows from both sides, so which way it is wound does not matter.
fn shade(positions: [Vector; 3], light: &Light) -> Vector {
    let edge = |to: usize| [positions[to][0] - positions[0][0], positions[to][1] - positions[0][1], positions[to][2] - positions[0][2]];
    let (a, b) = (edge(1), edge(2));
    let normal = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    let facing = if length < 1e-9 { 0.0 } else { ((normal[0] * light.direction[0] + normal[1] * light.direction[1] + normal[2] * light.direction[2]) / length).abs() };
    [0, 1, 2].map(|channel| (light.ambient[channel] + light.diffuse[channel] * facing).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vector, b: Vector) -> bool {
        (0..3).all(|axis| (a[axis] - b[axis]).abs() < 1e-4)
    }

    #[test]
    fn matrices_move_turn_and_stretch() {
        let moved = Matrix::translation([1.0, 2.0, 3.0]);
        assert!(close(moved.point([1.0, 1.0, 1.0]), [2.0, 3.0, 4.0]));
        let stretched = Matrix::scale([2.0, 3.0, 4.0]);
        assert!(close(stretched.point([1.0, 1.0, 1.0]), [2.0, 3.0, 4.0]));
        // A quarter turn around y takes x to -z.
        let turned = Matrix::rotation([0.0, 1.0, 0.0], std::f32::consts::FRAC_PI_2);
        assert!(close(turned.point([1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]));
        // The same turn as a quaternion.
        let half = std::f32::consts::FRAC_PI_4;
        assert!(close(Matrix::quaternion([0.0, half.sin(), 0.0, half.cos()]).point([1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]));
        // Stretched first, then moved.
        assert!(close(moved.after(&stretched).point([1.0, 1.0, 1.0]), [3.0, 5.0, 7.0]));
        assert!(close(stretched.after(&moved).point([1.0, 1.0, 1.0]), [4.0, 9.0, 16.0]));
        // No axis, no turn.
        assert_eq!(Matrix::rotation([0.0; 3], 1.0), Matrix::IDENTITY);
    }

    fn padded(name: &str, size: usize) -> Vec<u8> {
        let mut field = name.as_bytes().to_vec();
        field.resize(size, 0);
        field
    }

    fn floats(values: &[f32]) -> Vec<u8> {
        values.iter().flat_map(|value| value.to_le_bytes()).collect()
    }

    /// A RSW 2.1 with water, a sun, a light, a sound and one model.
    fn rsw_bytes() -> Vec<u8> {
        let mut file = b"GRSW\x02\x01".to_vec();
        file.extend(padded("", 160));
        file.extend(floats(&[-12.5]));
        file.extend(4i32.to_le_bytes());
        file.extend(floats(&[1.0, 2.0, 50.0]));
        file.extend(5i32.to_le_bytes());
        file.extend(45i32.to_le_bytes());
        file.extend(60i32.to_le_bytes());
        file.extend(floats(&[1.0, 0.9, 0.8, 0.3, 0.3, 0.4, 1.0]));
        file.extend([0u8; 16]);
        file.extend(3i32.to_le_bytes());
        file.extend(2i32.to_le_bytes());
        file.extend(vec![0u8; 108]);
        file.extend(3i32.to_le_bytes());
        file.extend(vec![0u8; 192]);
        file.extend(1i32.to_le_bytes());
        file.extend(vec![0u8; 52]);
        file.extend(padded("town\\house.rsm", 80));
        file.extend(padded("", 80));
        file.extend(floats(&[10.0, -5.0, 20.0, 0.0, 90.0, 0.0, 1.0, 2.0, 1.0]));
        file
    }

    fn text(name: &[u8]) -> String {
        String::from_utf8_lossy(name).into_owned()
    }

    #[test]
    fn reads_a_map_description() {
        let rsw = Rsw::parse(&rsw_bytes(), text).unwrap();
        assert_eq!(rsw.water, Water { level: 12.5, kind: 4, frame_ticks: 5 });
        assert_eq!((rsw.light.diffuse, rsw.light.ambient), ([1.0, 0.9, 0.8], [0.3, 0.3, 0.4]));
        // The sun is above the ground.
        assert!(rsw.light.direction[1] > 0.0);
        assert_eq!(rsw.placements, [Placement { model: "town\\house.rsm".into(), position: [10.0, -5.0, 20.0], rotation: [0.0, 90.0, 0.0], scale: [1.0, 2.0, 1.0] }]);

        assert!(Rsw::parse(b"GRSM\x02\x01", text).is_err());
        assert!(Rsw::parse(&rsw_bytes()[..300], text).is_err());
        let mut newer = rsw_bytes();
        newer[5] = 6;
        assert!(Rsw::parse(&newer, text).is_err());
    }

    /// A RSM 1.4 of one node: a flat square of two triangles, 10 wide, standing from y = -20 to y = 0 … moved up
    /// (y down) by its node, so its base is not at the origin until the model is assembled.
    fn rsm_bytes() -> Vec<u8> {
        let mut file = b"GRSM\x01\x04".to_vec();
        file.extend([0u8; 8]);
        file.push(255);
        file.extend([0u8; 16]);
        file.extend(1i32.to_le_bytes());
        file.extend(padded("wall.bmp", 40));
        file.extend(padded("root", 40));
        file.extend(1i32.to_le_bytes());

        file.extend(padded("root", 40));
        file.extend(padded("", 40));
        file.extend(1i32.to_le_bytes());
        file.extend(0i32.to_le_bytes());
        // No turn in the mesh, no offset; the node moved to (100, -7, 50), not turned, twice as wide.
        file.extend(floats(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]));
        file.extend(floats(&[0.0, 0.0, 0.0]));
        file.extend(floats(&[100.0, -7.0, 50.0]));
        file.extend(floats(&[0.0, 0.0, 1.0, 0.0]));
        file.extend(floats(&[2.0, 1.0, 1.0]));
        file.extend(4i32.to_le_bytes());
        file.extend(floats(&[0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 0.0, -20.0, 0.0, 10.0, -20.0, 0.0]));
        file.extend(4i32.to_le_bytes());
        for (u, v) in [(0.0f32, 1.0f32), (1.0, 1.0), (0.0, 0.0), (1.0, 0.0)] {
            file.extend(0u32.to_le_bytes());
            file.extend(floats(&[u, v]));
        }
        file.extend(2i32.to_le_bytes());
        for corners in [[0u16, 1, 2], [2, 1, 3]] {
            for corner in corners.iter().chain(&corners) {
                file.extend(corner.to_le_bytes());
            }
            file.extend(0u16.to_le_bytes());
            file.extend(0u16.to_le_bytes());
            file.extend(0i32.to_le_bytes());
            file.extend(0i32.to_le_bytes());
        }
        file.extend(0i32.to_le_bytes());
        file
    }

    #[test]
    fn puts_a_model_on_its_base() {
        let model = Rsm::parse(&rsm_bytes(), text).unwrap();
        assert_eq!(model.textures, ["wall.bmp"]);
        assert_eq!(model.triangles.len(), 2);
        // 20 wide after the node's stretch, centred on x; its lowest point (the largest y) at 0.
        let first = &model.triangles[0];
        assert!(close(first.positions[0], [-10.0, 0.0, 0.0]), "{:?}", first.positions);
        assert!(close(first.positions[1], [10.0, 0.0, 0.0]));
        assert!(close(first.positions[2], [-10.0, -20.0, 0.0]));
        assert_eq!(first.uvs, [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]]);
        assert_eq!(first.texture, 0);

        assert!(Rsm::parse(b"GRSW\x01\x04", text).is_err());
        assert!(Rsm::parse(&rsm_bytes()[..200], text).is_err());
        let mut newer = rsm_bytes();
        newer[4] = 2;
        newer[5] = 3;
        assert!(Rsm::parse(&newer, text).unwrap_err().contains("2.3"));
    }

    #[test]
    fn stands_a_model_on_the_map() {
        let model = Rsm::parse(&rsm_bytes(), text).unwrap();
        let light = Light { direction: [0.0, 0.0, 1.0], diffuse: [0.5; 3], ambient: [0.25; 3] };
        // On a map 400 by 600 units: 10 east and 20 north of its middle, 5 above the zero height.
        let straight = Placement { model: String::new(), position: [10.0, -5.0, 20.0], rotation: [0.0; 3], scale: [1.0; 3] };
        let placed = place(&model, &straight, (400.0, 600.0), &light);
        assert_eq!(placed.len(), 2);
        let (texture, corners) = &placed[0];
        assert_eq!(*texture, 0);
        // The base at height 5, the top 20 above it: y points up in the world.
        assert!(close(corners[0].position, [200.0, 5.0, 320.0]), "{:?}", corners[0].position);
        assert!(close(corners[1].position, [220.0, 5.0, 320.0]));
        assert!(close(corners[2].position, [200.0, 25.0, 320.0]));
        // A wall facing north and south, lit by a sun in the north: ambient and all of the sunlight.
        assert!(close(corners[0].colour, [0.75; 3]));

        // Turned a quarter around the upright axis, the wall runs north-south and the same sun only grazes it.
        let turned = Placement { rotation: [0.0, 90.0, 0.0], ..straight.clone() };
        let (_, corners) = &place(&model, &turned, (400.0, 600.0), &light)[0];
        assert!((corners[0].position[0] - corners[1].position[0]).abs() < 1e-3, "{:?}", corners);
        assert!((corners[0].position[2] - corners[1].position[2]).abs() > 19.9);
        assert!(close(corners[0].colour, [0.25; 3]));

        // Twice as tall.
        let tall = Placement { scale: [1.0, 2.0, 1.0], ..straight };
        let (_, corners) = &place(&model, &tall, (400.0, 600.0), &light)[0];
        assert!(close(corners[2].position, [200.0, 45.0, 320.0]));
    }
}
