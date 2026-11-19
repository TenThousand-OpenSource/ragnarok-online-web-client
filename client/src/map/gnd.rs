//! GND: the ground's shape, textures and baked light, and the mesh drawn from them. The layout follows
//! korangar's `ragnarok-formats` (MIT).

use crate::bytes::Reader;

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
        let mut reader = Reader::new(bytes);
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
            textures.push(name(reader.name(name_size)?));
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
            let corners = reader.floats::<8>()?;
            let (texture, lightmap) = (reader.u16()? as i16, reader.u16()?);
            // Stored blue, green, red, alpha.
            let colour = reader.take(4)?;
            surfaces.push(Surface {
                u: corners[..4].try_into().unwrap(),
                v: corners[4..].try_into().unwrap(),
                texture,
                lightmap,
                colour: [colour[2], colour[1], colour[0]]
            });
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
        (0..self.width * self.height)
            .filter(|&index| self.cubes[index].heights.iter().any(|&height| height < level))
            .map(|index| (index % self.width, index / self.width))
            .collect()
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
                vertices.push(Vertex {
                    position: positions[corner],
                    uv: [surface.u[corner], surface.v[corner]],
                    light_uv: light_uv(surface.lightmap, corner),
                    colour: colours[corner]
                });
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
                    face(
                        surface,
                        [
                            [west, cube.heights[2], north],
                            [east, cube.heights[3], north],
                            [west, next.heights[0], north],
                            [east, next.heights[1], north]
                        ],
                        colour
                    );
                }
                // The wall on the east side.
                if let (Some(surface), Some(next)) = (self.surface(cube.east), self.cube(x + 1, y)) {
                    let colour = [surface.colour.map(|channel| f32::from(channel) / 255.0); 4];
                    face(
                        surface,
                        [
                            [east, cube.heights[3], north],
                            [east, cube.heights[1], south],
                            [east, next.heights[2], north],
                            [east, next.heights[0], south]
                        ],
                        colour
                    );
                }
            }
        }
        Mesh { vertices, by_texture, light_atlas, light_atlas_size: (atlas_size.0 as u32, atlas_size.1 as u32) }
    }
}
