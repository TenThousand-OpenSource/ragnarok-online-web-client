//! How the primitives that stand in the world are shaped: the triangles of each kind, two by two, as the
//! client's drawing functions make them (`RenderArcBand`, `RenderRecordRings`, `RenderSphere`,
//! `RenderCrossPlanes`; the spike's `RenderPyramid` is in `horn.rs`). The client leaves out a triangle that falls off its screen; here all are given.

use super::*;

/// A corner in the world (east, up, north) with where of the picture it shows.
type Corner = ([f32; 3], [f32; 2]);

impl Prim {
    /// The triangles now of a kind that stands in the world, two by two, around `place`; none for the kinds
    /// that lie flat on the screen or show a sprite.
    pub fn solid(&self, place: [f32; 3]) -> Vec<Corner> {
        match self.kind {
            Shape::Cylinder3d => self.band(place),
            Shape::Circle3d => self.ring(place),
            Shape::Sphere3d => self.ball(place),
            Shape::CrossTexture3d => self.cross(place),
            Shape::QuadHorn3d => self.horn(place),
            Shape::Texture2d | Shape::Quad2d | Shape::Flash2d | Shape::Circle2d | Shape::Particle3d | Shape::ParticleGravity3d | Shape::ParticleOrbit3d | Shape::ParticleSpline3d | Shape::ParticleUp | Shape::Casting3d | Shape::Walls(_) | Shape::Tei(_) | Shape::Texture3d | Shape::Cast(_) | Shape::Streak { .. } | Shape::SpriteGlow | Shape::Kouenka | Shape::Fourfold | Shape::Roamer | Shape::Petal { .. } => Vec::new(),
        }
    }

