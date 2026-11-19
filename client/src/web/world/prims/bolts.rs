//! The mage's bolts and ice, each with the figures of its function of `CRagEffect`:
//!
//! - `EF_SOULSTRIKE` (15): Soul Strike (`SoulStrike`); `EF_SOULSTRIKE2` (451): its dark twin (`SoulStrike`
//!   with its flag);
//! - `EF_FIREBALL` (24): Fire Ball (`FireBall`);
//! - `EF_ICEARROW` (26), `EF_FIREARROW` (31): Cold Bolt and Fire Bolt (`IceArrow`, `FireArrow`);
//! - `EF_FROSTDIVER` (27): the ice that runs from the caster of Frost Diver to the one struck (`FrostDiver`);
//! - `EF_FROSTDIVER3` (801): the same spikes in a blue light (`FrostDiver(3)`);
//! - `EF_FROSTDIVER2` (28): the ice that stands up around the one frozen (`FrostDiver2`);
//! - `EF_SPEARBMR` (80): the spear of Spear Boomerang, thrown as the ball of fire is (`SpearBmr`);
//! - `EF_FIREBALL2` (751): the balls a Warlock releases, blue, green and red (`ShadedFireballVolley`).
//!
//! The client has an option to show fewer effects (`TT_MIN_EFFECT_ON_OFF`), with which `FrostDiver` shows
//! nothing; what is here is what it shows with the option off.

use super::launched::{bowed, thrown_level, FIREBALL, PARTICLE1, PARTICLE4, SPEAR};
use super::*;

pub(super) const FILE: File = File { known, thrown, stays, run, pictures: &[&PICTURES, &FIRE_BOLT], sprites: &[&SPRITES] };

/// The sprite of Soul Strike's dark twin, among the client's sprites, beside those several files show
/// (`launched.rs`).
const SPRITES: [&str; 1] = [PARTICLE5];
const PARTICLE5: &str = "이팩트/particle5";

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 4] = ["icearrow.tga", "ring_yellow.tga", "ring_blue.tga", "ice.tga"];

