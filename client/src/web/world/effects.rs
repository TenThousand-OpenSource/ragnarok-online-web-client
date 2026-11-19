//! The effects the client draws by code rather than from `.str` files: the rings of a cast, the mark on its target,
//! a portal's rings, the flash of a blow.

use super::*;

/// How the rings around a caster look, by the client's effect (`EF_BEGINSPELL` and on): the picture, its tint,
/// and whether it adds light. Water, fire and wind are the client's own figures (`QuadRingStackSized` with
/// `RenderTeiRect`'s colours); the others are drawn the same way with their own picture.
pub(super) fn cast_look(effect: u32) -> (&'static str, [f32; 3], bool) {
    match effect {
        54 => ("ring_blue.tga", [195.0 / 255.0, 195.0 / 255.0, 1.0], false),
        55 => ("ring_yellow.tga", [1.0, 175.0 / 255.0, 175.0 / 255.0], true),
        56 => ("magic_green.tga", [1.0, 1.0, 1.0], false),
        57 => ("ring_yellow.tga", [1.0, 1.0, 1.0], false),
        58 => ("ring_white.tga", [1.0, 1.0, 1.0], false),
        59 => ("magic_violet.tga", [1.0, 1.0, 1.0], false),
        _ => ("ring_yellow.tga", [1.0, 175.0 / 255.0, 175.0 / 255.0], true),
    }
}

/// The rings around a caster, `frame` frames into a cast of `frames` (the client's `PP_3DCASTING`: its
/// `UpdateStoneCurse` and `Render3DCasting`). Three arcs turn around the caster, each rising outwards to its own
/// height, inside a tall faint column; all come up over the first quarter turn of a sine and fade over the last
/// 40 frames. Each corner: place, texture coordinates, colour.
pub(super) fn cast_rings(centre: Vector, frame: f32, frames: f32, tint: [f32; 3], corners: &mut Vec<f32>) -> usize {
    // Of each arc: how far round it goes, how high it rises, where it starts, how far out it stands, how steeply
    // it rises, all in degrees and the client's units.
    const ARCS: [(i32, f32, f32, f32, f32); 4] = [(315, 25.0, 0.0, 4.5, 70.0), (315, 22.0, 90.0, 5.0, 57.0), (315, 19.0, 180.0, 5.5, 45.0), (360, 250.0, 0.0, 4.0, 89.0)];
    let mut quads = 0;
    for (index, (round, tall, start, out, steep)) in ARCS.into_iter().enumerate() {
        let column = index == 3;
        let (most, comes, goes) = if column { (60.0, 8.0, 2.0) } else { (180.0, 10.0, 5.0) };
        let alpha = if frame < frames - 40.0 { (comes * frame).min(most) } else { most - goes * (frame - (frames - 40.0)) };
        if alpha <= 0.0 {
            continue;
        }
        let turned = if column { start } else { start + (index as f32 + 3.0) * frame.floor() };
        let risen = frame.min(90.0).to_radians().sin();
        let (step, steep) = ((round / 20) as f32, steep.to_radians());
        let corner = |order: usize, top: bool| {
            let angle = (turned + order as f32 * step).to_radians();
            let height = if top { tall * (order as f32 * 9.0).to_radians().sin() * risen } else { 0.0 };
            let radius = out + steep.cos() * height;
            [centre[0] + angle.cos() * radius, centre[1] + 0.3 + steep.sin() * height, centre[2] + angle.sin() * radius]
        };
        for order in 1..=20usize {
            let (left, right) = ((order - 1) as f32 / 21.0, order as f32 / 21.0);
            for (order, top, u) in [(order - 1, false, left), (order, false, right), (order - 1, true, left), (order - 1, true, left), (order, false, right), (order, true, right)] {
                corners.extend(corner(order, top));
                corners.extend([u, if top { 0.0 } else { 1.0 }]);
                corners.extend([tint[0], tint[1], tint[2], alpha / 255.0]);
            }
            quads += 1;
        }
    }
    quads
}

/// The mark on the one a cast is aimed at (the client's `LockOnMarker`): two copies of its picture lying on the
/// ground, one an eighth of a turn ahead of the other, turning 4.5 degrees a frame and closing in from 30 across
/// to 10 over the first 20 frames.
pub(super) fn lock_on(centre: Vector, frame: f32, corners: &mut Vec<f32>) -> usize {
    let half = (30.0 - frame).max(10.0) / 2.0;
    for ahead in [0.0f32, 45.0] {
        let turned = (ahead + 4.5 * frame).to_radians();
        let corner = |x: f32, z: f32| [centre[0] + (x * turned.cos() - z * turned.sin()) * half, centre[1] + 0.4, centre[2] + (x * turned.sin() + z * turned.cos()) * half];
        for (x, z, u, v) in [(-1.0, -1.0, 0.0, 0.0), (1.0, -1.0, 1.0, 0.0), (-1.0, 1.0, 0.0, 1.0), (-1.0, 1.0, 0.0, 1.0), (1.0, -1.0, 1.0, 0.0), (1.0, 1.0, 1.0, 1.0)] {
            corners.extend(corner(x, z));
            corners.extend([u, v]);
            corners.extend([250.0 / 255.0, 150.0 / 255.0, 150.0 / 255.0, 1.0]);
        }
    }
    2
}

/// The rings of light of a portal standing at a place, at a moment: two sets of four bands opening upwards,
/// each turning its own way. (The original's `AlphaDownRipples` with `ring_blue.tga`; its sparks are not made.)
pub(super) fn rings(centre: Vector, time: f64, corners: &mut Vec<f32>) {
    const SIDES: usize = 20;
    for (size, turn, growth) in [(11.0f32, 0.0f32, 0.0f32), (4.0, 1.0, 0.2)] {
        for (slot, (start, distance, rise)) in [(270.0f32, 2.5f32, 53.0f32), (0.0, 5.0, 60.0), (90.0, 7.5, 55.0), (180.0, 10.0, 50.0)].into_iter().enumerate() {
            let rise = (rise - turn).to_radians();
            let (low, high, height) = (distance + growth, distance + growth + size * rise.cos(), size * rise.sin());
            let way = if slot % 2 == 0 { 1.0 } else { -1.0 };
            let spin = (start + turn).to_radians() + way * (time / 1000.0) as f32 * 1.2;
            let point = |side: usize, radius: f32, up: f32| {
                let angle = spin + side as f32 / SIDES as f32 * std::f32::consts::TAU;
                [centre[0] + radius * angle.cos(), centre[1] + 0.3 + up, centre[2] + radius * angle.sin()]
            };
            for side in 0..SIDES {
                // The picture goes around five times.
                let (left, right) = ((side % 4) as f32 / 4.0, (side % 4 + 1) as f32 / 4.0);
                let quad = [(side, low, 0.0, left, 1.0), (side + 1, low, 0.0, right, 1.0), (side, high, height, left, 0.0), (side, high, height, left, 0.0), (side + 1, low, 0.0, right, 1.0), (side + 1, high, height, right, 0.0)];
                for (side, radius, up, u, v) in quad {
                    corners.extend(point(side, radius, up));
                    corners.extend([u, v]);
                }
            }
        }
    }
}
