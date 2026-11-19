//! What else flies or runs from the one who strikes to a place it is told, beside the mage's bolts
//! (`bolts.rs`), each with the figures of its function of `CRagEffect`:
//!
//! - `EF_YUFITEL` (93): the ball of Jupitel Thunder (`Yufitel`);
//! - `EF_WATERBALL2` (117): the ball of Water Ball (`WaterBall2`);
//! - `EF_GRIMTOOTH` (123): Grimtooth (`GrimTooth`);
//! - `EF_ENERGYDRAIN` (217): what is drawn out of the one struck flies to the striker (`BloodDrain`);
//! - `EF_BLOODDRAIN` (216), `EF_BLOODDRAIN2` (1052): the same in red, three sparks or one (`RedParticleSpray`);
//! - `EF_ENERGYDRAIN2` (378): Soul Drain, three pale blue sparks from the one drained to the one who drains
//!   (`EnergyDrain2`), who shows blue for a moment as they arrive (`SetColor` on `state.parent`);
//! - `EF_FIREBALL3` (806), `EF_FIREBALL6` (935): the fists of Boost Knuckle and of the elemental's Typhoon
//!   Missile (`RocketPunch`, `GreenRocketPunch`);
//! - `EF_NEW_TRIPLEATTACK` (1409), `EF_NEW_CROSSIMPACT_HIT` (1568), `EF_NEW_GATEOFHELL` (1575): streaks of light
//!   that run over the ground from the striker towards the one struck (`LineStreak`, `LineStreakTextured`);
//!   and so `EF_CHEMICAL3` (439: the flying kick, Dark Illusion), `EF_CHEMICAL4` (623: Piercing Shot) and
//!   `EF_CHEMICAL2DASH2` (835: Hundred Spear, which also shakes the screen and colours the striker)
//!   (`LineStreakLayered`), and `EF_CHEMICAL3S` (893: Knuckle Arrow; `LineStreakKind`).
//! - `EF_TRIPLEATTACK2` (388), `EF_TRIPLEATTACK3` (393): the streaks of Sharp Shooting and of Arrow Vulcan,
//!   from near the archer to the one shot at and past (`AlphaCenterTrail`, `AlphaCenterStream`; the kind 69).
//!
//! None of these takes its parent's place every tick: all stay where they began (the two of the archers are
//! not named in `stays`: their primitives stay in the world by themselves).

use super::launched::{PARTICLE1, of_parts, part};
use super::*;
use crate::prim::{Tei, angle_of, quarter};

pub(super) const FILE: File = File { known, thrown, stays, run, pictures: &[&PICTURES, &THUNDER_BALL], sprites: &[&SPRITES] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 8] = ["thunder_center.bmp", "stone.bmp", "line3y.tga", "line3.tga", LINE_PITCH, LINE_RED, LINE_HELL, "alpha_center.tga"];

/// The pictures the ball of Jupitel Thunder runs through, one a tick.
const THUNDER_BALL: [&str; 6] =
    ["thunder_ball_a.bmp", "thunder_ball_b.bmp", "thunder_ball_c.bmp", "thunder_ball_d.bmp", "thunder_ball_e.bmp", "thunder_ball_f.bmp"];

/// The pictures of the streaks of Cross Impact and of Gate of Hell. (With the option to show fewer effects on,
/// `TT_MIN_EFFECT_ON_OFF`, the client takes pictures of the same names from `mineffect`; what is here is what
/// it shows with the option off.)
const LINE_PITCH: &str = "new_crossimpact/new_crossimpact_line/line_pitch.tga";
const LINE_RED: &str = "new_crossimpact/new_crossimpact_line/line_red.tga";
const LINE_HELL: &str = "new_gateofhell/line_hell.tga";

/// The sprites the effects here show, among the client's sprites, beside those of `sprites.rs`: the ball of
/// water and the fist (`이팩트\남_로켓트펀치.spr`).
const SPRITES: [&str; 2] = [WATERBALL, FIST];
const WATERBALL: &str = "이팩트/waterball";
const FIST: &str = "이팩트/남_로켓트펀치";

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        93 => (250, "effect/hunter_shockwavetrap.wav"),
        117 => (100, "effect/wizard_waterball_chulung.wav"),
        123 => (250, "effect/ef_frostdiver.wav"),
        216 | 217 | 378 | 1052 => (200, ""),
        // `FrostDiver` starts no sound.
        // The sound is started by what launches the effect for the skill, with it (`NC_BOOSTKNUCKLE` and
        // `EL_TYPOON_MIS` in `Contents/Jobs/JobInheritance.cpp`), not by the effect's own function.
        806 | 935 => (100, "effect/맹호경파산.wav"),
        // (`EF_CHEMICAL4` starts its sound at its 29th tick, in `run`.)
        439 | 623 | 835 | 893 | 1409 | 1568 | 1575 => (300, ""),
        388 | 393 => (200, ""),
        _ => return None,
    })
}

