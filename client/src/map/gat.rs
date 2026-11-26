//! GAT: where one can walk on a map, and how high the ground is there. The layout follows korangar's
//! `ragnarok-formats` (MIT). The ways found over it are in `paths.rs`.

use crate::bytes::Reader;

pub struct Gat {
    pub width: usize,
    pub height: usize,
    /// Per cell, the ground's height (pointing up) at its south-west, south-east, north-west and north-east
    /// corners.
    heights: Vec<[f32; 4]>,
    walkable: Vec<bool>,
    /// Per cell, whether it is a wall (type 1): what cannot be walked on and cannot be shot over either. A
    /// cliff (type 5) cannot be walked on and can be shot over.
    walls: Vec<bool>,
}

impl Gat {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let mut reader = Reader::new(bytes);
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
        let mut walls = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            heights.push([-reader.f32()?, -reader.f32()?, -reader.f32()?, -reader.f32()?]);
            let kind = reader.i32()?;
            walkable.push(Self::walked(kind));
            walls.push(kind == 1);
        }
        Ok(Self { width, height, heights, walkable, walls })
    }

    /// A flat, open map, for tests.
    pub fn open(width: usize, height: usize) -> Self {
        Self { width, height, heights: vec![[0.0; 4]; width * height], walkable: vec![true; width * height], walls: vec![false; width * height] }
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
            self.walls[y as usize * self.width + x as usize] = kind == 1;
        }
    }

    /// A cell becomes a wall, for tests.
    pub fn block(&mut self, x: usize, y: usize) {
        self.walkable[y * self.width + x] = false;
        self.walls[y * self.width + x] = true;
    }

    /// Whether a cell is a wall: one that stops a shot (rAthena's `CELL_CHKWALL`, neither walked on nor shot
    /// over). A cell outside the map is none, as there.
    pub fn wall(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height && self.walls[y as usize * self.width + x as usize]
    }

    pub fn walkable(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height && self.walkable[y as usize * self.width + x as usize]
    }

    /// The heights of a cell's own four corners (south-west, south-east, north-west, north-east), as the file has
    /// them; nothing for a cell outside the map. A cell beside it may have its corners elsewhere (a bridge over
    /// water, a ledge): what is laid on one cell goes by that cell's corners alone
    /// (`GroundGeometry::CellRectangle`).
    pub fn corners(&self, x: i32, y: i32) -> Option<[f32; 4]> {
        (x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height).then(|| self.heights[y as usize * self.width + x as usize])
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
}
