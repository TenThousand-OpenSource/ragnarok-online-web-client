//! The effects a state going on or off begins and ends on the one in it: the client's `AM_STATUS_CHANGE` in
//! `CGameActor::SendMsg` (`Contents/Actors/GameObject.cpp`), for the states whose effects live until they are
//! told to go. The client ends every such effect that runs on the actor (`AM_DELETE_EFFECT`, while `HasId`),
//! whether the state goes on or off, and with the state on launches it anew.
//!
//! The same effects come by a second way: the bits of the third option of one coming into sight (the
//! `virtue` of the packets that show a unit, the server's `opt3`; `CGameMode::ApplyActorStatusEffects`,
//! `Client/Mode/GameModePackets.cpp`), which `seen` has. And some effects that a skill shows are launched
//! only while another runs, or does not (`begins`). And two effects a bit of the effect state keeps going,
//! launched again and again for as long as it is set (`Kept`, `state_effects/kept.rs`).
//!
//! Only the states are here whose effects are drawn (`web/world/prims/`, or a file of the client's that is
//! played). Every state of that arm that launches or ends an effect, with those left out and why:
//!
//! - the copies of `EFST_NJ_BUNSINJYUTSU`, `EFST_WUGDASH`, `EFST_STRIKING` and `EFST_HALLUCINATIONWALK`
//!   (617, 752, 925, 792): not drawn (the blur of the quickenings, Overthrust, Energy Coat and Fury is:
//!   `EF_MAKEBLUR`, told its colour by `blur`);
//! - `EFST_DEVIL1` and `EFST_HANDICAPSTATE_DEEPBLIND`, which darken the screen of the player's own character
//!   (`EF_BLACKDEVIL` and the `EF_DEVIL`s, `EF_BLIND` with `EF_STATUS_STATE`): not drawn here; the veil of
//!   blindness comes by the health state (`crate::ailments`);
//! - `EFST_PROPERTYUNDEAD` (655, 656), `EFST_SMA_READY` (546), `EFST_RG_CCONFINE_M` and `_S` (604),
//!   `EFST_WZ_SIGHTBLASTER` (601), `EFST_DA_CONTRACT` (586), `EFST_ORATIO` (755), `EFST_BANDING` (909),
//!   `EFST_VENOMIMPRESS` (788), `EFST_FROSTMISTY` (789), `EFST_COLD` (940), `EFST_SIREN` (879),
//!   `EFST_MANHOLE` (823), `EFST_SHADOWFORM` (809), `EFST_CAMOUFLAGE`, `EFST_STEALTHFIELD` and
//!   `EFST_ELEMENTAL_VEIL` (744), `EFST_GRANITIC_ARMOR`, `EFST_KYOMU` and `EFST_GENSOU` (963),
//!   `EFST_KAGEMUSYA` (1004), `EFST_ZANGETSU` (1003), `EFST_MAXPAIN` (1037), `EFST_WIDEWEB` (404),
//!   `EFST_BURNT` (1054), `EFST_MISTY_FROST` (1420), `EFST_PRISON` (1525), `EFST_LOCKON_LASER` (1601),
//!   `EFST_HELLS_PLANT_ARMOR` (1610), the states that launch an `EF_STATUS_…_UP` (2288 to 2300), and the songs
//!   whose effect is an `EF_CIRCLEPOWER` not drawn (865, 867, 871, 873, 875, 877): their effects are not drawn;
//! - `EFST_MEIKYOUSISUI`, which launches Moonlit's wind in a colour of its own (the effect's
//!   `colorChannels`, which no effect here takes);
//! - what the client does through its other effects' manager (`g_effectMgr`: the balls of `EFST_SUMMON1` to
//!   `5`, `EFST_ROLLINGCUTTER`, the stances, the marks and the hats), which is another system than the
//!   effects of `CRagEffect`.
//!
//! Left out of what the client does for the states that are here: the darkened sky of a Soul Link on the
//! player's own character, that Steel Body's effect stays on a character who wears a thing that shows it too
//! (`appearanceFlags`), the words a mercenary's states say over it (`AM_CHAT`), and the grey body held
//! still of the third option's bit 0x20 (a blade stopped).
//!
//! A third way is the player's own character alone: the client launches the effects of the states it is in
//! anew when it is made on a map it has come to (`CPlayer::ReapplySkillEffects`, which goes through the states
//! it keeps for the player, `g_activeSkillEffects`); `reapplied` has that list, as far as its effects are
//! drawn. (The client calls the same function when the player's own states are told and when the map's kind
//! is; that is not done here: only the arrival.)

