//! What someone's body state and health state do to how they are seen and heard: the client's
//! `CGameActor::SetStatusStates` (`Contents/Actors/RenderObject.cpp`), which every actor goes through when the
//! server tells its states (ZC_STATE_CHANGE, with the word that it is news) and when it comes into sight (the
//! packets that show a unit, without that word).
//!
//! The server names the two `opt1` and `opt2`; the client keeps them as `bodyState` and `effectState`. Here they
//! are the body state (one of several: stone, frozen, stunned…) and the health state (bits: poisoned, cursed…),
//! as `net::statuses::body_and_health` reads them.
//!
//! What is here: the colour the whole sprite is drawn in, the sounds of a change, the marks that stay by the
//! one ailing (`CMsgEffect`s the actor makes for itself: `AM_MAKE_ICEEFFECT`, `AM_MAKE_CURSE_EFFECT`…), the
//! effects of the player's own character (`EF_POISON`, `EF_BLIND`) with their two veils over the screen, and
//! whether the motion is held still. `web/world/ailments.rs` keeps it for everyone and draws it.
//!
//! Not here, as the client's side of each was not ported: the burning state's `EF_BURNING` and the imprisoned
//! state's `EF_BOTTOM_BASILICA2`; what `AM_MAKE_ANGELUS_EFFECT` shows (an unseen `CMsgEffect` that sends its
//! owner a message every 34 frames; what that message does was not followed); the sleeper's own state
//! (`ST_SLEEP`, which `AM_MAKE_SLEEP_EFFECT` puts the actor in); the colours the client's "instant" states 3 and
//! 12 to 15 keep (they only stop the sprite from going white again), and `EF_BLACKBODY`, which keeps the
//! poisoned from being coloured.

/// The body states the client tells apart (`e_sc_opt1`).
pub const STONE: u16 = 1;
pub const FREEZE: u16 = 2;
pub const STUN: u16 = 3;
pub const SLEEP: u16 = 4;
pub const STONEWAIT: u16 = 6;

/// The bits of the health state the client looks at.
pub const POISON: u16 = 0x01;
pub const CURSE: u16 = 0x02;
pub const SILENCE: u16 = 0x04;
pub const CONFUSION: u16 = 0x08;
pub const BLIND: u16 = 0x10;
/// The bit 0x80, which colours as poison does, only deeper (rAthena's `OPT2_DPOISON`).
pub const DEADLY_POISON: u16 = 0x80;

/// The client's numbers of the two effects the player's own character gets.
pub const EF_BLIND: u32 = 334;
pub const EF_POISON: u32 = 335;

/// The colours a sprite is drawn in, red, green and blue: its own, and those the states give it. The sprite's
/// own colours are multiplied by it. A state's colour is written to the body of the one in it
/// (`crate::body::Body::stated`), which is where the colour of everyone is kept.
pub const TINTS: [[u8; 3]; 7] = [WHITE, [0xff, 0x80, 0xff], [0xff, 0x32, 0xff], [0xc8, 0x32, 0x32], [0x40, 0x40, 0x40], [0x80, 0x80, 0x80], [0x00, 0x80, 0xff]];
pub use crate::body::WHITE;
const POISONED: [u8; 3] = TINTS[1];
const DEADLY: [u8; 3] = TINTS[2];
const CURSED: [u8; 3] = TINTS[3];
const STONED: [u8; 3] = TINTS[4];
const HARDENING: [u8; 3] = TINTS[5];
const FROZEN: [u8; 3] = TINTS[6];

/// Someone's two states.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ailing {
    pub body: u16,
    pub health: u16,
}

/// A mark that stays by the one ailing for as long as its state holds: a sprite of the client's
/// (`CGameMode::SpawnMsgEffect`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Ice,
    Curse,
    Sleep,
    Stun,
    Silence,
}

impl Mark {
    /// The sprite it is, under the client's folder of sprites, and the first of its actions it shows
    /// (`SpawnMsgEffect`: silence is the eleventh action of the emotions' sprite).
    pub fn sprite(self) -> (&'static str, usize) {
        match self {
            Mark::Ice => ("이팩트/얼음땡", 0),
            Mark::Curse => ("이팩트/status-curse", 0),
            Mark::Sleep => ("이팩트/status-sleep", 0),
            Mark::Stun => ("이팩트/status-stun", 0),
            Mark::Silence => ("이팩트/emotion", 10),
        }
    }

