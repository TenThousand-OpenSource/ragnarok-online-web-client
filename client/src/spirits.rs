//! The spirit spheres about a monk and the coins about a gunslinger, as rules: which of the client's effects
//! a job shows them as, and where each sphere of that effect begins (`CRagEffect::ElementalOrbit`,
//! `Contents/Effects/Core/RagEffect.cpp`). Kept by `web/world/spirits.rs`, launched by
//! `web/world/prims/spirits.rs`, drawn as the primitive kind 32 (`prim/tei/spheres.rs`).
//!
//! The server tells how many someone has with ZC_SPIRITS, and with ZC_SPIRITS2 of one who comes into sight
//! (`CGameMode::UpdateActorSpiritCount` and `CGameMode::Zc_Spirits2`, `Client/Mode/GameModePackets.cpp`).
//!
//! Not this: the client's `CEnergyOrbEffect` (`Contents/Effects/Skills/EnergyOrbEffect.cpp`) is not what
//! these two packets show. It serves `SetCharm` and `EFST_GENSOU`, which are of later content, and is not
//! ported.

/// `EF_CHOOKGI`, `EF_CHOOKGI2` and `EF_CHOOKGI3`: the spheres of a monk, of a baby monk, the coins of a
/// gunslinger.
pub const SPHERES: u32 = 228;
pub const SMALL_SPHERES: u32 = 504;
pub const COINS: u32 = 629;

/// The three, for what ends whichever of them is on someone.
pub const EFFECTS: [u32; 3] = [SPHERES, SMALL_SPHERES, COINS];

/// The effect a job shows its spheres as: `EF_CHOOKGI3` for `JT_GUNSLINGER` (24) and `JT_GUNSLINGER_B`
/// (4228), `EF_CHOOKGI2` for `JT_MONK_B` (4038), `EF_CHOOKGI` for every other (the `default` of both
/// handlers: whoever the server tells it of).
///
/// Nothing for the jobs both handlers treat in other ways, with effects of later content that are not ported:
/// `JT_REBELLION` (4215), `JT_REBELLION_B` (4229) and `JT_NIGHT_WATCH` (4306), whose coins are
/// `CRichsCoinEffect`; `JT_DRAGON_KNIGHT` (4252, riding 4265 and 4280), whose weapons are
/// `CServantWeaponNewEffect`; `JT_ABYSS_CHASER` (4260), the effect 2109.
pub fn effect(job: u16) -> Option<u32> {
    match job {
        24 | 4228 => Some(COINS),
        4038 => Some(SMALL_SPHERES),
        4215 | 4229 | 4306 | 4252 | 4265 | 4280 | 4260 => None,
        _ => Some(SPHERES),
    }
}

/// Whether someone in that effect state shows none: one who hides (its bit 1) or cloaks (bit 2), the
/// client's `StateFlag01` and `StateFlag02`. `Zc_Spirits2` ends the effect on such a one and launches none.
pub fn hidden(state: u32) -> bool {
    state & 6 != 0
}

/// A sphere as `ElementalOrbit` launches one: how far round the one it is about it begins, in degrees
/// (`height[6]`), how high over the feet (`height[7]`, the client's y, which goes down), how far from its
/// middle to a corner it reaches (`distance`), whether it goes round nearer (`height[10]`), and the flag
/// that says how it turns and in which colours it is drawn (`flag1[0]`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere {
    pub round: f32,
    pub high: f32,
    pub out: f32,
    pub near: bool,
    pub flag: i8,
}

