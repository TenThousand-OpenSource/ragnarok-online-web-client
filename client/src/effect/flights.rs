//! The ways a carried effect file flies and is turned, each the rule of the class of the client's that carries
//! it: `CCartCannon` (ten steps), `CStormCannonEffect` (straight on, fading), `CAcidifiedThrow` (a bow), and
//! the angles on the screen (`ScreenAngle`, `CEZ2STRParticle::RenderScreen`). How a file is carried is in
//! `carried.rs`, through which these are named (`crate::effect::carried::steps`).
//!
//! Places are this client's: east, up, north, in the client's units (the client's own y points down).

use crate::vector::{length, subtract};

/// `CCartCannon::Open`: where its flight starts and how far a step of it goes, from where the one stood to
/// where the other stood, and how many times faster than ten steps it flies. Both ends are 10 over the ground;
/// a step is a tenth of the way, or for a way under 40 as much longer as makes ten of them 40; it starts 4
/// along the way. Nothing for no way at all (the client divides by its length).
fn shot(from: [f32; 3], to: [f32; 3]) -> Option<([f32; 3], [f32; 3], f32)> {
    let way = subtract(to, from);
    let length = length(way);
    if length == 0.0 {
        return None;
    }
    let speed = if length < 40.0 { 40.0 / length } else { 1.0 };
    let start = std::array::from_fn(|axis| from[axis] + way[axis] / length * 4.0 + if axis == 1 { 10.0 } else { 0.0 });
    Some((start, way.map(|part| part / 10.0 * speed), speed))
}

/// `CCartCannon::OnProcess` and `IsAlive`: where it is at a step of its flight; nothing once the steps are past
/// ten over its speed.
pub fn steps(from: [f32; 3], to: [f32; 3], step: i32) -> Option<[f32; 3]> {
    let (start, delta, speed) = shot(from, to)?;
    (step as f32 <= 10.0 / speed).then(|| std::array::from_fn(|axis| start[axis] + step as f32 * delta[axis]))
}

/// `CCartCannon::Open`: the angle it is turned by, the camera's yaw being `yaw` (the client's degrees; it takes
/// `CameraDesiredYaw`, the yaw the camera is turning to): 270 less the yaw and the angle of the way left to fly
/// over the ground (`atan2(north, east)`), each brought over nothing by a whole turn.
pub fn steps_turn(from: [f32; 3], to: [f32; 3], yaw: f32) -> f32 {
    let Some((start, ..)) = shot(from, to) else { return 0.0 };
    let mut angle = yaw + (to[2] - start[2]).atan2(to[0] - start[0]).to_degrees();
    if angle < 0.0 {
        angle += 360.0;
    }
    angle = 270.0 - angle;
    if angle < 0.0 { angle + 360.0 } else { angle }
}

/// `CStormCannonEffect::OnProcess`: where it is at a tick of its flight and how much of it shows (0 to 1). It
/// goes 1.7 a tick along the way from where the one stood to where the other stood, and each tick that finds
/// it farther than the other loses 10 of its 255; nothing once none is left, or for no way at all.
pub fn straight(from: [f32; 3], to: [f32; 3], tick: i32) -> Option<([f32; 3], f32)> {
    let way = subtract(to, from);
    let length = length(way);
    if length == 0.0 {
        return None;
    }
    let past = (0..=tick).filter(|tick| 1.7 * (tick + 1) as f32 > length).count() as i32;
    let alpha = (255 - 10 * past).max(0);
    let gone = 1.7 * (tick + 1) as f32;
    (alpha > 0).then(|| (std::array::from_fn(|axis| from[axis] + way[axis] / length * gone), alpha as f32 / 255.0))
}

/// `CAcidifiedThrow::OnProcess`: where what it throws is at a tick of its flight; nothing once it has landed.
/// `raised` is the thrower's place lifted 10 up the screen, `to` the place of the one thrown at, `rightwards`
/// whether that one is not to the left of the thrower on the screen. It flies a bow of three points (a
/// quadratic Bezier curve) whose time runs from 1 down by 0.07 a tick: from `raised` to `to`, by a third point
/// as far from `to` as `raised` is, the way from `raised` to `to` turned about the upright by 240 degrees
/// (rightwards) or 120.
pub fn bowed(raised: [f32; 3], to: [f32; 3], rightwards: bool, tick: i32) -> Option<[f32; 3]> {
    let time = 1.0 - 0.07 * (tick + 1) as f32;
    if time <= 0.0 {
        return None;
    }
    let way = subtract(to, raised);
    // `matrix::MakeYRotation`: east goes to (cos, -sin), north to (sin, cos).
    let (sine, cosine) = if rightwards { 4.1887894f32 } else { 2.0943947f32 }.sin_cos();
    let by = [to[0] + way[0] * cosine + way[2] * sine, to[1] + way[1], to[2] - way[0] * sine + way[2] * cosine];
    let (near, between, far) = ((1.0 - time) * (1.0 - time), 2.0 * time * (1.0 - time), time * time);
    Some(std::array::from_fn(|axis| between * by[axis] + far * raised[axis] + near * to[axis]))
}

/// `ScreenAngle` (`Core/EffectLists.cpp`, and written out in `CCrimsonArrowEffect::Open` and others): the angle
/// of the way from one point of the screen to another (x to the right, y down), in degrees from 0 up to 360:
/// the arc cosine of how far right it goes, taken from a whole turn when it goes up the screen. Nothing for no
/// way (the client's angle is then no number, and turns nothing).
pub fn screen_angle(from: [f32; 2], to: [f32; 2]) -> f32 {
    let (across, down) = (to[0] - from[0], to[1] - from[1]);
    let length = (across * across + down * down).sqrt();
    if length == 0.0 {
        return 0.0;
    }
    let angle = (across / length).clamp(-1.0, 1.0).acos() / 3.141592 * 180.0;
    if from[1] > to[1] { 360.0 - angle } else { angle }
}

/// `CEZ2STRParticle::RenderScreen`: a corner so far right and down the screen from the file's place, turned
/// about that place by its angle (`MakeZRotation`: clockwise, the screen's y pointing down); an angle not over
/// nothing turns nothing.
pub fn turned([x, y]: [f32; 2], angle: f32) -> [f32; 2] {
    if angle > 0.0 {
        let (sine, cosine) = angle.to_radians().sin_cos();
        [x * cosine - y * sine, x * sine + y * cosine]
    } else {
        [x, y]
    }
}
