//! What a skill used on a place shows there (`CGameActor::PlayGroundSkillEffect`, `crate::skill_units`), and
//! what stands on a cell of the later skills that leave units (`CSkill::OnProcess`): each with the figures of
//! its function of `CRagEffect`.
//!
//! - `EF_METEORSTORM` (92): the shake of the screen as a meteor lands. (The meteor is a file picked by lot:
//!   `crate::effect::played`.)
//! - `EF_DETECTING` (119): Detect, a ring that widens over the ground (`TexturedSkillAura` with no type);
//! - `EF_HEAVENSDRIVE` (142): Heaven's Drive, 25 stones pushed up out of the ground (`StoneSpike`);
//! - `EF_GANBANTEIN` (223), `EF_VOLCANO` (225), `EF_DELUGE` (236), `EF_VIOLENTGALE` (237),
//!   `EF_LANDPROTECTOR` (238): four walls that spin away from the place (`TexturedCrossKind`), white and low
//!   for Ganbantein (its level 2), red, blue, green and white for the others;
//! - `EF_DEMONSTRATION` (302): the fire on a cell of Demonstration (`Demonstration`);
//! - `EF_GRAVITATION` (522): Gravitation Field, a hundred faint spikes of stone and of ice that go up and down
//!   (`Gravitation`);
//! - `EF_PORTAL3` (341): the pink walls, rings and winds on the cell of a call to one's family
//!   (`VioletRingCloud` with its type 3);
//! - `EF_BOTTOM_GOSPEL` (370): the small cross that hovers over a cell of Gospel (`Bottom_Music` with its
//!   kind 9);
//! - `EF_BOTTOM_BASILICA` (374): the walls of Basilica and the lights that run along them
//!   (`TexturedFourRing`, its kinds 0 and 1);
//! - `EF_BOTTOM_SPIDER` (404): the web that breathes on a cell of Spider Web (`TexturedHalo` with its kind 2);
//! - `EF_SLIM`, `EF_SLIM2`, `EF_SLIM3` (497 to 499): Slim Potion Pitcher, the potion that comes down turning
//!   over a square of light that widens (`TexturedRain`, `TexturedLevelRing`): red, yellow and white;
//! - `EF_BOTTOM_HERMODE` (517): the small blue box that turns over a cell of Hermode's Rod (`TexturedSpin`);
//! - `EF_BOTTOM_SUITON` (620): the low blue wall of light that widens over and over on a cell of Suiton
//!   (`TexturedFlare` with the size 14).
//!
//! The grey box on a cell of Fog Wall (`EF_BOTTOM_FOGWALL`) is kept with the units themselves, as Sanctuary's
//! is (`crate::skill_units::Pulse`).
//!
//! `StoneSpike` and `Gravitation` shake the screen as they begin (`MM_QUAKE`). The sounds the client's handler
//! of a skill starts beside these (Ganbantein's, Gravitation's) are the skill's
//! (`crate::effect_tables::skill_sounds`), not the effect's.

use super::launched::{of_parts, ripples, skill_aura, square};
use super::*;
use crate::prim::{Change, Part, Spike, Tei};

pub(super) const FILE: File = File { known, thrown: none, stays, run, pictures: &[&PICTURES], sprites: &[&SPRITES] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 16] = [
    "fashasha.tga", "stone.bmp", "ice.tga", "ring_white.tga", "ring_red.tga", "ring_blue.tga", "magic_green.tga", "cloud11.tga",
    "cross_old.bmp", "alpha_down.tga", "spiderweb.tga", "white02.bmp", "bbbb.bmp", POTIONS[0], POTIONS[1], POTIONS[2]
];

/// The potions of `EF_SLIM`, `EF_SLIM2` and `EF_SLIM3`: pictures of items, from the folder of the interface's
/// pictures (`world/loading.rs` reads a name that begins with that folder from there).
const POTIONS: [&str; 3] = [
    "유저인터페이스\\item\\레드슬림포션.bmp", "유저인터페이스\\item\\옐로우슬림포션.bmp", "유저인터페이스\\item\\화이트슬림포션.bmp"
];

