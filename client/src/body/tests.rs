//! The tests of what effects and states make of a body (`body.rs`).

use super::*;

fn quads(body: &Body, character: bool) -> Vec<Quad> {
    body.quads(character, 100.0, &mut |_| 0)
}

#[test]
fn a_body_nothing_touches_is_its_picture_as_it_is() {
    assert_eq!(quads(&Body::default(), true), vec![Quad::PLAIN]);
    assert!(Quad::PLAIN.plain());
}

#[test]
fn the_colour_of_an_effect_fades_and_that_of_a_state_stays() {
    let mut body = Body::default();
    let mut touch = Touch::default();
    touch.colour(255, 100, 100);
    body.touch(&touch, true);
    assert_eq!(quads(&body, true)[0].colour, [255, 100, 100, 255]);
    body.pass(1);
    assert_eq!(quads(&body, true)[0].colour, [255, 108, 108, 255]);
    body.pass(100);
    assert_eq!(quads(&body, true)[0].colour, [255; 4]);
    // A state's colour is locked; an effect writes over it, and what it wrote stays.
    body.stated([0x40; 3]);
    body.pass(50);
    assert_eq!(quads(&body, true)[0].colour, [0x40, 0x40, 0x40, 255]);
    body.touch(&touch, true);
    body.pass(50);
    assert_eq!(quads(&body, true)[0].colour, [255, 100, 100, 255]);
    body.stated(WHITE);
    assert_eq!(quads(&body, true)[0].colour, [255; 4]);
}

#[test]
fn a_colour_is_written_only_at_a_tick_of_the_effect() {
    let mut body = Body::default();
    let mut touch = Touch::default();
    touch.flashes(0, [0, 0, 255]);
    body.touch(&touch, false);
    assert_eq!(quads(&body, true)[0].colour, [255; 4]);
    assert!(quads(&body, true)[0].adds);
    touch = Touch::default();
    touch.flashes(1, [0, 0, 255]);
    assert_eq!((touch.flags, touch.colour), (0, Some(WHITE)));
}

#[test]
fn flags_hold_until_they_are_cleared() {
    let mut body = Body::default();
    body.touch(&Touch { flags: flag::ADDS, ..Touch::default() }, true);
    assert!(quads(&body, false)[0].adds);
    body.clear();
    assert_eq!(quads(&body, false), vec![Quad::PLAIN]);
}

#[test]
fn a_turn_is_taken_in_eighths() {
    let mut body = Body::default();
    body.touch(&Touch { turn: 30.0, ..Touch::default() }, true);
    assert_eq!(body.turned(), 0);
    body.touch(&Touch { turn: 30.0, ..Touch::default() }, true);
    assert_eq!(body.turned(), 1);
    body.touch(&Touch { turn: 30.0, ..Touch::default() }, false);
    assert_eq!(body.turned(), 0);
    body.touch(&Touch { turn: 120.0, ..Touch::default() }, true);
    assert_eq!(body.turned(), 3);
}

#[test]
fn it_shrinks_to_half_and_grows_to_one_and_a_half() {
    let mut body = Body::default();
    body.touch(&Touch { flags: flag::SHRINKS, shrink_phase: Some(0), ..Touch::default() }, true);
    assert!((body.scale() - 1.0).abs() < 1e-6);
    body.touch(&Touch { flags: flag::SHRINKS, shrink_phase: Some(45), ..Touch::default() }, true);
    assert!((body.scale() - 0.5).abs() < 1e-6);
    body.clear();
    body.touch(&Touch { flags: flag::GROWS, grow_phase: Some(45), ..Touch::default() }, true);
    assert!((body.scale() - 1.5).abs() < 1e-6);
    assert!((quads(&body, true)[0].scale - 1.5).abs() < 1e-6);
}

#[test]
fn the_outline_of_a_character_is_two_copies_and_of_a_monster_one() {
    let mut body = Body::default();
    body.touch(&Touch { flags: flag::OUTLINE, ..Touch::default() }, true);
    let drawn = quads(&body, true);
    assert_eq!(drawn.len(), 3);
    assert_eq!((drawn[0].edges, drawn[0].colour, drawn[0].adds), ([-3.0, -3.0, 3.0, 3.0], [255, 255, 255, 100], true));
    assert_eq!((drawn[1].edges, drawn[1].colour), ([-5.0, -5.0, 5.0, 5.0], [255, 255, 255, 120]));
    assert_eq!(drawn[2], Quad::PLAIN);
    body.pass(90);
    assert_eq!(quads(&body, true)[0].edges, [-4.0, -4.0, 4.0, 4.0]);
    assert_eq!(quads(&body, false).len(), 2);
    assert_eq!(quads(&body, false)[0].edges, [-6.5, -6.5, 6.5, 6.5]);
}