    /// How far over the feet of its owner it is kept (`CMsgEffect::FollowOwner`).
    pub fn above(self) -> f32 {
        match self {
            Mark::Ice => 0.0,
            Mark::Curse => 15.0,
            Mark::Sleep | Mark::Stun | Mark::Silence => 20.0,
        }
    }

    /// How much of it shows: the curse is half there (`AM_SETMASTERGID`: its alpha 0x7F), the rest whole.
    pub fn alpha(self) -> f32 {
        if self == Mark::Curse { 127.0 / 255.0 } else { 1.0 }
    }

    /// Whether its action follows the way its owner faces, one action to each of the eight ways (the curse:
    /// `FollowOwner` adds `Get8Dir` of the owner's angle to its action).
    pub fn faces(self) -> bool {
        self == Mark::Curse
    }
}

/// The sprite of the ice that breaks as a frozen one thaws, and which of its actions it is
/// (`MAKETYPE_ICEBREAK_EFFECT`). It plays once, 1.2 pictures to each of the client's frames of 24 milliseconds
/// (`CMsgEffect::OnProcess`, `MET_ICEBREAKEFFECT`), where its owner stood as it broke.
pub const ICE_BREAKS: (&str, usize) = ("이팩트/얼음땡", 1);
pub const ICE_BREAK_PACE: f64 = 1.2;

/// The marks someone in those states has by them. (The client makes and removes each as a state comes and
/// goes; none is made twice, so what there is follows from the states alone.)
pub fn marks(ailing: Ailing) -> Vec<Mark> {
    let mut marks = Vec::new();
    match ailing.body {
        FREEZE => marks.push(Mark::Ice),
        STUN => marks.push(Mark::Stun),
        SLEEP => marks.push(Mark::Sleep),
        _ => {}
    }
    if ailing.health & CURSE != 0 {
        marks.push(Mark::Curse);
    }
    if ailing.health & SILENCE != 0 {
        marks.push(Mark::Silence);
    }
    marks
}

/// What a telling of someone's states does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// The colour the sprite is drawn in from now on.
    pub colour: [u8; 3],
    /// Whether its motion is held where it is (`isMotionFreezed`).
    pub still: bool,
    /// The sounds to play, among the client's sounds.
    pub sounds: Vec<&'static str>,
    /// The ice around it breaks.
    pub ice_breaks: bool,
    /// The effects to begin on the player's own character, unless it has them already, and those to end.
    pub begins: Vec<u32>,
    pub ends: Vec<u32>,
}

/// `CGameActor::SetStatusStates`, what of it is the body state's and the health state's: someone who was in the
/// states `before`, drawn in `colour` and held `still` or not, is told the states `now`. `news` is the client's
/// `notify`: the server's word that the states changed, not the showing of someone who comes into sight. `own`
/// tells that it is the player's own character.
///
/// The colour is kept from telling to telling, as the client keeps it: a body state colours only as it comes,
/// and the sprite goes white again only when nothing that colours holds (so one poisoned while turned to
/// stone, and then cured, stays in poison's colour until the stone goes too).
///
/// The client plays the sounds of the health state at every telling with news while the bit holds, not only
/// as it comes; so it is here.
pub fn change(before: Ailing, colour: [u8; 3], still: bool, now: Ailing, news: bool, own: bool) -> Change {
    let mut change = Change { colour, still, sounds: Vec::new(), ice_breaks: false, begins: Vec::new(), ends: Vec::new() };
    if before.body != now.body {
        match before.body {
            STONE => change.sounds.push("_stone_explosion.wav"),
            FREEZE => {
                change.ice_breaks = true;
                change.sounds.push("_frozen_explosion.wav");
                (change.still, change.colour) = (false, WHITE);
            }
            _ => {}
        }
        match now.body {
            STONE => (change.still, change.colour) = (true, STONED),
            FREEZE => {
                (change.still, change.colour) = (true, FROZEN);
                // (The client plays the sound of the stone curse as someone freezes, and none as they turn to
                // stone.)
                if news {
                    change.sounds.push("_stonecurse.wav");
                }
            }
            STUN if news => change.sounds.push("_stun.wav"),
            STONEWAIT => change.colour = HARDENING,
            _ => {}
        }
    }
    let poisoned = now.health & (DEADLY_POISON | POISON) != 0;
    if now.health & DEADLY_POISON != 0 {
        change.colour = DEADLY;
    } else if now.health & POISON != 0 {
        change.colour = POISONED;
    } else if own && before.health & (DEADLY_POISON | POISON) != 0 {
        change.ends.push(EF_POISON);
    }
    if poisoned && news {
        change.sounds.push("_poison.wav");
        if own {
            change.begins.push(EF_POISON);
        }
    }
    if now.health & CURSE != 0 {
        change.colour = CURSED;
    }
    for (bit, sound) in [(CURSE, "_curse.wav"), (SILENCE, "_silence.wav"), (CONFUSION, "_confusion.wav"), (BLIND, "_blind.wav")] {
        if now.health & bit != 0 && news {
            change.sounds.push(sound);
        }
    }
    if own && now.health & BLIND != 0 && news {
        change.begins.push(EF_BLIND);
    } else if own && now.health & BLIND == 0 && before.health & BLIND != 0 {
        change.ends.push(EF_BLIND);
    }
    if now.body == 0 {
        change.still = false;
    }
    if now.health & (DEADLY_POISON | CURSE | POISON) == 0 && !matches!(now.body, FREEZE | STONE | STONEWAIT) {
        change.colour = WHITE;
    }
    change
}

