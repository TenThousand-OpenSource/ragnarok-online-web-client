//! The client's effect primitives (`CEffectPrim`): the figures of one and how it lives and is shaped, kind by
//! kind as they are ported. Nothing here knows of the page or the renderer: `web/world/prims.rs` runs the
//! effects made of these and draws them.
//!
//! - `PP_2DTEXTURE` (0), `PP_2DFLASH` (3), `PP_2DCIRCLE` (4): flat on the screen, around the place on the screen
//!   of the one the effect is on (`CEffectPrim::OnProcess`, `Move2DPrimitive`, `ProcessAlpha`,
//!   `Render2DPrimitive`, `Render2DCircle`);
//! - `PP_3DCYLINDER` (13): a band between two rings in the world, turned as the effect says
//!   (`Prim3DCylinder`, `RenderArcBand`).

/// The kinds of primitive ported, by the client's names.
#[derive(Clone, Copy, PartialEq)]
pub enum Shape {
    Texture2d,
    Flash2d,
    Circle2d,
    Cylinder3d,
}

/// A primitive's figures: the client's `EffectPrimitiveState`, the fields the ported kinds use, with the values
/// it starts them at.
#[derive(Clone)]
pub struct Prim {
    pub kind: Shape,
    pub picture: &'static str,
    /// Ticks lived, and how many it lives.
    pub count: i32,
    pub duration: i32,
    /// The client's `pattern & 1`: what it means is each kind's own.
    pub first_pattern: bool,
    /// The tick from which a flat picture neither grows nor shrinks any more (the client's `pattern & 2` with
    /// its `chCnt`).
    pub settles_at: Option<i32>,
    /// Where it is, from the effect's place on the screen: where it was put (`deltaPos2`) and how far it has
    /// gone since (`deltaPos`).
    pub put: (f32, f32),
    pub gone: (f32, f32),
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
    pub size: f32,
    /// The band's two rings: how wide the near one and the far one are (the far one `height` away), how fast
    /// the far one widens, and how the band is tipped (`latitude`, about the line across) before it is turned
    /// (`longitude`, about the upright).
    pub inner: f32,
    pub outer: f32,
    pub outer_speed: f32,
    pub latitude: f32,
    /// How far it has gone in the world since it began (east, up, north).
    pub moved: [f32; 3],
    pub arc: f32,
    pub alpha: f32,
    pub max_alpha: f32,
    pub alpha_speed: f32,
    /// The tick it begins to fade at, to be gone by its last.
    pub fade_out: i32,
}

impl Prim {
    pub fn new(kind: Shape, picture: &'static str) -> Self {
        Self {
            kind,
            picture,
            count: 0,
            duration: 100,
            first_pattern: false,
            settles_at: None,
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
            width_accel: 0.0,
            height: 0.0,
            height_speed: 0.0,
            height_accel: 0.0,
            radius: 0.0,
            size: 1.0,
            inner: 0.0,
            outer: 0.0,
            outer_speed: 0.0,
            latitude: 0.0,
            moved: [0.0; 3],
            arc: 36.0,
            alpha: 254.0,
            max_alpha: 254.0,
            alpha_speed: 0.0,
            fade_out: -1,
        }
    }

