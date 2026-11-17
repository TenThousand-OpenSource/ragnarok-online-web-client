//! The camera over the character and the view it gives: what the screen shows of the world, and what of the
//! world a point of the screen is.

use super::*;

impl Camera {
    /// How far the camera may tilt and how far away it may be, outdoors or inside (the original's limits: 20
    /// degrees either way of 45 outdoors and 10 inside, and inside no farther than where it starts).
    pub(super) fn limits(indoor: bool, zoom_out: bool) -> ([f32; 2], [f32; 2]) {
        // Outdoors as far as the client's Zoom Out option lets it: 730 with it, 400 without.
        let far = if zoom_out { DISTANCE_RANGE[1] } else { DISTANCE_NEAR_MOST };
        if indoor { ([35.0, 55.0], [DISTANCE_RANGE[0], DISTANCE]) } else { (PITCH_RANGE, [DISTANCE_RANGE[0], far]) }
    }

    /// The camera a map starts with: looking north outdoors, turned 45 degrees inside, as near as it was left.
    pub(super) fn starting(indoor: bool, zoom_out: bool) -> Self {
        let (_, distances) = Self::limits(indoor, zoom_out);
        Self { yaw: if indoor { 45.0 } else { 0.0 }, pitch: PITCH, distance: Self::remembered().distance.clamp(distances[0], distances[1]) }
    }

    /// The camera as the player last left it, on any map and in any earlier visit; the game's own if never moved.
    pub(super) fn remembered() -> Self {
        let kept = web_sys::window().and_then(|window| window.local_storage().ok().flatten()).and_then(|storage| storage.get_item(CAMERA_KEPT).ok().flatten());
        let numbers: Vec<f32> = kept.unwrap_or_default().split(',').filter_map(|number| number.parse().ok()).filter(|number: &f32| number.is_finite()).collect();
        match numbers[..] {
            [yaw, pitch, distance] => Self { yaw: yaw.rem_euclid(360.0), pitch: pitch.clamp(PITCH_RANGE[0], PITCH_RANGE[1]), distance: distance.clamp(DISTANCE_RANGE[0], DISTANCE_RANGE[1]) },
            _ => Self::default(),
        }
    }

    /// Keeps the camera for the next map and the next visit.
    pub(super) fn remember(&self) {
        if let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok().flatten()) {
            let _ = storage.set_item(CAMERA_KEPT, &format!("{},{},{}", self.yaw, self.pitch, self.distance));
        }
    }

    /// The world is left-handed (x east, y up, z north), and so is this view: depth grows along `forward`.
    pub(super) fn view(&self, target: Vector, aspect: f32) -> View {
        let (yaw, pitch) = (self.yaw.to_radians(), self.pitch.to_radians());
        let horizontal = self.distance * pitch.cos();
        let eye = [target[0] - horizontal * yaw.sin(), target[1] + self.distance * pitch.sin(), target[2] - horizontal * yaw.cos()];
        let forward = normalize(subtract(target, eye));
        let right = normalize(cross([0.0, 1.0, 0.0], forward));
        let up = cross(forward, right);
        let focal = 1.0 / (FIELD_OF_VIEW.to_radians() / 2.0).tan();
        let depth = FAR / (FAR - NEAR);
        // Projection times view, written out: each column is what a world axis (or the origin) becomes.
        let column = |axis: Vector, offset: f32| {
            let (x, y, z) = (dot(right, axis) + offset * -dot(right, eye), dot(up, axis) + offset * -dot(up, eye), dot(forward, axis) + offset * -dot(forward, eye));
            [x * focal / aspect, y * focal, z * depth - offset * NEAR * depth, z]
        };
        let columns = [column([1.0, 0.0, 0.0], 0.0), column([0.0, 1.0, 0.0], 0.0), column([0.0, 0.0, 1.0], 0.0), column([0.0, 0.0, 0.0], 1.0)];
        let mut matrix = [0.0; 16];
        for (index, column) in columns.iter().enumerate() {
            matrix[index * 4..index * 4 + 4].copy_from_slice(column);
        }
        View { eye, forward, right, up, aspect, matrix }
    }
}

/// One frame of the client, which moves its camera a tenth of the way to the character each: taken as a
/// sixtieth of a second (the client's own pace is not in what was read of it).
const CAMERA_FRAME_MS: f64 = 1000.0 / 60.0;

impl Scene {
    /// Where the camera looks now: at the character's feet until it has started to follow them.
    pub(super) fn looked_at(&self) -> Vector {
        self.aim.map_or_else(|| on_ground(&self.gat, self.player.at), |(aim, _)| aim)
    }

    /// Moves the camera's aim towards the character's feet, as far as the time since it was last moved allows
    /// (the client's `CameraState::Smooth`: a tenth of what is left, each frame).
    pub(super) fn follow(&mut self, feet: Vector, time: f64) -> Vector {
        let aim = match self.aim {
            Some((aim, moved)) => {
                let share = 1.0 - 0.9f64.powf(((time - moved) / CAMERA_FRAME_MS).clamp(0.0, 600.0));
                [0, 1, 2].map(|axis| aim[axis] + (feet[axis] - aim[axis]) * share as f32)
            }
            None => feet,
        };
        self.aim = Some((aim, time));
        aim
    }
}

impl View {
    /// The direction from the eye through a point of the screen (0 to 1 on each axis, y down).
    pub(super) fn ray(&self, screen: (f32, f32)) -> Vector {
        let tangent = (FIELD_OF_VIEW.to_radians() / 2.0).tan();
        let across = (screen.0 * 2.0 - 1.0) * tangent * self.aspect;
        let upward = (1.0 - screen.1 * 2.0) * tangent;
        normalize(along(along(self.forward, self.right, across), self.up, upward))
    }

    /// Where on the screen a point of the world shows (0 to 1 on each axis, y down), if it is in front of the eye.
    pub(super) fn project(&self, [x, y, z]: Vector) -> Option<(f32, f32)> {
        let matrix = &self.matrix;
        let clip = |row: usize| matrix[row] * x + matrix[4 + row] * y + matrix[8 + row] * z + matrix[12 + row];
        let depth = clip(3);
        (depth > 0.0).then(|| ((clip(0) / depth + 1.0) / 2.0, (1.0 - clip(1) / depth) / 2.0))
    }

    /// The cell of the ground a point of the screen shows.
    pub(super) fn pick(&self, gat: &Gat, screen: (f32, f32)) -> Option<(i32, i32)> {
        let direction = self.ray(screen);
        let mut distance = NEAR;
        while distance < FAR {
            let point = along(self.eye, direction, distance);
            let (x, y) = (point[0] / CELL, point[2] / CELL);
            let inside = x >= 0.0 && y >= 0.0 && x < gat.width as f32 && y < gat.height as f32;
            if inside && point[1] <= gat.height_at(x, y) {
                return Some((x as i32, y as i32));
            }
            distance += 1.0;
        }
        None
    }
}
