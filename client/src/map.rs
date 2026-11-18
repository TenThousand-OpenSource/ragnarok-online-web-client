//! A map's ground: GAT (where one can walk, and how high the ground is there) and GND (the ground's shape,
//! textures and baked light), the mesh drawn from them, and the paths walked on them.
//!
//! Format layouts follow korangar's `ragnarok-formats` (MIT). The path search follows rAthena's `path.cpp`, as the
//! server decides where a character really goes.
//!
//! World space: x grows east and z north, 5 units per cell, y up (the files keep heights pointing down).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// World units per GAT cell.
pub const CELL: f32 = 5.0;
/// The longest path, in cells, the server walks in one go (rAthena's `MAX_WALKPATH`).
pub const MAX_WALK_PATH: usize = 32;
/// The longest path, in cells, a player may ask the server to walk in one go (rAthena's `max_walk_path`, 17 as
/// on the official servers): a longer one is refused without a word.
pub const MAX_ASKED_PATH: usize = 17;

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
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

    /// A count or size that the rest of the file could hold.
    fn count(&mut self, each: usize) -> Result<usize, String> {
        let count = usize::try_from(self.i32()?).map_err(|_| "a negative count")?;
        if count.saturating_mul(each) > self.bytes.len() - self.at {
            return Err("a count larger than the file".to_owned());
        }
        Ok(count)
    }
}

// ---------------------------------------------------------------------------------------------------------------
// GAT

pub struct Gat {
    pub width: usize,
    pub height: usize,
    /// Per cell, the ground's height (pointing up) at its south-west, south-east, north-west and north-east
    /// corners.
    heights: Vec<[f32; 4]>,
    walkable: Vec<bool>,
}

impl Gat {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let mut reader = Reader { bytes, at: 0 };
        if reader.take(4)? != b"GRAT" {
            return Err("not a GAT file".to_owned());
        }
        reader.take(2)?;
        let (width, height) = (reader.count(0)?, reader.count(0)?);
        if width.saturating_mul(height).saturating_mul(20) > bytes.len() {
            return Err("a GAT larger than its file".to_owned());
        }
        let mut heights = Vec::with_capacity(width * height);
        let mut walkable = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            heights.push([-reader.f32()?, -reader.f32()?, -reader.f32()?, -reader.f32()?]);
            // Types 1 (a wall) and 5 (a cliff one can shoot over) cannot be walked on.
            walkable.push(!matches!(reader.i32()?, 1 | 5));
        }
        Ok(Self { width, height, heights, walkable })
    }

    /// A flat, open map, for tests.
    pub fn open(width: usize, height: usize) -> Self {
        Self { width, height, heights: vec![[0.0; 4]; width * height], walkable: vec![true; width * height] }
    }

    pub fn block(&mut self, x: usize, y: usize) {
        self.walkable[y * self.width + x] = false;
    }

    pub fn walkable(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height && self.walkable[y as usize * self.width + x as usize]
    }

    /// The ground's height at a point given in cells (`3.5` is the middle of cell 3), between the cell's corners.
    pub fn height_at(&self, x: f32, y: f32) -> f32 {
        if self.width == 0 || self.height == 0 {
            return 0.0;
        }
        let cell_x = (x.max(0.0) as usize).min(self.width - 1);
        let cell_y = (y.max(0.0) as usize).min(self.height - 1);
        let (across, along) = ((x - cell_x as f32).clamp(0.0, 1.0), (y - cell_y as f32).clamp(0.0, 1.0));
        let [south_west, south_east, north_west, north_east] = self.heights[cell_y * self.width + cell_x];
        let south = south_west + (south_east - south_west) * across;
        let north = north_west + (north_east - north_west) * across;
        south + (north - south) * along
    }

    /// The path from one cell to another, the start left out and the goal last; empty when there is none within
    /// the server's reach. Steps are to the eight neighbours; a diagonal needs both cells beside it free.
    pub fn path(&self, from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
        if from == to || !self.walkable(to.0, to.1) || !self.walkable(from.0, from.1) {
            return Vec::new();
        }
        const STRAIGHT: u32 = 10;
        const DIAGONAL: u32 = 14;
        let index = |(x, y): (i32, i32)| y as usize * self.width + x as usize;
        // The server's estimate of what is left: straight steps along both axes.
        let estimate = |(x, y): (i32, i32)| STRAIGHT * ((x - to.0).unsigned_abs() + (y - to.1).unsigned_abs());

        let mut cost = vec![u32::MAX; self.width * self.height];
        let mut came_from = vec![usize::MAX; self.width * self.height];
        let mut open = BinaryHeap::new();
        cost[index(from)] = 0;
        // Ordered by cost and estimate, then by the order found, so equal paths come out the same every time.
        let mut found = 0u32;
        open.push(Reverse((estimate(from), found, from)));
        while let Some(Reverse((_, _, cell))) = open.pop() {
            if cell == to {
                let mut path = Vec::new();
                let mut at = index(to);
                while at != index(from) {
                    path.push(((at % self.width) as i32, (at / self.width) as i32));
                    at = came_from[at];
                }
                path.reverse();
                return if path.len() <= MAX_WALK_PATH { path } else { Vec::new() };
            }
            let here = cost[index(cell)];
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let next = (cell.0 + dx, cell.1 + dy);
                if !self.walkable(next.0, next.1) {
                    continue;
                }
                let diagonal = dx != 0 && dy != 0;
                if diagonal && (!self.walkable(cell.0 + dx, cell.1) || !self.walkable(cell.0, cell.1 + dy)) {
                    continue;
                }
                let through = here + if diagonal { DIAGONAL } else { STRAIGHT };
                // Nothing farther than the server walks is worth looking at.
                if through > (MAX_WALK_PATH as u32) * DIAGONAL || through >= cost[index(next)] {
                    continue;
                }
                cost[index(next)] = through;
                came_from[index(next)] = index(cell);
                found += 1;
                open.push(Reverse((through + estimate(next), found, next)));
            }
        }
        Vec::new()
    }

    /// Whether the server would walk a player from one cell to another when asked: there is a path, and it is
    /// no longer than a player may ask for.
    fn askable(&self, from: (i32, i32), to: (i32, i32)) -> bool {
        (1..=MAX_ASKED_PATH).contains(&self.path(from, to).len())
    }

    /// Where a press on `to` sends a character standing on `from`: there, if the server would walk it there
    /// when asked; else the cell nearest to it on the straight line towards it that the server would walk to (a
    /// press too far away, or on a wall). None when there is nowhere to go that way.
    pub fn goal_towards(&self, from: (i32, i32), to: (i32, i32)) -> Option<(i32, i32)> {
        if self.askable(from, to) {
            return Some(to);
        }
        let steps = (to.0 - from.0).abs().max((to.1 - from.1).abs());
        // Nothing farther than a player may ask for is worth trying: a path is at least as long as the way is.
        (1..=steps.min(MAX_ASKED_PATH as i32)).rev().find_map(|step| {
            let along = step as f32 / steps as f32;
            let cell = (from.0 + ((to.0 - from.0) as f32 * along).round() as i32, from.1 + ((to.1 - from.1) as f32 * along).round() as i32);
            self.askable(from, cell).then_some(cell)
        })
    }
}

