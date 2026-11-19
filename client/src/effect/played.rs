//! Which of the client's effects are played from files, and how: the one answer to "is this effect a file".
//!
//! Three sorts, asked in this order:
//!
//! - an effect whose function picks one of several files by lot (`SelectRagEffectResource`), played as every
//!   `.str` effect is: `EF_FIREWALL` (25), `EF_FIREHIT` (49), `EF_WINDHIT` (52), `EF_BUBBLE` (109), `EF_PONG`
//!   (199);
//! - an effect a class of the client's own plays and carries (`crate::effect::carried` has the rules of the
//!   ways and the turns): moved from the one who strikes to the one struck, turned on the screen, put aside,
//!   begun late or anew:
//!   - `EF_NEW_CART_CANNON` (1357), `CCartCannon`: a ball flown in ten steps from the one who shoots to the one
//!     shot at, turned by its way;
//!   - `EF_STORM_CANNON` (1755), `CStormCannonEffect`: two files flown at 1.7 a tick to the one aimed at and
//!     on, fading once past;
//!   - `EF_CRIMSONARROW` (1756), `EF_FRENZY_SHOT` (2114), `CCrimsonArrowEffect`, and `EF_SAVAGE_IMPACT` (1837),
//!     `CSavageImpactEffect`: a file over the striker turned on the screen towards the one struck;
//!   - `EF_SECOND_FAITH_HIT` (2080), `EF_SECOND_JUDGEMENT` (2082), `CRandomHitEffect`: a hit at one of three
//!     places by the one struck, each with its turn and its delay, taken in turn;
//!   - `EF_ACIDIFIED_ZONE_FIRE`, `_WATER`, `_WIND`, `_GROUND` (2206, 2214, 2222, 2230), `CAcidifiedThrow`: a
//!     bottle thrown in a bow, and three files where it lands;
//!   - `EF_SHADOW_STAB` (1953), `EF_CHAIN_REACTION_SHOT` (2018), `EF_ROSEBLOSSOM_SHOT` (2050),
//!     `EF_WOODEN_WARRIOR_ATTACK` (2250), `EF_WOODEN_WARRIOR_THROWROCK` (2252): files `CEZ2STRScreenEffect`
//!     plays over the striker as it plays any, turned in its `InitEvents` by the way on the screen to the one
//!     struck: the first by that way itself, the others half a turn on;
//! - any other effect that is a file: the one file the client's table has for it, with the sound it starts
//!   with (`crate::effect_tables`).
//!
//! Each carried effect the client launches on the one who uses the skill and tells the other
//! (`g_effectMgr->Spawn` with the target as its source actor, in `CGameMode`'s handler of a skill used), but
//! for the two hits, which follow a blow on the one struck.
//!
//! `tools/client-tables/skill_effects.py` reads the numbers `played` names: those effects are not plain files,
//! whatever file the client's table has for them, and are left out of `effect_tables/effect_files.txt`. The
//! world plays what this gives (`web/world/launch.rs`, `web/world/frame/effects.rs`).

use crate::effect::carried::{BOW_LANDS, Carried, TICK_MS, Turn, Way};

/// What an effect that is played from files launches: its files in the client's folder of effects, in the
/// order they are drawn, each with how it is carried (none: over the one the effect is on, as every plain file
/// plays), and its sounds, each with the tick of the effect it starts at.
#[derive(Debug, Clone, PartialEq)]
pub struct Launch {
    pub files: Vec<(String, Option<Carried>)>,
    pub sounds: Vec<(i32, String)>,
}

impl Launch {
    /// One file carried so, with a sound at its first tick (none: empty).
    fn one(file: &str, how: Option<Carried>, sound: &str) -> Self {
        Launch { files: vec![(file.to_owned(), how)], sounds: if sound.is_empty() { Vec::new() } else { vec![(0, sound.to_owned())] } }
    }
}

