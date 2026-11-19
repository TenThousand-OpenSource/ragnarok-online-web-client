//! A map's ground: GAT (where one can walk, and how high the ground is there) and GND (the ground's shape,
//! textures and baked light), the mesh drawn from them, and the paths walked on them.
//!
//! Format layouts follow korangar's `ragnarok-formats` (MIT). The path search follows rAthena's `path.cpp`, as the
//! server decides where a character really goes.
//!
//! World space: x grows east and z north, 5 units per cell, y up (the files keep heights pointing down).

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

/// World units per GAT cell.
pub const CELL: f32 = 5.0;
/// The longest path, in cells, the server walks in one go (rAthena's `MAX_WALKPATH`).
pub const MAX_WALK_PATH: usize = 32;
/// The longest path, in cells, a player may ask the server to walk in one go (rAthena's `max_walk_path`, 17 as
/// on the official servers): a longer one is refused without a word.
pub const MAX_ASKED_PATH: usize = 17;
/// How many cells the Windows client's search for a way looks at before it gives up (the 149 nodes of
/// `CPathFinder::FindOrCreateNode`, rodecomp Source/Contents/Movement/TerrainPathSearch.cpp).
pub const CLIENT_PATH_NODES: usize = 149;

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
            walkable.push(Self::walked(reader.i32()?));
        }
        Ok(Self { width, height, heights, walkable })
    }

    /// A flat, open map, for tests.
    pub fn open(width: usize, height: usize) -> Self {
        Self { width, height, heights: vec![[0.0; 4]; width * height], walkable: vec![true; width * height] }
    }

    /// Whether a cell of that type can be walked on: types 1 (a wall) and 5 (a cliff one can shoot over) cannot.
    fn walked(kind: i32) -> bool {
        !matches!(kind, 1 | 5)
    }

    /// A cell becomes of another type while the map is played on (the server's ZC_UPDATE_MAPINFO: an Ice Wall
    /// shuts its cells and opens them again when it melts). A cell outside the map changes nothing.
    pub fn retype(&mut self, x: i32, y: i32, kind: i32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.walkable[y as usize * self.width + x as usize] = Self::walked(kind);
        }
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
        let longest = MAX_WALK_PATH;
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
                return if path.len() <= longest { path } else { Vec::new() };
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
                if through > (longest as u32) * DIAGONAL || through >= cost[index(next)] {
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

    /// The way the Windows client finds from one cell to another before it asks the server for a walk
    /// (`CPathFinder::FindPath`, rodecomp Source/Contents/Movement/TerrainPathSearch.cpp): the start left out
    /// and the goal last, or nothing. Its search looks at no more than `CLIENT_PATH_NODES` cells and gives up
    /// altogether when it needs one more, so it finds a way round what is near and never one across a map.
    /// The neighbours are tried in the client's order, and what is left is estimated in straight steps.
    pub fn client_path(&self, from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
        if from == to || !self.walkable(to.0, to.1) {
            return Vec::new();
        }
        let estimate = |(x, y): (i32, i32)| 10 * ((x - to.0).unsigned_abs() + (y - to.1).unsigned_abs());
        // Of each cell looked at: what the way to it costs, and the cell before it.
        let mut nodes = HashMap::from([(from, (0u32, from))]);
        let mut open = BinaryHeap::from([Reverse((estimate(from), 0u32, 0u32, from))]);
        let mut found = 0u32;
        while let Some(Reverse((_, _, costed, cell))) = open.pop() {
            if cell == to {
                let mut path = Vec::new();
                let mut at = to;
                while at != from {
                    path.push(at);
                    at = nodes[&at].1;
                }
                path.reverse();
                return path;
            }
            // An older, dearer entry of a cell since reached more cheaply.
            if nodes[&cell].0 != costed {
                continue;
            }
            for (dx, dy) in [(1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1)] {
                let next = (cell.0 + dx, cell.1 + dy);
                let diagonal = dx != 0 && dy != 0;
                if !self.walkable(next.0, next.1) || diagonal && (!self.walkable(cell.0, next.1) || !self.walkable(next.0, cell.1)) {
                    continue;
                }
                let through = costed + if diagonal { 14 } else { 10 };
                match nodes.get(&next) {
                    Some((cost, _)) if *cost <= through => continue,
                    None if nodes.len() >= CLIENT_PATH_NODES => return Vec::new(),
                    _ => {}
                }
                nodes.insert(next, (through, cell));
                found += 1;
                open.push(Reverse((through + estimate(next), found, through, next)));
            }
        }
        Vec::new()
    }

    /// Where a press on `to` sends a character standing on `from`, as the Windows client decides it
    /// (`CGameMode`'s press on the ground, rodecomp Source/Client/Mode/GameMode.cpp, and
    /// `GetClosestValidCell`, Source/Contents/Messenger/ClientMessenger.cpp). To the place itself when it is
    /// walkable, nobody stands on it (`taken`: the client's `IsRegisterCell`) and the client's search finds a
    /// way to it. Else the client goes along the straight line from the place back to the character and takes
    /// the first cell on it or beside it that is walkable and not taken: there, if its search finds a way to
    /// that one, and nowhere if not — it does not look further, and it does not walk on by itself afterwards.
    ///
    /// The client asks for the cell however long the way found is, and this server refuses without a word a
    /// way longer than `MAX_ASKED_PATH`: such a press does nothing, there as here. With `far` — this project's
    /// own switch, off unless the player turns it on — the cell that far along the way found is given instead,
    /// from which the character is sent on (`world/pursuit.rs`, the journey).
    pub fn goal_towards(&self, from: (i32, i32), to: (i32, i32), taken: impl Fn((i32, i32)) -> bool, far: bool) -> Option<(i32, i32)> {
        let free = |cell: (i32, i32)| self.walkable(cell.0, cell.1) && !taken(cell);
        let reached = |cell: (i32, i32)| {
            let way = self.client_path(from, cell);
            if far { way.get(way.len().min(MAX_ASKED_PATH).wrapping_sub(1)).copied() } else { way.last().copied() }
        };
        if let Some(cell) = Some(to).filter(|to| free(*to)).and_then(reached) {
            return Some(cell);
        }
        const BESIDE: [(i32, i32); 9] = [(0, 0), (1, 0), (1, 1), (1, -1), (-1, 0), (-1, 1), (-1, -1), (0, 1), (0, -1)];
        let (dx, dy) = (from.0 - to.0, from.1 - to.1);
        // Along the longer side of the line a cell at a time, the other side by whole division as the client.
        let wide = dx.abs() >= dy.abs();
        let (long, short) = if wide { (dx, dy) } else { (dy, dx) };
        for step in 0..long.abs() {
            let along = step * long.signum();
            let across = along * short / long;
            let on_line = if wide { (to.0 + along, to.1 + across) } else { (to.0 + across, to.1 + along) };
            if let Some(cell) = BESIDE.iter().map(|(x, y)| (on_line.0 + x, on_line.1 + y)).find(|cell| free(*cell)) {
                return reached(cell);
            }
        }
        None
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

    /// The cubes (east, north) with a corner lower than `level`: where water of that level shows
    /// (`GroundGeometry::DrawWaterCell` draws a cell's water when a corner of its ground is under it).
    pub fn under(&self, level: f32) -> Vec<(usize, usize)> {
        (0..self.width * self.height).filter(|&index| self.cubes[index].heights.iter().any(|&height| height < level)).map(|index| (index % self.width, index / self.width)).collect()
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
