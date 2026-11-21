//! Walls of light round the one an effect is on, each with the figures of its function of `CRagEffect`:
//!
//! - the rings of a spell being cast (`TexturedQuadRingSized`, twice): `EF_BEGINSPELL` (12, yellow, kind 1),
//!   `EF_BEGINSPELL6` (58, white, kind 0),
//!   `EF_BLUECASTING` (441, `BlueSpellRings`, kind 5), `EF_DARKCASTING` (454) and `EF_DARKCASTING2` (826,
//!   `BlackSpellRings`, kind 6), `EF_HOMUNCASTING` (564, `TexturedSpellRings`, kind 3), `EF_GREENCASTING` (911,
//!   `WhiteSpellRings`, kind 15), and the white and the yellow rings of `EF_AURABLADE` (367,
//!   `WhiteYellowRings`, kind 22); and `EF_COUPLECASTING` (342, `TexturedSpellRings`, kind 3), which lives
//!   1000 ticks: its walls show from the first and begin anew over and over; `EF_BEGINASURA` (328,
//!   `AsuraStrike` with its kind 0, the rings' kind 2), as long, with the six signs of Asura Strike that
//!   flash up over the one who casts, three to the left and three to the right (`PP_ASURACASTING`);
//! - `EF_HEARTCASTING` (343): twenty pairs of small pink walls that stand in the shape of a heart round the
//!   one it is on, the heart turned to face the camera (`RingRedPulse`, `PP_HEAL_LOW`), for 1000 ticks;
//! - the walls and the pillar of a spell of an element being cast (`QuadRingStackSized`): `EF_BEGINSPELL2`
//!   (54, water, blue, kind 2), `EF_BEGINSPELL3` (55, fire, yellow, kind 1), `EF_BEGINSPELL5` (57, earth,
//!   yellow, kind 0), and with a pillar that breathes (its kind 33) `EF_BEGINSPELL4` (56, wind, green) and
//!   `EF_BEGINSPELL7` (59, holy, violet). (These seven are what an effect by its number shows, as a script of
//!   an NPC sends one; a cast itself is still drawn by `world/effects.rs`.)
//! - `EF_CHANGEDARK` (172) to `EF_CHANGEPOISON` (179): the pillar of light as a monster changes its element,
//!   each with a ring of its own colour for its picture (`QuadRingStackSized`, kind 3);
//! - `EF_DEFENDER` (222), `EF_REFLECTSHIELD` (252): Defender's black and Reflect Shield's yellow walls that
//!   turn about the one guarded (`QuadRingStack`);
//! - four walls that spin away (`TexturedCrossKind`): `EF_GUMGANG3` (263, yellow, level 1), `EF_GUMGANG4` (773,
//!   purple, level 1), `EF_GUMGANG5` (886, level 3), `EF_GUMGANG6` (934), `EF_GUMGANG7` (936), `EF_GUMGANG8`
//!   (937) and `EF_GUMGANG9` (938), all of level 4;
//! - four walls that spin away slower and lean further (`TexturedCrossLevel`, `TexturedQuadSpiral`):
//!   `EF_GUMGANG2` (261, yellow, numbered 0) and `EF_THE_ONE_FIGHTER_RISES` (2338, numbered 5);
//! - `EF_ENTRY2` (344): the walls round one who arrives by a teleport, which rise and fall once
//!   (`PortalRings`);
//! - `EF_BEGINSPELL_N3` to `EF_BEGINSPELL_N8` (840 to 845): the swirl of bubbles a song or a dance begins
//!   with, in six colours (`BeginSpellSwirl`);
//! - `EF_FLOWERCAST` (488): the Comforts of the Sun, the Moon and the Stars, eight blue walls like petals
//!   round the one comforted, in two rings (`TexturedSlashAir` twice; `PP_FLOWERCAST`). Blue copies of that
//!   one's body come into sight behind it over 200 ticks and go over the next 200 (its `render.flags` of
//!   0x800 with `outlineAlpha`), and a sound starts at the effect's sixth tick.
//!
//! `RenderTeiRect` draws a wall by its primitive's `size`, which the effect's function sets from the kind or
//! level it is told; what that makes of each is said at its function here. The walls themselves are in
//! `crate::prim` (`prim/walls.rs`, `prim/casts.rs`).
//!
//! Beside its walls the client's arm of several of these changes the body of the one the effect is on
//! (`crate::body`; `touched`): `EF_GUMGANG5`, `EF_GUMGANG7`, `EF_GUMGANG8` and `EF_GUMGANG9` blur it all their
//! life (`render.flags | 0x1000`) and for their first 30 ticks light it, outline it and colour it;
//! `EF_GUMGANG6` lights and outlines it on and off from its tenth tick to its 34th.