/// The pictures a bolt of fire runs through as it falls.
const FIRE_BOLT: [&str; 6] = ["불화살1.tga", "불화살2.tga", "불화살3.tga", "불화살4.tga", "불화살5.tga", "불화살6.tga"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`); nothing for any other effect.
/// None starts with a sound: the bolts, the ice and the spear start their sounds themselves, at the tick the
/// client does.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        15 | 451 => (100, ""),
        24 | 28 => (100, ""),
        26 | 31 => (160, ""),
        27 | 801 => (150, ""),
        80 => (250, ""),
        751 => (100, ""),
        _ => return None,
    })
}

/// Whether an effect is one the client has the striker launch and then tells a place, that of the one struck
/// (`CSession::PlaySkillEffect`: `source->LaunchEffect` and `AM_SETPOS` with the target's place; and
/// `CGameActor::ProcessSkillMessage` for the effect of its table `skillEffectIds`, as Soul Strike's). The table
/// of what a skill's blow shows (`effect_tables::skill_hits`) keeps only whose place it is, not who launches.
/// (`EF_FROSTDIVER3` is what Jack Frost is cast with, on the caster, in `effect_tables::skill_effects`;
/// whether the client tells it a place was not looked for, and it is not named here.)
fn thrown(effect: u32) -> bool {
    matches!(effect, 15 | 24 | 27 | 80 | 451 | 751)
}

/// Whether an effect stays where it began: one whose function does not take its parent's place every tick.
fn stays(effect: u32) -> bool {
    matches!(effect, 15 | 24 | 27 | 28 | 80 | 451 | 751 | 801)
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    match effect {
        15 => soul_strike(course, prims),
        // `SoulStrike` with its flag: the same bolts, of `particle5.spr`.
        451 => {
            let before = prims.len();
            soul_strike(course, prims);
            prims[before..].iter_mut().for_each(|bolt| bolt.picture = PARTICLE5);
        }
        24 => fire_ball(course, prims),
        26 => bolt(false, course, prims),
        27 => frost_diver(course, prims),
        801 => frost_diver_blue(course, prims),
        28 if course.tick == 0 => frozen(course, prims),
        31 => bolt(true, course, prims),
        80 => spear(course, prims),
        751 => shaded_volley(course, prims),
        _ => {}
    }
}

/// `CRagEffect::SoulStrike`: a bolt for each hit of the blow, up to seven, the first at the seventh tick and one
/// every eleven ticks after, each with the sound. A bolt starts 10 above the caster and comes to where the one
/// struck stood in 40 ticks: it draws back first and then flies, in a bow that stands 21 out at its widest.
/// Which way each bows is its turn about the line of flight: one bolt alone bows upwards (turned by 1 degree,
/// as the client's figures have it); of several the first bows to one side and each next is turned on by 180,
/// 90, 60, 45, 30 or 25 degrees for two to seven bolts. Each is drawn twelve times, a trail.
fn soul_strike(course: &mut Course, prims: &mut Vec<Prim>) {
    let (base, step) = match course.count {
        1 => (0, 1),
        2 => (-90, 180),
        3 => (-90, 90),
        4 => (-90, 60),
        5 => (-90, 45),
        6 => (-90, 30),
        7 => (-90, 25),
        _ => return,
    };
    let frame = course.tick + 5;
    if frame % 11 != 0 || frame / 11 > course.count {
        return;
    }
    course.sound("effect/ef_soulstrike.wav");
    let aim = course.shared.aim;
    let mut bolt = bowed(PARTICLE1, 40, 3.5, (base + step * (frame / 11)) as f32);
    bolt.longitude = -faced(aim[0], aim[2]);
    bolt.speed = f32::hypot(aim[0], aim[2]) / bolt.duration as f32;
    (bolt.size, bolt.segments) = (2.75, 12);
    prims.push(bolt);
}

/// How faint the second, third and fourth of four thrown one after another are (`FireBall`, `SpearBmr`).
fn fainter(nth: i32) -> f32 {
    match nth {
        1 => 180.0,
        2 => 130.0,
        _ => 80.0,
    }
}

/// `CRagEffect::FireBall`: at ticks 20, 24, 28 and 32 a ball of fire from 15 above the caster, which flies level
/// to where the one struck stood, there in 20 ticks, turned on the screen the way it flies; with each the
/// sound. The three after the first are yellower and each fainter than the one before.
fn fire_ball(course: &mut Course, prims: &mut Vec<Prim>) {
    if !matches!(course.tick, 20 | 24 | 28 | 32) {
        return;
    }
    course.sound("effect/ef_fireball.wav");
    let mut ball = thrown_level(FIREBALL, 20, course.shared.aim);
    ball.size = 1.3;
    if course.tick != 20 {
        ball.colour = [246.0, 199.0, 76.0];
        ball.alpha = fainter((course.tick - 20) / 4);
    }
    prims.push(ball);
}

/// `CRagEffect::ShadedFireballVolley`: three volleys of four balls of fire, one ball a tick, from 15 above the
/// caster to where the one struck stood, there in 20 ticks: at the first four ticks pale blue ones straight at
/// that one, from the seventh pale green ones aimed 5 degrees to one side, and from the thirteenth pale red
/// ones 5 degrees to the other. Each ball of a volley is fainter than the one before (200, 160, 120, 80). The
/// sound of the ball of fire comes once, at the sixth tick.
///
/// (A ball aimed to a side flies as far as the others; it ends within 3 of the place aimed at, as every such
/// sprite does, which it never comes to, so it lives its 20 ticks.)
fn shaded_volley(course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick == 5 {
        course.sound("effect/ef_fireball.wav");
    }
    let (turn, colour, fade) = match course.tick {
        0..=3 => (0.0, [155.0, 155.0, 255.0], 5),
        6..=9 => (-5.0, [155.0, 255.0, 155.0], 11),
        12..=15 => (5.0, [255.0, 155.0, 155.0], 17),
        _ => return,
    };
    let aim = course.shared.aim;
    let mut ball = thrown_level(FIREBALL, 20, aim);
    let angle = (faced(aim[0], aim[2]) + turn).rem_euclid(360.0);
    (ball.longitude, ball.points) = (-angle, Some((180.0 - angle).rem_euclid(360.0)));
    (ball.size, ball.colour, ball.alpha) = (1.0, colour, ((fade - course.tick) * 40) as f32);
    prims.push(ball);
}

/// `CRagEffect::SpearBmr`: at ticks 0, 4, 8 and 12 a spear from 15 above the one who throws it, which flies
/// level to where the one struck stood, there in 15 ticks, turned on the screen the way it flies; with each
/// the sound of the ball of fire, which is the one the client plays here. The three after the first are each
/// fainter than the one before.
///
/// (The effect also sets an action on itself by the way the thrower faces, `SetAction(Get8Dir2(..))`; the
/// primitive that shows the spear draws the first action of `창.spr`, as every primitive of its kind does.)
fn spear(course: &mut Course, prims: &mut Vec<Prim>) {
    if !matches!(course.tick, 0 | 4 | 8 | 12) {
        return;
    }
    course.sound("effect/ef_fireball.wav");
    let mut spear = thrown_level(SPEAR, 15, course.shared.aim);
    if course.tick != 0 {
        spear.alpha = fainter(course.tick / 4);
    }
    prims.push(spear);
}

/// `CRagEffect::IceArrow` and `CRagEffect::FireArrow` (`fire`): from the twelfth tick a bolt every ten ticks,
/// as many as the blow has hits, each from 60 above and some 25 and 15 to the side of the one struck, flying
/// at 2.75 a tick to a place within 2 of that one; with the first the sound, one of three. A bolt tells the
/// effect where it is and, on reaching the ground, that it has: then a ring widens on the ground under the one
/// struck. Of fire, every fourth tick a spark besides, from where the last bolt is, thrown about the way that
/// bolt was turned.
fn bolt(fire: bool, course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick == 12 {
        course.sound(&format!("effect/{}{}.wav", if fire { "ef_firearrow" } else { "ef_icearrow" }, lot(3) + 1.0));
        course.count *= 10;
    }
    let since = course.tick - 12;
    if since >= 0 && since % 10 == 0 && since < course.count {
        let mut bolt = Prim::new(Shape::CrossTexture3d, if fire { FIRE_BOLT[0] } else { "icearrow.tga" });
        bolt.duration = 70;
        (bolt.leads, bolt.lands) = (true, true);
        (bolt.deep, bolt.put) = (lot(10) + 15.0, (lot(10) + 25.0, -60.0));
        let (aim_north, aim_east) = (lot(4) - 2.0, lot(4) - 2.0);
        bolt.longitude = -heading(aim_east - bolt.put.0, aim_north - bolt.deep);
        // The client's `Heading(-|y|, run, 0, 0)`: how steeply it comes down.
        bolt.latitude = heading(bolt.put.1.abs(), -f32::hypot(bolt.put.0, bolt.deep));
        (course.longitude, course.latitude) = (bolt.longitude, bolt.latitude);
        bolt.speed = -2.75;
        (bolt.width, bolt.height) = if fire { (14.0, 3.5) } else { (11.5, 3.8) };
        bolt.fade_out = bolt.duration - 20;
        if fire {
            (bolt.pictures, bolt.anim_speed) = (&FIRE_BOLT, 1);
        }
        prims.push(bolt);
    }
    if fire && course.tick > 0 && course.tick % 4 == 0 {
        prims.push(fire_spark(course));
    }
    if course.shared.landed {
        course.shared.landed = false;
        let mut ring = Prim::new(Shape::Circle3d, if fire { "ring_yellow.tga" } else { "ring_blue.tga" });
        ring.duration = 30;
        ring.latitude = 90.0;
        ring.radius_speed = 1.2;
        ring.radius_accel = -(ring.radius_speed / (ring.duration + 40) as f32) * 2.0;
        ring.inner = 10.0;
        ring.alpha = 0.0;
        ring.alpha_speed = ring.max_alpha / 10.0;
        ring.fade_out = ring.duration - 10;
        prims.push(ring);
    }
}

/// The spark of `CRagEffect::FireArrow`: from where the effect's last bolt is, turned within 20 degrees round
/// and 10 up or down of that bolt's turn, it flies on as it slows, shrinking to nothing and fading from its
/// first tick.
fn fire_spark(course: &Course) -> Prim {
    let mut spark = Prim::new(Shape::Particle3d, PARTICLE4);
    spark.duration = lot(24) as i32 + 6;
    spark.longitude = lot(40) + (course.longitude - 20.0);
    spark.latitude = lot(20) + (course.latitude - 10.0);
    let at = course.shared.aim;
    (spark.put, spark.deep) = ((at[0], -at[1]), at[2]);
    spark.speed = (lot(90) + 60.0) / 100.0;
    spark.accel = -(spark.speed / spark.duration as f32) / 1.5;
    spark.size = (lot(30) + 20.0) / 100.0;
    spark.size_speed = -(spark.size / spark.duration as f32);
    spark.fade_out = 0;
    spark
}

/// A spike of ice as `FrostDiver` and `FrostDiver2` launch it: from 30 under the ground, somewhere `away`
/// round a place, turned any way round, pushed up at 3 a tick, slowing, and stopped at its twentieth tick; it
/// fades over the last 10 of its 40.
fn ice(away: f32, at: [f32; 3]) -> Prim {
    let mut ice = Prim::new(Shape::QuadHorn3d, "ice.tga");
    ice.duration = 40;
    ice.first_pattern = true;
    let round = lot(360);
    (ice.put, ice.deep) = ((sine(round) * away + at[0], 30.0 - at[1]), -away * cosine(round) + at[2]);
    ice.speed = 3.0;
    ice.accel = -(ice.speed / ice.duration as f32) * 2.0;
    ice.settles_at = Some(20);
    ice.fade_out = ice.duration - 10;
    ice
}

/// `CRagEffect::FrostDiver`: a spike of ice a tick, the first 5 from the caster towards the place of the one
/// struck and each 2 further, until one stands within 2.5 of that place. Each is 0.6 to 1 wide and 15 to 18
/// long, tipped within 15 degrees of upright.
fn frost_diver(course: &mut Course, prims: &mut Vec<Prim>) {
    let aim = course.shared.aim;
    if course.tick == 0 {
        // `Face` and a turn about the upright by the angle taken back: its forward is the way from the place.
        let faces = faced(aim[0], aim[2]);
        let back = |far: f32| [sine(-faces) * far, 0.0, cosine(-faces) * far];
        course.reach = back(-5.0);
        course.step = back(2.0);
    }
    if course.shared.landed {
        return;
    }
    if f32::hypot(aim[0] - course.reach[0], aim[2] - course.reach[2]) <= 2.5 {
        course.shared.landed = true;
    }
    let mut spike = ice((lot(10) + 5.0) / 10.0, course.reach);
    spike.size = (lot(40) + 60.0) / 100.0;
    spike.height = (lot(30) + 150.0) / 10.0;
    (spike.longitude, spike.latitude) = (lot(360), lot(30) + 75.0);
    prims.push(spike);
    course.reach = std::array::from_fn(|axis| course.reach[axis] - course.step[axis]);
}

/// `CRagEffect::FrostDiver2`: eight spikes of ice at once around the one frozen, within half a unit of its
/// place, each 1 to 3.5 wide and 20 to 30 long, tipped within 10 degrees of upright, with the sound.
fn frozen(course: &mut Course, prims: &mut Vec<Prim>) {
    course.sound("effect/ef_frostdiver2.wav");
    for _ in 0..8 {
        let mut spike = ice((lot(4) + 1.0) / 10.0, [0.0; 3]);
        spike.size = (lot(250) + 100.0) / 100.0;
        spike.height = (lot(100) + 200.0) / 10.0;
        (spike.longitude, spike.latitude) = (lot(360), lot(20) + 80.0);
        spike.alpha = 200.0;
        prims.push(spike);
    }
}

/// `CRagEffect::FrostDiver` with its type of 3: the spikes of Frost Diver (`frost_diver`), each told to
/// be drawn in a light of its own (`slots[0].flag1[0]` of 3). `RenderPyramid` draws such a spike with its
/// light added, in a blue whose red and green are each 40 to 50 of 255, and whole whatever the spike's alpha
/// is, so it does not fade. (The client draws that red and green by lot anew at every drawing; here a spike
/// keeps what it drew at first.)
fn frost_diver_blue(course: &mut Course, prims: &mut Vec<Prim>) {
    let before = prims.len();
    frost_diver(course, prims);
    for spike in &mut prims[before..] {
        (spike.colour, spike.adds, spike.fade_out) = ([40.0 + lot(11), 40.0 + lot(11), 255.0], true, -1);
    }
}