#[test]
fn it_is_thrown_up_pressed_and_spun_by_the_count() {
    let mut body = Body::default();
    let mut touch = Touch::default();
    touch.phased(flag::KICKED, 18);
    body.touch(&touch, true);
    assert!((quads(&body, true)[0].of_height[0] + 0.3).abs() < 1e-6);
    body.clear();
    touch = Touch::default();
    touch.phased(flag::PRESSED, 2);
    body.touch(&touch, true);
    assert!((quads(&body, true)[0].of_height[0] - 0.4).abs() < 1e-6);
    assert_eq!(quads(&body, true)[0].of_height[1], 0.0);
    touch.phased(flag::PRESSED, 10);
    body.touch(&touch, true);
    assert!((body.quads(true, 100.0, &mut |_| 5)[0].of_height[0] - 0.65).abs() < 1e-6);
    body.clear();
    touch = Touch::default();
    touch.phased(flag::SPUN, 18);
    body.touch(&touch, true);
    assert!((quads(&body, true)[0].turn - 180.0).abs() < 1e-3);
    for still in [0, 36, 37] {
        touch.phased(flag::SPUN, still);
        body.touch(&touch, true);
        assert!(quads(&body, true)[0].turn.abs() < 1e-3);
    }
    touch.phased(flag::SPUN, 9);
    body.touch(&touch, true);
    assert!((quads(&body, true)[0].turn - (180.0 - (135.0f32).to_radians().sin() * 180.0)).abs() < 1e-3);
}

#[test]
fn the_lights_over_it_come_and_go() {
    let mut body = Body::default();
    let mut touch = Touch::default();
    touch.tinted(flag::BLUE, 4);
    body.touch(&touch, true);
    let drawn = quads(&body, true);
    assert_eq!(drawn.len(), 3);
    assert_eq!((drawn[1].colour, drawn[1].adds, drawn[2]), ([5, 5, 255, 60], true, drawn[1]));
    touch.tinted(flag::RED, 51);
    body.touch(&touch, true);
    assert_eq!(quads(&body, true).len(), 1);
    body.clear();
    touch = Touch::default();
    touch.phased(flag::FLASH, 7);
    body.touch(&touch, true);
    assert_eq!(quads(&body, false)[1].colour, [255; 4]);
    touch.phased(flag::FLASH, 12);
    body.touch(&touch, true);
    assert_eq!(quads(&body, false).len(), 1);
}

#[test]
fn the_copies_of_a_blur_fly_apart_in_front() {
    let mut body = Body::default();
    let mut touch = Touch::default();
    touch.phased(flag::BLUR, 10);
    body.touch(&touch, true);
    let drawn = quads(&body, true);
    assert_eq!(drawn.len(), 5);
    assert_eq!(drawn[0], Quad::PLAIN);
    assert_eq!((drawn[1].edges, drawn[1].colour, drawn[1].adds), ([40.0, 0.0, 40.0, 0.0], [255, 255, 255, 100], false));
    assert_eq!(drawn[4].edges, [0.0, -40.0, 0.0, -40.0]);
    assert_eq!(quads(&body, false)[1].edges, [50.0, 0.0, 50.0, 0.0]);
}

#[test]
fn a_character_comes_down_into_sight() {
    let mut body = Body::default();
    let mut touch = Touch::default();
    touch.tinted(flag::LANDS, 15);
    touch.flags |= flag::OUTLINE;
    body.touch(&touch, true);
    let drawn = quads(&body, true);
    assert_eq!(drawn.len(), 1);
    assert_eq!((drawn[0].edges, drawn[0].colour), ([0.0, -200.0, 0.0, -200.0], [255, 255, 255, 85]));
}

#[test]
fn the_shadows_of_a_monster_widen_by_the_count() {
    let mut body = Body::default();
    let mut touch = Touch::default();
    touch.phased(flag::SHADOWS, 100);
    body.touch(&touch, true);
    let drawn = quads(&body, false);
    assert_eq!(drawn.len(), 11);
    assert_eq!((drawn[0].edges, drawn[0].colour), ([-3.0, -3.0, 3.0, 3.0], [255, 255, 255, 45]));
    assert_eq!(drawn[9].colour[3], 0);
    body.touch(&Touch { outline_alpha: Some(77), ..Touch::default() }, true);
    let drawn = quads(&body, true);
    assert_eq!((drawn.len(), drawn[0].colour), (3, [0x37, 0x37, 0xff, 77]));
    assert!((drawn[1].scale - 1.02).abs() < 1e-6);
}