/// What an effect launches, if it is played from files; nothing for one that is not. `lot` is the client's
/// `rand() % under`; `turn` the number the client counts its hits at three places round with
/// (`CRandomHitEffect`), which such a hit moves on.
pub fn played(effect: u32, lot: &mut dyn FnMut(u32) -> u32, turn: &mut usize) -> Option<Launch> {
    Some(match effect {
        // `EF_FIREWALL`: what a cell of Fire Wall launches again and again, and what `NPC_HELLBURNING` shows.
        // (The sound of a Fire Wall is its unit's, as it appears.)
        25 => Launch::one(&format!("firewall{}.str", lot(2) + 1), None, ""),
        // The hit of fire, with one of two sounds besides (`CSession::GetEnemyHitWaveName` for fire).
        49 => {
            let file = format!("firehit{}.str", lot(3) + 1);
            Launch::one(&file, None, &format!("_enemy_hit_fire{}.wav", lot(2) + 1))
        }
        52 => Launch::one(&format!("windhit{}.str", lot(3) + 1), None, ""),
        // `EF_BUBBLE` and `EF_PONG`: what the maps under the sea and in the caves place about themselves.
        109 => Launch::one(&format!("bubble{}.str", lot(4) + 1), None, ""),
        199 => Launch::one(&format!("pong{}.str", lot(3) + 1), None, ""),
        1357 => cart_cannon(),
        1755 => storm_cannon(),
        1756 => shot("crimsonarrow/crimsonarrow/crimsonarrow.str", 13.0, "effect/AG_CRIMSON_ARROW1.wav"),
        1837 => shot("savage_impact/savage_impact/savage_impact.str", 12.0, "effect/SHC_SAVAGE_IMPACT.wav"),
        // (No sound of its own: `InitEvents` gives it none.)
        1953 => turned_by_way("shadow_stab/shadow_stab/shadow_stab.str", 0.0, ""),
        // (Its sound the client starts beside it, in its handler of `ABC_CHAIN_REACTION_SHOT`.)
        2018 => turned_by_way("chain_reaction_shot/chain_reaction_shot/chain_reaction_shot.str", 180.0, "effect/ABC_CHAIN_REACTION_SHOT.wav"),
        2050 => turned_by_way("roseblossom/roseblossom_shot/roseblossom_shot.str", 180.0, "effect/TR_ROSEBLOSSOM_SHOT.wav"),
        2080 => hit("4ig_secondfalth/secondfalth_hit/secondfalth_hit.str", turn),
        2082 => hit("4iq_secondjudgement/secondjudgement/secondjudgement.str", turn),
        2114 => shot("frenzy_shot/frenzy_shot/frenzy_shot.str", 4.0, "effect/ABC_FRENZY_SHOT.wav"),
        2206 => thrown("fire", "effect/BO_ACIDIFIED_ZONE_FIRE.wav"),
        2214 => thrown("water", "effect/BO_ACIDIFIED_ZONE_WATER.wav"),
        2222 => thrown("wind", "effect/BO_ACIDIFIED_ZONE_WIND.wav"),
        2230 => thrown("ground", "effect/BO_ACIDIFIED_ZONE_GROUND.wav"),
        2250 => turned_by_way("wooden_warrior/wooden_attack/wooden_attack/wooden_attack.str", 180.0, "effect/BO_WOODEN_ATTACK.wav"),
        2252 => turned_by_way("wooden_warrior/throwrock/throwrock/throwrock.str", 180.0, "effect/BO_THROWROCK.wav"),
        _ => return plain(effect),
    })
}

/// An effect that is one file of the client's table, played over the one it is on, with the sound the table
/// has for it.
fn plain(effect: u32) -> Option<Launch> {
    let file = crate::effect_tables::effect_file(effect)?.to_lowercase();
    Some(Launch::one(&file, None, crate::effect_tables::effect_sound(effect).unwrap_or("")))
}

/// `CCartCannon`: one file, begun anew for as long as it flies, flown in ten steps and turned by its way; it
/// waits the 200 milliseconds the handler of `GN_CARTCANNON` gives it (`SetDelay`), which starts the sound as
/// it launches it.
fn cart_cannon() -> Launch {
    let ball = Carried::new(Way::Steps).turned(Turn::Flight).delayed(200).looped();
    Launch::one("new_cart_cannon/new_cart_cannon/new_cart_cannon.str", Some(ball), "effect/mon_침투경.wav")
}

