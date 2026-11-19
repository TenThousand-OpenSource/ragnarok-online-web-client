//! Small lights round the one an effect is on: streaks that rise and fall, sparks that loop out and back, and
//! motes. Each with the figures of its function of `CRagEffect`:
//!
//! - `EF_INCAGILITY` (37), `EF_DECAGILITY` (38): Increase and Decrease Agility (`IncAgility`, `DecAgility`);
//! - `EF_ENERGYDRAIN3` (383): the green sparks of Health Conversion (`SonicSlashFan`, kind 0);
//! - `EF_PEONG_BLACK` (1207), `EF_ENERGYDRAIN_BLACK` (1208): the same sparks in grey (`SonicSlashFan`, kinds 1
//!   and 2);
//! - `EF_HATED` (543) and `EF_HATED2` (572): the blue motes of the Comforts and of Hatred, and the red ones of
//!   the Warmths (`ParticleTime`); `EF_HATED` lays a blue light over the one it is on besides, that comes over
//!   its first 25 ticks and goes from its 151st to its 175th (`render.flags` of 0x4000, `tintPhase`);
//! - `EF_AGIUP2` (777), `EF_AGIUP3` (779): the streaks of Increase Agility without its sign, paler and wider
//!   (`IncAgilityCenter`), the one it is on outlined and flashing a pale blue for its first 60 ticks;
//!   `EF_AGIUP` (456): the same function with its kind 0, narrower and slower streaks, and nothing done to
//!   the body;
//! - `EF_DANCE1` (847), `EF_DANCE2` (848), `EF_ENERVATION7` (864): streaks of light rising round the one it is
//!   on, and falling (`RisingSparkles`, `TintedRisingSparkles`);
//! - `EF_SPRINKLESAND` (310): the grains of Sprinkle Sand, small yellow motes that rise drifting from round
//!   the feet of the one it is thrown at (`ThunderSparks`);
//! - `EF_CHOOKGI_N` (833): the two lights of Duple Light that go round the one in that state
//!   (`ThunderFreeze`; the kind 145), for as long as the state holds (`crate::state_effects`).
//!
//! Beside its primitives the client's arm of several of these colours the one the effect is on for a time
//! (`SetColor` on `state.parent`, `crate::body`): `EF_ENERGYDRAIN3` outlines it and turns it blue from its
//! 50th tick to its 80th; `EF_DANCE1`, `EF_DANCE2` and `EF_ENERVATION7` flash it pink or violet from their
//! fourth tick to their 90th.

use super::launched::PARTICLE1;
use super::*;

pub(super) const FILE: File = File { known, thrown: none, stays, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 7] = ["ac_center2.tga", "agi_up.bmp", "slow.bmp", "pok1.tga", "pok3.tga", "thunder_center.bmp", "freeze_a.bmp"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        37 | 456 => (100, "effect/ef_incagility.wav"),
        38 => (100, "effect/ef_decagility.wav"),
        310 => (300, ""),
        383 => (200, ""),
        543 => (500, ""),
        // `EF_HATED2` starts its sound at its eleventh tick.
        572 => (300, ""),
        777 => (100, "effect/ef_incagility.wav"),
        779 => (100, "effect/소울 체인지.wav"),
        833 => (999_999, ""),
        847 | 848 | 864 => (100, ""),
        1207 | 1208 => (200, ""),
        _ => return None,
    })
}