/// Whether an effect is told a place it aims at. The striker launches it and it is told where the one struck
/// stands (`CSession::PlaySkillEffect`: `source->LaunchEffect` and `AM_SETPOS`), but for the drain, which is
/// on the one struck and is told where the striker stands (`AM_SETPOS` as the blow lands, in
/// `CGameActor::SendMsg`).
fn thrown(effect: u32) -> bool {
    matches!(effect, 93 | 117 | 123 | 216 | 217 | 378 | 388 | 393 | 439 | 623 | 806 | 835 | 893 | 935 | 1052 | 1409 | 1568 | 1575)
}

/// Whether an effect stays where it began: all of this file but the archers' two do, their functions do not take their parent's
/// place every tick (and `PrimChemical3`, the streak, does not follow its master).
fn stays(effect: u32) -> bool {
    matches!(effect, 93 | 117 | 123 | 216 | 217 | 378 | 439 | 623 | 806 | 835 | 893 | 935 | 1052 | 1409 | 1568 | 1575)
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let first = course.tick == 0;
    match effect {
        93 if first => prims.extend(jupitel(course.shared.aim)),
        117 if first => prims.push(water_ball(course.shared.aim)),
        123 => grim_tooth(course, prims),
        216 if first => prims.extend(drain(course.shared.aim, [250.0, 100.0, 100.0])),
        217 if first => prims.extend(drain(course.shared.aim, [100.0, 100.0, 250.0])),
        1052 if first => prims.extend(drain(course.shared.aim, [250.0, 100.0, 100.0]).into_iter().take(1)),
        378 => {
            // From its 55th tick to its 65th the one who drains is lit and blue.
            if (55..=65).contains(&course.tick) {
                course.touch.flagged(crate::body::flag::ADDS, 100, 100, 255);
            }
            if first {
                prims.extend(soul_drain(course.shared.aim));
            }
        }
        806 | 935 => fists(effect == 935, course, prims),
        388 | 393 => {
            course.starts(&[(30, if effect == 388 { "effect/샤프슈팅.wav" } else { "effect/애로우 발칸.wav" })]);
            arrow_streaks(effect == 393, course, prims);
        }
        439 if first => streaks_both_sides(0, course, prims),
        623 => {
            if course.tick == 28 {
                course.sound("effect/피어싱샷.wav");
            }
            if first {
                streaks_both_sides(20, course, prims);
            }
        }
        835 => hundred_spear_streaks(course, prims),
        893 if first => streaks_both_sides(30, course, prims),
        1409 if first => (0..8).for_each(|turn| line_streak(None, turn % 2 == 1, course, prims)),
        1568 if first => {
            for picture in [LINE_PITCH, LINE_RED, LINE_PITCH, LINE_RED] {
                line_streak(Some(picture), false, course, prims);
                line_streak(Some(picture), true, course, prims);
            }
        }
        1575 if first => (0..8).for_each(|turn| line_streak(Some(LINE_HELL), turn % 2 == 1, course, prims)),
        _ => {}
    }
}

/// `EF_CHEMICAL2DASH2` in `CRagEffect::OnProcess`: the yellow streaks of Hundred Spear at its first tick; at
/// its 45th the screen shakes; and from its 21st to its 101st the one who strikes has a pale copy behind
/// (`render.flags` with 8) and is a pale yellow.
fn hundred_spear_streaks(course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick == 0 {
        streaks_both_sides(10, course, prims);
    }
    if course.tick == 44 {
        course.quake = Some(0);
    }
    if (20..=100).contains(&course.tick) {
        course.touch.flagged(crate::body::flag::OUTLINE, 255, 255, 150);
    }
}