mod kept;
mod numbers;
mod own;
mod seen;

pub use kept::Kept;
use numbers::*;
pub use own::reapplied;
pub use seen::{Seen, seen};

/// What `EF_MAKEBLUR` is told as a state launches it (`AM_SETCOUNT` with 0xFFFF and the state's own number,
/// which `CRagEffect::SendMsg` turns into the colour of the body and of its afterimages; with 0xFFFE, Fury's,
/// into -1: a flicker and no afterimage: `crate::body::copies::blur`). None for a state that launches none.
pub fn blur(state: u32) -> Option<[f32; 3]> {
    Some(match state {
        // `EFST_TWOHANDQUICKEN`, `EFST_ONEHANDQUICKEN` (told 130), `EFST_SPEARQUICKEN` (250).
        2 | 68 | 161 => [200.0, 200.0, 0.0],
        // `EFST_OVERTHRUST`, `EFST_OVERTHRUSTMAX` (128).
        25 | 188 => [250.0, 150.0, 150.0],
        // `EFST_ENERGYCOAT` (169).
        31 => [180.0, 180.0, 250.0],
        // `EFST_EXPLOSIONSPIRITS` (0xFFFE, 128).
        86 => [-1.0; 3],
        _ => return None,
    })
}

/// What a state going on or off does to the effects on the one in it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Change {
    /// The effects ended, and then those begun.
    pub ends: &'static [u32],
    pub begins: &'static [u32],
    /// An effect begun only if another ran as the state changed: the one that ran, and the one begun.
    pub after: Option<(u32, u32)>,
    /// The sound the change starts (none: empty).
    pub sound: &'static str,
}