use super::*;
use crate::prim::{Cast, Part, Tei, Walls};

pub(super) const FILE: File = File { known, thrown: none, stays, run, pictures: &[&PICTURES, &BUBBLES, &ASURA[0], &ASURA[1]], sprites: &[] };

/// The signs of `AsuraStrike`, in the order it names them: those of its first primitive and of its second.
const ASURA: [[&str; 3]; 2] = [["asura3.tga", "asura2.tga", "asura1.tga"], ["asura4.tga", "asura5.tga", "asura6.tga"]];

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 11] = [
    "magic_green.tga",
    "magic_violet.tga",
    "the_one_fighter_rises\\the_one_fighter_rises\\the_ring_00.tga",
    "ring_white.tga",
    "ring_black.tga",
    "ring_red.tga",
    "ring_blue.tga",
    "ring_yellow.tga",
    "ring_jadu.tga",
    "ring_brown.tga",
    "ring_purple.tga",
];

/// The 27 pictures the swirl of bubbles runs through, one a tick (`BeginSpellSwirl`: `effect\W_bubble%02d.tga`,
/// 1 to 27).
const BUBBLES: [&str; 27] = [
    "W_bubble01.tga",
    "W_bubble02.tga",
    "W_bubble03.tga",
    "W_bubble04.tga",
    "W_bubble05.tga",
    "W_bubble06.tga",
    "W_bubble07.tga",
    "W_bubble08.tga",
    "W_bubble09.tga",
    "W_bubble10.tga",
    "W_bubble11.tga",
    "W_bubble12.tga",
    "W_bubble13.tga",
    "W_bubble14.tga",
    "W_bubble15.tga",
    "W_bubble16.tga",
    "W_bubble17.tga",
    "W_bubble18.tga",
    "W_bubble19.tga",
    "W_bubble20.tga",
    "W_bubble21.tga",
    "W_bubble22.tga",
    "W_bubble23.tga",
    "W_bubble24.tga",
    "W_bubble25.tga",
    "W_bubble26.tga",
    "W_bubble27.tga",
];

/// How many ticks an effect of this file lives and the sound it starts with (none: empty); nothing for any
/// other effect. The times are the client's `EffectLifetime`, but that `TexturedQuadRingSized` gives an effect
/// shorter than that 56 ticks, and that the swirl of bubbles lives 120 ticks at least (`BeginSpellSwirl` sets
/// the effect's `duration`; its table says 100), and that `QuadRingStackSized` gives one shorter than that 70
/// (the table has 40 for the rings of a cast).
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        12 | 58 => (56, "effect/ef_beginspell.wav"),
        54..=57 | 59 => (70, "effect/ef_beginspell.wav"),
        172..=179 => (100, ""),
        222 | 252 => (200, ""),
        261 | 263 => (300, ""),
        328 | 342 | 343 => (1000, "effect/ef_beginspell.wav"),
        344 => (100, "effect/ef_portal.wav"),
        367 => (60, ""),
        441 | 454 | 564 | 911 => (60, "effect/ef_beginspell.wav"),
        488 => (400, ""),
        773 => (300, ""),
        // `BlackSpellRings(2)` is told to be silent.
        826 => (60, ""),
        840..=845 => (120, "effect/ef_beginspell.wav"),
        886 | 934 | 936 | 937 | 938 => (300, ""),
        2338 => (70, ""),
        _ => return None,
    })
}