    /// The band's triangles now, two by two, in the world: around a middle `put.1` over `place` (the client's
    /// y, which points down), moved as far as it has gone. Its near ring lies in its own flat, its far ring its
    /// height up from it (`RenderArcBand`).
    pub(super) fn band(&self, place: [f32; 3]) -> Vec<Corner> {
        let middle = self.middle(place);
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

    /// `RenderRecordRings`, for a ring of one segment as all here are: a ring in its own flat, turned as the
    /// primitive is, in pieces of its arc, from its radius inwards by its size; the picture a quarter to a
    /// piece, its bottom at the inner edge.
    fn ring(&self, place: [f32; 3]) -> Vec<Corner> {
        let middle = self.middle(place);
        let inner = self.radius - self.size;
        let point = |angle: f32, radius: f32, u: f32, v: f32| (add(middle, self.turned([radius * cosine(angle), radius * sine(angle), 0.0])), [u, v]);
        let mut corners = Vec::new();
        let (mut angle, mut u) = (0.0, 0.0);
        while angle <= 360.0 - self.arc {
            let next = angle + self.arc;
            corners.extend([point(angle, inner, u, 1.0), point(next, inner, u + 0.25, 1.0), point(angle, self.radius, u, 0.0)]);
            corners.extend([point(next, self.radius, u + 0.25, 0.0), point(angle, self.radius, u, 0.0), point(next, inner, u + 0.25, 1.0)]);
            u = if u + 0.25 >= 1.0 { 0.0 } else { u + 0.25 };
            angle = next;
        }
        corners
    }

    /// `RenderSphere`: a ball of its radius in pieces of its arc both round and from its bottom to its top,
    /// turned round by its longitude; the picture is wrapped once round it, its top row at the ball's bottom.
    /// The far side of a piece is a quarter of a degree further round than the next piece begins, which the
    /// client's whole degrees swallow.
    fn ball(&self, place: [f32; 3]) -> Vec<Corner> {
        let middle = self.middle(place);
        let point = |latitude: f32, longitude: f32, u: f32, v: f32| (add(middle, turn(latitude, longitude, [0.0, 0.0, self.radius])), [u, v]);
        let (u_step, v_step) = (self.arc / 360.0, self.arc / 180.0);
        let mut corners = Vec::new();
        let (mut latitude, mut v) = (-90.0, 0.0);
        while latitude <= 90.0 - self.arc {
            let (mut round, mut u) = (0.0, 0.0);
            while round <= 360.0 - self.arc {
                let (near, far) = (round + self.longitude, round + self.arc + 0.25 + self.longitude);
                let (higher, next_u, next_v) = (latitude + self.arc, u + u_step, v + v_step);
                corners.extend([point(latitude, near, u, v), point(higher, near, u, next_v), point(latitude, far, next_u, v)]);
                corners.extend([point(higher, far, next_u, next_v), point(latitude, far, next_u, v), point(higher, near, u, next_v)]);
                u = next_u;
                round += self.arc;
            }
            v += v_step;
            latitude += self.arc;
        }
        corners
    }

    /// `RenderCrossPlanes`: the picture twice, crossed along its own forward: once across its own x and once
    /// across its own y, its height to each side and its width forward and back, turned as the primitive is.
    fn cross(&self, place: [f32; 3]) -> Vec<Corner> {
        let middle = self.middle(place);
        let (height, width) = (self.height, self.width);
        let planes = [
            [[-height, 0.0, width], [-height, 0.0, -width], [height, 0.0, width], [height, 0.0, -width]],
            [[0.0, -height, width], [0.0, -height, -width], [0.0, height, width], [0.0, height, -width]],
        ];
        let mut corners = Vec::new();
        for plane in planes {
            let point = |index: usize, u: f32, v: f32| (add(middle, self.turned(plane[index])), [u, v]);
            corners.extend([point(0, 0.0, 1.0), point(1, 1.0, 1.0), point(2, 0.0, 0.0)]);
            corners.extend([point(3, 1.0, 0.0), point(2, 0.0, 0.0), point(1, 1.0, 1.0)]);
        }
        corners
    }
}

pub(super) fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

impl Prim {
    /// `Prim3DCylinder`: it goes along its own upright by its speed, turns (twice its turning speed a
    /// tick, as the client adds it twice), and its two rings and its height change by their speeds,
    /// which change by theirs. With the first pattern its far ring closes over the time it fades.
    pub(super) fn drill(&mut self) {
        self.speed += self.accel;
        self.height_speed += self.height_accel;
        self.outer_speed += self.outer_accel;
        self.inner_speed += self.inner_accel;
        self.long_speed += self.long_accel;
        self.longitude += self.long_speed;
        let step = self.turned([0.0, -self.speed, 0.0]);
        self.moved = [self.moved[0] + step[0], self.moved[1] + step[1], self.moved[2] + step[2]];
        self.longitude += self.long_speed;
        self.inner += self.inner_speed;
        self.outer += self.outer_speed;
        self.height = (self.height + self.height_speed).min(100.0);
        if self.first_pattern && self.count == self.fade_out {
            self.outer_speed = -(self.outer / (self.duration - self.count + 1) as f32);
        }
    }

    /// `Prim3DCircle`: it widens by its speed, turns, and goes along its own forward by its speed; it
    /// is filled from its edge inwards by its inner size, or to its middle with the first pattern or
    /// while it is narrower than that. (The client can change its inner size by a speed too; none of the
    /// effects here gives it one.)
    pub(super) fn ring_out(&mut self) {
        self.radius_speed += self.radius_accel;
        self.long_speed += self.long_accel;
        self.speed += self.accel;
        self.longitude += self.long_speed;
        self.radius += self.radius_speed;
        self.go_forward();
        self.size = if self.first_pattern || self.inner > self.radius { self.radius } else { self.inner };
    }

    /// `CEffectPrim::OnProcess`, `PP_3DSPHERE`: it swells by its speed and turns by its turning speed.
    pub(super) fn swell(&mut self) {
        self.radius_speed += self.radius_accel;
        self.radius += self.radius_speed;
        self.longitude += self.long_speed;
    }
}
