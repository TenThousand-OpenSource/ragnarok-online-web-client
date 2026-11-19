//! Effects of the client's that draw nothing of their own and change the body of the one they are on
//! (`crate::body`): each arm of `CRagEffect::OnProcess` (or `OnProcess2`) here reaches `state.parent` and
//! colours it (`SetColor`), sets its `render.flags` and the counts beside them, or turns it (`render.angle`),
//! at the ticks it says; some start a sound or shake the screen (`MM_QUAKE`) besides.
//!
//! - the bodies of states, which live until the state ends (`crate::state_effects`): `EF_REDBODY` (368,
//!   Berserk), `EF_ASSUMPTIO` (375), `EF_PINKBODY` (396, Marionette Control), `EF_REFLECTBODY` (419, Kaite),
//!   `EF_BABYBODY` (420), `EF_ASURABODY` (424, a Soul Link), `EF_ASURABODY_MONSTER` (427), `EF_REDLIGHTBODY`
//!   (544, a Warmth);
//! - a colour for a time: `EF_TRANSBLUEBODY` (379), `EF_MAGICCRASHER` (380), `EF_LINKLIGHT` (385),
//!   `EF_MAGICCRASHER2` (403);
//! - a flash between a colour and white: `EF_CHEMICALBODY` (500), `EF_MEMORIZE` (505), `EF_DOUBLECASTBODY`
//!   (521), `EF_GREENBODY` (538), `EF_SHRINK` (599), `EF_PURPLEBODY` (768);
//! - a light over the body: `EF_HITBODY` (417), `EF_BLUEBODY` (542), `EF_RED_HIT` (548), `EF_BLUE_HIT` (549),
//!   `EF_MADNESS_BLUE` (625), `EF_MADNESS_RED` (626);
//! - the body moved: `EF_PRESSEDBODY` (413), `EF_KICKEDBODY` (415), `EF_4WAYBODY` (425), `EF_LANDBODY` (446),
//!   `EF_QUAKEBODY2` (458), `EF_SPINEDBODY2` (466), `EF_QUAKEBODY3` (550), `EF_QUAKEBODY4` (602),
//!   `EF_SOLAR_EXPLOSION` (1224), `EF_NEW_MAGNUM_BREAK` (1545);
//! - the body turned: `EF_FALCONASSAULT` (387), `EF_SPINMOVE` (890).
//!
//! - the weapon of a character drawn by itself (`crate::body`): `EF_LIGHTBLADE` (382, Mystical
//!   Amplification) has it drawn twice, `EF_AURABLADE2` (474, Aura Blade) with its light added; both live
//!   until their state ends;
//! - `EF_RUN` (442): one who runs leaves footprints (the flag 0x8000), until the state of running ends;
//!   `EF_STOPRUN` (443), which that state begins as it changes, does nothing: while it lives the skill shows
//!   the blades of a stop (`crate::state_effects::begins`);
//! - `EF_QUAKEBODY` (426): one who ran into a wall is shaken, as by `EF_QUAKEBODY2`;
//! - `EF_JUMPKICK` (457): the one who kicks is held in the sixth picture of its casting action from its
//!   fifth tick to its 35th. (The client puts its parent into a state of its own for that time, 27, and
//!   holds the picture with `SetFrameSequence`; here the picture alone is held.)
//! - `EF_ESCAPE` (1081): the one who slides away is yellow, and leaves four yellow copies of itself standing
//!   on the way (`escape`).

