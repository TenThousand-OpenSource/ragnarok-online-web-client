//! The camera over the character and the view it gives: what the screen shows of the world, and what of the
//! world a point of the screen is.

use super::*;
use crate::scenery::views;

pub(super) const FIELD_OF_VIEW: f32 = 15.0;
pub(super) const NEAR: f32 = 10.0;
pub(super) const FAR: f32 = 5000.0;
/// How far away and how tilted the camera is when never moved (the rules of it: `crate::scenery::views`).
pub(super) use views::DISTANCE;
use views::PITCH;

/// The camera: where it looks, and from where.
#[derive(Clone, Copy)]
pub(super) struct Camera {
    /// Degrees; 0 looks north, growing clockwise.
    pub(super) yaw: f32,
    /// Degrees above the horizon.
    pub(super) pitch: f32,
    pub(super) distance: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self { yaw: 0.0, pitch: PITCH, distance: DISTANCE }
    }
}

/// Under which name the browser keeps the camera.
pub(super) const CAMERA_KEPT: &str = "roweb.camera";

/// The camera at one moment: its place and axes, and the matrix that takes the world to the screen.
pub(super) struct View {
    pub(super) eye: Vector,
    pub(super) forward: Vector,
    pub(super) right: Vector,
    pub(super) up: Vector,
    pub(super) aspect: f32,
    /// Column by column, for the shaders.
    pub(super) matrix: [f32; 16],
}

impl Camera {
    /// The camera a map starts with: looking north outdoors, turned 45 degrees inside, as near as it was left.
    pub(super) fn starting(indoor: bool, zoom_out: bool) -> Self {
        let (_, distances) = views::limits(indoor, zoom_out);
        Self { yaw: if indoor { 45.0 } else { 0.0 }, pitch: PITCH, distance: Self::remembered().distance.clamp(distances[0], distances[1]) }
    }

    /// The camera a map is entered with: the one the client's table of views gives the map, if it has one
    /// (`CGameMode::InitializeCamera`: its distance, turn and tilt `_IN`), else the one a map starts with.
    pub(super) fn entering(indoor: bool, zoom_out: bool, view: Option<&crate::scenery::views::MapView>) -> Self {
        view.map_or_else(|| Self::starting(indoor, zoom_out), |view| Self { yaw: view.yaw.rem_euclid(360.0), pitch: view.pitch, distance: view.distance })
    }

    /// The camera as the player last left it, on any map and in any earlier visit; the game's own if never moved.
    pub(super) fn remembered() -> Self {
        let kept = web_sys::window().and_then(|window| window.local_storage().ok().flatten()).and_then(|storage| storage.get_item(CAMERA_KEPT).ok().flatten());
        match views::kept(&kept.unwrap_or_default()) {
            Some([yaw, pitch, distance]) => Self { yaw, pitch, distance },
            None => Self::default(),
        }
    }

