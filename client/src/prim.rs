//! The client's effect primitives (`CEffectPrim`): the figures of one and how it lives and is shaped, kind by
//! kind as they are ported. Nothing here knows of the page or the renderer: `web/world/prims.rs` runs the
//! effects made of these and draws them. This file has the figures alone; one tick of a primitive's life is in
//! `prim/life.rs`, how it is turned in the world in `prim/turn.rs`, and each kind in the file named for it
//! below.
//!
//! - `PP_2DTEXTURE` (0), `PP_2DQUAD` (2), `PP_2DFLASH` (3), `PP_2DCIRCLE` (4): flat on the screen, around the
//!   place on the screen of the one the effect is on (`CEffectPrim::OnProcess`, `Move2DPrimitive`,
//!   `ProcessAlpha`); how they are shaped is in `prim/flat.rs`;
//! - `PP_3DCYLINDER` (13): a band between two rings in the world, turned as the effect says
//!   (`Prim3DCylinder`, `RenderArcBand`);
//! - `PP_3DCIRCLE` (12), `PP_3DSPHERE` (14), `PP_3DCROSSTEXTURE` (16): a flat ring, a ball and two crossed
//!   pictures in the world; how they are shaped is in `prim/solid.rs`.
//! - `PP_3DPARTICLE` (5), `PP_3DPARTICLEGRAVITY` (6), `PP_3DPARTICLEORBIT` (7), `PP_3DPARTICLESPLINE` (8): a
//!   frame of a sprite at a place in the world that moves;
//!   how they move and what of the sprite shows is in `prim/particle.rs`.
//! - `PP_3DQUADHORN` (9): a spike of four sides that is pushed along its own forward (`prim/horn.rs`).
//! - `PP_PARTICLE_UP` (59) and the kind 60: kinds made of up to four parts that each live by themselves, motes
//!   that rise facing the screen and walls of light around the one the effect is on (`prim/slots.rs`).
//!
//! What a primitive shares with the effect it belongs to (the place the effect aims at, the word that something
//! has landed) is in `prim/master.rs`; the ways on the ground and the camera's turn as the client measures them
//! (and `Get_Angle`, the whole degrees of a way) in `prim/facing.rs`; which of the four ways across the screen an
//! effect points, for the sprites whose action is picked by it, in `prim/quarter.rs`; the colour that goes
//! round in `prim/tint.rs`.
//!
//! - the kinds 30, 31 and 149: walls of light as the kind 60 has them, that widen, spin or change their
//!   picture, and the kind 88, a level square that widens (`prim/walls.rs`).
//! - `PP_CHEMICAL3` (56): streaks of light flat over the ground that run towards a place (`prim/streak.rs`).
//!
//! - `PP_3DTEXTURE` (15): one picture standing in the world (`prim/plate.rs`); and what a primitive may be told
//!   to become at a tick of its life, the speeds along the three directions and the ring on the screen that
//!   widens (`prim/change.rs`).
//!
//! - the kinds 62, 69, 77, 89, 91, 92, 100, 103, 115 and 116 and the others its file names (34, 37, 42 to 45,
//!   61, 70, 73, 87 among them), built corner by corner out of the client's parts kept whole (`prim/tei.rs`).
//!
//! - the kind 153: a sprite standing on the ground that comes up and goes, with a rim of its own light
//!   (`prim/glow.rs`).
//!
//! - `PP_KOUENKA` (124): a sprite that rises wandering from side to side (`prim/petals.rs`).
//!
//! - the kind 167: a sprite drawn four times, once at each of its parts, in colours of its own
//!   (`prim/fourfold.rs`).
//!
//! The other kind that shows a sprite (`PP_3DPARTICLE_NOMASTER`, 21) is not ported.

