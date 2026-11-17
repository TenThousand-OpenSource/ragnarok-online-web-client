//! The effects the client draws by code, out of its primitives.
//!
//! In the client such an effect (`CRagEffect`) launches a few primitives (`CEffectPrim`), sets their figures and
//! lets them run: each tick a primitive moves by its speeds and is drawn by its kind. This has the kinds ported
//! so far and the effects made of them, figure for figure:
//!
//! - `PP_2DTEXTURE` (0), `PP_2DFLASH` (3), `PP_2DCIRCLE` (4): flat on the screen, around the place on the screen
//!   of the one the effect is on (`CEffectPrim::OnProcess`, `Move2DPrimitive`, `ProcessAlpha`,
//!   `Render2DPrimitive`, `Render2DCircle`);
//! - `PP_3DCYLINDER` (13): a band between two rings in the world, turned as the effect says
//!   (`Prim3DCylinder`, `RenderArcBand`);
//! - `EF_HIT1` (0), the ring of a blow that lands (`CRagEffect::Hit1`; its four small sparks, sprites that
//!   fly off, are not ported yet), `EF_HIT2` (1), the flare where a skill lands (`CRagEffect::Hit2`), and
//!   `EF_BASH` (16) (`CRagEffect::Bash`).
//!
//! The client measures the flat kinds in pixels of its screen. Here a pixel of them is a pixel of a sprite, as
//! for the effects played from files: they keep their size beside the characters when the camera moves.

use super::*;

/// The kinds of primitive ported, by the client's names.
#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Texture2d,
    Flash2d,
    Circle2d,
    Cylinder3d,
}

/// A primitive's figures: the client's `EffectPrimitiveState`, the fields the ported kinds use, with the values
/// it starts them at.
#[derive(Clone)]
struct Prim {
    kind: Shape,
    picture: &'static str,
    /// Ticks lived, and how many it lives.
    count: i32,
    duration: i32,
    /// The client's `pattern & 1`: what it means is each kind's own.
    first_pattern: bool,
    /// Where it is, from the effect's place on the screen: where it was put (`deltaPos2`) and how far it has
    /// gone since (`deltaPos`).
    put: (f32, f32),
    gone: (f32, f32),
    longitude: f32,
    long_speed: f32,
    long_accel: f32,
    roll: f32,
    speed: f32,
    accel: f32,
    width: f32,
    width_speed: f32,
    height: f32,
    height_speed: f32,
    height_accel: f32,
    radius: f32,
    size: f32,
    /// The band's two rings: how wide the near one and the far one are (the far one `height` away).
    inner: f32,
    outer: f32,
    arc: f32,
    alpha: f32,
    max_alpha: f32,
    alpha_speed: f32,
    /// The tick it begins to fade at, to be gone by its last.
    fade_out: i32,
}

impl Prim {
    fn new(kind: Shape, picture: &'static str) -> Self {
        Self {
            kind,
            picture,
            count: 0,
            duration: 100,
            first_pattern: false,
            put: (0.0, 0.0),
            gone: (0.0, 0.0),
            longitude: 0.0,
            long_speed: 0.0,
            long_accel: 0.0,
            roll: 0.0,
            speed: 0.0,
            accel: 0.0,
            width: 0.0,
            width_speed: 0.0,
            height: 0.0,
            height_speed: 0.0,
            height_accel: 0.0,
            radius: 0.0,
            size: 1.0,
            inner: 0.0,
            outer: 0.0,
            arc: 36.0,
            alpha: 254.0,
            max_alpha: 254.0,
            alpha_speed: 0.0,
            fade_out: -1,
        }
    }