/// The direction of a step, as the game numbers them: 0 is south, then clockwise (1 south-west, 2 west, … 7
/// south-east).
pub fn direction(dx: i32, dy: i32) -> u8 {
    match (dx.signum(), dy.signum()) {
        (0, -1) => 0,
        (-1, -1) => 1,
        (-1, 0) => 2,
        (-1, 1) => 3,
        (0, 1) => 4,
        (1, 1) => 5,
        (1, 0) => 6,
        (1, -1) => 7,
        _ => 0,
    }
}

/// A walk along a path, at a speed in milliseconds per cell; diagonal steps take 1.4 times as long, as on the
/// server.
pub struct Walk {
    /// The cell walked from, then every cell of the path.
    cells: Vec<(i32, i32)>,
    /// When each cell of `cells` is reached, in milliseconds from the start.
    reached: Vec<f64>,
}

impl Walk {
    pub fn new(from: (i32, i32), path: &[(i32, i32)], speed: f64) -> Self {
        let mut cells = vec![from];
        let mut reached = vec![0.0];
        for &cell in path {
            let last = *cells.last().unwrap();
            let diagonal = cell.0 != last.0 && cell.1 != last.1;
            reached.push(reached.last().unwrap() + if diagonal { speed * 1.4 } else { speed });
            cells.push(cell);
        }
        Self { cells, reached }
    }

    pub fn duration(&self) -> f64 {
        *self.reached.last().unwrap()
    }

    pub fn goal(&self) -> (i32, i32) {
        *self.cells.last().unwrap()
    }

    /// Where the walker is `elapsed` milliseconds in, in cells (whole numbers are cell centres), and which way it
    /// faces.
    pub fn at(&self, elapsed: f64) -> ((f32, f32), u8) {
        let last = self.cells.len() - 1;
        if last == 0 {
            return ((self.cells[0].0 as f32, self.cells[0].1 as f32), 0);
        }
        let step = self.reached.iter().rposition(|&time| time <= elapsed).unwrap_or(0).min(last - 1);
        let (from, to) = (self.cells[step], self.cells[step + 1]);
        let along = ((elapsed - self.reached[step]) / (self.reached[step + 1] - self.reached[step])).clamp(0.0, 1.0) as f32;
        let position = (from.0 as f32 + (to.0 - from.0) as f32 * along, from.1 as f32 + (to.1 - from.1) as f32 * along);
        (position, direction(to.0 - from.0, to.1 - from.1))
    }
}