/// Whether an effect stays where it began: the pillars are left there, as `QuadRingStackSized` takes its
/// parent's place as it is called, which is at the effect's first tick alone. All the others take it every
/// tick.
fn stays(effect: u32) -> bool {
    matches!(effect, 172..=179)
}

/// What the Gumgangs of the later jobs do at a tick to the body of the one they are on, as their arms of
/// `CRagEffect::OnProcess2` have it.
fn touched(effect: u32, tick: i32, touch: &mut crate::body::Touch) {
    use crate::body::flag;
    let colour = match effect {
        886 => [0, 0, 255],
        936 => [255, 0, 0],
        937 => [255, 255, 0],
        938 => [80, 40, 0],
        // `EF_FLOWERCAST`.
        488 => {
            touch.flags |= flag::SHADOWS;
            touch.outline_alpha = if tick < 200 { Some(tick as u8) } else { Some((400 - tick) as u8) };
            return;
        }
        // `EF_GUMGANG6`: lit and outlined two ticks of every four.
        934 if (10..=34).contains(&tick) && tick % 4 < 2 => return touch.flags |= flag::ADDS | flag::OUTLINE,
        _ => return,
    };
    touch.phased(flag::BLUR, tick);
    if tick <= 30 {
        touch.flagged(flag::ADDS | flag::OUTLINE, colour[0], colour[1], colour[2]);
    }
}

/// One tick of an effect of this file: all launch what they launch at their first tick.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    touched(effect, course.tick, &mut course.touch);
    if effect == 488 && course.tick == 5 {
        course.sound("effect/t_안락한마법.wav");
    }
    if effect == 328 && matches!(course.tick, 1 | 21) {
        prims.push(asura_signs(course.tick == 21, course));
    }
    if course.tick != 0 {
        return;
    }
    let lasts = course.lasts;
    let rings = |picture: &'static str, kind: i32| [spell_ring(picture, kind, lasts), spell_ring(picture, kind, lasts)];
    match effect {
        12 => prims.extend(rings("ring_yellow.tga", 1)),
        54 => prims.push(element_walls("ring_blue.tga", 2, lasts)),
        55 => prims.push(element_walls("ring_yellow.tga", 1, lasts)),
        56 => prims.push(element_walls("magic_green.tga", 33, lasts)),
        57 => prims.push(element_walls("ring_yellow.tga", 0, lasts)),
        58 => prims.extend(rings("ring_white.tga", 0)),
        59 => prims.push(element_walls("magic_violet.tga", 33, lasts)),
        172 => prims.push(pillar("ring_black.tga", lasts)),
        173 => prims.push(pillar("ring_red.tga", lasts)),
        174 => prims.push(pillar("ring_blue.tga", lasts)),
        175 => prims.push(pillar("ring_yellow.tga", lasts)),
        176 => prims.push(pillar("ring_jadu.tga", lasts)),
        177 => prims.push(pillar("ring_brown.tga", lasts)),
        178 => prims.push(pillar("ring_white.tga", lasts)),
        179 => prims.push(pillar("ring_purple.tga", lasts)),
        222 => prims.push(turning_walls("ring_black.tga", lasts)),
        252 => prims.push(turning_walls("ring_yellow.tga", lasts)),
        261 => prims.push(slow_walls("ring_yellow.tga", 0, lasts)),
        263 => prims.push(spinning_walls("ring_yellow.tga", 1, lasts)),
        328 => prims.extend([45, 25].map(|alpha| lasting_ring("ring_white.tga", 2, alpha, lasts))),
        342 => prims.extend([45, 25].map(|alpha| lasting_ring("ring_red.tga", 3, alpha, lasts))),
        343 => prims.extend(HEART.map(|(east, north)| heart_walls(east as f32 * 0.27, north as f32 * 0.27, lasts))),
        344 => prims.push(entry_walls(course)),
        367 => prims.extend([spell_ring("ring_white.tga", 22, lasts), spell_ring("ring_yellow.tga", 22, lasts)]),
        441 => prims.extend(rings("ring_blue.tga", 5)),
        454 | 826 => prims.extend(rings("ring_black.tga", 6)),
        488 => prims.extend([petals(0, course), petals(1, course)]),
        564 => prims.extend(rings("ring_red.tga", 3)),
        773 => prims.push(spinning_walls("ring_purple.tga", 1, lasts)),
        840..=845 => prims.extend(swirl(effect as i32 - 837, lasts)),
        886 => prims.push(spinning_walls("ring_blue.tga", 3, lasts)),
        911 => prims.extend(rings("ring_white.tga", 15)),
        934 => prims.push(spinning_walls("ring_white.tga", 4, lasts)),
        936 => prims.push(spinning_walls("ring_red.tga", 4, lasts)),
        937 => prims.push(spinning_walls("ring_yellow.tga", 4, lasts)),
        938 => prims.push(spinning_walls("ring_brown.tga", 4, lasts)),
        2338 => prims.push(slow_walls("the_one_fighter_rises\\the_one_fighter_rises\\the_ring_00.tga", 5, lasts)),
        _ => {}
    }
}