    /// One tick of its life. `left` is how many ticks the effect it belongs to still has. Gives back whether it
    /// lives on.
    fn tick(&mut self, left: i32) -> bool {
        match self.kind {
            Shape::Texture2d => {
                self.height_speed += self.height_accel;
                self.speed += self.accel;
                self.width += self.width_speed;
                self.height += self.height_speed;
                self.gone.0 += sine(self.longitude) * self.speed;
                self.gone.1 -= cosine(self.longitude) * self.speed;
            }
            Shape::Flash2d => {
                if self.height < 0.0 {
                    self.height_speed = -self.height_speed;
                    self.height = 0.0;
                }
                self.long_speed += self.long_accel;
                self.height_speed += self.height_accel;
                self.longitude += self.long_speed;
                self.height += self.height_speed;
            }
            // The ring neither grows nor moves unless told a speed, and none of the effects here tells it one;
            // with the first pattern it is filled to its middle.
            Shape::Circle2d => {
                if self.first_pattern {
                    self.size = self.radius;
                }
            }
            // The band goes the way it is turned, by its speed (its rings keep their size here: no effect
            // ported gives them a speed).
            Shape::Cylinder3d => {
                self.speed += self.accel;
                self.gone.0 += self.speed;
            }
        }
        // `ProcessAlpha`: one that would outlive its effect fades with it; at its own time to fade it sets off
        // to be gone by its last tick; and it stays between nothing and its most.
        if self.duration - self.count > left + 1 {
            self.alpha_speed = -(self.alpha / left.max(1) as f32);
        }
        if self.count == self.fade_out {
            self.alpha_speed = -(self.alpha / (self.duration - self.count) as f32);
        }
        self.alpha = (self.alpha + self.alpha_speed).clamp(0.0, self.max_alpha);
        self.count += 1;
        self.count <= self.duration
    }

    /// Its triangles now, two by two: corners on the screen from the effect's place (x to the right, y down)
    /// with where of the picture each shows.
    fn corners(&self) -> Vec<[f32; 4]> {
        let (x, y) = (self.put.0 + self.gone.0, self.put.1 + self.gone.1);
        match self.kind {
            // A picture turned by its roll: with the first pattern it reaches from its place twice its height
            // one way, else its height both ways.
            Shape::Texture2d => {
                let (sin, cos) = (sine(self.roll), cosine(self.roll));
                let (far, near) = if self.first_pattern { (-2.0 * self.height, 0.0) } else { (-self.height, self.height) };
                let corner = |across: f32, along: f32, u: f32, v: f32| [x + across * cos - along * sin - 0.5, y + across * sin + along * cos - 0.5, u, v];
                let (low_left, high_left) = (corner(-self.width, near, 0.0, 1.0), corner(-self.width, far, 0.0, 0.0));
                let (low_right, high_right) = (corner(self.width, near, 1.0, 1.0), corner(self.width, far, 1.0, 0.0));
                vec![low_left, high_left, low_right, low_right, high_left, high_right]
            }
            // A wedge from its place, its height long, its arc to each side of where it points.
            Shape::Flash2d => {
                let point = |angle: f32, u: f32| [x + self.height * cosine(angle), y - self.height * sine(angle), u, 1.0];
                // One triangle; the renderer takes them in twos, so a second with no room in it goes along.
                let (tip, left) = ([x, y, 0.5, 0.0], point(self.longitude + self.arc, 0.0));
                vec![tip, point(self.longitude - self.arc, 1.0), left, left, left, left]
            }
            // A ring in pieces of its arc, from its radius inwards by its size, the picture a quarter to a piece.
            Shape::Circle2d => {
                let inner = self.radius - self.size;
                let point = |angle: f32, radius: f32, u: f32, v: f32| [x + radius * cosine(angle), y - radius * sine(angle), u, v];
                let mut corners = Vec::new();
                let (mut angle, mut u) = (0.0, 0.0);
                while angle <= 360.0 - self.arc {
                    let next = angle + self.arc;
                    corners.extend([point(angle, inner, u, 1.0), point(next, self.radius, u + 0.25, 0.0), point(angle, self.radius, u, 0.0)]);
                    corners.extend([point(angle, inner, u, 1.0), point(next, inner, u + 0.25, 1.0), point(next, self.radius, u + 0.25, 0.0)]);
                    u = if u + 0.25 >= 1.0 { 0.0 } else { u + 0.25 };
                    angle = next;
                }
                corners
            }
            Shape::Cylinder3d => Vec::new(),
        }
    }

