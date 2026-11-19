//! The weather: effects that fall over the whole view round the player's character, which the client launches
//! on that character and lets live until the map is left (`CWeatherEffect`, Source/Contents/Map/MapSkyMgr.cpp;
//! a server's map flags send the same effects by number):
//!
//! - `EF_SNOW` (162): two flakes a tick, each 320 ticks falling from 100 over the ground somewhere within 300
//!   of the player (`CRagEffect::Snow`);
//! - `EF_SAKURA` (163): 150 petals, one every other tick, that fall wandering from 200 over the player within
//!   150 either way and begin anew from there once they are down (`CRagEffect::Sakura`, `PrimSakura`);
//! - `EF_MAPLE` (333): 100 leaves, the same way, slower (`CRagEffect::Maple`);
//! - the clouds and the stars of a sky (`CRagEffect::SkyCloud`, the kind 33 of the primitives): `EF_CLOUD2`
//!   (230: 60 lots of four clouds a little under the height nothing), `EF_CLOUD3` (233: 40 lots, lower),
//!   `EF_CLOUD5` (516: 80 lots that drift eastwards), `EF_CLOUD6` (592: 80 lots of dark clouds) and
//!   `EF_CLOUD9` (945: 65 lots of stars, far under the map).
//!
//! What falls stays in the world where it is as the player walks on: a flake where it was launched, a petal
//! until it is down, and then it comes again round where the player is by then.

use super::launched::{of_parts, part};
use super::*;
use crate::prim::{PUT_ANEW, Tei};

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&CLOUDS, &STARS], sprites: &[&SPRITES] };

/// The pictures of the clouds, one by lot for each lot of four, and the six a star runs through.
const CLOUDS: [&str; 3] = ["cloud4.tga", "cloud1.tga", "cloud2.tga"];
const STARS: [&str; 6] = ["star01.bmp", "star02.bmp", "star03.bmp", "star04.bmp", "star05.bmp", "star06.bmp"];

const SNOW: &str = "이팩트/ef_snow";
const SAKURA: &str = "이팩트/sakura01";
const MAPLE: &str = "이팩트/단풍";

/// The sprites the effects here show, among the client's sprites.
const SPRITES: [&str; 3] = [SNOW, SAKURA, MAPLE];

/// The client's `EffectLifetime` of all three: as good as for ever.
const FOR_EVER: i32 = 99_999_999;

fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        162 | 163 | 333 => (FOR_EVER, ""),
        230 | 233 | 516 | 592 | 945 => (FOR_EVER, ""),
        _ => return None,
    })
}

/// The kind of `SkyCloud` an effect of a sky launches, and how many lots of it; nothing for any other effect.
fn sky(effect: u32) -> Option<(i32, usize)> {
    Some(match effect {
        230 => (1, 60),
        233 => (2, 40),
        516 => (4, 80),
        592 => (5, 80),
        945 => (9, 65),
        _ => return None,
    })
}

/// One tick of an effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    match effect {
        162 if course.lasts - course.tick > 160 => prims.extend([flake(course), flake(course)]),
        163 | 333 => {
            let leaves = effect == 333;
            if course.tick % 2 == 0 && course.tick < if leaves { 200 } else { 300 } {
                prims.push(petal(leaves, course));
            }
            for petal in prims.iter_mut().filter(|prim| matches!(prim.kind, Shape::Petal { .. })) {
                // Down under the place the effect is at now, it begins anew over it.
                let left = petal.left.unwrap_or_default();
                if left[1] + petal.moved[1] - course.walked[1] < 0.0 {
                    petal.moved = std::array::from_fn(|axis| over_the_view()[axis] + course.walked[axis] - left[axis]);
                }
            }
        }
        _ => {
            let Some((kind, lots)) = sky(effect) else { return };
            if course.tick == 0 {
                prims.extend((0..lots).map(|_| clouds(kind, course)));
            }
            // A cloud that has gone comes again somewhere round where the effect is by now.
            for prim in prims.iter_mut().filter(|prim| prim.kind == Shape::Tei(Tei::Sky)) {
                let left = prim.left.unwrap_or_default();
                for cloud in prim.parts.iter_mut().filter(|cloud| cloud.flags[PUT_ANEW] != 0) {
                    cloud.flags[PUT_ANEW] = 0;
                    let (east, north) = round_the_view(kind);
                    (cloud.at[0][0], cloud.at[0][2]) = (east + course.walked[0] - left[0], north + course.walked[2] - left[2]);
                }
            }
        }
    }
}