/// Whether an effect stays where it began: `SonicSlashFan` with the kind 0 does not take its parent's place.
fn stays(effect: u32) -> bool {
    effect == 383
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let (tick, lasts) = (course.tick, course.lasts);
    match effect {
        383 if (50..=80).contains(&tick) => course.touch.flagged(crate::body::flag::OUTLINE, (125 - tick) * 2, (125 - tick) * 2, 250),
        847 if (4..=90).contains(&tick) => course.touch.flashes(tick, [255, 89, 182]),
        848 | 864 if (4..=90).contains(&tick) => course.touch.flashes(tick, [182, 89, 255]),
        _ => {}
    }
    match effect {
        37 => agility(true, tick, lasts, prims),
        38 => agility(false, tick, lasts, prims),
        310 if tick == 0 => prims.extend((0..20).map(|_| sand(course))),
        833 if tick == 0 => prims.extend(duple_lights(course)),
        383 => sonic_slash_fan(0, tick, prims),
        572 => {
            if tick == 10 {
                course.sound("effect/t_따듯한마법.wav");
            }
            warm_motes([250.0, 100.0, 100.0], course, prims);
        }
        543 => {
            if tick < 200 {
                course.touch.flags |= crate::body::flag::BLUE;
                course.touch.tint_phase = match tick {
                    ..=25 => Some(tick as u8),
                    151..=175 => Some((175 - tick) as u8),
                    _ => None,
                };
            }
            warm_motes([150.0, 150.0, 250.0], course, prims);
        }
        777 | 779 => {
            if tick < 61 {
                course.touch.flags |= crate::body::flag::OUTLINE;
                course.touch.flashes(tick, [155, 155, 255]);
            }
            if tick % 2 == 0 {
                prims.push(centre_streak(true));
            }
        }
        456 if tick % 2 == 0 => prims.push(centre_streak(false)),
        847 if tick % 2 == 0 => prims.push(rising_streak(false)),
        848 if tick % 2 == 0 => prims.push(rising_streak(true)),
        864 if tick % 2 == 0 => prims.extend([rising_streak(true), falling_streak()]),
        1207 => sonic_slash_fan(1, tick, prims),
        1208 => sonic_slash_fan(2, tick, prims),
        _ => {}
    }
}

/// A thin streak of light standing somewhere `nearest` to `nearest` and 6 round the one the effect is on, tipped
/// up (a latitude of 90) or down (-90), as `IncAgility`, `DecAgility`, `RisingSparkles` and
/// `TintedRisingSparkles` launch one every other tick: it comes up over 20 ticks and fades over the last 20 of
/// its 50.
fn light_streak(nearest: f32, latitude: f32) -> Prim {
    let mut streak = Prim::new(Shape::CrossTexture3d, "ac_center2.tga");
    streak.duration = 50;
    let (round, away) = (lot(360), lot(7) + nearest);
    (streak.put.0, streak.deep) = (sine(round) * away, cosine(round) * away);
    streak.latitude = latitude;
    streak.width = (lot(60) + 30.0) / 10.0;
    streak.height = 0.18;
    streak.alpha = 0.0;
    streak.max_alpha = 200.0;
    streak.alpha_speed = streak.max_alpha / 20.0;
    streak.fade_out = streak.duration - 20;
    streak
}

/// `CRagEffect::IncAgility` (`quickens`) and `CRagEffect::DecAgility`: a sign on the screen, and every other
/// tick a thin streak of light standing somewhere 2 to 8 around the one it is on.
///
/// Quickened: the sign rises from that one's place on the screen as it slows, and the streaks rise from the
/// ground, each at a speed of its own. Slowed: the sign sinks from 80 above as it slows to a stop, and the
/// streaks fall from 20 above the ground, faster and faster.
///
/// The client moves the sign and the streaks of Increase Agility (not those of Decrease Agility) on every pass
/// of its loop, not only on its ticks (`CEffectPrim::OnProcess`, a first slot whose `life` is 99), so how fast
/// they go there depends on the machine; here they move a tick at a time like everything else.
fn agility(quickens: bool, tick: i32, lasts: i32, prims: &mut Vec<Prim>) {
    if tick == 0 {
        let mut sign = Prim::new(Shape::Texture2d, if quickens { "agi_up.bmp" } else { "slow.bmp" });
        sign.duration = lasts;
        if quickens {
            sign.speed = 1.5;
            sign.accel = -(sign.speed / sign.duration as f32) * 1.2;
            (sign.width, sign.height) = (40.0, 20.0);
        } else {
            sign.put.1 = -80.0;
            sign.longitude = 180.0;
            sign.speed = 1.0;
            sign.accel = -(sign.speed / sign.duration as f32);
            (sign.width, sign.height) = (40.0, 10.0);
        }
        sign.alpha = 0.0;
        sign.max_alpha = 200.0;
        sign.alpha_speed = sign.max_alpha / 15.0;
        sign.fade_out = sign.duration - 15;
        prims.push(sign);
    }
    if tick % 2 != 0 {
        return;
    }
    let mut streak = light_streak(2.0, if quickens { 90.0 } else { -90.0 });
    if quickens {
        streak.speed = (lot(50) + 20.0) / 100.0;
    } else {
        streak.put.1 = -20.0;
        streak.accel = 0.015;
    }
    prims.push(streak);
}

