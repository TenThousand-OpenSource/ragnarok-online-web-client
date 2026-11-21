//! The hits that land: the one struck flinches and stops, the hit sounds from where it lands, and its numbers
//! and words rise (the client's handler of `AM_ATTACKED` and its like in `CGameActor::SendMsg`; the rules are
//! `crate::blow::signs`).

use super::*;
use crate::blow::landing::{Hit, How};
use crate::blow::signs::{self, Between, Sign};

/// The arrow's sprite, and how long it flies: 8 of the client's frames of 24 milliseconds (`CArrowEffect`).
pub(in crate::web::world) const ARROW: &str = "몬스터/skel_archer_arrow";
pub(in crate::web::world) const ARROW_MS: f64 = 192.0;

/// A number or a word of a fight flying over someone, as the original's `CMsgEffect` flies it.
pub(in crate::web::world) struct Flying {
    pub(in crate::web::world) began: f64,
    /// Whom it is over (none named: the player's own character), and where that one stood on the ground when
    /// last seen: it stays over its owner while that one is in sight.
    pub(in crate::web::world) on: Option<u32>,
    pub(in crate::web::world) base: Vector,
    /// Which way it drifts, east and north.
    pub(in crate::web::world) drift: (f32, f32),
    /// Its pictures, left to right: the digits, or the one word.
    pub(in crate::web::world) pictures: Vec<SpriteKey>,
    pub(in crate::web::world) flight: Flight,
}

/// A hit on its way to someone.
pub(in crate::web::world) struct Landing {
    /// The player's own ID, who is not among the others.
    pub own: u32,
    pub striker: u32,
    pub hit: Hit,
    /// Whether it lands with the ring of a plain blow (`EF_HIT1`): a skill's hit shows what the client has
    /// for that skill instead, which the app asks for.
    pub ring: bool,
    /// The one struck is the player's own character under a state that wards blows off (Kyrie Eleison,
    /// Parrying): a monster's blow that does nothing then shows the guard, not a miss.
    pub guarded: bool,
    /// The effect that follows the skill on the one struck, which the message of a skill's first hit carries
    /// (the client's `effectId`).
    pub follow: Option<u32>,
}

/// What a hit that landed does to the body of the one struck (`frame/sprites.rs` does it): whether it stops
/// and flinches, and the colour its sprite is drawn in for a moment (`crate::blow::signs::flash`).
pub(in crate::web::world) struct Struck {
    pub target: u32,
    pub own: u32,
    pub flinches: bool,
    pub flash: Option<[u8; 3]>,
}

/// The sprite of the numbers (the client's damage sprite), and of its words.
const NUMBERS: &str = "이팩트/숫자";
const WORDS: &str = "이팩트/msg";
/// The colour of the mark of a critical blow (`SpawnMsgEffect` of `MAKETYPE_CRITICAL_EFFECT`: 0xFFAAAAAA), and
/// of the blue word (`MET_BLUE_EFFECT`: 150, 150, 255).
const MARK_GREY: u32 = 0xaaaaaa;
const WORD_BLUE: u32 = 0x9696ff;
/// The client's effect of a blow warded off.
const EF_GUARD: u32 = 336;

/// A sprite's name with the colour it is tinted in; white is the sprite as it is.
fn tinted(sprite: &str, colour: u32) -> String {
    if colour == signs::WHITE { sprite.to_owned() } else { format!("{sprite}#{colour:06x}") }
}

/// The hits that land now, out of the one queue of them (`State::blows`): each sounds and shows its numbers
/// here; what the body of the one struck does is given back for `frame/sprites.rs`.
pub(in crate::web::world) fn landings(state: &mut State, time: f64) -> Vec<Struck> {
    if state.scene.is_none() {
        return Vec::new();
    }
    let due: Vec<_> = state.blows.extract_if(.., |blow| blow.0 <= time).collect();
    due.into_iter().filter_map(|(_, target, landing)| land(state, time, target, &landing)).collect()
}