/// The sprite of Demonstration's fire, among the client's sprites (`이팩트\데몬스트레이션.spr`).
const FIRE: &str = "이팩트/데몬스트레이션";
const SPRITES: [&str; 1] = [FIRE];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        // The client's lives 9999 ticks, or as long as its file plays; here it is its file and, besides,
        // this: the shake at its 40th tick and nothing after.
        92 => (41, ""),
        119 => (250, "effect/hunter_detecting.wav"),
        // `StoneSpike` gives itself 50 ticks more than its 250.
        142 => (300, "effect/wizard_earthspike.wav"),
        223 | 225 => (300, ""),
        236..=238 => (999, ""),
        302 => (99_999, ""),
        341 => (9999, ""),
        370 => (19999, ""),
        374 | 404 | 517 | 620 => (29999, ""),
        497..=499 => (250, ""),
        522 => (2000, ""),
        _ => return None,
    })
}

/// Whether an effect stays where it began: `TexturedSkillAura` never takes its parent's place.
fn stays(effect: u32) -> bool {
    effect == 119
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let (tick, lasts) = (course.tick, course.lasts);
    match effect {
        92 if tick == 40 => course.quake = Some(0),
        119 if tick == 0 => prims.push(skill_aura(0)),
        142 if tick == 0 => {
            course.quake = Some(0);
            prims.extend(grid().map(|(east, north)| stone(east, north, lasts)));
        }
        223 if tick == 0 => prims.push(cross("ring_white.tga", 2, lasts)),
        225 if tick == 0 => prims.push(cross("ring_red.tga", 0, lasts)),
        236 if tick == 0 => prims.push(cross("ring_blue.tga", 0, lasts)),
        237 if tick == 0 => prims.push(cross("magic_green.tga", 0, lasts)),
        238 if tick == 0 => prims.push(cross("ring_white.tga", 0, lasts)),
        302 if tick == 0 => prims.push(fire(lasts)),
        // `VioletRingCloud` with its type 3: the walls of a Warp Portal, its rings and its winds, one a tick,
        // all of `ring_red.tga` (the winds of `cloud11.tga`) in pink.
        341 => match tick {
            0 => prims.push(grounds::portal_walls("ring_red.tga", 10.0, course)),
            1 => prims.push(closing_rings(course)),
            2 => prims.push(grounds::portal_winds(10.0, course)),
            _ => {}
        },
        370 if tick == 0 => prims.push(hovering_cross(course)),
        374 if tick == 0 => prims.extend([four_ring(0, course), four_ring(1, course)]),
        404 if tick == 0 => prims.push(grounds::halo("spiderweb.tga", 2.0, 12.0, course)),
        497..=499 if tick == 0 => prims.extend(potion(effect as usize - 497, course)),
        517 if tick == 0 => prims.push(turning_box(course)),
        620 if tick == 0 => prims.push(grounds::flare("ring_blue.tga", 14.0, course)),
        522 if tick == 0 => {
            course.quake = Some(0);
            prims.extend(grid().flat_map(|(east, north)| ["stone.bmp", "stone.bmp", "ice.tga", "ice.tga"].map(|picture| pumping(picture, east, north))));
        }
        _ => {}
    }
}

/// The 25 places `StoneSpike` and `Gravitation` put their spikes at: every fifth of a cell's side from 10 west
/// and south of the middle to 10 east and north (the client's x and z).
fn grid() -> impl Iterator<Item = (f32, f32)> {
    (-2..=2).flat_map(|north| (-2..=2).map(move |east| (east as f32 * 5.0, north as f32 * 5.0)))
}