    /// Keeps the camera for the next map and the next visit.
    pub(super) fn remember(&self) {
        if let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok().flatten()) {
            let _ = storage.set_item(CAMERA_KEPT, &format!("{},{},{}", self.yaw, self.pitch, self.distance));
        }
    }

    /// The world is left-handed (x east, y up, z north), and so is this view: depth grows along `forward`.
    /// The eye is never under the ground it is over (`CView::Refresh`, rodecomp
    /// Source/Contents/Camera/CameraFrame.cpp:23-27: an eye below the terrain's height there is put at that
    /// height), or a camera laid low behind a hill looks out from inside it and sees nothing of the map.
    pub(super) fn view(&self, target: Vector, aspect: f32, gat: &Gat) -> View {
        let (yaw, pitch) = (self.yaw.to_radians(), self.pitch.to_radians());
        let horizontal = self.distance * pitch.cos();
        let mut eye = [target[0] - horizontal * yaw.sin(), target[1] + self.distance * pitch.sin(), target[2] - horizontal * yaw.cos()];
        eye[1] = eye[1].max(gat.height_at(eye[0] / CELL, eye[2] / CELL));
        let forward = normalize(subtract(target, eye));
        let right = normalize(cross([0.0, 1.0, 0.0], forward));
        let up = cross(forward, right);
        let focal = 1.0 / (FIELD_OF_VIEW.to_radians() / 2.0).tan();
        let depth = FAR / (FAR - NEAR);
        // Projection times view, written out: each column is what a world axis (or the origin) becomes.
        let column = |axis: Vector, offset: f32| {
            let (x, y, z) = (
                dot(right, axis) + offset * -dot(right, eye),
                dot(up, axis) + offset * -dot(up, eye),
                dot(forward, axis) + offset * -dot(forward, eye)
            );
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

/// The camera as a shake of the screen has it at this frame, and where it looks (`crate::quake`, the client's
/// `CView::Apply`): the place looked at thrown across, the distance and the tip changed, by what the shake has
/// added so far. A shake whose time is over is over, and the camera is as it was.
pub(super) fn shaken(quake: &mut Option<crate::quake::Quake>, camera: Camera, aim: Vector, time: f64) -> (Camera, Vector) {
    let Some(shake) = quake.as_mut().filter(|shake| time < shake.until) else {
        *quake = None;
        return (camera, aim);
    };
    shake.frame(&mut || (js_sys::Math::random() * 5.0) as u32);
    let [across, distance, pitch] = shake.off;
    let yaw = camera.yaw.to_radians();
    (Camera { distance: camera.distance + distance, pitch: camera.pitch + pitch, ..camera }, along(aim, [yaw.cos(), 0.0, -yaw.sin()], across))
}

/// One frame of the client, which moves its camera a tenth of the way to the character each.
const CAMERA_FRAME_MS: f64 = scenery::CLIENT_FRAME_MS;

impl Scene {
    /// Where the camera looks now: at the character's feet until it has started to follow them.
    pub(super) fn looked_at(&self) -> Vector {
        self.aim.map_or_else(|| on_ground(&self.gat, self.player.at), |(aim, _)| aim)
    }

    /// The view the camera gives now on that canvas: what the mouse is told the world by between frames.
    pub(super) fn view(&self, gpu: &Gpu) -> View {
        self.camera.view(self.looked_at(), gpu.config.width as f32 / gpu.config.height as f32, &self.gat)
    }

    /// Moves the camera's aim towards the character's feet, as far as the time since it was last moved allows
    /// (the client's `CameraState::Smooth`: a tenth of what is left, each frame).
    pub(super) fn follow(&mut self, feet: Vector, time: f64) -> Vector {
        let aim = match self.aim {
            Some((aim, moved)) => {
                views::followed(aim, feet, views::share((time - moved) / CAMERA_FRAME_MS))
            }
            None => feet,
        };
        self.aim = Some((aim, time));
        aim
    }
}

impl Scene {
    /// How far the camera may tilt and how far away it may be on this map, and between which turns it may be
    /// moved, if not all the way round: what the client's table of views says of the map, if it has it
    /// (`ProcessRightButtonDrag`, `ProcessMouseWheel`), else the limits of a map outdoors or inside.
    pub(super) fn camera_limits(&self, zoom_out: bool) -> ([f32; 2], [f32; 2], Option<[f32; 2]>) {
        match &self.scenery.view {
            Some(view) => (view.pitches, view.distances, Some(view.yaws)),
            None => {
                let (pitches, distances) = views::limits(self.indoor, zoom_out);
                (pitches, distances, None)
            }
        }
    }

    /// Asks the camera to be as `change` makes it, within what this map allows: it goes there by and by.
    pub(super) fn ask_camera(&mut self, zoom_out: bool, change: impl FnOnce(&mut Camera)) {
        let (pitches, distances, yaws) = self.camera_limits(zoom_out);
        let asked = &mut self.wanted.0;
        change(asked);
        asked.pitch = views::held_tilt(asked.pitch, pitches);
        asked.distance = views::held_distance(asked.distance, distances);
        // (A map of the table holds the turn between two marks; one it lets go all the way round, from
        // -360 to 360, turns as any other.)
        asked.yaw = views::held_turn(asked.yaw, yaws);
        // Only the camera of a map without a view of its own is the player's to keep.
        if !self.indoor && self.scenery.view.is_none() {
            asked.remember();
        }
    }

    /// Moves the camera towards what was asked of it, as far as the time since it was last moved allows: a
    /// tenth of the way in its turn, its tilt and its distance, each of the client's frames
    /// (`CameraState::Smooth`).
    pub(super) fn ease_camera(&mut self, time: f64) {
        let (asked, eased) = self.wanted;
        let share = views::share((time - eased) / CAMERA_FRAME_MS);
        self.camera.yaw = views::eased_turn(self.camera.yaw, asked.yaw, share);
        self.camera.pitch = views::eased(self.camera.pitch, asked.pitch, share);
        self.camera.distance = views::eased(self.camera.distance, asked.distance, share);
        self.wanted.1 = time;
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
