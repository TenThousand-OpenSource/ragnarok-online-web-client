//! The tests of the effects a state begins and ends (`state_effects.rs`, `own.rs`, `seen.rs`).

use super::*;

#[test]
fn the_blur_is_told_its_colour_by_the_state() {
    assert_eq!((changed(2, true).begins, changed(2, false)), (&[EF_MAKEBLUR][..], Change { ends: &[EF_MAKEBLUR], ..Change::default() }));
    assert_eq!(changed(86, true).begins, &[EF_GUMGANG, EF_MAKEBLUR][..]);
    assert_eq!((blur(2), blur(68), blur(161)), (Some([200.0, 200.0, 0.0]), Some([200.0, 200.0, 0.0]), Some([200.0, 200.0, 0.0])));
    assert_eq!((blur(25), blur(31), blur(86), blur(87)), (Some([250.0, 150.0, 150.0]), Some([180.0, 180.0, 250.0]), Some([-1.0; 3]), None));
}

#[test]
fn a_state_going_on_ends_its_effect_and_begins_it_anew() {
    assert_eq!(changed(107, true), Change { ends: &[EF_REDBODY], begins: &[EF_REDBODY], ..Change::default() });
    assert_eq!(changed(107, false), Change { ends: &[EF_REDBODY], ..Change::default() });
    assert_eq!(changed(166, true).begins, &[EF_DOUBLEGUMGANG, EF_REDLIGHTBODY][..]);
    // Aura Blade's walls and the light of the weapon; Mystical Amplification's weapon; the two lights.
    assert_eq!(changed(103, true).begins, &[EF_AURABLADE, EF_AURABLADE2][..]);
    assert_eq!((changed(113, true).begins, changed(113, false).ends), (&[EF_LIGHTBLADE][..], &[EF_LIGHTBLADE][..]));
    assert_eq!(changed(350, true).begins, &[EF_CHOOKGI_N][..]);
    assert_eq!(changed(730, true).begins.len(), 3);
}

#[test]
fn some_states_only_end_what_a_skill_began() {
    assert_eq!(changed(95, true), Change { ends: &[EF_NPC_STOP], ..Change::default() });
    assert_eq!(changed(149, true), Change::default());
    assert_eq!(changed(149, false).ends, &[EF_ASURABODY][..]);
    assert_eq!(changed(121, false).ends, &[EF_LINELINK3, EF_PINKBODY][..]);
}

#[test]
fn running_ends_with_a_stop_and_a_wall_shakes_the_runner() {
    // Whichever way the state goes: the footprints end, and what follows them begins if there were any.
    for on in [true, false] {
        assert_eq!(changed(133, on), Change { ends: &[EF_RUN], after: Some((EF_RUN, EF_STOPRUN)), ..Change::default() });
        assert_eq!((changed(134, on).begins, changed(134, on).sound), (&[EF_QUAKEBODY][..], "effect/t_벽튕김.wav"));
    }
    // The skill shows the footprints, or the blades of a stop while what follows the footprints runs.
    assert!(begins(EF_RUN, |_| false) && !begins(EF_STOPEFFECT, |_| false));
    assert!(!begins(EF_RUN, |effect| effect == EF_STOPRUN) && begins(EF_STOPEFFECT, |effect| effect == EF_STOPRUN));
    assert!(!begins(EF_RUN, |effect| effect == EF_RUN));
    assert!(begins(EF_CHOOKGI_N, |_| false) && !begins(EF_CHOOKGI_N, |effect| effect == EF_CHOOKGI_N));
    assert!(begins(EF_REDBODY, |_| true));
}

#[test]
fn any_other_state_has_none() {
    assert_eq!(changed(10, true), Change::default());
}

#[test]
fn one_who_comes_into_sight_shows_the_states_of_its_third_option() {
    // A player with Fury and Steel Body on: each effect unless it runs already, the sound of Steel Body
    // going by its body.
    let player = seen(0x18, false, true);
    assert_eq!(player.begins, [(EF_GUMGANG, EF_GUMGANG), (EF_STEELBODY, EF_STEELBODY), (EF_SU_STOOP, EF_STEELBODY)]);
    assert!(player.ends.is_empty());
    assert_eq!(seen(0x140, false, true).begins, [(EF_AURABLADE2, EF_AURABLADE2), (EF_LIGHTBLADE, EF_LIGHTBLADE)]);
    assert_eq!(seen(0x800, false, true).begins, [(EF_ASSUMPTIO, EF_ASSUMPTIO)]);
    // One who hides shows none of them.
    assert_eq!(seen(0x18, true, true), Seen { ends: vec![EF_GUMGANG, EF_STEELBODY, EF_SU_STOOP, EF_DOUBLEGUMGANG], begins: Vec::new() });
    // A monster's bits are others, and one not set ends its effect.
    let monster = seen(0x6, false, false);
    assert_eq!(monster.begins, [(EF_BABYBODY, EF_BABYBODY), (EF_ASURABODY_MONSTER, EF_ASURABODY_MONSTER)]);
    assert_eq!(monster.ends, [EF_STEELBODY, EF_SU_STOOP, EF_ASSUMPTIO]);
}

#[test]
fn the_players_own_states_launch_their_effects_anew_on_arrival() {
    assert_eq!(reapplied(86), &[EF_GUMGANG][..]);
    assert_eq!((reapplied(2), reapplied(188)), (&[EF_MAKEBLUR][..], &[EF_MAKEBLUR][..]));
    assert_eq!(reapplied(87), &[EF_STEELBODY, EF_SU_STOOP][..]);
    // Aura Blade: the light of the weapon alone, not the walls the state began with.
    assert_eq!((reapplied(103), changed(103, true).begins), (&[EF_AURABLADE2][..], &[EF_AURABLADE, EF_AURABLADE2][..]));
    assert_eq!((reapplied(166), reapplied(232)), (&[EF_DOUBLEGUMGANG, EF_REDLIGHTBODY][..], &[EF_DOUBLEGUMGANG, EF_REDLIGHTBODY][..]));
    // A Soul Link's body, which the state itself does not begin; Dark Crow three times over.
    assert_eq!((reapplied(149), changed(149, true).begins), (&[EF_ASURABODY][..], &[][..]));
    assert_eq!(reapplied(730).len(), 3);
    // A state that is only ended as it changes, or one not drawn, launches nothing.
    assert_eq!((reapplied(95), reapplied(133), reapplied(10)), (&[][..], &[][..], &[][..]));
}