/// A primitive the client draws by adding its picture's colours to what is behind it, whatever its alpha is
/// (a `renderFlag` with 2: both blends are "one"). Its alpha is put at its most and left there: nothing the
/// client does to the alpha of such a primitive shows.
fn glowing(mut prim: Prim) -> Prim {
    (prim.adds, prim.alpha, prim.fade_out) = (true, prim.max_alpha, -1);
    prim
}

/// `CRagEffect::Yufitel`: two pictures turned to the camera, 5 above the caster, that fly level to where the
/// one struck stood, there in 20 ticks, and end within 3 of that place: a glow 3.5 to each side and over it the
/// ball, 4.5 to each side, a new picture of its six every tick. Both are drawn by adding their colours.
fn jupitel(aim: [f32; 3]) -> [Prim; 2] {
    let ball = |picture: &'static str, size: f32| {
        let mut ball = Prim::new(Shape::Texture3d, picture);
        (ball.duration, ball.faces_camera, ball.seeks) = (20, true, true);
        (ball.width, ball.height, ball.put.1) = (size, size, -5.0);
        ball.longitude = -faced(aim[0], aim[2]);
        ball.speed = -(f32::hypot(aim[0], aim[2]) / ball.duration as f32);
        glowing(ball)
    };
    let mut lightning = ball(THUNDER_BALL[0], 4.5);
    (lightning.pictures, lightning.anim_speed) = (&THUNDER_BALL, 1);
    [ball("thunder_center.bmp", 3.5), lightning]
}

/// `CRagEffect::WaterBall2`: a ball of water from 10 above the caster that comes to where the one struck stood
/// in 40 ticks, as a bolt of Soul Strike does (`launched::bowed`): it draws back first and then flies, in a bow
/// that stands 21 out upwards at its widest. It is drawn twelve times, a trail.
fn water_ball(aim: [f32; 3]) -> Prim {
    let mut ball = launched::bowed(WATERBALL, 40, 3.5, 0.0);
    ball.longitude = -faced(aim[0], aim[2]);
    ball.speed = f32::hypot(aim[0], aim[2]) / ball.duration as f32;
    (ball.size, ball.segments) = (0.75, 12);
    ball
}

/// `CRagEffect::BloodDrain`, which `EF_ENERGYDRAIN` runs: three blue sparks from 10 above the one struck to
/// where the striker stood, there in 60 ticks, each in a bow like a bolt of Soul Strike (`launched::bowed`): one
/// to each side and one upwards. Each is drawn seven times, a trail. `CRagEffect::RedParticleSpray` launches
/// the same in red, as many of the three as it is told, the one to the left first.
fn drain(aim: [f32; 3], colour: [f32; 3]) -> [Prim; 3] {
    [-90.0, 0.0, 90.0].map(|roll| {
        let mut spark = launched::bowed(PARTICLE1, 60, 2.0, roll);
        spark.longitude = -faced(aim[0], aim[2]);
        spark.speed = f32::hypot(aim[0], aim[2]) / spark.duration as f32 * 2.0;
        spark.colour = colour;
        (spark.size, spark.segments) = (1.7, 7);
        spark
    })
}

/// `CRagEffect::EnergyDrain2`: three sparks as `BloodDrain` launches them, but from 10 above the place the
/// effect is told (the one drained), turned the way from there to the one who drains, at half the speed; each
/// a pale blue drawn by lot and 1.5 to 2 its size.
fn soul_drain(aim: [f32; 3]) -> [Prim; 3] {
    [-90.0, 0.0, 90.0].map(|roll| {
        let mut spark = launched::bowed(PARTICLE1, 60, 2.0, roll);
        (spark.put.0, spark.deep) = (aim[0], aim[2]);
        spark.longitude = -faced(-aim[0], -aim[2]);
        spark.speed = f32::hypot(aim[0], aim[2]) / spark.duration as f32;
        spark.colour = [lot(81) + 160.0, lot(81) + 160.0, 255.0];
        (spark.size, spark.segments) = (lot(6) * 0.1 + 1.5, 7);
        spark
    })
}