/// The streak of `CRagEffect::IncAgilityCenter`, somewhere 2 to 8 round the one it is on. With a kind of 2 or
/// 3 (`wide`) it rises from the ground at 0.4 to 0.89 a tick, 4 to 13.9 wide and 0.1 to 0.35 thick; with a
/// kind under 2 at 0.2 to 0.69 a tick, 2 to 11.9 wide and 0.1 to 0.4 thick. It comes up to an alpha of 60
/// only.
fn centre_streak(wide: bool) -> Prim {
    let mut streak = light_streak(2.0, 90.0);
    let least = if wide { 40.0 } else { 20.0 };
    streak.speed = (lot(50) + least) / 100.0;
    (streak.width, streak.height) = ((lot(100) + least) / 10.0, if wide { lot(6) * 0.05 + 0.1 } else { lot(4) * 0.1 + 0.1 });
    streak.max_alpha = 60.0;
    streak.alpha_speed = streak.max_alpha / 20.0;
    streak
}

/// The streak of `CRagEffect::RisingSparkles`, somewhere 3 to 9 round the one it is on: from 5 above the ground
/// it rises faster and faster. With the colour mode 2 (`blue`) it is blue to white, its red and green one
/// number drawn by lot; else red to white, its green and blue one number.
fn rising_streak(blue: bool) -> Prim {
    let mut streak = light_streak(3.0, -90.0);
    streak.put.1 = -5.0;
    streak.accel = -0.015;
    let pale = lot(256);
    streak.colour = if blue { [pale, pale, 255.0] } else { [255.0, pale, pale] };
    streak
}

/// The streak of `CRagEffect::TintedRisingSparkles`, somewhere 3 to 9 round the one it is on: from 20 above the
/// ground it falls faster and faster; its red, green and blue are each whole one time in three and 140 of 255
/// else.
fn falling_streak() -> Prim {
    let mut streak = light_streak(3.0, -90.0);
    streak.put.1 = -20.0;
    streak.accel = 0.015;
    streak.colour = std::array::from_fn(|_| if lot(3) == 0.0 { 255.0 } else { 140.0 });
    streak
}

/// `CRagEffect::SonicSlashFan`, by its kind: at each of its first five ticks three sparks from 10 above the
/// place of the one it is on. Each is told to fly to its own place, so it only draws back at 2 a tick, slowing,
/// and returns along a way on the ground drawn by lot (towards a place up to 3 east or west and 3 north or
/// south, in whole steps; the lot falling on its own place, the way of an angle of nothing), swinging out to a
/// side and back as it does, in a bow that stands 30 out at its widest: the three of a tick swing to the left,
/// upwards and to the right. Each lives 60 ticks, is 1 to 1.5 times the sprite's size and is drawn seven
/// times, a trail. The kind 0's are green to white, the kind 1's a white or light grey (200 to 254), the kind
/// 2's a dark grey (20 to 119).
///
/// (The client measures how far a spark flies from the spark's place to the effect's, both that of the one the
/// effect is on as it begins: nothing, as it is taken here.)
fn sonic_slash_fan(kind: i32, tick: i32, prims: &mut Vec<Prim>) {
    if tick > 4 {
        return;
    }
    for roll in [-90.0, 0.0, 90.0] {
        let mut spark = launched::bowed(PARTICLE1, 60, 2.0, roll);
        // The client draws the place to the east first.
        let east = lot(7) - 3.0;
        spark.longitude = -faced(east, lot(7) - 3.0);
        spark.colour = match kind {
            0 => [lot(81) + 160.0, 255.0, lot(81) + 160.0],
            1 => [lot(55) + 200.0; 3],
            _ => [lot(100) + 20.0; 3],
        };
        (spark.size, spark.segments) = (lot(6) * 0.1 + 1.0, 7);
        prims.push(spark);
    }
}

