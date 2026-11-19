//! Effects of the client's that are sounds and nothing drawn, each as its arm of `CRagEffect::OnProcess` has
//! it:
//!
//! - `EF_NAPALMVALCAN` (401): Napalm Vulcan, five sounds (the client calls `Hit2` at five ticks of it, and
//!   `Hit2` launches its flares at an effect's first tick only, so nothing is drawn);
//! - `EF_STIN6` (769): five kicks;
//! - `EF_POISONWAV` (771): the two sounds of poison;
//! - `EF_VULCANWAV` (776): the sound of Desperado;
//! - `EF_SU_STOOP` (1132): the sound of the state it shows;
//! - `EF_TALK_FROSTJOKE` (295), `EF_TALK_SCREAM` (296): no sound either, a line of talk (`NotifyGameMode`:
//!   the client has the player's own character say a line of `BA_frostjoke.txt` or `DC_scream.txt`). The world
//!   says nothing: the app says the line when it is told of the effect (`crate::song_talk`,
//!   `app/incoming/combat.rs`), and here the two only have their place, so that they are known.
//!
//! The client places a sound where the effect is (`PlayEffectWave`) and plays the kicks of `EF_STIN6` each from
//! further off; here a sound is only started, and all are as loud.

use super::*;

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[], sprites: &[] };

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect. The sounds an effect starts later are in `sounds`.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        // The two lines of talk: nothing here but their first tick (the client's live 999 and do nothing more).
        295 | 296 => (1, ""),
        401 | 769 => (200, ""),
        771 => (300, ""),
        776 => (100, ""),
        // `EF_SU_STOOP` lives as long as the state it shows (the client ends it by `AM_DELETE_EFFECT`) and
        // draws nothing: it is its sound alone, so a tick is all it is given here.
        1132 => (1, "effect/mon_금강불괴.wav"),
        _ => return None,
    })
}

/// The sounds an effect starts after its first tick, each with its tick.
fn sounds(effect: u32) -> &'static [(i32, &'static str)] {
    const KICK: &str = "effect/t_날라차기.wav";
    const NAPALM: &str = "effect/ef_napalmbeat.wav";
    match effect {
        401 => &[(20, NAPALM), (30, NAPALM), (40, NAPALM), (50, NAPALM), (60, NAPALM)],
        769 => &[(5, KICK), (20, KICK), (35, KICK), (50, KICK), (65, KICK)],
        771 => &[(1, "effect/assasin_enchantpoison.wav"), (1, "effect/ef_poisonattack.wav")],
        776 => &[(45, "effect/데스페라도.wav")],
        _ => &[],
    }
}

/// One tick of an effect of this file: the sounds it starts at that tick.
fn run(effect: u32, course: &mut Course, _prims: &mut Vec<Prim>) {
    course.starts(sounds(effect));
}