/// A spike as `StoneSpike` and `Gravitation` begin theirs: at its place of the grid 10 under the ground, 3 to
/// 3.5 wide, turned any way round, pushed up at 1 a tick, quickening by a hundredth.
fn spike(picture: &'static str, east: f32, north: f32) -> Prim {
    let mut spike = Prim::new(Shape::QuadHorn3d, picture);
    spike.first_pattern = true;
    (spike.put, spike.deep) = ((east, 10.0), north);
    spike.size = (lot(50) + 300.0) / 100.0;
    spike.longitude = lot(360);
    (spike.speed, spike.accel) = (1.0, 0.01);
    spike
}

/// `CRagEffect::StoneSpike`, one of its stones: 9.1 to 15 long, within 10 degrees of upright; it draws back at
/// 1.2 from its twelfth tick, stands still from its fifteenth, and 30 ticks before its end goes down again at
/// 1 a tick, quickening (its second pattern, with which it does not fade). It lives 10 ticks less than the
/// effect and up to 29 more by lot.
fn stone(east: f32, north: f32, lasts: i32) -> Prim {
    let mut stone = spike("stone.bmp", east, north);
    stone.duration = lot(30) as i32 + lasts - 10;
    (stone.settles_at, stone.spike) = (Some(14), Spike::Withdraws(-1.0, -0.01));
    stone.height = (lot(60) + 1.0) / 10.0 + 9.0;
    stone.latitude = lot(20) + 80.0;
    stone.changes = Some(Change { speed: -1.2, accel: 0.0, ..Change::at(11) });
    stone
}

/// `CRagEffect::Gravitation`, one of its spikes (`PP_3DQUADHORN2`): 18 long, within 30 degrees of upright,
/// all but unseen at an alpha of 20; it goes down at 1.2 for three ticks and up again for three, over and
/// over, for 250 ticks, fading over the last ten.
fn pumping(picture: &'static str, east: f32, north: f32) -> Prim {
    let mut spike = spike(picture, east, north);
    (spike.duration, spike.spike) = (250, Spike::Pumps(-1.2, 0.0));
    spike.height = 18.0;
    spike.latitude = lot(40) + 60.0;
    spike.alpha = 20.0;
    spike.fade_out = spike.duration - 10;
    spike
}

/// `CRagEffect::TexturedCrossKind` with its level of nothing or of 2: the four walls that spin away
/// (`walls::spinning_walls`); those of the level 2 are 15 tall and rise at 70 degrees, and fade four times as
/// fast.
fn cross(picture: &'static str, level: i32, lasts: i32) -> Prim {
    let mut walls = walls::spinning_walls(picture, level, lasts);
    if level == 2 {
        walls.slots.iter_mut().for_each(|wall| (wall.tall, wall.rise) = (15.0, 70));
    }
    walls
}

/// `CRagEffect::Demonstration`: the sprite of the fire 1 over the ground where the cell is, a fifth larger
/// than it is, at an alpha of 200, a frame every fourth tick, over and over. (The kind 51 of the client's
/// primitives, told not to follow its effect, stays at the place its part holds; a cell does not move, and
/// here it is a sprite of `PP_3DPARTICLE` with no speed, as the sign of Stop is: `held.rs`.)
fn fire(lasts: i32) -> Prim {
    let mut fire = Prim::new(Shape::Particle3d, FIRE);
    (fire.duration, fire.alpha, fire.anim_speed, fire.size) = (lasts, 200.0, 4, 1.2);
    fire.put.1 = -1.0;
    fire
}

/// `CRagEffect::VioletRingCloud` with its type 3, its second tick: the rings of a Warp Portal
/// (`launched::ripples`) of `ring_red.tga` in pink, of the kind 3, which close in from 14 out instead of
/// widening; the second begins 20 ticks and the third 40 after the first.
fn closing_rings(course: &Course) -> Prim {
    let mut rings = ripples("ring_red.tga", 10.0, 3111, 3, course);
    (rings.parts[1].process, rings.parts[2].process) = (-20, -40);
    rings
}