/// One hit lands on `target`: what it does to that one's body, if that one is in sight.
fn land(state: &mut State, time: f64, target: u32, landing: &Landing) -> Option<Struck> {
    let scene = state.scene.as_ref()?;
    let (own, hit) = (landing.own, landing.hit);
    let (on, by) = ((target != own).then_some(target), (landing.striker != own).then_some(landing.striker));
    // Who struck, as far as the hit goes: whether it is a character, and the kind of weapon it holds (a
    // monster holds none the client knows of, `CGameActor::GetWeapon` gives -1: its blow, as that of one
    // out of sight, sounds as a skill's hit does).
    let striker = scene.one(landing.striker, own);
    let striker_is_character = striker.map(|striker| matches!(striker.look, Look::Character(_)));
    let weapon = striker.and_then(|striker| match &striker.look {
        Look::Character(figure) => Some(
            data::about(figure.weapon)
                .and_then(|(_, facts)| facts.into_iter().find(|(key, _)| key == "SubType"))
                .and_then(|(_, kind)| crate::jobs::weapon_kind(&kind))
                .unwrap_or(0)
        ),
        Look::Creature(_) | Look::Model(_) => None,
    });
    let hurt = scene.one(target, own)?;
    let between = Between { struck_is_character: matches!(hurt.look, Look::Character(_)), striker_is_character, guarded: landing.guarded && target == own };
    // The client sends a number off by the quarter the one struck faces (`MsgEffectDrift`).
    let drift = match hurt.facing {
        0 | 1 => (-0.8, 0.8),
        2 | 3 => (-0.8, -0.8),
        4 | 5 => (0.8, -0.8),
        _ => (0.8, 0.8),
    };
    let base = on_ground(&scene.gat, hurt.at);
    let hurts = hit.damage > 0 && hit.how != How::Lucky;
    // What it sounds like, heard from where it lands, unless one struck itself: a skill's hit as one of the
    // client's four plain blows, a character by what its job wears, a monster by the weapon that struck it.
    if hurts && landing.striker != target {
        let four = lot(4);
        let sound = if hit.how.of_skill() {
            signs::weapon_sound(None, data::weapon_hit, four)
        } else if between.struck_is_character {
            signs::struck_sound(hurt.job).to_owned()
        } else {
            signs::weapon_sound(weapon, data::weapon_hit, four)
        };
        earshot::heard(scene, &sound, on);
    }
    if hurts {
        // The ring of a plain blow that lands (the client's `EF_HIT1`).
        if landing.ring {
            prims::begin(0, on, None, time, &mut state.bursts, &mut state.sounds_due);
        }
        // A critical blow lands with the flare of a skill's hit over that (the client's `EF_HIT2`, what its
        // `ProcessHitMessage` gives a `DMG_CRITICAL`), and with that effect's own sound.
        if matches!(hit.how, How::Critical | How::ManyCritical) {
            if let Some((burst, sound)) = prims::burst(1, on, None, time) {
                state.bursts.push(burst);
                state.heard.push((time, sound.to_owned(), on));
            }
        }
    }
    for sign in signs::signs(hit.how, hit.damage, hit.total, between) {
        match sign {
            // A miss is said over the one who struck, and only when the player has it so (the client's
            // `/miss`).
            Sign::Miss { red } => {
                let stands = state.scene.as_ref().and_then(|scene| scene.place_of(by).map(|at| on_ground(&scene.gat, at)));
                if let Some(stands) = stands.filter(|_| state.options.misses) {
                    let word = vec![SpriteKey::Creature(tinted(WORDS, if red { signs::RED } else { signs::WHITE }), 0, 0)];
                    state.flying.push(Flying { began: time, on: by, base: stands, drift, pictures: word, flight: Flight::Miss });
                }
            }
            Sign::Guard => {
                prims::begin(EF_GUARD, on, None, time, &mut state.bursts, &mut state.sounds_due);
            }
            sign => shown(state, time, on, base, drift, sign),
        }
    }
    // The one struck stops and flinches as a hit that hurts lands, unless it endures it (the client's two
    // messages "no motion"); a skill's hit may flash it.
    let hurts = hit.damage > 0;
    Some(Struck { target, own, flinches: hurts && hit.how.flinches(), flash: signs::flash(hit.how, landing.follow).filter(|_| hurts) })
}

/// A number or a word rises over someone standing at `base`.
fn shown(state: &mut State, time: f64, on: Option<u32>, base: Vector, drift: (f32, f32), sign: Sign) {
    let word = |sprite: String, action: usize, flight: Flight| Flying {
        began: time,
        on,
        base,
        drift,
        pictures: vec![SpriteKey::Creature(sprite, action, 0)],
        flight
    };
    let flying = match sign {
        Sign::Number(value, colour, flight) => {
            // A new total takes the place of the one over the same head (`SpawnDamageDigits`).
            if flight == Flight::Total {
                state.flying.retain(|flying| flying.flight != Flight::Total || flying.on != on);
            }
            let sprite = tinted(NUMBERS, colour);
            // (The client writes no more than 999999, without its option for longer numbers.)
            let pictures = value.clamp(0, 999_999).to_string().bytes().map(|digit| SpriteKey::Creature(sprite.clone(), 0, usize::from(digit - b'0'))).collect();
            Flying { began: time, on, base, drift, pictures, flight }
        }
        // The third action of the numbers' sprite, greyed.
        Sign::CriticalMark => word(tinted(NUMBERS, MARK_GREY), 2, Flight::Mark),
        // The third action of the words' sprite (`MAKETYPE_BLUE_EFFECT`).
        Sign::Word => word(tinted(WORDS, WORD_BLUE), 2, Flight::Word),
        // The fourth action of the numbers' sprite (`MAKETYPE_LUCKY_EFFECT`).
        Sign::Lucky => word(NUMBERS.to_owned(), 3, Flight::Lucky),
        Sign::Miss { .. } | Sign::Guard => return,
    };
    state.flying.push(flying);
}

/// A number rises over someone in a colour, straight up (the client's `AM_MAKE_NUMBER` of kind 114): life or
/// spirit recovered (`crate::blow::signs::GREEN`, `BLUE`: what a potion gives the player's own character, what
/// Heal gives anyone), or a number the server has shown with an effect.
pub fn number_over(who: u32, own: u32, amount: i32, colour: u32) {
    STATE.with_borrow_mut(|state| {
        let on = (who != own).then_some(who);
        let Some(base) = state.scene.as_ref().and_then(|scene| scene.place_of(on).map(|at| on_ground(&scene.gat, at))) else { return };
        shown(state, now(), on, base, (0.0, 0.0), Sign::Number(amount, colour, Flight::Recovery));
    });
}