/// `CStormCannonEffect`: two files, the lower drawn first, both begun anew as they fly and both 1.5 west of
/// and 17 over where the flight is.
fn storm_cannon() -> Launch {
    let flown = Carried::new(Way::Straight).offset([-1.5, 17.0, 0.0]).looped();
    let files = ["stormcannon/stormcannon_bottom/stormcannon_bottom.str", "stormcannon/stormcannon/stormcannon.str"];
    Launch { files: files.into_iter().map(|file| (file.to_owned(), Some(flown))).collect(), sounds: vec![(0, "effect/AG_STORM_CANNON.wav".to_owned())] }
}

/// `CCrimsonArrowEffect` and `CSavageImpactEffect`: a file so high over the striker, played once, turned by the
/// way on the screen to the one struck as it begins.
fn shot(file: &str, high: f32, sound: &str) -> Launch {
    Launch::one(file, Some(Carried::new(Way::Stays).offset([0.0, high, 0.0]).turned(Turn::Towards(0.0))), sound)
}

/// A file `CEZ2STRScreenEffect` plays 80 pixels over the striker and turns by the way on the screen to the one
/// struck and `more` degrees: nothing more for `EF_SHADOW_STAB` (the angle written out in `InitEvents`), half
/// a turn for the others (`ScreenAngle(...) + 180`, which `EF_CHAIN_REACTION_SHOT` and `EF_ROSEBLOSSOM_SHOT`
/// write out).
fn turned_by_way(file: &str, more: f32, sound: &str) -> Launch {
    Launch::one(file, Some(Carried::new(Way::Stays).lifted(80.0).turned(Turn::Towards(more))), sound)
}

/// `CRandomHitEffect`: a file at the next of three places by the one struck (east and up), turned by that
/// place's angle and waiting 200 milliseconds more at each, and moved on twice a tick (its `OnProcess` calls
/// `Process` twice).
fn hit(file: &str, turn: &mut usize) -> Launch {
    const PLACES: [([f32; 3], f32); 3] = [([0.0, 7.0, 0.0], 0.0), ([4.0, 6.0, 0.0], 320.0), ([-4.0, 5.0, 0.0], 40.0)];
    let index = *turn % 3;
    *turn = (index + 1) % 3;
    let (offset, angle) = PLACES[index];
    Launch::one(file, Some(Carried::new(Way::Stays).offset(offset).turned(Turn::By(angle)).delayed(index as i32 * 200).paced(2)), "")
}