/// `CRagEffect::Bottom_Music` with its kind 9: one picture of `cross_old.bmp` facing the screen
/// (`PP_BOTTOM`), 1.5 from its middle to a corner, turned 45 degrees, at an alpha of 200, that hovers about 6
/// over the cell from anywhere in its swing by lot; white, its light added (`RenderFieldPanel`, 9 with 6).
fn hovering_cross(course: &Course) -> Prim {
    let mut cross = of_parts(Tei::Bottom, "cross_old.bmp", true, course);
    cross.size = 0.0;
    let mut part = Part { out: 1.5, alpha: 200, turned: lot(360) as i32, round: 45, ..Part::default() };
    (part.flags[2], part.flags[3]) = (9, 6);
    cross.parts.push(part);
    cross
}

/// `CRagEffect::TexturedFourRing` with its kind 0 or 1 (`PP_RECT_UP2`), of `alpha_down.tga`: two walls round
/// a square, one 0.4 outside the other, and two lights, each a stretch 26.6 long, that run round a square 0.8
/// outside the first, half the way round apart. Of the kind 0 the walls are 12.5 and 12.9 from the middle, 30
/// and 31 tall, at an alpha of 65, and the lights 32 and 33 tall; of the kind 1 everything is 1.5 further
/// out and 10 lower, the walls at an alpha of 32, and its lights begin 26.6 further along than the
/// others. The lights show at an alpha of 15. All sink after some 2400 ticks.
fn four_ring(kind: i32, course: &Course) -> Prim {
    let mut ring = of_parts(Tei::RectUp2, "alpha_down.tga", true, course);
    ring.size = 0.0;
    let (tall, out) = if kind == 0 { (30.0, 12.5) } else { (20.0, 14.0) };
    ring.parts.extend((0..4).map(|index| {
        let mut part = Part { tall: index as f32 + tall, ..Part::default() };
        let wall = index < 2;
        (part.alpha, part.alpha_top) = match (wall, kind) {
            (true, 0) => (65, 2415),
            (true, _) => (32, 2385),
            (false, _) => (15, 2415),
        };
        let half = if wall { index as f32 * 0.4 + out } else { out + 0.8 };
        (part.heights[0], part.heights[1], part.heights[4]) = (half, half, kind as f32);
        if !wall {
            (part.out, part.heights[3]) = ((index - 2) as f32 * 53.2 + if kind == 0 { 0.0 } else { 26.6 }, 26.6);
        }
        part
    }));
    ring
}

/// `CRagEffect::TexturedSpin`: one small box of `white02.bmp` (`PP_HERMODE`), 1.6 tall and as far from its
/// middle to its corners, 12 over the cell, its turn and its breath begun anywhere, in a blue with up to 30
/// of red and green by lot.
fn turning_box(course: &Course) -> Prim {
    let mut cube = of_parts(Tei::Hermode, "white02.bmp", true, course);
    cube.size = 5.0;
    let mut part = Part { out: 1.6, rise: lot(360) as i32, turned: lot(360) as i32, ..Part::default() };
    (part.heights[0], part.flags[0]) = (-12.0, lot(31) as i8);
    cube.parts.push(part);
    cube
}

/// `EF_SLIM`, `EF_SLIM2` and `EF_SLIM3` in `CRagEffect::OnProcess3` (`which` of 0, 1 and 2): the potion that
/// comes down (`TexturedRain` with its kinds 1, 2 and 3: `blows::falling`) and, 16 ticks later, a square of
/// `bbbb.bmp` that widens round the place (`TexturedLevelRing` with its levels 1, 2 and 0): red, which the
/// client draws twice over, yellow and white.
fn potion(which: usize, course: &Course) -> Vec<Prim> {
    let ring = || square("bbbb.bmp", [1, 2, 0][which], 16, course.lasts);
    let mut prims = vec![blows::falling(POTIONS[which], which as i8 + 1, course), ring()];
    if which == 0 {
        prims.push(ring());
    }
    prims
}