mod casts;
mod change;
mod facing;
mod flat;
mod fourfold;
mod glow;
mod horn;
mod life;
mod master;
mod outdoors;
mod particle;
mod petals;
mod plate;
mod quarter;
mod slots;
mod solid;
mod streak;
mod tei;
mod tint;
mod turn;
mod walls;
#[cfg(test)]
mod horn_tests;
#[cfg(test)]
mod particle_tests;
#[cfg(test)]
mod slots_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod walls_tests;

pub use casts::Cast;
pub use change::{Change, UNSET};
pub use facing::{angle_of, camera_yaw, faced, heading, pointing};
pub use fourfold::Rim;
pub use master::Master;
pub use particle::Seen;
pub use quarter::quarter;
pub use slots::{Lit, Slot};
pub use tei::{PUT_ANEW, Part, Rect, Tei};
pub use tint::Cycle;
pub use turn::{cosine, sine};
use turn::turn;
pub use walls::Walls;

/// The kinds of primitive ported, by the client's names.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Texture2d,
    Quad2d,
    Flash2d,
    Circle2d,
    Cylinder3d,
    Circle3d,
    Sphere3d,
    CrossTexture3d,
    Particle3d,
    ParticleGravity3d,
    ParticleOrbit3d,
    ParticleSpline3d,
    QuadHorn3d,
    ParticleUp,
    Casting3d,
    /// `PP_3DTEXTURE` (15): one picture standing in the world (`prim/plate.rs`).
    Texture3d,
    /// `PP_3DCASTING` (23) and `PP_3DCASTING_2` (24): walls of light that rise once, or turn (`prim/casts.rs`).
    Cast(Cast),
    /// The kinds 30, 31 and 149: walls of light that live in other ways, and the kind 88, a square that
    /// widens (`prim/walls.rs`).
    Walls(Walls),
    /// `PP_CHEMICAL3` (56): streaks of light flat over the ground (`prim/streak.rs`); `quick` ones come up
    /// and run faster and fade slower.
    Streak { quick: bool },
    /// The kinds built corner by corner out of the client's parts kept whole (`prim/tei.rs`).
    Tei(Tei),
    /// The kind 153: a sprite on the ground with a rim of light (`prim/glow.rs`).
    SpriteGlow,
    /// `PP_KOUENKA` (124): a sprite that rises wandering (`prim/petals.rs`).
    Kouenka,
    /// The kind 167: a sprite drawn four times, a part each (`prim/fourfold.rs`).
    Fourfold,
    /// `PP_3DPARTICLE` (5) with its `pattern & 4`: a sprite whose way wanders, a firefly (`prim/outdoors.rs`).
    Roamer,
    /// `PP_SAKURA` (40): a sprite that falls wandering from high over the ground, a petal or a leaf; a `slow`
    /// one drifts half as far to the sides (`prim/outdoors.rs`).
    Petal { slow: bool },
}