use super::*;
use crate::body::flag;

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[], sprites: &[] };

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect. The sounds an effect starts later are in `sounds`.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        368 | 375 | 420 => (99999, ""),
        382 | 396 | 419 | 424 | 427 | 474 | 544 => (99_999_999, ""),
        // `EF_RUN` lives until its state ends.
        442 => (9999, ""),
        426 => (14, ""),
        443 => (30, ""),
        457 => (100, ""),
        // `EF_ESCAPE` is told as it is launched to live 20 ticks (`AM_SETCOUNT` with 0xFFFF and 1058).
        1081 => (20, ""),
        379 | 385 | 387 | 542 | 599 => (200, ""),
        380 | 403 | 415 | 417 | 425 | 890 => (100, ""),
        413 | 466 | 1224 => (50, ""),
        446 | 458 => (35, ""),
        500 | 521 | 538 | 768 => (120, ""),
        505 => (200, "effect/priest_suffragium.wav"),
        548 | 549 | 550 | 602 | 625 | 626 => (60, ""),
        1545 => (300, ""),
        _ => return None,
    })
}

/// The sounds an effect starts after its first tick, each with its tick.
fn sounds(effect: u32) -> &'static [(i32, &'static str)] {
    match effect {
        380 => &[(25, "effect/매직 크래쉬.wav")],
        403 => &[(25, "effect/swordman_provoke.wav")],
        413 => &[(5, "effect/t_내려찍기.wav")],
        415 => &[(17, "effect/ef_hit2.wav"), (25, "effect/ef_hit4.wav")],
        457 => &[(5, "effect/t_날라차기.wav")],
        466 => &[(5, "effect/t_돌려차기.wav")],
        599 => &[(5, "effect/ef_beginspell.wav")],
        602 => &[(20, "effect/ef_hit6.wav")],
        1224 => &[(5, "effect/mon_침투경.wav")],
        _ => &[],
    }
}

/// One tick of an effect of this file: what it does to the body of the one it is on at that tick, the sound it
/// starts and the shake it asks for.
fn run(effect: u32, course: &mut Course, _prims: &mut Vec<Prim>) {
    let tick = course.tick;
    course.starts(sounds(effect));
    // `EF_MAGICCRASHER` and `EF_FALCONASSAULT` shake the screen at their 30th tick.
    if matches!(effect, 380 | 387) && tick == 30 {
        course.quake = Some(0);
    }
    if effect == 1081 {
        return escape(course);
    }
    let touch = &mut course.touch;
    match effect {
        // `EF_LIGHTBLADE`, `EF_AURABLADE2`: the weapon drawn twice, or with its light added.
        382 => touch.flags |= flag::WEAPON_TWICE,
        474 => touch.flags |= flag::WEAPON_ADDS,
        // `EF_RUN`: footprints, and the tick it is at, a byte.
        442 => (touch.flags, touch.run_tick) = (touch.flags | flag::RUNS, Some(tick as u8)),
        443 => {}
        // `EF_JUMPKICK`: the picture of the kick (the action at 96, its frame 5).
        457 if (5..35).contains(&tick) => touch.held = Some((96, 5)),
        457 => {}
        368 | 375 | 396 | 419 | 420 | 424 | 427 | 544 => stated(effect, tick, touch),
        379 | 380 | 385 | 403 => coloured(effect, tick, touch),
        500 | 505 | 521 | 538 | 599 | 768 => flashed(effect, tick, touch),
        417 | 542 | 548 | 549 | 625 | 626 => lit(effect, tick, touch),
        387 | 890 => touch.turn = turned(effect, tick, touch),
        _ => moved(effect, tick, course.lasts, touch),
    }
}