/// `CRagEffect::GrimTooth`: spikes of stone that run from the striker to the one struck, as the ice of Frost
/// Diver does: the place goes 2 further every tick and a spike comes up there every third, until one stands
/// within 2.5 of the place of the one struck. Each is 0.6 to 1 wide and 10 long, tipped within 15 degrees of
/// upright, pushed up from 20 under the ground at 3 a tick, slowing, and stopped at its sixth tick; it fades
/// over the last 10 of its 40.
fn grim_tooth(course: &mut Course, prims: &mut Vec<Prim>) {
    let aim = course.shared.aim;
    if course.tick == 0 {
        // `Face` and a turn about the upright by the angle taken back: its forward is the way from the place.
        let faces = faced(aim[0], aim[2]);
        course.step = [sine(-faces) * 2.0, 0.0, cosine(-faces) * 2.0];
        course.reach = course.step.map(|far| -far);
    }
    if course.shared.landed {
        return;
    }
    if course.tick % 3 == 0 {
        if f32::hypot(aim[0] - course.reach[0], aim[2] - course.reach[2]) <= 2.5 {
            course.shared.landed = true;
        }
        let mut spike = Prim::new(Shape::QuadHorn3d, "stone.bmp");
        (spike.duration, spike.first_pattern, spike.settles_at) = (40, true, Some(6));
        (spike.put, spike.deep) = ((course.reach[0], 20.0 - course.reach[1]), course.reach[2]);
        (spike.size, spike.height) = ((lot(40) + 60.0) / 100.0, 10.0);
        (spike.longitude, spike.latitude) = (lot(360), lot(30) + 75.0);
        spike.speed = 3.0;
        spike.accel = -(spike.speed / spike.duration as f32) * 2.0;
        spike.fade_out = spike.duration - 10;
        prims.push(spike);
    }
    course.reach = std::array::from_fn(|axis| course.reach[axis] - course.step[axis]);
}

/// `CRagEffect::RocketPunch` and `CRagEffect::GreenRocketPunch` (`green`): at ticks 0, 2, 4 and 6 a fist
/// thrown as Fire Ball's balls are (`launched::thrown_level`), there in 20 ticks, but not turned on the screen:
/// it shows the action of its sprite for the quarter of the screen it flies into (0, 6, 4 or 2).
///
/// `RocketPunch`: the first fist in the colours of its sprite (`pattern & 0x200`), the three after it fainter
/// and fainter (60, 40 and 20 of 255). `GreenRocketPunch`: all in a green whose light is added; the first is
/// told the sprite's own colours too, which is what shows; the next two are at 120 and 40 of 255 and the last
/// at nothing.
fn fists(green: bool, course: &mut Course, prims: &mut Vec<Prim>) {
    if !matches!(course.tick, 0 | 2 | 4 | 6) {
        return;
    }
    let aim = course.shared.aim;
    let mut fist = launched::thrown_level(FIST, 20, aim);
    fist.points = None;
    fist.action = [0, 6, 4, 2][quarter(course.yaw, aim[0], aim[2])];
    fist.own_colours = course.tick == 0;
    if green {
        (fist.colour, fist.adds) = ([105.0, 255.0, 105.0], true);
        fist.alpha = ((5 - course.tick) * 40) as f32;
    } else if course.tick != 0 {
        fist.alpha = ((8 - course.tick) * 10) as f32;
    }
    prims.push(fist);
}

/// `CRagEffect::LineStreak` (no picture named: its own, `line3y.tga`) and `CRagEffect::LineStreakTextured`, for
/// a kind that ends in 0 (`right`: no) or in 1 (`right`: yes) and is under 20: the streaks of the look 0, and
/// with a picture named those of the look 10 (`streaks`).
fn line_streak(picture: Option<&'static str>, right: bool, course: &Course, prims: &mut Vec<Prim>) {
    streaks(picture.unwrap_or("line3y.tga"), if picture.is_some() { 10 } else { 0 }, right, course, prims);
}