#[test]
fn a_weapon_is_drawn_by_itself_with_its_light_added() {
    let mut body = Body::default();
    assert_eq!(body.weapon(), None);
    // The flag 2: the weapon alone over the body without it, at every drawing.
    body.touch(&Touch { flags: flag::WEAPON_ADDS, ..Touch::default() }, true);
    assert_eq!(body.weapon(), Some((Quad { adds: true, ..Quad::PLAIN }, true)));
    // The flag 4: a second time over the body as it is, at every other drawing.
    body.clear();
    body.touch(&Touch { flags: flag::WEAPON_TWICE, ..Touch::default() }, true);
    assert_eq!(body.weapon(), Some((Quad { adds: true, ..Quad::PLAIN }, false)));
    body.pass(1);
    assert_eq!(body.weapon(), None);
    body.pass(1);
    assert!(body.weapon().is_some());
    // A character's own picture is not changed by either.
    assert_eq!(quads(&body, true), [Quad::PLAIN]);
}

#[test]
fn a_monster_with_the_weapons_flag_is_blue_with_two_copies() {
    let mut body = Body::default();
    body.touch(&Touch { flags: flag::WEAPON_ADDS, outline_alpha: Some(90), ..Touch::default() }, true);
    let drawn = quads(&body, false);
    assert_eq!(drawn.len(), 3);
    assert_eq!((drawn[0].edges, drawn[0].colour, drawn[0].adds), ([-1.0, -1.0, 1.0, 1.0], [0x37, 0x37, 0xff, 90], true));
    assert_eq!((drawn[1].edges, drawn[2].colour, drawn[2].adds), ([-2.0, -2.0, 2.0, 2.0], [0x37, 0x37, 0xff, 100], false));
}

#[test]
fn one_who_runs_leaves_a_footprint_every_seventh_pass() {
    let mut body = Body::default();
    body.touch(&Touch { flags: flag::RUNS, run_tick: Some(20), ..Touch::default() }, true);
    assert!(body.runs());
    // East a step a pass: the left foot at the seventh pass, the right at the fourteenth, each turned 135
    // degrees on from the way gone.
    let prints: Vec<_> = (0..28).map(|pass| body.stepped([pass as f32, 0.0])).collect();
    assert_eq!(prints.iter().flatten().count(), 4);
    assert_eq!((prints[6], prints[13]), (Some(Print { left: true, turned: 135 }), Some(Print { left: false, turned: 135 })));
    assert_eq!(prints[20], Some(Print { left: true, turned: 135 }));
    // North is a quarter on, the west a half, the south three quarters.
    for (way, turned) in [((0.0, 1.0), 225), ((-1.0, 0.0), 315), ((0.0, -1.0), 45)] {
        let mut body = Body::default();
        body.touch(&Touch { flags: flag::RUNS, run_tick: Some(8), ..Touch::default() }, true);
        let prints: Vec<_> = (0..7).map(|pass| body.stepped([way.0 * pass as f32, way.1 * pass as f32])).collect();
        assert_eq!(prints[6], Some(Print { left: true, turned }));
    }
}

#[test]
fn no_footprint_of_one_standing_or_of_an_effect_just_begun() {
    let mut still = Body::default();
    still.touch(&Touch { flags: flag::RUNS, run_tick: Some(20), ..Touch::default() }, true);
    assert_eq!((0..14).filter_map(|_| still.stepped([3.0, 4.0])).count(), 0);
    let mut early = Body::default();
    early.touch(&Touch { flags: flag::RUNS, run_tick: Some(7), ..Touch::default() }, true);
    assert_eq!((0..14).filter_map(|pass| early.stepped([pass as f32, 0.0])).count(), 0);
}

#[test]
fn an_effect_holds_a_body_in_one_picture_for_as_long_as_it_says_so() {
    let mut body = Body::default();
    body.touch(&Touch { held: Some((96, 5)), ..Touch::default() }, false);
    assert_eq!(body.held(), Some((96, 5)));
    body.clear();
    assert_eq!(body.held(), None);
}

#[test]
fn the_colour_its_states_gave_it_is_kept_beside_what_effects_write() {
    let mut body = Body::default();
    assert_eq!(body.state_colour(), WHITE);
    body.stated([0x40; 3]);
    // An effect writes over the colour it is drawn in, and one may lock its own: the states' stays.
    body.touch(&Touch { colour: Some([1, 2, 3]), ..Touch::default() }, true);
    body.locked([255, 128, 255]);
    assert_eq!(body.state_colour(), [0x40; 3]);
    body.pass(100);
    let drawn = body.quads(true, 100.0, &mut |_| 0);
    assert_eq!(drawn.last().map(|quad| quad.colour), Some([255, 128, 255, 255]));
}