/// `CRagEffect::TexturedQuadRingSized` for an effect of 60 ticks or fewer, by the kind it is told: four walls
/// of the kind 30 round the one it is on, 4.1 out and rising at 80 degrees, begun a quarter round from each
/// other, 20, 19, 18 and 17 tall (15, 14, 13 and 12 for the kind 22, 25 to 22 for the kind 2); nothing of them shows at first, and each
/// begins five ticks after the one before. The effects call the function twice, with two alphas for the walls
/// to begin with, which an effect this short sets to nothing: the same walls are drawn twice.
///
/// `RenderTeiRect` draws the walls by the primitive's `size`, which the kind gives: the kind 0 (a `size` of 5)
/// white, the kind 1 (8) a pale yellow, the kind 3 (10) pink, the kind 5 (4) blue, the kind 15 (7) green and the kind 22 (26) a pale blue,
/// the light of each added to what is behind it; the kind 6 (12) a dark grey mixed in.
fn spell_ring(picture: &'static str, kind: i32, lasts: i32) -> Prim {
    let mut walls = Prim::new(Shape::Walls(Walls::Spell), picture);
    walls.duration = lasts;
    let tallest = match kind {
        2 => 25.0,
        22 => 15.0,
        _ => 20.0,
    };
    (walls.colour, walls.adds) = match kind {
        1 => ([255.0, 255.0, 170.0], true),
        3 => ([255.0, 89.0, 182.0], true),
        5 => ([100.0, 100.0, 255.0], true),
        6 => ([50.0; 3], false),
        15 => ([100.0, 255.0, 100.0], true),
        22 => ([164.0, 205.0, 254.0], true),
        _ => ([255.0; 3], true),
    };
    walls.slots = [180, 270, 0, 90]
        .into_iter()
        .enumerate()
        .map(|(index, turned)| {
            let mut wall = Slot::wall(turned, 4.1, 80, tallest - index as f32, kind);
            wall.process = index as i32 * -5;
            wall
        })
        .collect();
    walls
}

/// `CRagEffect::TexturedQuadRingSized` for an effect of more than 60 ticks: the same four walls, but all begun
/// at once and showing from the first, the last at the alpha the function is told (45 for the first of an
/// effect's two calls and 25 for the second) and each one before it 45 more.
fn lasting_ring(picture: &'static str, kind: i32, alpha: i32, lasts: i32) -> Prim {
    let mut walls = spell_ring(picture, kind, lasts);
    for (index, wall) in walls.slots.iter_mut().enumerate() {
        (wall.process, wall.alpha) = (0, alpha + 45 * (3 - index as i32));
    }
    walls
}

