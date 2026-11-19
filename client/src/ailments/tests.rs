//! The tests of what the body state and the health state do (`ailments.rs`, with its marks and veils).

use super::*;

fn told(before: Ailing, colour: [u8; 3], now: Ailing, news: bool, own: bool) -> Change {
    change(before, colour, false, now, news, own)
}

#[test]
fn poison_colours_and_sounds_and_veils_the_players_own() {
    let poisoned = Ailing { body: 0, health: POISON };
    let other = told(Ailing::default(), WHITE, poisoned, true, false);
    assert_eq!((other.colour, other.sounds.as_slice(), other.begins.as_slice()), ([0xff, 0x80, 0xff], &["_poison.wav"][..], &[][..]));
    let own = told(Ailing::default(), WHITE, poisoned, true, true);
    assert_eq!(own.begins, vec![EF_POISON]);
    // One who walks into sight poisoned is coloured without a sound.
    let seen = told(Ailing::default(), WHITE, poisoned, false, false);
    assert_eq!((seen.colour, seen.sounds.len()), ([0xff, 0x80, 0xff], 0));
    // Cured: white again, and the player's own veil goes.
    let cured = told(poisoned, other.colour, Ailing::default(), true, true);
    assert_eq!((cured.colour, cured.ends.as_slice(), cured.sounds.len()), (WHITE, &[EF_POISON][..], 0));
    assert_eq!(told(Ailing::default(), WHITE, Ailing { body: 0, health: DEADLY_POISON }, true, false).colour, [0xff, 0x32, 0xff]);
}

#[test]
fn curse_colours_over_poison() {
    let both = told(Ailing::default(), WHITE, Ailing { body: 0, health: POISON | CURSE }, true, false);
    assert_eq!(both.colour, [0xc8, 0x32, 0x32]);
    assert_eq!(both.sounds, vec!["_poison.wav", "_curse.wav"]);
    assert_eq!(marks(Ailing { body: 0, health: CURSE | SILENCE }), vec![Mark::Curse, Mark::Silence]);
}

#[test]
fn stone_and_ice_hold_the_motion_and_colour() {
    let stone = change(Ailing::default(), WHITE, false, Ailing { body: STONE, health: 0 }, true, false);
    assert_eq!((stone.colour, stone.still, stone.sounds.len()), ([0x40; 3], true, 0));
    let waits = told(Ailing::default(), WHITE, Ailing { body: STONEWAIT, health: 0 }, true, false);
    assert_eq!((waits.colour, waits.still), ([0x80; 3], false));
    let frozen = told(Ailing::default(), WHITE, Ailing { body: FREEZE, health: 0 }, true, false);
    assert_eq!((frozen.colour, frozen.still, frozen.sounds.as_slice()), ([0x00, 0x80, 0xff], true, &["_stonecurse.wav"][..]));
    assert_eq!(marks(Ailing { body: FREEZE, health: 0 }), vec![Mark::Ice]);
    // The ice breaks as it goes.
    let thawed = change(Ailing { body: FREEZE, health: 0 }, frozen.colour, true, Ailing::default(), true, false);
    assert_eq!((thawed.colour, thawed.still, thawed.ice_breaks, thawed.sounds.as_slice()), (WHITE, false, true, &["_frozen_explosion.wav"][..]));
    let crumbled = change(Ailing { body: STONE, health: 0 }, stone.colour, true, Ailing::default(), true, false);
    assert_eq!((crumbled.colour, crumbled.still, crumbled.sounds.as_slice()), (WHITE, false, &["_stone_explosion.wav"][..]));
}

#[test]
fn a_colour_stays_until_nothing_colours() {
    // Poisoned while stone, then cured: the body state did not change, so nothing colours it stone again.
    let both = Ailing { body: STONE, health: POISON };
    let poisoned = change(Ailing { body: STONE, health: 0 }, [0x40; 3], true, both, true, false);
    assert_eq!(poisoned.colour, [0xff, 0x80, 0xff]);
    let cured = change(both, poisoned.colour, true, Ailing { body: STONE, health: 0 }, true, false);
    assert_eq!((cured.colour, cured.still), ([0xff, 0x80, 0xff], true));
}

#[test]
fn stun_and_the_rest_sound_only_as_news() {
    assert_eq!(told(Ailing::default(), WHITE, Ailing { body: STUN, health: 0 }, true, false).sounds, vec!["_stun.wav"]);
    assert!(told(Ailing::default(), WHITE, Ailing { body: STUN, health: 0 }, false, false).sounds.is_empty());
    let many = told(Ailing::default(), WHITE, Ailing { body: 0, health: SILENCE | CONFUSION | BLIND }, true, true);
    assert_eq!((many.sounds.as_slice(), many.begins.as_slice(), many.colour), (&["_silence.wav", "_confusion.wav", "_blind.wav"][..], &[EF_BLIND][..], WHITE));
    let sees = told(Ailing { body: 0, health: BLIND }, WHITE, Ailing::default(), true, true);
    assert_eq!(sees.ends, vec![EF_BLIND]);
    assert_eq!(marks(Ailing { body: SLEEP, health: 0 }), vec![Mark::Sleep]);
}

#[test]
fn the_veils_cover_what_the_client_covers() {
    let poison = poison_veil();
    assert_eq!(poison.len(), 60);
    assert_eq!(poison[0].corners[0], ([-150.0, -90.0], [0.0, 1.0]));
    assert_eq!(poison[59].corners[2], ([150.0, 90.0], [1.0, 0.0]));
    assert_eq!((poison_alpha(0), poison_alpha(3), poison_alpha(400)), (0.0, 15.0, 50.0));
    let blind = blind_veil();
    assert_eq!(blind.iter().filter(|pane| pane.picture == "fullb.tga").count(), 4);
    // Each inner pane has the picture's left bottom in the middle.
    assert!(blind[..4].iter().all(|pane| pane.corners[0] == ([0.0, 0.0], [0.0, 1.0])));
    assert_eq!(blind[4].corners[2].0, [165.0, 165.0]);
    assert_eq!((blind_alpha(7), blind_alpha(999)), (7.0, 255.0));
    assert!(TINTS.contains(&[0x00, 0x80, 0xff]) && TINTS[0] == WHITE);
}