/// What a state (the client's `EFST_` number) going on or off does: the effects to end on the one in it, and
/// then those to begin.
pub fn changed(state: u32, on: bool) -> Change {
    let kept = |effects: &'static [u32]| Change { ends: effects, begins: if on { effects } else { &[] }, ..Change::default() };
    match state {
        // `EFST_APPLEIDUN`.
        73 => kept(&[EF_CIRCLEPOWER2]),
        // `EFST_DONTFORGETME`.
        75 => kept(&[EF_DECAGILITY]),
        // `EFST_ETERNALCHAOS`, `EFST_ROKISWEIL`, `EFST_SATURDAY_NIGHT_FEVER`.
        79 | 82 | 448 => kept(&[EF_ENERVATION7]),
        // `EFST_TWOHANDQUICKEN`, `EFST_OVERTHRUST`, `EFST_ENERGYCOAT`, `EFST_SPEARQUICKEN`,
        // `EFST_ONEHANDQUICKEN`, `EFST_OVERTHRUSTMAX`: the blur, told its colour (`blur`).
        2 | 25 | 31 | 68 | 161 | 188 => kept(&[EF_MAKEBLUR]),
        // `EFST_EXPLOSIONSPIRITS`: the sign of Fury, and the blur that flickers the body.
        86 => kept(&[EF_GUMGANG, EF_MAKEBLUR]),
        // `EFST_STEELBODY`, `EFST_DBARRIER`.
        87 | 235 => kept(&[EF_STEELBODY, EF_SU_STOOP]),
        // `EFST_STOP`: the effect is begun by the skill (`NPC_STOP`), and only ended here.
        95 => Change { ends: &[EF_NPC_STOP], ..Change::default() },
        // `EFST_POWERUP`.
        98 => kept(&[EF_GUMGANGNPC]),
        // `EFST_AURABLADE`: the walls round the one in it, and the light of its weapon.
        103 => kept(&[EF_AURABLADE, EF_AURABLADE2]),
        // `EFST_LKCONCENTRATION`.
        105 => kept(&[EF_LKCONCENTRATION]),
        // `EFST_BERSERK`, `EFST_BLOOD_SUCKER`.
        107 | 464 => kept(&[EF_REDBODY]),
        // `EFST_ASSUMPTIO`, `EFST_ASSUMPTIO2`, `EFST_ASSUMPTIO_BUFF`.
        110 | 473 | 1121 => kept(&[EF_ASSUMPTIO]),
        // `EFST_MAGICPOWER`: the weapon drawn twice.
        113 => kept(&[EF_LIGHTBLADE]),
        // `EFST_MARIONETTE_MASTER`: the strings to the puppet go too.
        121 => Change { ends: &[EF_LINELINK3, EF_PINKBODY], begins: if on { &[EF_PINKBODY] } else { &[] }, ..Change::default() },
        // `EFST_MARIONETTE`, `EFST_MAGIC_POISON`.
        122 | 1142 => kept(&[EF_PINKBODY]),
        // `EFST_MOON`.
        123 => kept(&[EF_SPHEREWIND2]),
        // `EFST_RUN`, going on or off: the footprints end, and if there were any `EF_STOPRUN` begins, which
        // draws nothing and for as long as it lives has the skill show the blades of one who stops.
        133 => Change { ends: &[EF_RUN], after: Some((EF_RUN, EF_STOPRUN)), ..Change::default() },
        // `EFST_TING`, going on or off: one who ran into a wall is shaken, with a sound.
        134 => Change { ends: &[EF_RUN], begins: &[EF_QUAKEBODY], sound: "effect/t_벽튕김.wav", ..Change::default() },
        // `EFST_SOULLINK` and the soul states after it: the skill begins the effect; the state's end ends it.
        149 | 1056..=1059 if !on => Change { ends: &[EF_ASURABODY], ..Change::default() },
        // `EFST_KAITE`.
        153 => kept(&[EF_REFLECTBODY]),
        // `EFST_SG_SUN_WARM`, `EFST_SG_MOON_WARM`, `EFST_SG_STAR_WARM`, `EFST_DAURA`.
        165..=167 | 232 => kept(&[EF_DOUBLEGUMGANG, EF_REDLIGHTBODY]),
        // `EFST_MER_FLEE`, `_ATK`, `_HP`, `_SP`, `_HIT`: a mercenary's state, going on or off.
        277..=281 => Change { begins: &[EF_AGIUP], ..Change::default() },
        // `EFST_DUPLELIGHT`: the two lights that go round the one in it.
        350 => kept(&[EF_CHOOKGI_N]),
        // `EFST_SHADOWFORM`: the tube to the one it is used on, which the skill begins (`SC_SHADOWFORM`), goes
        // with the state. (The client ends it beside the state's own effect, `EF_CLOAKING3`, which is not
        // drawn, and only while that one runs: so not as the state comes on.)
        394 if !on => Change { ends: &[EF_LINELINK4], ..Change::default() },
        // `EFST_NETHERWORLD`.
        433 => kept(&[EF_NPC_STOP]),
        // `EFST_MELODYOFSINK`.
        452 => kept(&[EF_LEXDIVINA]),
        // `EFST_BEYOND_OF_WARCRY`.
        453 => kept(&[EF_STASIS]),
        // `EFST_AKAITSUKI`.
        660 => kept(&[EF_REDLIGHTBODY]),
        // `EFST_RAY_OF_PROTECTION`.
        676 => kept(&[EF_ALL_RAY_OF_PROTECTION]),
        // `EFST_OFFERTORIUM`.
        716 => kept(&[EF_AB_OFFERTORIUM_RING]),
        // `EFST_TELEKINESIS_INTENSE`.
        717 => kept(&[EF_WL_TELEKINESIS_INTENSE]),
        // `EFST_DARKCROW`: the client launches its effect three times.
        730 => Change { ends: &[EF_GC_DARKCROW], begins: if on { &[EF_GC_DARKCROW; 3] } else { &[] }, ..Change::default() },
        // `EFST_ILLUSIONDOPING`.
        734 => kept(&[EF_GN_ILLUSIONDOPING]),
        // `EFST_CHILL`.
        736 => kept(&[EF_NPC_CHILL]),
        _ => Change::default(),
    }
}

/// Whether an effect that a skill shows is launched, `running` saying which effects run on the one it is for:
/// some the client launches only if they do not run already (`HasId` in its handler of the skill), and the
/// two of `TK_RUN` by whether `EF_STOPRUN` runs: the blades of a stop (`EF_STOPEFFECT`) if it does, the
/// footprints (`EF_RUN`) if it does not.
pub fn begins(effect: u32, running: impl Fn(u32) -> bool) -> bool {
    match effect {
        EF_RUN => !running(EF_STOPRUN) && !running(EF_RUN),
        EF_STOPEFFECT => running(EF_STOPRUN),
        EF_CHOOKGI_N | EF_ESCAPE => !running(effect),
        _ => true,
    }
}

#[cfg(test)]
mod tests;