/// One pane of a veil over the screen: its picture among the client's pictures of effects, and its four
/// corners in `RenderTeiRect`'s order, each as how far across the screen and down it from the veil's middle
/// (in the world's measure) and where of the picture it shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pane {
    pub picture: &'static str,
    pub corners: [([f32; 2], [f32; 2]); 4],
}

/// A pane between four corners, the picture laid as `RenderTeiRect` lays it: its left bottom on the first
/// corner, then its right bottom, right top and left top.
fn pane(picture: &'static str, corners: [[f32; 2]; 4]) -> Pane {
    let uv = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    Pane { picture, corners: std::array::from_fn(|index| (corners[index], uv[index])) }
}

/// The veil of `EF_POISON` (`CRagEffect::POISON`, drawn by `CEffectPrim::RenderGrid`): sixty panes of
/// `poison_f.bmp`, each 30 a side, ten across and six down, around a place 9 above the feet of the player's
/// own character, facing the screen. White, mixed in by its alpha. (The client holds the panes 10 off their
/// plane, which changes nothing of where they show on a screen they face.)
pub fn poison_veil() -> Vec<Pane> {
    let mut panes = Vec::with_capacity(60);
    for across in (-150..150).step_by(30) {
        for down in (-90..90).step_by(30) {
            let (left, top) = (across as f32, down as f32);
            panes.push(pane("poison_f.bmp", [[left, top], [left + 30.0, top], [left + 30.0, top + 30.0], [left, top + 30.0]]));
        }
    }
    panes
}

/// How much of the poison's veil shows after so many of the effect's ticks, of 255 (`CEffectPrim::PrimPoison`:
/// 5 more a tick until it is past 45).
pub fn poison_alpha(ticks: i32) -> f32 {
    (ticks.max(0) * 5).min(50) as f32
}

/// The veil of `EF_BLIND` (`CRagEffect::BLIND` with no flag, drawn by `CEffectPrim::RenderBlind`): around the
/// place 9 above the feet of the player's own character, four panes of `fullb.tga`, 65 a side, that meet in
/// the middle (the picture's left bottom there in each), and round them four panes of `white02.bmp` 100 wide
/// that close the frame. All are drawn in the colour 10, 10, 10 of 255, mixed in by their alpha.
pub fn blind_veil() -> Vec<Pane> {
    let (near, far) = (65.0, 165.0);
    let mut panes = Vec::with_capacity(8);
    for (across, down) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
        panes.push(pane("fullb.tga", [[0.0, 0.0], [0.0, down * near], [across * near, down * near], [across * near, 0.0]]));
    }
    panes.push(pane("white02.bmp", [[near, -near], [far, -near], [far, far], [near, far]]));
    panes.push(pane("white02.bmp", [[near, near], [near, far], [-far, far], [-far, near]]));
    panes.push(pane("white02.bmp", [[-near, near], [-far, near], [-far, -far], [-near, -far]]));
    panes.push(pane("white02.bmp", [[-near, -near], [-near, -far], [far, -far], [far, -near]]));
    panes
}

/// The colour of the blindness's veil, each of 255.
pub const BLIND_COLOUR: [f32; 3] = [10.0; 3];

/// How much of the blindness's veil shows after so many of the effect's ticks, of 255
/// (`CEffectPrim::PrimFadeInFollow`: one more a tick).
pub fn blind_alpha(ticks: i32) -> f32 {
    ticks.clamp(0, 255) as f32
}

#[cfg(test)]
mod tests {
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
}