    /// The band's triangles now, two by two, in the world: around a middle `put.1` over `place` and as far
    /// along `facing` (a step of one on the ground) as it has gone. It stands upright across the way it faces:
    /// the near ring there, the far one its height further along.
    fn band(&self, place: Vector, facing: (f32, f32)) -> Vec<(Vector, [f32; 2])> {
        let (ahead, aside) = ([facing.0, 0.0, facing.1], [facing.1, 0.0, -facing.0]);
        let middle = along(along(place, [0.0, 1.0, 0.0], -self.put.1), ahead, self.gone.0);
        let point = |angle: f32, radius: f32, far: f32, u: f32, v: f32| (along(along(along(middle, aside, radius * cosine(angle)), [0.0, 1.0, 0.0], radius * sine(angle)), ahead, far), [u, v]);
        let mut corners = Vec::new();
        let (mut angle, mut u) = (0.0, 0.0);
        while angle <= 360.0 - self.arc {
            let next = angle + self.arc;
            corners.extend([point(angle, self.inner, 0.0, u, 1.0), point(angle, self.outer, self.height, u, 0.0), point(next, self.inner, 0.0, u + 0.25, 1.0)]);
            corners.extend([point(next, self.inner, 0.0, u + 0.25, 1.0), point(angle, self.outer, self.height, u, 0.0), point(next, self.outer, self.height, u + 0.25, 0.0)]);
            u = if u + 0.25 >= 1.0 { 0.0 } else { u + 0.25 };
            angle = next;
        }
        corners
    }
}

/// The client's sines and cosines are of whole degrees.
fn sine(degrees: f32) -> f32 {
    degrees.trunc().to_radians().sin()
}

fn cosine(degrees: f32) -> f32 {
    degrees.trunc().to_radians().cos()
}

/// A whole number from 0 up to, not reaching, `under` (the client's `rand() % under`).
fn lot(under: u32) -> f32 {
    (js_sys::Math::random() * f64::from(under)).floor() as f32
}

/// An effect of the client's under way on someone.
pub(super) struct Burst {
    /// On whom: no one named is the player's own character.
    pub(super) on: Option<u32>,
    /// When it begins, and how many of its ticks have been lived through.
    pub(super) begins: f64,
    ticks: i32,
    /// How many ticks the effect lives (the client's `EffectLifetime`).
    lasts: i32,
    prims: Vec<Prim>,
    /// The way the one it is on faced as it began: a step of one on the ground.
    facing: Option<(f32, f32)>,
}

/// The effect of that number of the client's, to begin at `begins` on someone, and the sound it starts with; none
/// for an effect not ported yet.
pub(super) fn burst(effect: u32, on: Option<u32>, begins: f64) -> Option<(Burst, &'static str)> {
    let (lasts, prims, sound) = match effect {
        0 => (50, hit1(), ""),
        1 => (50, hit2(), "effect/ef_hit2.wav"),
        16 => (40, bash(), "effect/ef_bash.wav"),
        _ => return None,
    };
    Some((Burst { on, begins, ticks: 0, lasts, prims, facing: None }, sound))
}

/// The pictures the effects here are drawn with, among the client's pictures of effects.
pub(super) const PICTURES: [&str; 5] = ["ring_blue.tga", "lens1.tga", "lens2.tga", "alpha_down.tga", "alpha_center.tga"];

/// `CRagEffect::Hit1`: a blue ring 10 above the feet of the one struck, upright and across the way that one
/// faces, that goes that way as it slows and fades over its second half.
fn hit1() -> Vec<Prim> {
    let mut ring = Prim::new(Shape::Cylinder3d, "ring_blue.tga");
    ring.duration = 10;
    ring.put.1 = -10.0;
    ring.speed = 0.7;
    ring.accel = -(ring.speed / ring.duration as f32) * 0.5;
    (ring.outer, ring.inner, ring.height) = (10.0, 5.0, 3.5);
    ring.fade_out = ring.duration - ring.duration / 2;
    vec![ring]
}