/// `CRagEffect::ParticleTime` with its kind of 3, as `EF_HATED2` runs it, or of 1, as `EF_HATED` does: every
/// fourth tick up to the 80th, four motes of light, each somewhere within 5 east and north of the one the
/// effect is on and 6 above the feet, 2 to 2.9 from its middle to its corners, rising 0.1 to 0.29 a tick. They
/// are red (`RenderFieldPanel`, kind 13) or blue (kind 1), their light added, show `pok3.tga` for two ticks
/// on the way, and stay in the world where they were launched. (`RenderFieldPanel` gives the corners of the
/// kind 1 in another order, which turns the picture over; that is not done.)
fn warm_motes(colour: [f32; 3], course: &Course, prims: &mut Vec<Prim>) {
    if course.tick > 80 || course.tick % 4 != 0 {
        return;
    }
    let mut motes = Prim::new(Shape::ParticleUp, "pok1.tga");
    motes.pictures = &["pok1.tga", "pok3.tga"];
    (motes.duration, motes.colour, motes.adds, motes.left) = (course.lasts, colour, true, Some(course.walked));
    motes.slots = (0..4)
        .map(|_| {
            let out = lot(10) * 0.1 + 2.0;
            let east = lot(11) - 5.0;
            let north = lot(11) - 5.0;
            Slot::mote([east, 6.0, north], out, lot(20) * 0.01 + 0.1, true)
        })
        .collect();
    prims.push(motes);
}

/// `CRagEffect::ThunderSparks` with `thunder_center.bmp`, which `EF_SPRINKLESAND` calls twenty times: four
/// motes, each somewhere within 4 east and north of the feet of the one the effect is on, 0.1 to 0.55 from its
/// middle to its corners, that wait up to 10 ticks and rise 0.2 to 0.59 a tick, drifting up to 0.09 a tick
/// along a way of their own. They are a pale yellow, their light added (`RenderFieldPanel`, kind 10), and stay
/// in the world where they were launched.
///
/// (The client draws a mote's way by lot as the mote begins; here it is drawn as the mote is launched.)
fn sand(course: &Course) -> Prim {
    let mut motes = Prim::new(Shape::ParticleUp, "thunder_center.bmp");
    (motes.duration, motes.colour, motes.adds, motes.left) = (course.lasts, [250.0, 250.0, 150.0], true, Some(course.walked));
    motes.slots = (0..4)
        .map(|_| {
            let process = lot(10) as i32 - 10;
            let out = lot(16) * 0.03 + 0.1;
            let (east, north) = (lot(9) - 4.0, lot(9) - 4.0);
            let drifts = lot(10) * 0.01;
            let mut mote = Slot::mote([east, 0.0, north], out, lot(40) * 0.01 + 0.2, false);
            (mote.process, mote.drifts, mote.turned) = (process, drifts, lot(360) as i32);
            mote
        })
        .collect();
    motes
}

/// `CRagEffect::ThunderFreeze`: the lights of Duple Light (the kind 145), spread evenly round the one the
/// effect is on, each 15 over the feet with four squares of `freeze_a.bmp` about it that begin 20 ticks one
/// after the other, turned and at a place of their breath by lot.
///
/// The client's effect is told how many lights it is (the first figure of `deltaPos`, five at most): both
/// what launches it, the skill and the state `EFST_DUPLELIGHT`, tell it 2, and so it is two here.
fn duple_lights(course: &Course) -> Vec<Prim> {
    const LIGHTS: i32 = 2;
    (0..LIGHTS)
        .map(|light| {
            let mut prim = launched::of_parts(crate::prim::Tei::DriftingSparks, "thunder_center.bmp", true, course);
            prim.pictures = &["thunder_center.bmp", "freeze_a.bmp"];
            prim.parts.extend((0..4).map(|square| {
                let mut part = launched::part();
                (part.live, part.process, part.tall, part.rise) = (square == 0, -20 * square, 3.0, 200);
                (part.round, part.turned) = (lot(360) as i32, lot(360) as i32);
                part
            }));
            let first = &mut prim.parts[0];
            first.heights[..6].iter_mut().for_each(|turn| *turn = lot(360));
            (first.heights[6], first.heights[7], first.heights[8]) = ((light * 360 / LIGHTS) as f32, -15.0, lot(360));
            (first.out, first.flags[0]) = (1.0, 1);
            prim
        })
        .collect()
}