/// `CRagEffect::ElementalOrbit` for an effect told `told` spheres (the first figure of its `deltaPos`, told
/// with `AM_SETPOS`): the spheres it launches at its first tick, five at most and none for a count of
/// nothing; none for any other effect. The client calls it with the kind 0 for `EF_CHOOKGI`, 1 for
/// `EF_CHOOKGI2` and 3 for `EF_CHOOKGI3`:
///
/// - kind 0: each sphere 72 degrees on from the one before, 15 over the feet, reaching 2. Told more than
///   five, the spheres whose number is at least six under the count are flagged 20, and told more than
///   ten all are, and those at least eleven under it 21 (drawn larger and bluer: one sphere for six);
/// - kind 1: the same, 14 over the feet, reaching 1.3, going round nearer;
/// - kind 3: the spheres spread evenly over the round (72 degrees apart once told more than five), 15
///   over the feet, reaching 1 (2 for those at least six under a count over five), flagged 1.
///
/// (Its kinds 4 and 10 to 13, the coins of `rcoin1.bmp` and the four elemental spirits, are of other
/// effects of later content and are not here.)
pub fn spheres(effect: u32, told: f32) -> Vec<Sphere> {
    let count = (told as i32).min(5);
    (0..count.max(0)).filter_map(|index| {
        let (angle, under) = ((index * 72) as f32, told - index as f32);
        Some(match effect {
            SMALL_SPHERES => Sphere { round: angle, high: -14.0, out: 1.3, near: true, flag: 0 },
            COINS => Sphere { round: if told > 5.0 { angle } else { (index * 360) as f32 / told }, high: -15.0, out: if told > 5.0 && under >= 6.0 { 2.0 } else { 1.0 }, near: false, flag: 1 },
            SPHERES => {
                let flag = match () {
                    () if told > 10.0 => if under >= 11.0 { 21 } else { 20 },
                    () if told > 5.0 && under >= 6.0 => 20,
                    () => 0,
                };
                Sphere { round: angle, high: -15.0, out: 2.0, near: false, flag }
            }
            _ => return None,
        })
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_job_shows_its_spheres_as_its_own_effect() {
        // A monk and whoever else the server tells it of; a baby monk; a gunslinger and a baby one.
        assert_eq!((effect(15), effect(4016), effect(0)), (Some(SPHERES), Some(SPHERES), Some(SPHERES)));
        assert_eq!(effect(4038), Some(SMALL_SPHERES));
        assert_eq!((effect(24), effect(4228)), (Some(COINS), Some(COINS)));
        // The later jobs' own effects are not ported: nothing is shown for them.
        assert!([4215, 4229, 4306, 4252, 4265, 4280, 4260].into_iter().all(|job| effect(job).is_none()));
    }

    #[test]
    fn one_who_hides_or_cloaks_shows_none() {
        assert!(hidden(2) && hidden(4) && hidden(6 | 1));
        assert!(!hidden(0) && !hidden(1) && !hidden(0x40));
    }

    #[test]
    fn a_monk_has_up_to_five_spheres_a_fifth_of_a_round_apart() {
        assert!(spheres(SPHERES, 0.0).is_empty());
        let three = spheres(SPHERES, 3.0);
        assert_eq!(three.iter().map(|sphere| sphere.round).collect::<Vec<_>>(), [0.0, 72.0, 144.0]);
        assert!(three.iter().all(|sphere| (sphere.high, sphere.out, sphere.near, sphere.flag) == (-15.0, 2.0, false, 0)));
        assert_eq!(spheres(SPHERES, 5.0).len(), 5);
        // Told seven there are five, the first two of them standing for six each.
        assert_eq!(spheres(SPHERES, 7.0).iter().map(|sphere| sphere.flag).collect::<Vec<_>>(), [20, 20, 0, 0, 0]);
        // Told twelve, the first two stand for eleven each and the others for six.
        assert_eq!(spheres(SPHERES, 12.0).iter().map(|sphere| sphere.flag).collect::<Vec<_>>(), [21, 21, 20, 20, 20]);
    }

    #[test]
    fn a_baby_monk_has_them_smaller_lower_and_nearer() {
        let two = spheres(SMALL_SPHERES, 2.0);
        assert_eq!(two, [Sphere { round: 0.0, high: -14.0, out: 1.3, near: true, flag: 0 }, Sphere { round: 72.0, high: -14.0, out: 1.3, near: true, flag: 0 }]);
    }

    #[test]
    fn a_gunslinger_has_its_coins_spread_evenly_over_the_round() {
        assert_eq!(spheres(COINS, 3.0).iter().map(|coin| coin.round).collect::<Vec<_>>(), [0.0, 120.0, 240.0]);
        assert!(spheres(COINS, 4.0).iter().all(|coin| (coin.high, coin.out, coin.near, coin.flag) == (-15.0, 1.0, false, 1)));
        // Told more than five they are a fifth of a round apart, and those that stand for six reach 2.
        let ten = spheres(COINS, 10.0);
        assert_eq!(ten.iter().map(|coin| (coin.round, coin.out)).collect::<Vec<_>>(), [(0.0, 2.0), (72.0, 2.0), (144.0, 2.0), (216.0, 2.0), (288.0, 2.0)]);
        assert!(spheres(7, 3.0).is_empty());
    }
}
