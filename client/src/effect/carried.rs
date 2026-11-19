//! An effect file that a class of the client's own plays and carries: moved along a way from one actor to
//! another, turned on the screen, put aside, begun late, begun anew when it is over.
//!
//! The client's `CEZ2STRParticle` (`Contents/Effects/Particles/Particle.cpp`) is one playing file. Each tick of
//! the effect that holds it, the effect gives it a place in the world and calls its `Process`, which counts its
//! delay (`m_delay`) down by 16 or moves it on by one frame; a file that loops (`m_loop`) begins anew when its
//! layers are through. `RenderScreen` then draws the layers across the screen about the place, the corners
//! turned about it by `m_angle` (and not at all by an angle not over nothing) and faded by `m_alpha`.
//!
//! What gives the place and the angle is the effect: `CCartCannon`, `CStormCannonEffect`, `CAcidifiedThrow`,
//! `CCrimsonArrowEffect`, `CSavageImpactEffect`, `CRandomHitEffect`, and `CEZ2STRScreenEffect` for the plain
//! files it turns towards someone. Their rules are the functions here; which effect is carried how is listed in
//! `crate::effect::played`, and `web/world/frame/effects.rs` draws them.
//!
//! Places are this client's: east, up, north, in the client's units (the client's own y points down).

/// A tick of the client's effects in milliseconds: what `CEZ2STRParticle::Process` takes off a delay each time.
pub const TICK_MS: i32 = 16;

/// The tick of its flight at which what `CAcidifiedThrow` throws has landed: its time runs from 1 down by 0.07
/// a tick, and is not over nothing after the fifteenth.
pub const BOW_LANDS: i32 = 14;

/// Where a carried file is, tick by tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Way {
    /// On the one the effect is on, wherever that one goes.
    Stays,
    /// On the other one the effect was told of, wherever that one goes (`CAcidifiedThrow`'s `impactPosition`).
    There,
    /// From where the one stood to where the other stood as it began, in ten steps (`CCartCannon`).
    Steps,
    /// From where the one stood towards where the other stood and on, fading once it is past (`CStormCannonEffect`).
    Straight,
    /// From the one to the other in a bow to the side (`CAcidifiedThrow`).
    Bowed,
}

/// How a carried file is turned on the screen (`m_angle`, degrees, clockwise).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Turn {
    None,
    /// By so much.
    By(f32),
    /// By the angle on the screen of the way from the one to the other as it began (`screen_angle`), and so
    /// much more.
    Towards(f32),
    /// By the way of its flight over the ground and the camera's yaw as it began (`steps_turn`).
    Flight,
}

/// How one file of an effect is carried.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Carried {
    pub way: Way,
    /// How far from the place its way gives it stands: east, up, north.
    pub offset: [f32; 3],
    /// How many pixels up the screen its middle is from that place: 80 for a file `CEZ2STRScreenEffect` plays
    /// (`Process(0, -80, 0)`), nothing for the classes that carry theirs (`Process(0, 0, 0)`).
    pub lift: f32,
    pub turn: Turn,
    /// The milliseconds it waits before its first frame (`m_delay`, or the effect's own `delay`).
    pub delay: i32,
    /// Whether it begins anew when it is over (`m_loop`).
    pub looped: bool,
    /// How many times a tick it is moved on (`CRandomHitEffect::OnProcess` calls `Process` twice).
    pub pace: i32,
}

impl Carried {
    /// A file on that way, as a class of the client's that carries its own plays it.
    pub const fn new(way: Way) -> Self {
        Carried { way, offset: [0.0; 3], lift: 0.0, turn: Turn::None, delay: 0, looped: false, pace: 1 }
    }

    pub const fn offset(self, offset: [f32; 3]) -> Self {
        Carried { offset, ..self }
    }

    pub const fn lifted(self, lift: f32) -> Self {
        Carried { lift, ..self }
    }

    pub const fn turned(self, turn: Turn) -> Self {
        Carried { turn, ..self }
    }

    pub const fn delayed(self, delay: i32) -> Self {
        Carried { delay, ..self }
    }

    pub const fn looped(self) -> Self {
        Carried { looped: true, ..self }
    }

    pub const fn paced(self, pace: i32) -> Self {
        Carried { pace, ..self }
    }