/// Four streaks of the kind 56 along the way from the striker to the one struck, 15 to 25 to one side of it
/// or to the other (`right`), each with its tail somewhere along 200 of that way; what `LineStreak`,
/// `LineStreakTextured`, `LineStreakLayered` and `LineStreakKind` launch. `look` is the ten of the number
/// they give each streak (`flag1[0]`, to which a lot of 0 to 2 is added that makes it fainter):
///
/// - 0: 0.6 to 1.2 wide and 20 to 50 long, running 1.5 to 3.5 a tick, white with less and less of red and
///   green (225, 205 or 185 of 255);
/// - 10: 1 to 2 wide and 40 to 70 long, starting at 1 to 2 a tick, white with less and less of blue;
/// - 20: 0.8 to 1.4 wide and 30 to 60 long, starting at 1.5 to 3.5 a tick, white with less and less of red
///   and blue;
/// - 30: as 0 in size and speed, blue (105, 65 or 25 of red and green), its light added (the client draws
///   these over everything; here they are drawn as the other effects are).
///
/// All but those of the look 0 are of the quick sort; all live as long as the effect, and all but those of
/// the look 30 are mixed in by their alpha.
///
/// (The client makes the four one primitive; each is its own here, as each has its own colour. It draws its
/// lots in the order they are drawn here.)
fn streaks(picture: &'static str, look: i32, right: bool, course: &Course, prims: &mut Vec<Prim>) {
    let aim = (course.shared.aim[0], course.shared.aim[2]);
    for _ in 0..4 {
        let side = if right { lot(11) + 15.0 } else { -15.0 - lot(11) };
        let tail = lot(200);
        let half_width = match look {
            10 => lot(6) * 0.1 + 0.5,
            20 => lot(4) * 0.1 + 0.4,
            _ => lot(4) * 0.1 + 0.3,
        };
        let length = lot(31) + [20.0, 40.0, 30.0, 20.0][(look / 10).clamp(0, 3) as usize];
        let fainter = 225.0 - lot(3) * 20.0;
        let speed = if look == 10 { lot(11) * 0.1 + 1.0 } else { lot(21) * 0.1 + 1.5 };
        let mut streak = Prim::new(Shape::Streak { quick: look >= 10 }, picture);
        streak.duration = course.lasts;
        streak.colour = match look {
            0 => [fainter, fainter, 255.0],
            10 => [255.0, 255.0, fainter],
            20 => [fainter, 255.0, fainter],
            _ => [fainter * 2.0 - 345.0, fainter * 2.0 - 345.0, 255.0],
        };
        streak.adds = look >= 30;
        streak.slots = vec![Slot::streak(aim, side, tail, length, speed, half_width)];
        prims.push(streak);
    }
}

/// `CRagEffect::LineStreakLayered` (`layered`) or `LineStreakKind`, called four times for each side as the
/// arms of `CRagEffect::OnProcess` do: 32 streaks of `line3.tga`, or for the looks 10 and 20 of `line3y.tga`.
fn streaks_both_sides(look: i32, course: &Course, prims: &mut Vec<Prim>) {
    let picture = if look == 10 || look == 20 { "line3y.tga" } else { "line3.tga" };
    for call in 0..8 {
        streaks(picture, look, call % 2 == 1, course, prims);
    }
}

/// `CRagEffect::AlphaCenterTrail` (Sharp Shooting) and `AlphaCenterStream` (`stream`, Arrow Vulcan): at each
/// of the first four ticks, for the stream nine, four streaks from near the archer towards the place of the
/// one shot at and a little past it. Each begins up to 4 to each side of the archer and up to 15 behind, waits
/// (45 to 64 ticks; the stream's 30 to 139), and flies 8 to 12 over the ground (the stream's 4 to 10, thicker
/// and pink).
fn arrow_streaks(stream: bool, course: &Course, prims: &mut Vec<Prim>) {
    if course.tick >= if stream { 9 } else { 4 } {
        return;
    }
    let (east, north) = (course.shared.aim[0] - course.walked[0], course.shared.aim[2] - course.walked[2]);
    let (far, way) = (east.hypot(north), angle_of(east, north));
    let mut streaks = of_parts(Tei::PentaRing, "alpha_center.tga", false, course);
    streaks.parts.extend((0..4).map(|_| {
        let mut streak = part();
        streak.process = if stream { -30.0 - lot(110) } else { -45.0 - lot(20) } as i32;
        streak.turned = way;
        let (aside, ahead, back) = (lot(100) * 0.08 - 4.0, lot(100) * 0.08 - 4.0, lot(100) * 0.15);
        streak.heights[1] = back;
        let near = [aside - cosine(way as f32) * back, 0.0, ahead - sine(way as f32) * back];
        (streak.at[0], streak.at[1]) = (near, near);
        if stream {
            streak.tall = lot(11) * 0.008 + 0.02;
            streak.heights[0] = lot(100) * 0.06 + 4.0;
            streak.out = lot(100) * 0.3 + far;
            streak.heights[4] = 3.0;
        } else {
            streak.tall = 0.05;
            streak.heights[0] = lot(100) * 0.04 + 8.0;
            streak.out = lot(100) * 0.6 + far;
            streak.heights[4] = 2.0;
        }
        streak
    }));
    prims.push(streaks);
}