/// The bodies of states: what each does at every tick of its life.
fn stated(effect: u32, tick: i32, touch: &mut crate::body::Touch) {
    match effect {
        // `EF_REDBODY`: lit and red.
        368 => touch.flagged(flag::ADDS, 255, 100, 100),
        // `EF_ASSUMPTIO`: outlined.
        375 => touch.flags |= flag::OUTLINE,
        // `EF_PINKBODY`: lit, outlined and pink.
        396 => touch.flagged(flag::ADDS | flag::OUTLINE, 255, 89, 182),
        // `EF_REFLECTBODY`: the pale copies that widen behind it.
        419 => touch.flags |= flag::AURA,
        // `EF_BABYBODY`: it shrinks over 45 ticks and stays small.
        420 => {
            touch.flags |= flag::SHRINKS;
            touch.shrink_phase = (tick <= 45).then_some(tick as u8);
        }
        // `EF_ASURABODY`: blue copies behind it that come into sight over 200 ticks.
        424 => {
            touch.flags |= flag::SHADOWS;
            touch.outline_alpha = Some(tick.min(200) as u8);
        }
        // `EF_ASURABODY_MONSTER`: the white copies, whose count runs from 50 for 150 ticks.
        427 => {
            touch.flags |= flag::SHADOWS;
            touch.effect_phase = (tick < 150).then_some((tick + 50) as i16);
        }
        // `EF_REDLIGHTBODY`: the red light, coming up over 200 ticks and staying.
        _ => touch.tinted(flag::RED, if tick <= 200 { (tick as f32 * 0.125) as i32 } else { 25 }),
    }
}

/// A colour for a time.
fn coloured(effect: u32, tick: i32, touch: &mut crate::body::Touch) {
    match effect {
        // `EF_TRANSBLUEBODY`: bluer and bluer.
        379 => touch.colour(205 - tick, 205 - tick, 255),
        // `EF_MAGICCRASHER`: lit and in a colour by lot, from the 30th tick to the 60th.
        380 if (30..=60).contains(&tick) => touch.flagged(flag::ADDS, lot(255) as i32, lot(255) as i32, lot(255) as i32),
        // `EF_LINKLIGHT`: lit and golden, from the 40th tick to the 70th.
        385 if (40..=70).contains(&tick) => touch.flagged(flag::ADDS, 200, 150, 50),
        // `EF_MAGICCRASHER2`: a colour by lot for the first 60 ticks.
        403 if tick <= 60 => touch.colour(lot(255) as i32, lot(255) as i32, lot(255) as i32),
        _ => {}
    }
}

/// A flash between a colour and white (`Touch::flashes`), for some with an outline, over the ticks each has.
fn flashed(effect: u32, tick: i32, touch: &mut crate::body::Touch) {
    let (ticks, outlined, colour) = match effect {
        500 => (0..=30, true, [0, 0, 255]),
        505 => (10..=35, false, [250, 250, 100]),
        521 => (10..=50, true, [255, 0, 0]),
        538 => (0..=30, true, [0, 255, 0]),
        599 => (5..=25, false, [250, 250, 100]),
        _ => (0..=30, true, [255, 0, 255]),
    };
    if ticks.contains(&tick) {
        if outlined {
            touch.flags |= flag::OUTLINE;
        }
        touch.flashes(tick, colour);
    }
}

/// A light over the body, by the count each gives it.
fn lit(effect: u32, tick: i32, touch: &mut crate::body::Touch) {
    match effect {
        // `EF_HITBODY`: the white flash, its count the first 15 ticks.
        417 => {
            touch.flags |= flag::FLASH;
            touch.effect_phase = (tick < 15).then_some(tick as i16);
        }
        // `EF_BLUEBODY`: the blue light, a count every third tick up to the 150th.
        542 if tick <= 150 => touch.tinted(flag::BLUE, tick / 3),
        // `EF_RED_HIT`, `EF_BLUE_HIT`: three counts a tick for 16 ticks.
        548 if tick <= 16 => touch.tinted(flag::RED, tick * 3),
        549 if tick <= 16 => touch.tinted(flag::BLUE, tick * 3),
        // `EF_MADNESS_BLUE`, `EF_MADNESS_RED`: the light at its fullest, every fourth tick.
        625 if tick % 4 == 0 => touch.tinted(flag::BLUE, 20),
        626 if tick % 4 == 0 => touch.tinted(flag::RED, 20),
        _ => {}
    }
}