    /// How many frames it has been moved on before the one it shows at a tick of its effect (the first tick is
    /// 0); nothing while it waits. A delay takes one call of `Process` for every 16 milliseconds begun.
    pub fn lived(&self, tick: i32) -> Option<i32> {
        let waits = if self.delay > 0 { (self.delay + TICK_MS - 1) / TICK_MS } else { 0 };
        let moved = (tick + 1) * self.pace - waits;
        (moved > 0).then_some(moved - 1)
    }

    /// The frame of its file it shows, having lived so many, the file's last frame being `last`; nothing once
    /// it is over. (The client ends a file, or begins it anew, the call after its layers are all through their
    /// keys; here that is taken to be the frame after the file's last.)
    pub fn frame(&self, lived: i32, last: i32) -> Option<i32> {
        if self.looped { Some(lived % (last + 1).max(1)) } else { (lived <= last).then_some(lived) }
    }
}

fn length(way: [f32; 3]) -> f32 {
    (way[0] * way[0] + way[1] * way[1] + way[2] * way[2]).sqrt()
}

/// `CCartCannon::Open`: where its flight starts and how far a step of it goes, from where the one stood to
/// where the other stood, and how many times faster than ten steps it flies. Both ends are 10 over the ground;
/// a step is a tenth of the way, or for a way under 40 as much longer as makes ten of them 40; it starts 4
/// along the way. Nothing for no way at all (the client divides by its length).
fn shot(from: [f32; 3], to: [f32; 3]) -> Option<([f32; 3], [f32; 3], f32)> {
    let way = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
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
    let way = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
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
    let way = [to[0] - raised[0], to[1] - raised[1], to[2] - raised[2]];
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

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 3], b: [f32; 3]) -> bool {
        (0..3).all(|axis| (a[axis] - b[axis]).abs() < 1e-3)
    }

    #[test]
    fn a_delay_is_counted_down_by_sixteen_a_call() {
        // 200 takes 13 calls, 400 takes 25: the frame after is the first.
        let late = Carried::new(Way::Stays).delayed(200);
        assert_eq!((late.lived(12), late.lived(13), late.lived(14)), (None, Some(0), Some(1)));
        assert_eq!(Carried::new(Way::Stays).delayed(400).lived(25), Some(0));
        assert_eq!(Carried::new(Way::Stays).lived(0), Some(0));
        // Moved on twice a tick, it waits half as long and shows every other frame.
        let twice = Carried::new(Way::Stays).delayed(200).paced(2);
        assert_eq!((twice.lived(5), twice.lived(6), twice.lived(7)), (None, Some(0), Some(2)));
        assert_eq!(Carried::new(Way::Stays).paced(2).lived(0), Some(1));
    }

    #[test]
    fn a_file_is_over_after_its_last_frame_unless_it_loops() {
        let once = Carried::new(Way::Stays);
        assert_eq!((once.frame(10, 10), once.frame(11, 10)), (Some(10), None));
        let again = once.looped();
        assert_eq!((again.frame(10, 10), again.frame(11, 10), again.frame(25, 10)), (Some(10), Some(0), Some(3)));
    }

    #[test]
    fn a_cannon_ball_flies_in_ten_steps() {
        // 100 to the east: it starts 4 along the way and 10 up, and goes a tenth of the way a step.
        let (from, to) = ([0.0, 2.0, 0.0], [100.0, 2.0, 0.0]);
        assert!(close(steps(from, to, 0).unwrap(), [4.0, 12.0, 0.0]));
        assert!(close(steps(from, to, 10).unwrap(), [104.0, 12.0, 0.0]));
        assert_eq!(steps(from, to, 11), None);
        // A way of 20 is flown twice as fast: steps of 4, and none after the fifth.
        let near = [0.0, 2.0, 20.0];
        assert!(close(steps(from, near, 5).unwrap(), [0.0, 12.0, 24.0]));
        assert_eq!(steps(from, near, 6), None);
        assert_eq!(steps(from, from, 0), None);
    }

    /// The ball's picture points down the screen, and a turn is clockwise: with the camera looking north (a
    /// yaw of 0) one flying east is turned three quarters (to point right) and one flying north a half (up);
    /// looking east (90), the one flying east is turned a half.
    #[test]
    fn a_cannon_ball_is_turned_by_its_way_and_the_cameras_yaw() {
        let from = [0.0; 3];
        assert!((steps_turn(from, [50.0, 0.0, 0.0], 0.0) - 270.0).abs() < 1e-3);
        assert!((steps_turn(from, [0.0, 0.0, 50.0], 0.0) - 180.0).abs() < 1e-3);
        assert!((steps_turn(from, [50.0, 0.0, 0.0], 90.0) - 180.0).abs() < 1e-3);
        // South with the camera looking north: 270 - (0 + 270) = 0; west: 270 - 180.
        assert!(steps_turn(from, [0.0, 0.0, -50.0], 0.0).abs() < 1e-3);
        assert!((steps_turn(from, [-50.0, 0.0, 0.0], 0.0) - 90.0).abs() < 1e-3);
        // A yaw under nothing that brings the sum under nothing is brought round first.
        assert!((steps_turn(from, [50.0, 0.0, 0.0], -30.0) - 300.0).abs() < 1e-3);
        let [x, y] = turned([0.0, 1.0], steps_turn(from, [50.0, 0.0, 0.0], 0.0));
        assert!((x - 1.0).abs() < 1e-3 && y.abs() < 1e-3);
    }

    #[test]
    fn a_storm_cannon_flies_on_and_fades_once_past() {
        let (from, to) = ([0.0; 3], [17.5, 0.0, 0.0]);
        // 1.7 a tick, whole until it is farther than the other: the tenth tick is at 17, not past.
        let (at, alpha) = straight(from, to, 9).unwrap();
        assert!(close(at, [17.0, 0.0, 0.0]) && alpha == 1.0);
        let (at, alpha) = straight(from, to, 10).unwrap();
        assert!(close(at, [18.7, 0.0, 0.0]) && (alpha - 245.0 / 255.0).abs() < 1e-6);
        // 26 ticks past it nothing is left of 255.
        assert!(straight(from, to, 34).is_some_and(|(_, alpha)| (alpha - 5.0 / 255.0).abs() < 1e-6));
        assert_eq!(straight(from, to, 35), None);
        assert_eq!(straight(from, from, 0), None);
    }

    #[test]
    fn what_is_thrown_flies_a_bow_and_lands() {
        let (raised, to) = ([0.0, 10.0, 0.0], [10.0, 0.0, 0.0]);
        // Half way through its time (no tick is, the rule is the same): a quarter of each end and half the
        // third point, which stands as far from the end as the start does, the way turned by 240 degrees.
        let third = [10.0 + 10.0 * -0.5, -10.0, 10.0 * 0.8660254];
        let first = bowed(raised, to, true, 0).unwrap();
        let (near, between, far) = (0.07f32 * 0.07, 2.0 * 0.93 * 0.07, 0.93f32 * 0.93);
        assert!(close(first, std::array::from_fn(|axis| between * third[axis] + far * raised[axis] + near * to[axis])));
        // To the left the bow goes the other way round.
        assert!((bowed(raised, to, false, 0).unwrap()[2] + first[2]).abs() < 1e-3);
        // It is nearly there at the fourteenth tick and has landed at the fifteenth.
        let last = bowed(raised, to, true, BOW_LANDS - 1).unwrap();
        assert!(length([last[0] - to[0], last[1] - to[1], last[2] - to[2]]) < 1.0);
        assert_eq!(bowed(raised, to, true, BOW_LANDS), None);
    }

    #[test]
    fn the_angle_on_the_screen_goes_round_clockwise_from_the_right() {
        let from = [100.0, 100.0];
        assert!(screen_angle(from, [150.0, 100.0]).abs() < 1e-3);
        assert!((screen_angle(from, [100.0, 150.0]) - 90.0).abs() < 1e-3);
        assert!((screen_angle(from, [50.0, 100.0]) - 180.0).abs() < 1e-3);
        assert!((screen_angle(from, [100.0, 50.0]) - 270.0).abs() < 1e-3);
        assert_eq!(screen_angle(from, from), 0.0);
        // What points right, turned by it, points along the way.
        let [x, y] = turned([1.0, 0.0], screen_angle(from, [130.0, 60.0]));
        assert!((x - 0.6).abs() < 1e-3 && (y + 0.8).abs() < 1e-3, "{x} {y}");
        // No angle, and one under nothing, turn nothing.
        assert_eq!(turned([3.0, 4.0], 0.0), [3.0, 4.0]);
        assert_eq!(turned([3.0, 4.0], -90.0), [3.0, 4.0]);
    }
}