/// `CAcidifiedThrow` of an element: the bottle, begun anew as it flies its bow from the thrower; then, a tick
/// after it has landed, three files 1 west of and 14 over the one thrown at, played once each, and at the tick
/// it lands the sound of every such throw and the element's own.
fn thrown(element: &str, sound: &str) -> Launch {
    let file = |part: &str| format!("acidified_zone_{element}/acidified_zone_{element}_{part}/acidified_zone_{element}_{part}.str");
    let landed = Carried::new(Way::There).offset([-1.0, 14.0, 0.0]).delayed((BOW_LANDS + 1) * TICK_MS);
    let mut files = vec![(file("c"), Some(Carried::new(Way::Bowed).looped()))];
    files.extend(["b", "b_bottom", "water"].map(|part| (file(part), Some(landed))));
    Launch { files, sounds: vec![(BOW_LANDS, "effect/BO_ACIDIFIED_ZONE.wav".to_owned()), (BOW_LANDS, sound.to_owned())] }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What an effect launches with every lot drawn as `drawn` and the hits' count at `turn`.
    fn launched(effect: u32, drawn: u32, turn: &mut usize) -> Option<Launch> {
        played(effect, &mut |under| drawn % under, turn)
    }

    #[test]
    fn a_file_picked_by_lot_plays_as_a_plain_one() {
        let fire_wall = launched(25, 1, &mut 0).unwrap();
        assert_eq!((fire_wall.files, fire_wall.sounds), (vec![("firewall2.str".to_owned(), None)], Vec::new()));
        let fire_hit = launched(49, 2, &mut 0).unwrap();
        assert_eq!((fire_hit.files, fire_hit.sounds), (vec![("firehit3.str".to_owned(), None)], vec![(0, "_enemy_hit_fire1.wav".to_owned())]));
        assert_eq!(launched(52, 0, &mut 0).unwrap().files, vec![("windhit1.str".to_owned(), None)]);
        assert_eq!(launched(109, 3, &mut 0).unwrap().files, vec![("bubble4.str".to_owned(), None)]);
        assert_eq!(launched(199, 4, &mut 0).unwrap().files, vec![("pong2.str".to_owned(), None)]);
    }

    #[test]
    fn a_plain_file_is_the_tables_with_its_sound() {
        // `EF_BENEDICTIO`: one file, lower case as the gateway serves it, played over the one it is on.
        let file = crate::effect_tables::effect_file(91).unwrap().to_lowercase();
        assert_eq!(launched(91, 0, &mut 0).unwrap().files, vec![(file, None)]);
        // An effect the client draws by code alone is no file.
        assert_eq!(launched(0, 0, &mut 0), None);
        // None of those named here is left in the table of plain files.
        for effect in [25, 49, 52, 1357, 1755, 1756, 1837, 1953, 2018, 2050, 2080, 2082, 2114, 2206, 2214, 2222, 2230, 2250, 2252] {
            assert_eq!(crate::effect_tables::effect_file(effect), None, "{effect}");
        }
    }

    #[test]
    fn the_shots_are_turned_towards_the_one_struck() {
        let how = |effect: u32| launched(effect, 0, &mut 0).unwrap().files[0].1.unwrap();
        assert_eq!((how(1756).turn, how(1756).offset, how(1756).lift), (Turn::Towards(0.0), [0.0, 13.0, 0.0], 0.0));
        // The files `CEZ2STRScreenEffect` turns stand 80 pixels up the screen, as every file it plays.
        assert_eq!((how(1953).turn, how(1953).lift), (Turn::Towards(0.0), 80.0));
        for effect in [2018, 2050, 2250, 2252] {
            assert_eq!((how(effect).turn, how(effect).lift, how(effect).way), (Turn::Towards(180.0), 80.0, Way::Stays), "{effect}");
        }
        assert_eq!(launched(1953, 0, &mut 0).unwrap().sounds, Vec::new());
        assert_eq!(launched(2050, 0, &mut 0).unwrap().sounds, vec![(0, "effect/TR_ROSEBLOSSOM_SHOT.wav".to_owned())]);
    }

    #[test]
    fn the_hits_go_round_three_places() {
        let mut turn = 0;
        let hows: Vec<Carried> = (0..4).map(|_| launched(2080, 0, &mut turn).unwrap().files[0].1.unwrap()).collect();
        assert_eq!(
            hows.iter().map(|how| (how.offset, how.turn, how.delay)).collect::<Vec<_>>(),
            vec![
                ([0.0, 7.0, 0.0], Turn::By(0.0), 0),
                ([4.0, 6.0, 0.0], Turn::By(320.0), 200),
                ([-4.0, 5.0, 0.0], Turn::By(40.0), 400),
                ([0.0, 7.0, 0.0], Turn::By(0.0), 0)
            ]
        );
        assert_eq!((hows[0].pace, turn), (2, 1));
    }

    #[test]
    fn what_is_thrown_flies_and_lands() {
        let cannon = launched(1357, 0, &mut 0).unwrap();
        assert_eq!(cannon.files[0].1, Some(Carried::new(Way::Steps).turned(Turn::Flight).delayed(200).looped()));
        assert_eq!(launched(1755, 0, &mut 0).unwrap().files.len(), 2);
        let bottle = launched(2214, 0, &mut 0).unwrap();
        assert_eq!(
            bottle.files.iter().map(|(file, _)| file.as_str()).collect::<Vec<_>>()[..2],
            [
                "acidified_zone_water/acidified_zone_water_c/acidified_zone_water_c.str",
                "acidified_zone_water/acidified_zone_water_b/acidified_zone_water_b.str"
            ]
        );
        assert_eq!(bottle.files[1].1.unwrap().delay, (BOW_LANDS + 1) * TICK_MS);
        assert_eq!(bottle.sounds, vec![(BOW_LANDS, "effect/BO_ACIDIFIED_ZONE.wav".to_owned()), (BOW_LANDS, "effect/BO_ACIDIFIED_ZONE_WATER.wav".to_owned())]);
    }
}