// ---------------------------------------------------------------------------------------------------------------
// GND

/// One textured face of the ground: its texture coordinates at four corners, its texture and baked light.
struct Surface {
    u: [f32; 4],
    v: [f32; 4],
    texture: i16,
    lightmap: u16,
    /// Red, green, blue, as the face's tint.
    colour: [u8; 3],
}

struct Cube {
    /// Heights (pointing up) at the south-west, south-east, north-west and north-east corners.
    heights: [f32; 4],
    top: i32,
    north: i32,
    east: i32,
}

pub struct Gnd {
    pub width: usize,
    pub height: usize,
    /// World units per cube: two GAT cells.
    pub zoom: f32,
    /// Texture files, by their path under `data\texture\`.
    pub textures: Vec<String>,
    /// How many pixels a baked light patch is wide and high, and the patches: brightness, then colour.
    lightmap_size: (usize, usize),
    lightmaps: Vec<Vec<u8>>,
    surfaces: Vec<Surface>,
    cubes: Vec<Cube>,
}

/// One corner of the ground's mesh.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    /// Where in the atlas of baked light.
    pub light_uv: [f32; 2],
    pub colour: [f32; 3],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    /// The triangles of each texture, as indices into `vertices`, three per triangle.
    pub by_texture: Vec<Vec<u32>>,
    /// The atlas of baked light: RGBA, colour to add and (in alpha) brightness to multiply by.
    pub light_atlas: Vec<u8>,
    pub light_atlas_size: (u32, u32),
}