/// `CRagEffect::AsuraStrike` with its kind 0, one of its two primitives of signs (`PP_ASURACASTING`, its mode
/// 0), launched at the effect's second tick and at its 22nd: three signs, each 18 from its middle to a
/// corner, 20 ticks apart. Those of the first stand 6, 18 and 30 to the left of the middle, the farthest
/// begun at once and the nearest 40 ticks late; those of the `second` as far to the right, the nearest 40
/// ticks late and the farthest 80.
fn asura_signs(second: bool, course: &Course) -> Prim {
    let signs = &ASURA[usize::from(second)];
    let mut prim = launched::of_parts(Tei::AsuraCasting, signs[0], true, course);
    prim.pictures = signs;
    prim.parts.extend((0..3).map(|index| {
        let mut part = Part { process: if second { -40 - index * 20 } else { index * 20 - 40 }, out: 18.0, ..Part::default() };
        let aside = (6 + index * 12) as f32;
        (part.heights[0], part.flags[0]) = (if second { aside } else { -aside }, index as i8);
        part
    }));
    prim
}

/// `CRagEffect::QuadRingStackSized` with the kind 3, as the effects of a changed element call it at their first
/// tick: a primitive of the kind 23 with four walls, of which one of that size lets only the fourth live: a
/// pillar all the way round, 4 out and all but upright, that rises to 250 over 90 ticks, its top a bow along
/// the wall. It is mixed in white at an alpha of 80 for as long as the effect lives, and is gone with it.
fn pillar(picture: &'static str, lasts: i32) -> Prim {
    stack(Cast::Stone { last_only: true }, picture, lasts)
}

/// `CRagEffect::QuadRingStackSized` with the kinds the rings of a cast have: all four walls live, the three
/// low ones turning round the caster inside the pillar. `RenderTeiRect` draws them by the primitive's `size`,
/// which is the kind: 0 white and mixed in, 1 a pale red whose light is added, 2 a pale blue mixed in. The
/// kind 33 is a primitive of the kind 25 (`Cast::Fire`: its pillar breathes) of the size 0.
fn element_walls(picture: &'static str, kind: i32, lasts: i32) -> Prim {
    let mut walls = stack(if kind == 33 { Cast::Fire } else { Cast::Stone { last_only: false } }, picture, lasts);
    (walls.colour, walls.adds) = match kind {
        1 => ([255.0, 175.0, 175.0], true),
        2 => ([195.0, 195.0, 255.0], false),
        _ => ([255.0; 3], false),
    };
    walls
}

/// The four walls `QuadRingStackSized` gives its primitive: three that go most of a round, begun a quarter
/// round from each other, each wider, lower and flatter than the one before, showing as the client starts
/// them; and the pillar, all the way round.
fn stack(cast: Cast, picture: &'static str, lasts: i32) -> Prim {
    let mut walls = Prim::new(Shape::Cast(cast), picture);
    walls.duration = lasts;
    walls.slots = [(0, 4.5, 70, 25.0), (90, 5.0, 57, 22.0), (180, 5.5, 45, 19.0)]
        .into_iter()
        .map(|(turned, out, rise, tall)| {
            let mut wall = Slot::wall(turned, out, rise, tall, 0);
            (wall.round, wall.alpha) = (315, 180);
            wall
        })
        .collect();
    let mut fourth = Slot::wall(0, 4.0, 89, 250.0, 0);
    fourth.alpha = 70;
    walls.slots.push(fourth);
    walls
}

/// `CRagEffect::QuadRingStack`: a primitive of the kind 24 with four walls all the way round the one it is
/// on, upright, 8, 7.9, 7.8 and 7.7 out and 40, 39, 38 and 37 tall, begun a quarter round from each other. They
/// come up slowly, turn, and fade over the effect's last 20 ticks; the client mixes them in white by their
/// alpha (a primitive of walls whose `size` is nothing).
fn turning_walls(picture: &'static str, lasts: i32) -> Prim {
    let mut walls = Prim::new(Shape::Cast(Cast::Turning), picture);
    walls.duration = lasts;
    walls.slots = [180, 270, 0, 90]
        .into_iter()
        .enumerate()
        .map(|(index, turned)| Slot::wall(turned, 8.0 - index as f32 * 0.1, 90, 40.0 - index as f32, 0))
        .collect();
    walls
}