/// How far an effect turns its parent at a tick, the client's degrees.
fn turned(effect: u32, tick: i32, touch: &mut crate::body::Touch) -> f32 {
    if effect == 387 {
        // `EF_FALCONASSAULT`: 30 a tick from the 30th to the 54th, lit every other five ticks up to the 50th.
        if (30..=50).contains(&tick) && tick % 10 < 5 {
            touch.flags |= flag::ADDS;
        }
        return if (30..=54).contains(&tick) { 30.0 } else { 0.0 };
    }
    // `EF_SPINMOVE`: faster and then slower, 45 ticks in all.
    match tick {
        ..=7 => 12.0,
        8..=15 => 18.0,
        16..=20 => 24.0,
        21..=25 => 48.0,
        26..=32 => 24.0,
        33..=37 => 18.0,
        38..=45 => 12.0,
        _ => 0.0,
    }
}

/// The body moved: pressed, thrown, spun, shaken, blurred, by the count each gives it.
fn moved(effect: u32, tick: i32, lasts: i32, touch: &mut crate::body::Touch) {
    match effect {
        // `EF_PRESSEDBODY`.
        413 if (17..=50).contains(&tick) => touch.phased(flag::PRESSED, tick - 17),
        // `EF_KICKEDBODY`.
        415 if tick >= 30 => touch.phased(flag::KICKED, tick - 30),
        // `EF_4WAYBODY`.
        425 if tick >= 30 => touch.phased(flag::BLUR, tick - 30),
        // `EF_LANDBODY`: it comes down into sight, turning for its first 25 ticks.
        446 => {
            touch.tinted(flag::LANDS, tick);
            if tick < 25 {
                touch.phased(flag::SPUN, tick + 11);
            }
        }
        // `EF_QUAKEBODY` and `EF_QUAKEBODY2`, and `EF_QUAKEBODY3` from its 31st tick to its 50th.
        426 | 458 => touch.phased(flag::JITTER, tick),
        550 if (31..=50).contains(&tick) => touch.phased(flag::JITTER, tick),
        // `EF_SPINEDBODY2`, and `EF_SOLAR_EXPLOSION` to its end.
        466 if (14..=49).contains(&tick) => touch.phased(flag::SPUN, tick - 14),
        1224 if tick >= 14 && lasts > tick => touch.phased(flag::SPUN, tick - 14),
        // `EF_QUAKEBODY4`: shaken, red at every other tick.
        602 if tick > 20 => {
            touch.phased(flag::JITTER, tick);
            if tick % 2 == 0 {
                touch.colour(250, 50, 50);
            } else {
                touch.colour(255, 255, 255);
            }
        }
        // `EF_NEW_MAGNUM_BREAK`: blurred all its life; lit, outlined and blue for its first 30 ticks.
        1545 => {
            touch.phased(flag::BLUR, tick);
            if tick <= 30 {
                touch.flagged(flag::ADDS | flag::OUTLINE, 0, 0, 255);
            }
        }
        _ => {}
    }
}

/// `EF_ESCAPE`: the one it is on is yellow (200, 200, 0) at every tick, and at every fifth a copy of it is
/// left standing in that colour (`CHalluPC`): the first where it stood as the effect was launched (the place
/// the effect is told), each next one half of what is left of the way from the one before to where it is
/// now. `Course::reach` is where the next copy stands, from where the effect began.
///
/// (With nothing left of the way the client's place for the next copy is no number any more, and no copy
/// after that one is seen; `Course::step` remembers that here.)
fn escape(course: &mut Course) {
    course.touch.colour(200, 200, 0);
    if course.tick % 5 != 0 || course.step[0] != 0.0 {
        return;
    }
    let way: [f32; 3] = std::array::from_fn(|axis| course.walked[axis] - course.reach[axis]);
    course.copies.push((way.map(|part| -part), [200, 200, 0]));
    if way == [0.0; 3] {
        course.step[0] = 1.0;
    }
    course.reach = std::array::from_fn(|axis| course.reach[axis] + way[axis] * 0.5);
}