    /// One tick of its life. `left` is how many ticks the effect it belongs to still has. Gives back whether it
    /// lives on.
    pub fn tick(&mut self, left: i32) -> bool {
        match self.kind {
            Shape::Texture2d => {
                self.width_speed += self.width_accel;
                self.height_speed += self.height_accel;
                self.speed += self.accel;
                if self.settles_at.is_some_and(|tick| tick <= self.count) {
                    (self.width_accel, self.width_speed, self.height_accel, self.height_speed) = (0.0, 0.0, 0.0, 0.0);
                }
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
            // `Prim3DCylinder`: it goes along its own upright by its speed, turns (twice its turning speed a
            // tick, as the client adds it twice), and its far ring and its height change by theirs. With the
            // first pattern its far ring closes over the time it fades.
            Shape::Cylinder3d => {
                self.speed += self.accel;
                self.height_speed += self.height_accel;
                self.long_speed += self.long_accel;
                self.longitude += self.long_speed;
                let step = self.turned([0.0, -self.speed, 0.0]);
                self.moved = [self.moved[0] + step[0], self.moved[1] + step[1], self.moved[2] + step[2]];
                self.longitude += self.long_speed;
                self.outer += self.outer_speed;
                self.height = (self.height + self.height_speed).min(100.0);
                if self.first_pattern && self.count == self.fade_out {
                    self.outer_speed = -(self.outer / (self.duration - self.count + 1) as f32);
                }
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
    pub fn corners(&self) -> Vec<[f32; 4]> {
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

    /// A point of the primitive's own space turned as the primitive is, in the world's directions (east, up,
    /// north). The client tips it about its x by the latitude and then turns it about its y by the longitude
    /// (`MakeXRotation`, `AppendYRotation`, on rows); its y points down, the world's up.
    fn turned(&self, [x, y, z]: [f32; 3]) -> [f32; 3] {
        let (tip_sin, tip_cos) = (sine(self.latitude), cosine(self.latitude));
        let (y, z) = (y * tip_cos - z * tip_sin, y * tip_sin + z * tip_cos);
        let (turn_sin, turn_cos) = (sine(self.longitude), cosine(self.longitude));
        [x * turn_cos + z * turn_sin, -y, -x * turn_sin + z * turn_cos]
    }

    /// The band's triangles now, two by two, in the world: around a middle `put.1` over `place` (the client's
    /// y, which points down), moved as far as it has gone. Its near ring lies in its own flat, its far ring its
    /// height up from it (`RenderArcBand`).
    pub fn band(&self, place: [f32; 3]) -> Vec<([f32; 3], [f32; 2])> {
        let middle = [place[0] + self.put.0 + self.moved[0], place[1] - self.put.1 + self.moved[1], place[2] + self.moved[2]];
        let point = |angle: f32, radius: f32, lift: f32, u: f32, v: f32| {
            let turned = self.turned([radius * cosine(angle), -lift, radius * sine(angle)]);
            ([middle[0] + turned[0], middle[1] + turned[1], middle[2] + turned[2]], [u, v])
        };
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
pub fn sine(degrees: f32) -> f32 {
    degrees.trunc().to_radians().sin()
}

pub fn cosine(degrees: f32) -> f32 {
    degrees.trunc().to_radians().cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    #[test]
    fn a_primitive_lives_its_ticks_and_fades_to_nothing() {
        let mut prim = Prim::new(Shape::Circle2d, "");
        (prim.duration, prim.fade_out) = (10, 5);
        let mut lived = 0;
        while prim.tick(100) {
            lived += 1;
        }
        // It is told to go on for its ten ticks, and the eleventh is its last.
        assert_eq!(lived, 10);
        // From its fifth tick it fades evenly, and is gone by its last.
        assert!(close(prim.alpha, 0.0));
    }

    #[test]
    fn a_primitive_that_would_outlive_its_effect_fades_with_it() {
        let mut prim = Prim::new(Shape::Circle2d, "");
        prim.duration = 100;
        // Four ticks of the effect left: it loses a quarter of what it has left each tick.
        prim.tick(4);
        assert!(close(prim.alpha, 254.0 * 0.75));
    }

    #[test]
    fn a_flat_picture_stops_changing_size_when_it_settles() {
        let mut prim = Prim::new(Shape::Texture2d, "");
        (prim.width, prim.width_speed, prim.settles_at) = (10.0, -1.0, Some(3));
        for _ in 0..3 {
            prim.tick(100);
        }
        assert!(close(prim.width, 7.0));
        for _ in 0..5 {
            prim.tick(100);
        }
        assert!(close(prim.width, 7.0));
    }

    #[test]
    fn a_ring_on_the_screen_is_ten_pieces_of_two_triangles() {
        let mut ring = Prim::new(Shape::Circle2d, "");
        (ring.radius, ring.size) = (10.0, 2.0);
        let corners = ring.corners();
        assert_eq!(corners.len(), 10 * 6);
        // Its first corner is on the inner edge, to the right of the middle.
        assert!(close(corners[0][0], 8.0) && close(corners[0][1], 0.0));
        // Every corner is between the inner edge and the outer.
        assert!(corners.iter().all(|corner| (7.99..=10.01).contains(&corner[0].hypot(corner[1]))));
    }

    #[test]
    fn a_band_lying_flat_has_its_far_ring_above_its_near_one() {
        let mut band = Prim::new(Shape::Cylinder3d, "");
        (band.inner, band.outer, band.height) = (5.0, 6.0, 3.0);
        band.put.1 = -10.0;
        let corners = band.band([100.0, 20.0, 200.0]);
        assert_eq!(corners.len(), 10 * 6);
        // The near ring is 10 over the place, the far ring its height above that, each around the place.
        let (near, far) = (corners[0].0, corners[1].0);
        assert!(close(near[1], 30.0) && close(far[1], 33.0));
        assert!(close((near[0] - 100.0).hypot(near[2] - 200.0), 5.0));
        assert!(close((far[0] - 100.0).hypot(far[2] - 200.0), 6.0));
    }

    #[test]
    fn a_band_stood_up_goes_the_way_it_is_turned() {
        // As the ring of a blow is: tipped up, and turned to face south (the one struck looks south: 180 - 0).
        let mut band = Prim::new(Shape::Cylinder3d, "");
        (band.latitude, band.longitude, band.speed) = (-90.0, 180.0, 1.0);
        band.tick(100);
        // South is towards smaller north; it has gone one that way and not up.
        assert!(close(band.moved[0], 0.0) && close(band.moved[1], 0.0) && close(band.moved[2], -1.0));
    }
}