/// `CRagEffect::TexturedCrossKind`, by its level: four walls of the kind 31 round the one it is on, 1, 2, 3
/// and 4 out, 25 tall and rising at 80 degrees, begun a quarter round from each other, that spin away for as
/// long as the effect lives. The level is the number its walls go by (`alphaT`). `RenderTeiRect` draws them
/// by the primitive's `size`: 17 for level 3, a blue whose light is added; nothing for the others, the
/// picture's own colours mixed in by their alpha.
pub(super) fn spinning_walls(picture: &'static str, level: i32, lasts: i32) -> Prim {
    let mut walls = Prim::new(Shape::Walls(Walls::Spin), picture);
    walls.duration = lasts;
    if level == 3 {
        (walls.colour, walls.adds) = ([0.0, 0.0, 255.0], true);
    }
    walls.slots = (0..4).map(|index| Slot::wall(index * 90, index as f32 + 1.0, 80, 25.0, level)).collect();
    walls
}

/// `CRagEffect::PortalRings` with `ring_blue.tga`: four walls all the way round the one who arrives (the kind
/// 60, its columns of the kind 2, which rise and fall once over 45 ticks and are then gone), each begun at any
/// turn and turning: two upright, 3.7 and 3.4 out and 30 tall, and two that lie all but flat, 3.6 and 3.7 out,
/// 4 tall and rising at 10 and 5 degrees. Blue (`RenderTeiRect` for a primitive of size 4), their light added.
fn entry_walls(course: &Course) -> Prim {
    let mut walls = launched::of_parts(Tei::Columns, "ring_blue.tga", true, course);
    walls.size = 4.0;
    walls.parts.extend([(30.0, 3.7, 90), (30.0, 3.4, 90), (4.0, 3.6, 10), (4.0, 3.7, 5)].map(|(tall, out, rise)| Part {
        round: 360,
        tall,
        turned: lot(360) as i32,
        out,
        rise,
        alpha_top: 1400,
        flags: [2; 21],
        ..Part::default()
    }));
    walls
}

/// `CRagEffect::TexturedCrossLevel` and `CRagEffect::TexturedQuadSpiral`: the same four walls numbered `level`
/// (the spiral's are numbered 5), of a primitive whose `accel` is 99: they turn and widen slower and lean
/// further (`Walls::SlowSpin`). The picture's own colours are mixed in by their alpha.
fn slow_walls(picture: &'static str, level: i32, lasts: i32) -> Prim {
    let mut walls = spinning_walls(picture, level, lasts);
    walls.kind = Shape::Walls(Walls::SlowSpin);
    walls
}

/// `CRagEffect::BeginSpellSwirl`, by the number it is told (3 to 8 for `EF_BEGINSPELL_N3` to `N8`): four walls
/// of the kind 149 round the one who begins, all 4.3 out and 23 tall, begun a quarter round from each other
/// and rising at 75, 63, 52 and 40 degrees, with the 27 pictures of the bubbles. `RenderGroundArcGlow` draws
/// every piece twice, which is two primitives here that move alike: once white and mixed in, and once with
/// its light added in the colour of the number, red (at three times the alpha), green, yellow, white, purple
/// or cyan (at twice the alpha).
fn swirl(number: i32, lasts: i32) -> [Prim; 2] {
    let (colour, gain) = match number {
        3 => ([255.0, 0.0, 0.0], 3),
        4 => ([0.0, 255.0, 0.0], 2),
        5 => ([255.0, 255.0, 0.0], 2),
        7 => ([255.0, 0.0, 255.0], 2),
        8 => ([0.0, 255.0, 255.0], 2),
        _ => ([255.0; 3], 2),
    };
    let walls = |gain: i32, colour: [f32; 3], adds: bool| {
        let mut walls = Prim::new(Shape::Walls(Walls::Wave { gain }), BUBBLES[0]);
        (walls.pictures, walls.duration, walls.colour, walls.adds) = (&BUBBLES, lasts, colour, adds);
        walls.slots = [(180, 75), (270, 63), (0, 52), (90, 40)].into_iter().map(|(turned, rise)| Slot::wall(turned, 4.3, rise, 23.0, number)).collect();
        walls
    };
    [walls(1, [255.0; 3], false), walls(gain, colour, true)]
}

