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

mod marks;
mod veils;
#[cfg(test)]
mod tests;

pub use marks::{ICE_BREAK_PACE, ICE_BREAKS, Mark, marks};
pub use veils::{BLIND_COLOUR, Pane, blind_alpha, blind_veil, poison_alpha, poison_veil};

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
use crate::body::WHITE;
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
