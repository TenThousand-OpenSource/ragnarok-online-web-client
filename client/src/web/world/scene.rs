//! The map being shown and who is on it (`Scene`), and what is asked of it everywhere: where someone is, in
//! cells and in the world, and where a press sends the character.

use super::*;

pub(super) struct Scene {
    /// Inside a building the camera is the original's: turned 45 degrees and not turning, never far away.
    pub(super) indoor: bool,
    pub(super) gat: Gat,
    pub(super) ground: Ground,
    pub(super) models: Models,
    pub(super) water: Option<WaterSurface>,
    pub(super) player: Mover,
    /// Everyone else in sight, by the server's ID.
    pub(super) units: HashMap<u32, Mover>,
    /// The things lying on the ground, by the server's ID.
    pub(super) drops: HashMap<u32, Lying>,
    /// When each thing that fell out of a monster began to fall.
    pub(super) falls: HashMap<u32, f64>,
    /// Where the camera looks, and when it was last moved: it follows the character a tenth of the way each of
    /// the client's frames (`CameraState::Smooth`), so a step the server corrects does not jolt the view. None
    /// until the first frame, and after the character is put somewhere else: then it looks straight at it.
    pub(super) aim: Option<(Vector, f64)>,
    /// Where the units of skills on the ground and the places skills were used on are, by ID, in cells
    /// (`skill_units.rs`): effects play on them as on someone.
    pub(super) spots: HashMap<u32, (f32, f32)>,
    /// The portals in sight, by the server's ID: the cells they are on.
    pub(super) portals: HashMap<u32, (i32, i32)>,
    pub(super) camera: Camera,
    /// The camera as the player asked for it, and when `camera` last went towards it: a tenth of the way
    /// each of the client's frames (`CameraState::Smooth`, `camera.rs`).
    pub(super) wanted: (Camera, f64),
    /// What surrounds the player here: the effects and sounds the map places, its fog, sky and weather
    /// (`scenery.rs`).
    pub(super) scenery: scenery::Surroundings,
    /// The last cell told to the page, so it is told only when it changes.
    pub(super) told: (i32, i32),
}

/// Where a point given in cells is in the world: cell centres, on the ground.
pub(super) fn on_ground(gat: &Gat, (x, y): (f32, f32)) -> Vector {
    // A character on cell 3 stands at 3.5 cells.
    [(x + 0.5) * CELL, gat.height_at(x + 0.5, y + 0.5), (y + 0.5) * CELL]
}

impl Scene {
    /// Someone in sight by the server's ID; no one named is the player's own character.
    pub(super) fn mover(&self, who: Option<u32>) -> Option<&Mover> {
        match who {
            Some(id) => self.units.get(&id),
            None => Some(&self.player),
        }
    }

    pub(super) fn mover_mut(&mut self, who: Option<u32>) -> Option<&mut Mover> {
        match who {
            Some(id) => self.units.get_mut(&id),
            None => Some(&mut self.player),
        }
    }

    /// The same for one named by an ID that may be the player's `own`, as the server names everyone in a
    /// packet: the player's own character is not among the others.
    pub(super) fn one(&self, id: u32, own: u32) -> Option<&Mover> {
        self.mover((id != own).then_some(id))
    }

    pub(super) fn one_mut(&mut self, id: u32, own: u32) -> Option<&mut Mover> {
        self.mover_mut((id != own).then_some(id))
    }

    /// Where someone is now, in cells: no one named is the player's own character; nothing for one out of sight.
    pub(super) fn place_of(&self, who: Option<u32>) -> Option<(f32, f32)> {
        match who {
            Some(id) => self.units.get(&id).map(|unit| unit.at).or_else(|| self.spots.get(&id).copied()),
            None => Some(self.player.at),
        }
    }

    /// Where someone, or a place effects play on, stands in the world now (none named: the player's own
    /// character): on the ground, but for a place a map put its effect at, which is as high as the map says.
    pub(super) fn stands(&self, who: Option<u32>) -> Option<Vector> {
        let mut place = on_ground(&self.gat, self.place_of(who)?);
        if let Some(height) = who.and_then(|id| self.scenery.raised.get(&id)) {
            place[1] = *height;
        }
        Some(place)
    }

    /// Where a press on `to` sends the player's character (`Gat::goal_towards`), the cells where somebody
    /// stands being taken. The client keeps those in a list (`CGameMode::actorPosList`): an actor is put on it
    /// when it stands, sits, strikes or casts (`RegisterPos` in `CRenderObject::SetState`) and taken off when
    /// it walks, has died or goes out of sight; portals and unseen NPCs are never on it, and one who hides
    /// (the effect state's bit 2) does not count (`IsRegisterCell`).
    pub(super) fn goal_towards(&self, from: (i32, i32), to: (i32, i32), far: bool) -> Option<(i32, i32)> {
        let taken = |cell: (i32, i32)| self.units.values().any(|unit| unit.cell == cell && unit.walk.is_none() && !unit.dead && unit.dying.is_none() && unit.state & 2 == 0);
        self.gat.goal_towards(from, to, taken, far)
    }
}