/// `CRagEffect::TexturedSlashAir` with `ring_blue.tga` and its size 4 (blue, the light added): four walls
/// (`PP_FLOWERCAST`) round the one the effect is on, each 315 degrees of a round and arched, a quarter round
/// from each other. Those of the `phase` 0 are 4.5 to 5.5 out, lean out by 5 to 20 degrees and are 27 to 30
/// tall; those of the phase 1 are 5.5 to 6.5 out and lean by 25 to 40 degrees.
fn petals(phase: usize, course: &Course) -> Prim {
    // How far out, how steep, how tall and from where round, wall by wall.
    const WALLS: [[(f32, i32, f32, i32); 4]; 2] = [
        [(4.5, 85, 27.0, 0), (5.0, 80, 28.0, 90), (5.5, 70, 29.0, 180), (5.0, 75, 30.0, 270)],
        [(5.5, 50, 30.0, 90), (6.0, 55, 29.0, 180), (6.5, 65, 28.0, 270), (6.0, 60, 27.0, 0)]
    ];
    let mut prim = launched::of_parts(Tei::FlowerCast, "ring_blue.tga", true, course);
    prim.size = 4.0;
    prim.parts.extend(WALLS[phase].map(|(out, rise, tall, turned)| {
        let mut part = Part { round: 315, out, rise, tall, turned, ..Part::default() };
        part.at[1] = [rise as f32, tall, 0.0];
        part
    }));
    prim
}

/// Where the twenty pairs of walls of `EF_HEARTCASTING` stand from the middle of the heart, in steps of 0.27:
/// to the right as the camera sees it, and away from it (the arm of `CRagEffect::OnProcess2`).
const HEART: [(i32, i32); 20] = [
    (0, 37), (-13, 44), (-29, 48), (-46, 42), (-56, 26), (-57, 6), (-51, -12), (-40, -26), (-25, -38), (-10, -47),
    (0, -60), (11, -47), (26, -38), (41, -26), (52, -12), (58, 6), (57, 26), (47, 42), (30, 48), (14, 44)
];

/// `CRagEffect::RingRedPulse`: two small walls of `ring_red.tga` all the way round a point so far aside
/// (`PP_HEAL_LOW`: ten pieces each, the way aside turned with the camera), each begun at any turn: one 2.3
/// out, 12 tall and rising at 86 degrees, one 2 out, 25 tall and upright. They come up over 15 ticks, swing
/// up and down, and go down over the effect's last 25 ticks. Pink, their light added (`RenderTeiRect` for a
/// primitive of size 10).
fn heart_walls(east: f32, north: f32, lasts: i32) -> Prim {
    let mut walls = Prim::new(Shape::CastingLow, "ring_red.tga");
    (walls.duration, walls.adds, walls.colour) = (lasts, true, [255.0, 89.0, 182.0]);
    walls.slots = [(2.3, 86, 12.0), (2.0, 90, 25.0)]
        .map(|(out, rise, tall)| Slot { at: [east, 0.0, north], ..Slot::wall(lot(360) as i32, out, rise, tall, lasts - 25) })
        .to_vec();
    // (The third part keeps the turn of the heart: `crate::prim`'s `low_walls.rs`.)
    walls.slots.extend([(); 2].map(|()| Slot { live: false, ..Slot::wall(0, 0.0, 0, 0.0, 0) }));
    walls
}