/// A primitive's figures: the client's `EffectPrimitiveState`, the fields the ported kinds use, with the values
/// it starts them at.
#[derive(Clone, Debug)]
pub struct Prim {
    pub kind: Shape,
    /// The picture it is drawn with, among the client's pictures of effects; for a kind that shows a sprite, the
    /// sprite's path under the client's folder of sprites, without its ending.
    pub picture: &'static str,
    /// The pictures it runs through, one every `anim_speed` ticks, when it has more than one (the client's
    /// `textures` of `totalTexture`); none: `picture` alone.
    pub pictures: &'static [&'static str],
    /// Ticks lived, and how many it lives.
    pub count: i32,
    pub duration: i32,
    /// The client's `pattern & 1`: what it means is each kind's own.
    pub first_pattern: bool,
    /// The tick from which a flat picture neither grows nor shrinks any more (the client's `pattern & 2` with
    /// its `chCnt`); of a spike, the tick at which it stops (its `pattern & 1` with its `chCnt`).
    pub settles_at: Option<i32>,
    /// Of a flat picture or kite: the tick at which its width and its height take new speeds (the client's
    /// `changePoint` with `changeWidthSpeed` and `changeHeightSpeed`); none: never.
    pub turns_at: Option<(i32, f32, f32)>,
    /// Where it is, from the effect's place on the screen: where it was put (`deltaPos2`) and how far it has
    /// gone since (`deltaPos`).
    pub put: (f32, f32),
    pub gone: (f32, f32),
    /// Where a kind of the world was put along the client's third direction (`deltaPos2.z`), north here.
    pub deep: f32,
    pub longitude: f32,
    pub long_speed: f32,
    pub long_accel: f32,
    pub roll: f32,
    pub speed: f32,
    pub accel: f32,
    pub width: f32,
    pub width_speed: f32,
    pub width_accel: f32,
    pub height: f32,
    pub height_speed: f32,
    pub height_accel: f32,
    pub radius: f32,
    pub radius_speed: f32,
    pub radius_accel: f32,
    pub size: f32,
    /// The band's two rings: how wide the near one and the far one are (the far one `height` away), how fast
    /// the far one widens, and how the band is tipped (`latitude`, about the line across) before it is turned
    /// (`longitude`, about the upright).
    pub inner: f32,
    pub outer: f32,
    pub outer_speed: f32,
    pub latitude: f32,
    /// Of a band: how fast its near ring widens, and how the speeds of both rings change (`innerSpeed`,
    /// `innerAccel`, `outerAccel`).
    pub inner_speed: f32,
    pub inner_accel: f32,
    pub outer_accel: f32,
    /// How far it has gone in the world since it began (east, up, north).
    pub moved: [f32; 3],
    pub arc: f32,
    /// Its red, green and blue, each of 255.
    pub colour: [f32; 3],
    pub alpha: f32,
    pub max_alpha: f32,
    pub alpha_speed: f32,
    /// The tick it begins to fade at, to be gone by its last.
    pub fade_out: i32,
    /// The client's `pattern & 0x80`: its alpha runs up to its most and back down to nothing, over and over.
    pub flickers: bool,
    /// Its turn is made about the upright first and then tipped (the client's `MakeYRotation` and then
    /// `AppendXRotation`); else tipped first and then turned, as the other kinds are.
    pub yaw_first: bool,
    /// How fast it turns in the plane of the screen (`rollSpeed`) and changes size (`sizeSpeed`), a tick.
    pub roll_speed: f32,
    pub size_speed: f32,
    /// How fast it falls, down being more, and how that changes (`gravSpeed`, `gravAccel`).
    pub grav_speed: f32,
    pub grav_accel: f32,
    /// Of a sprite that flies in a bow: how fast it swings out to the side and how that changes (`latiSpeed`,
    /// `latiAccel`; how far out it is now is kept in `latitude`), and where that puts it beside its flight
    /// (east, up, north; `deltaPos3`).
    pub lati_speed: f32,
    pub lati_accel: f32,
    pub aside: [f32; 3],
    /// Of a sprite: which frame of its first action shows (`curMotion`), every how many ticks the next one
    /// comes (`animSpeed`), and whether the frames start over after the last (`repeatAnim`) or it ends there.
    pub motion: usize,
    pub anim_speed: i32,
    pub repeats: bool,
    /// Of a sprite: which action of its file plays (the client's `SetAction`; the first unless told), and
    /// whether each layer of its frames is drawn in the colour and alpha its file gives that layer, whatever the
    /// primitive's own are (`pattern & 0x200`, `RenderSegmentSprites2`).
    pub action: usize,
    pub own_colours: bool,
    /// Of a sprite: how many times it is drawn (`numSegment`), and where each was, the newest first, from the
    /// effect's place (east, up, north): the places it has been a tick, two ticks... ago (`records`).
    pub segments: usize,
    pub trail: Vec<[f32; 3]>,
    /// Of a crossed picture: it tells its effect where it is, every tick (`pattern & 0x10`), and it stops on
    /// reaching the ground and tells its effect so (`pattern & 4`).
    pub leads: bool,
    pub lands: bool,
    /// Of a sprite: it ends within 3 of the place its effect aims at (`pattern & 0x100`).
    pub seeks: bool,
    /// Of a sprite: the way it points in the world, the client's degrees (`render.angle`), by which it is turned
    /// on the screen against the camera's own turn instead of by its roll (`pattern & 0x20`).
    pub points: Option<f32>,
    /// Its light is added to what is behind it (the client's blend of the source's alpha with one: a
    /// `renderFlag` of 5 for a sprite, `RenderTeiRect`'s `destBlend` of 2 for the kinds made of parts);
    /// else it is mixed in by its alpha.
    pub adds: bool,
    /// Of the kinds made of parts (`slots`, the client's four `EffectPrimitiveSlot`): the parts.
    pub slots: Vec<Slot>,
    /// Of the kinds of `prim/tei.rs`: the parts, each the client's `EffectPrimitiveSlot` whole.
    pub parts: Vec<Part>,
    /// It stays in the world where it was launched, whatever the one its effect is on does after: how far that
    /// one had gone since the effect began as it was launched (the client gives such a primitive places of the
    /// world, not places from its effect's).
    pub left: Option<[f32; 3]>,
    /// What it becomes at a tick of its life (`changePoint`); its speeds along the client's three directions
    /// (x east, y down, z north: `speed3d`) and how they change (`deltaPosAccel`), and whether it goes by them
    /// (`pattern & 0x400` of a sprite, `0x100` of a picture in the world): `prim/change.rs`.
    pub changes: Option<Change>,
    pub speed3: [f32; 3],
    pub accel3: [f32; 3],
    pub by_axes: bool,
    /// Of a picture in the world: it is turned to the camera (`pattern & 0x40`).
    pub faces_camera: bool,
    /// Of a picture in the world: its colour goes round (`pattern & 0x800`, `prim/tint.rs`).
    pub cycle: Option<Cycle>,
}

