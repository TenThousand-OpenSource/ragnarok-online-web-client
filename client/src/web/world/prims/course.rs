//! What an effect drawn by code remembers from tick to tick (`Course`: the fields of the client's `CRagEffect`
//! that its functions use), and what the effects under way ask of the world around them in a frame (`Asked`).

use super::*;

/// What an effect remembers from tick to tick: the fields of the client's `CRagEffect` that its functions use.
pub(in crate::web::world) struct Course {
    /// The tick it is at (`stateCnt`) of those it lives (`duration`), and the way the one it is on faced as it
    /// began, in the client's degrees (0 south, 90 east).
    pub(in crate::web::world) tick: i32,
    pub(in crate::web::world) lasts: i32,
    pub(in crate::web::world) facing: f32,
    /// What its primitives share with it: the place it aims at (`deltaPos`) and whether one has landed (`flag`).
    pub(in crate::web::world) shared: Master,
    /// How many it was told (`count`): the hits of the blow.
    pub(in crate::web::world) count: i32,
    /// Two places it keeps for itself (`deltaPos3`, `deltaPos2`) and two turns (`longitude`, `latitude`), each
    /// used as the effect's function says.
    pub(in crate::web::world) reach: [f32; 3],
    pub(in crate::web::world) step: [f32; 3],
    pub(in crate::web::world) longitude: f32,
    pub(in crate::web::world) latitude: f32,
    /// How far the one it is on has gone since it began (east, up, north): what a primitive that stays in the
    /// world where it was launched is told (`Prim::left`).
    pub(in crate::web::world) walked: [f32; 3],
    /// The camera's yaw now, the client's degrees (`crate::prim::camera_yaw`): what an effect that picks a
    /// sprite's action by the way it points across the screen looks at as it launches it.
    pub(in crate::web::world) yaw: f32,
    /// The sounds it starts at this tick, among the client's sounds.
    pub(in crate::web::world) sounds: Vec<String>,
    /// What it does at this tick to the body of the one it is on (`crate::body`: `SetColor` on `state.parent`,
    /// its `render.flags` and `render.angle`): written again at every tick it does so.
    pub(in crate::web::world) touch: crate::body::Touch,
    /// The shake of the screen it asks for at this tick (`MM_QUAKE` with that number: `crate::quake`).
    pub(in crate::web::world) quake: Option<u32>,
    /// The effects it ends at this tick on the one it is on, by the client's numbers (`AM_DELETE_EFFECT`).
    pub(in crate::web::world) ends: Vec<u32>,
    /// Where the other one it was told of is now, from the one it is on (none told, or out of sight: nothing):
    /// what an effect that follows a second actor looks at every tick (the client's `state.target`).
    pub(in crate::web::world) other: Option<[f32; 3]>,
    /// The copies of the one it is on that it leaves standing at this tick (the client's `CHalluPC`): where
    /// each stands from where that one is now (east, up, north), the colour it is in, and of which kind it is
    /// (`crate::body::copies::Kind`).
    pub(in crate::web::world) copies: Vec<([f32; 3], [u8; 3], crate::body::copies::Kind)>,
    /// How high over the feet of the one it is on that one's name stands (`CGameActor::GetNameOffset`), which
    /// the signs of Fury and its like are lifted by (`Prim::lifted`).
    pub(in crate::web::world) lift: f32,
    /// The four figures it was told (`param`, the client's `AM_SETPARAM`: those of an effect a map places),
    /// nothing unless told; and how high the ground is where the one it is on stands, by the world's own
    /// measure (`Burst::figured`, `Burst::grounded`).
    pub(in crate::web::world) figures: [f32; 4],
    pub(in crate::web::world) ground: f32,
}

impl Course {
    /// Starts a sound at this tick, among the client's sounds.
    pub(in crate::web::world) fn sound(&mut self, sound: &str) {
        self.sounds.push(sound.to_owned());
    }

    /// Starts those of an effect's sounds whose tick this is: `sounds` has each with the tick it starts at.
    pub(in crate::web::world) fn starts(&mut self, sounds: &[(i32, &str)]) {
        let tick = self.tick;
        self.sounds.extend(sounds.iter().filter(|(at, _)| *at == tick).map(|(_, sound)| (*sound).to_owned()));
    }
}

/// What effects under way ask of the world around them in a frame: what each does to the body of the one it is
/// on (and whether it lived a tick since it was last asked), the shakes of the screen, the effects to end
/// on someone, and the effects to begin on someone at a place so far east, up and north of that one
/// (`launches`: what a primitive asks for where it is, `Prim::asks`).
#[derive(Default)]
pub(in crate::web::world) struct Asked {
    pub(in crate::web::world) touches: Vec<(Option<u32>, crate::body::Touch, bool)>,
    pub(in crate::web::world) quakes: Vec<u32>,
    pub(in crate::web::world) ends: Vec<(Option<u32>, u32)>,
    pub(in crate::web::world) launches: Vec<(Option<u32>, u32, Vector)>,
    /// The copies to leave standing: of whom, where from that one's place, in which colour, of which kind.
    pub(in crate::web::world) copies: Vec<(Option<u32>, [f32; 3], [u8; 3], crate::body::copies::Kind)>,
}