/// `CRagEffect::Hit2`: eight flares around a place 20 above the one struck, each a narrowing ray that runs
/// outwards and slows, lengthening as it goes.
fn hit2() -> Vec<Prim> {
    (0..8)
        .map(|index| {
            let degree = index as f32 * 45.0;
            let mut flare = Prim::new(Shape::Texture2d, if index % 2 == 0 { "lens1.tga" } else { "lens2.tga" });
            flare.first_pattern = true;
            flare.duration = lot(20) as i32 + 10;
            let heading = degree - 15.0 + lot(30);
            (flare.roll, flare.longitude) = (heading, heading);
            flare.speed = (lot(45) + 5.0) / 10.0;
            // Half of what would stop it by its last tick (the client's `g_depthAverageHalf`, 0.5).
            flare.accel = -(flare.speed / flare.duration as f32) * 0.5;
            flare.width = lot(15) + 5.0;
            flare.height = lot(20) + 20.0;
            flare.width_speed = -(flare.width / flare.duration as f32);
            (flare.height_speed, flare.height_accel) = (1.5, 0.25);
            let spread = lot(5);
            flare.put = (sine(degree) * spread, -cosine(degree) * spread - 20.0);
            flare.alpha = 0.0;
            flare.alpha_speed = flare.max_alpha * 0.125;
            flare
        })
        .collect()
}

/// `CRagEffect::Bash`: 40 above the one it is on, a disc of light a hundred across its radius that comes up and
/// fades, and twenty thin wedges turning about its middle, growing as they slow.
fn bash() -> Vec<Prim> {
    let mut disc = Prim::new(Shape::Circle2d, "alpha_down.tga");
    disc.duration = 40;
    disc.first_pattern = true;
    disc.put.1 = -40.0;
    disc.radius = 100.0;
    disc.alpha = 0.0;
    disc.max_alpha = 170.0;
    disc.alpha_speed = disc.max_alpha / 6.0;
    disc.fade_out = disc.duration - 10;
    let mut prims = vec![disc];
    prims.extend((0..20).map(|_| {
        let mut wedge = Prim::new(Shape::Flash2d, "alpha_center.tga");
        wedge.duration = 40;
        wedge.put.1 = -40.0;
        wedge.longitude = lot(360);
        wedge.long_speed = (lot(60) + 10.0) / 10.0;
        wedge.long_accel = -(wedge.long_speed / wedge.duration as f32) / 1.5;
        wedge.arc = (lot(25) + 5.0) / 10.0;
        wedge.height = lot(40) + 20.0;
        wedge.height_speed = (lot(30) + 20.0) / 10.0;
        wedge.alpha = 0.0;
        wedge.max_alpha = 200.0;
        wedge.alpha_speed = wedge.max_alpha / 10.0;
        wedge.fade_out = wedge.duration - wedge.duration / 3;
        wedge
    }));
    prims
}

impl Burst {
    /// Lives up to `time`, a tick of the client's at a time. Gives back whether anything of it is left.
    pub(super) fn live(&mut self, time: f64) -> bool {
        let due = ((time - self.begins) / CAST_FRAME_MS) as i32;
        while self.ticks < due && self.ticks < self.lasts {
            let left = self.lasts - self.ticks;
            self.prims.retain_mut(|prim| prim.tick(left));
            self.ticks += 1;
        }
        self.ticks < self.lasts && !self.prims.is_empty()
    }

    /// Its triangles now, two at a time: around `place`, the feet of the one it is on, who faces `facing` now;
    /// the flat kinds across the screen along `right` and `up`. `draw` is given the picture, each of the six corners as its place in the world and where
    /// of the picture it shows, and the colour.
    pub(super) fn draw(&mut self, place: Vector, facing: (f32, f32), right: Vector, up: Vector, mut draw: impl FnMut(&'static str, [(Vector, [f32; 2]); 6], [f32; 4])) {
        let facing = *self.facing.get_or_insert(facing);
        for prim in self.prims.iter().filter(|prim| prim.alpha >= 1.0) {
            let colour = [1.0, 1.0, 1.0, prim.alpha.trunc() / 255.0];
            for pair in prim.band(place, facing).chunks_exact(6).filter(|_| prim.kind == Shape::Cylinder3d) {
                draw(prim.picture, std::array::from_fn(|index| pair[index]), colour);
            }
            for pair in prim.corners().chunks_exact(6) {
                let corner = |index: usize| (along(along(place, right, pair[index][0] * SPRITE_SCALE), up, -pair[index][1] * SPRITE_SCALE), [pair[index][2], pair[index][3]]);
                draw(prim.picture, std::array::from_fn(corner), colour);
            }
        }
    }
}