impl Prim {
    pub fn new(kind: Shape, picture: &'static str) -> Self {
        Self {
            kind,
            picture,
            pictures: &[],
            count: 0,
            duration: 100,
            first_pattern: false,
            settles_at: None,
            turns_at: None,
            put: (0.0, 0.0),
            gone: (0.0, 0.0),
            deep: 0.0,
            longitude: 0.0,
            long_speed: 0.0,
            long_accel: 0.0,
            roll: 0.0,
            speed: 0.0,
            accel: 0.0,
            width: 0.0,
            width_speed: 0.0,
            width_accel: 0.0,
            height: 0.0,
            height_speed: 0.0,
            height_accel: 0.0,
            radius: 0.0,
            radius_speed: 0.0,
            radius_accel: 0.0,
            size: 1.0,
            inner: 0.0,
            outer: 0.0,
            outer_speed: 0.0,
            latitude: 0.0,
            inner_speed: 0.0,
            inner_accel: 0.0,
            outer_accel: 0.0,
            moved: [0.0; 3],
            arc: 36.0,
            colour: [255.0; 3],
            alpha: 254.0,
            max_alpha: 254.0,
            alpha_speed: 0.0,
            fade_out: -1,
            flickers: false,
            yaw_first: false,
            roll_speed: 0.0,
            size_speed: 0.0,
            grav_speed: 0.0,
            grav_accel: 0.0,
            lati_speed: 0.0,
            lati_accel: 0.0,
            aside: [0.0; 3],
            motion: 0,
            anim_speed: 4,
            repeats: true,
            action: 0,
            own_colours: false,
            segments: 1,
            trail: Vec::new(),
            leads: false,
            lands: false,
            seeks: false,
            points: None,
            adds: false,
            slots: Vec::new(),
            parts: Vec::new(),
            left: None,
            changes: None,
            speed3: [0.0; 3],
            accel3: [0.0; 3],
            by_axes: false,
            faces_camera: false,
            cycle: None,
        }
    }
}