/// Where a cloud of a kind is put, east and north of the effect's place: the kinds 2 and 5 up to 150 either
/// way, the others 25 to 224 either way.
fn round_the_view(kind: i32) -> (f32, f32) {
    if matches!(kind, 2 | 5) {
        return (lot(300) - 150.0, lot(300) - 150.0);
    }
    let either = |far: f32| if lot(2) == 0.0 { -far } else { far };
    (either(lot(200) + 25.0), either(lot(200) + 25.0))
}

/// `CRagEffect::SkyCloud`: a lot of four squares facing the screen (`Tei::Sky`), each 30 to 49 from its middle
/// to a corner, at a height of the map's own counted from nothing (the kind 2 up to 10 under it, the kind 5
/// 20 to 29, the stars 80 to 89, the others 40 to 49), that stands 300 to 499 ticks before it goes. The clouds
/// are white and mixed in, the kind 5 dark red; the stars' light is added.
fn clouds(kind: i32, course: &Course) -> Prim {
    let mut prim = of_parts(Tei::Sky, CLOUDS[lot(3) as usize], false, course);
    if kind == 9 {
        prim.pictures = &STARS;
    }
    let under = match kind {
        2 => 0.0,
        5 => 20.0,
        9 => 80.0,
        _ => 40.0,
    };
    prim.parts.extend((0..4).map(|_| {
        let mut cloud = part();
        (cloud.round, cloud.turned, cloud.heights[7]) = (45, lot(200) as i32 + 300, kind as f32);
        // (A part's place is measured from the effect's, the client's y pointing down: the map's own height
        // is that less how high the effect's place is.)
        let down = lot(10) + under + course.ground;
        let (east, north) = round_the_view(kind);
        cloud.at[0] = [east, down, north];
        cloud.heights[..4].copy_from_slice(&[lot(360), lot(360), lot(360), lot(360)]);
        (cloud.out, cloud.heights[8]) = (lot(20) + 30.0, lot(360));
        cloud.flags[2] = match kind {
            5 => 14,
            9 => 23,
            _ => 0,
        };
        cloud
    }));
    prim
}

/// Where a petal begins, from the effect's place: 200 over it, up to 150 east or west and north or south.
fn over_the_view() -> [f32; 3] {
    [lot(300) - 150.0, 200.0, lot(300) - 150.0]
}

/// `CRagEffect::Snow`: a flake up to 300 from the player any way round, 100 over the height the client
/// reckons there (the ground's height counted twice: once as the player's own, once more as its function adds
/// it), that falls half a unit a tick for 320 ticks; half to four fifths of its sprite's size; it comes up over
/// 10 ticks and fades over its last fifth.
fn flake(course: &Course) -> Prim {
    let mut flake = Prim::new(Shape::Particle3d, SNOW);
    flake.duration = 320;
    flake.left = Some(course.walked);
    let (round, away) = (lot(360), lot(300) - 300.0);
    (flake.put, flake.deep) = ((sine(round) * away, -(course.ground + 100.0)), cosine(round) * away);
    // (Its turn, a quarter about the line across, and its speed of -0.5: straight down.)
    (flake.latitude, flake.speed) = (90.0, -0.5);
    flake.size = (lot(15) + 25.0) / 50.0;
    (flake.alpha, flake.alpha_speed) = (0.0, flake.max_alpha / 10.0);
    flake.fade_out = flake.duration - flake.duration / 5;
    flake
}

/// `CRagEffect::Sakura` and `Maple`: a petal (one of three by lot, a little over a fifth of its sprite's size,
/// falling 0.2 or 0.3 a tick) or a leaf (a little over a third of its size, falling 0.06 to 0.15 a tick and
/// drifting half as far), at four fifths of full alpha, for as long as the effect lives.
fn petal(leaf: bool, course: &Course) -> Prim {
    let mut petal = Prim::new(Shape::Petal { slow: leaf }, if leaf { MAPLE } else { SAKURA });
    (petal.duration, petal.alpha) = (course.lasts, 200.0);
    petal.left = Some(course.walked);
    let mut wander = part();
    wander.tall = if leaf { (lot(4) + 2.0) * 0.03 } else { (lot(2) + 2.0) * 0.1 };
    petal.size = (lot(6) + if leaf { 35.0 } else { 22.0 }) / 100.0;
    wander.heights[..4].copy_from_slice(&[lot(360), lot(360), lot(360), lot(360)]);
    petal.moved = over_the_view();
    petal.parts.push(wander);
    petal.action = if leaf { 0 } else { lot(3) as usize };
    petal
}
