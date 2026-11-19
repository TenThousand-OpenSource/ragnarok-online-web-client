//! The kinds that lie flat on the screen: how a picture and a kite move (`Move2DPrimitive`), and how each kind
//! is shaped, its triangles as the client's drawing functions make them (`Render2DPrimitive`,
//! `Render2DDiamond`, the wedge in `CEffectPrim::Render`, `Render2DCircle`).

use super::*;

impl Prim {
    /// `Move2DPrimitive`. (Of its change at a tick only the two speeds of size are ported, and of its turning
    /// the speed alone, `rollSpeed`: no effect here gives it `rollAccel`.)
    pub(super) fn slide(&mut self) {
        if let Some((_, width_speed, height_speed)) = self.turns_at.filter(|(tick, ..)| *tick == self.count) {
            (self.width_speed, self.height_speed) = (width_speed, height_speed);
        }
        self.width_speed += self.width_accel;
        self.height_speed += self.height_accel;
        self.speed += self.accel;
        if self.settles_at.is_some_and(|tick| tick <= self.count) {
            (self.width_accel, self.width_speed, self.height_accel, self.height_speed) = (0.0, 0.0, 0.0, 0.0);
        }
        self.roll += self.roll_speed;
        self.width += self.width_speed;
        self.height += self.height_speed;
        self.gone.0 += sine(self.longitude) * self.speed;
        self.gone.1 -= cosine(self.longitude) * self.speed;
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
            Shape::Quad2d => self.kite(x, y),
            Shape::Cylinder3d | Shape::Circle3d | Shape::Sphere3d | Shape::CrossTexture3d | Shape::Particle3d | Shape::ParticleGravity3d | Shape::ParticleOrbit3d | Shape::ParticleSpline3d | Shape::QuadHorn3d | Shape::ParticleUp | Shape::Casting3d | Shape::Walls(_) | Shape::Tei(_) | Shape::Texture3d | Shape::Cast(_) | Shape::Streak { .. } | Shape::SpriteGlow | Shape::Kouenka | Shape::Fourfold | Shape::Roamer | Shape::Petal { .. } => Vec::new(),
        }
    }

    /// `Render2DDiamond`: a kite of two triangles about its place, turned by its roll: half its height up and
    /// down, and half its width to each side, the left corner a sixth of the half height above the middle and
    /// the right one as far below it. The picture's upright middle runs from tip to tip.
    fn kite(&self, x: f32, y: f32) -> Vec<[f32; 4]> {
        let (sin, cos) = (sine(self.roll), cosine(self.roll));
        let (half_tall, half_wide) = (self.height * 0.5, self.width * 0.5);
        let notch = half_tall / 1.2;
        let corner = |across: f32, along: f32, u: f32, v: f32| [across * cos - along * sin + x, along * cos + across * sin + y, u, v];
        let (low, high) = (corner(0.0, half_tall, 0.5, 1.0), corner(0.0, -half_tall, 0.5, 0.0));
        vec![corner(-half_wide, notch - half_tall, 0.0, 0.5), low, high, corner(half_wide, half_tall - notch, 1.0, 0.5), low, high]
    }

    /// Whether a corner of its triangles is drawn white and whole whatever the primitive's colour and alpha
    /// are: the second corner of each triangle of a kite, its lower tip (`Render2DDiamond` gives that one
    /// `0xFFFFFFFF`).
    pub fn glares(&self, corner: usize) -> bool {
        self.kind == Shape::Quad2d && corner % 3 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    #[test]
    fn a_kite_has_its_tips_up_and_down_and_its_sides_off_the_middle() {
        let mut kite = Prim::new(Shape::Quad2d, "");
        (kite.width, kite.height, kite.put) = (10.0, 120.0, (3.0, -20.0));
        let corners = kite.corners();
        assert_eq!(corners.len(), 6);
        // The left corner 10 above the middle (a sixth of the half height), the right one 10 below.
        assert!(close(corners[0][0], -2.0) && close(corners[0][1], -30.0) && corners[0][2..] == [0.0, 0.5]);
        assert!(close(corners[3][0], 8.0) && close(corners[3][1], -10.0) && corners[3][2..] == [1.0, 0.5]);
        // Both triangles share the lower and the upper tip; the lower one glares.
        assert_eq!((corners[1], corners[2]), (corners[4], corners[5]));
        assert!(close(corners[1][0], 3.0) && close(corners[1][1], 40.0) && corners[1][2..] == [0.5, 1.0]);
        assert!(close(corners[2][1], -80.0) && corners[2][2..] == [0.5, 0.0]);
        assert_eq!([0, 1, 2, 3, 4, 5].map(|corner| kite.glares(corner)), [false, true, false, false, true, false]);
        assert!(!Prim::new(Shape::Texture2d, "").glares(1));
        // Rolled a quarter clockwise, its lower tip points left (the screen's y points down).
        kite.roll = 90.0;
        let corners = kite.corners();
        assert!(close(corners[1][0], -57.0) && close(corners[1][1], -20.0));
    }

    #[test]
    fn a_flat_picture_takes_new_speeds_of_size_at_its_tick_and_rolls() {
        // As the smoke of a hit of cold: fast for ten ticks, slowly after.
        let mut smoke = Prim::new(Shape::Texture2d, "");
        (smoke.width, smoke.height, smoke.width_speed, smoke.height_speed) = (1.0, 1.0, 6.0, 6.0);
        (smoke.turns_at, smoke.roll_speed) = (Some((10, 0.5, 0.25)), 2.0);
        for _ in 0..10 {
            smoke.tick(100);
        }
        assert!(close(smoke.width, 61.0) && close(smoke.height, 61.0) && close(smoke.roll, 20.0));
        smoke.tick(100);
        assert!(close(smoke.width, 61.5) && close(smoke.height, 61.25));
    }
}