impl Gnd {
    /// Reads a GND 1.7 (the version of nearly every map). Texture names are EUC-KR and given to `name` to decode.
    pub fn parse(bytes: &[u8], name: impl Fn(&[u8]) -> String) -> Result<Self, String> {
        let mut reader = Reader { bytes, at: 0 };
        if reader.take(4)? != b"GRGN" {
            return Err("not a GND file".to_owned());
        }
        let version = reader.u16()?.swap_bytes();
        if !(0x0107..=0x0109).contains(&version) {
            return Err(format!("GND version {version:#06x} is not supported"));
        }
        let (width, height) = (reader.count(0)?, reader.count(0)?);
        let zoom = reader.f32()?;
        let texture_count = reader.count(1)?;
        let name_size = reader.count(0)?;
        let mut textures = Vec::with_capacity(texture_count);
        for _ in 0..texture_count {
            let field = reader.take(name_size)?;
            textures.push(name(&field[..field.iter().position(|&byte| byte == 0).unwrap_or(field.len())]));
        }

        let lightmap_count = reader.count(1)?;
        let lightmap_size = (reader.count(0)?, reader.count(0)?);
        reader.i32()?;
        let patch = lightmap_size.0.saturating_mul(lightmap_size.1).saturating_mul(4);
        if patch == 0 || lightmap_count.saturating_mul(patch) > bytes.len() {
            return Err("baked light larger than the file".to_owned());
        }
        let lightmaps = (0..lightmap_count).map(|_| reader.take(patch).map(<[u8]>::to_vec)).collect::<Result<_, _>>()?;

        let surface_count = reader.count(40)?;
        let mut surfaces = Vec::with_capacity(surface_count);
        for _ in 0..surface_count {
            let mut corners = [0.0f32; 8];
            for corner in &mut corners {
                *corner = reader.f32()?;
            }
            let (texture, lightmap) = (reader.u16()? as i16, reader.u16()?);
            // Stored blue, green, red, alpha.
            let colour = reader.take(4)?;
            surfaces.push(Surface { u: corners[..4].try_into().unwrap(), v: corners[4..].try_into().unwrap(), texture, lightmap, colour: [colour[2], colour[1], colour[0]] });
        }

        if width.saturating_mul(height).saturating_mul(28) > bytes.len() {
            return Err("a GND larger than its file".to_owned());
        }
        let mut cubes = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            let heights = [-reader.f32()?, -reader.f32()?, -reader.f32()?, -reader.f32()?];
            cubes.push(Cube { heights, top: reader.i32()?, north: reader.i32()?, east: reader.i32()? });
        }
        Ok(Self { width, height, zoom, textures, lightmap_size, lightmaps, surfaces, cubes })
    }

    fn cube(&self, x: usize, y: usize) -> Option<&Cube> {
        (x < self.width && y < self.height).then(|| &self.cubes[y * self.width + x])
    }

    fn surface(&self, index: i32) -> Option<&Surface> {
        usize::try_from(index).ok().and_then(|index| self.surfaces.get(index))
    }

    /// The ground as triangles: the top of every cube, and the walls between cubes of different heights.
    pub fn mesh(&self) -> Mesh {
        // The atlas: every patch in a grid, as close to square as it goes.
        let (patch_width, patch_height) = self.lightmap_size;
        let columns = ((self.lightmaps.len() as f32).sqrt().ceil() as usize).max(1);
        let rows = self.lightmaps.len().div_ceil(columns).max(1);
        let atlas_size = (columns * patch_width, rows * patch_height);
        let mut light_atlas = vec![0u8; atlas_size.0 * atlas_size.1 * 4];
        for (index, patch) in self.lightmaps.iter().enumerate() {
            let (left, top) = ((index % columns) * patch_width, (index / columns) * patch_height);
            let pixels = patch_width * patch_height;
            for pixel in 0..pixels {
                let to = ((top + pixel / patch_width) * atlas_size.0 + left + pixel % patch_width) * 4;
                // Brightness comes first in the file, one byte a pixel; then the colour, three.
                light_atlas[to..to + 3].copy_from_slice(&patch[pixels + pixel * 3..pixels + pixel * 3 + 3]);
                light_atlas[to + 3] = patch[pixel];
            }
        }
        // A patch's outermost pixels are a border for filtering: a face uses what is inside it.
        let light_uv = |lightmap: u16, corner: usize| -> [f32; 2] {
            let index = usize::from(lightmap) % self.lightmaps.len().max(1);
            let (left, top) = ((index % columns) * patch_width, (index / columns) * patch_height);
            let across = if corner % 2 == 0 { 1.0 } else { patch_width as f32 - 1.0 };
            let down = if corner < 2 { 1.0 } else { patch_height as f32 - 1.0 };
            [(left as f32 + across) / atlas_size.0 as f32, (top as f32 + down) / atlas_size.1 as f32]
        };
        // A face's tint belongs to its south-west corner; the other corners take the tints of the faces that
        // start there, so the shading runs smoothly over the ground.
        let tint = |x: usize, y: usize, own: [u8; 3]| -> [f32; 3] {
            let colour = self.cube(x, y).and_then(|cube| self.surface(cube.top)).map_or(own, |surface| surface.colour);
            colour.map(|channel| f32::from(channel) / 255.0)
        };

        let mut vertices = Vec::new();
        let mut by_texture = vec![Vec::new(); self.textures.len()];
        let mut face = |surface: &Surface, positions: [[f32; 3]; 4], colours: [[f32; 3]; 4]| {
            let Some(triangles) = usize::try_from(surface.texture).ok().and_then(|texture| by_texture.get_mut(texture)) else { return };
            let first = vertices.len() as u32;
            for corner in 0..4 {
                vertices.push(Vertex { position: positions[corner], uv: [surface.u[corner], surface.v[corner]], light_uv: light_uv(surface.lightmap, corner), colour: colours[corner] });
            }
            triangles.extend([first, first + 1, first + 2, first + 2, first + 1, first + 3]);
        };

        let zoom = self.zoom;
        for y in 0..self.height {
            for x in 0..self.width {
                let cube = &self.cubes[y * self.width + x];
                let (west, east, south, north) = (x as f32 * zoom, (x + 1) as f32 * zoom, y as f32 * zoom, (y + 1) as f32 * zoom);
                if let Some(surface) = self.surface(cube.top) {
                    let own = surface.colour;
                    face(
                        surface,
                        [[west, cube.heights[0], south], [east, cube.heights[1], south], [west, cube.heights[2], north], [east, cube.heights[3], north]],
                        [tint(x, y, own), tint(x + 1, y, own), tint(x, y + 1, own), tint(x + 1, y + 1, own)],
                    );
                }
                // The wall on the north side, down (or up) to the cube north of this one.
                if let (Some(surface), Some(next)) = (self.surface(cube.north), self.cube(x, y + 1)) {
                    let colour = [surface.colour.map(|channel| f32::from(channel) / 255.0); 4];
                    face(surface, [[west, cube.heights[2], north], [east, cube.heights[3], north], [west, next.heights[0], north], [east, next.heights[1], north]], colour);
                }
                // The wall on the east side.
                if let (Some(surface), Some(next)) = (self.surface(cube.east), self.cube(x + 1, y)) {
                    let colour = [surface.colour.map(|channel| f32::from(channel) / 255.0); 4];
                    face(surface, [[east, cube.heights[3], north], [east, cube.heights[1], south], [east, next.heights[2], north], [east, next.heights[0], south]], colour);
                }
            }
        }
        Mesh { vertices, by_texture, light_atlas, light_atlas_size: (atlas_size.0 as u32, atlas_size.1 as u32) }
    }
}

#[cfg(test)]
mod tests;
