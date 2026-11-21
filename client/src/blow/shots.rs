//! What a monster sends flying as it strikes from afar, by its job: the arrow of the archers, and the things
//! a few others throw.
//!
//! From the client's `CGameActor::LaunchAttackProjectile` (`Source/Contents/Actors/RenderObject.cpp`), which
//! the monsters' `ST_ATTACK` calls for every blow with the frame of the swing the blow is on
//! (`GetAttackMotion`) as the frame it leaves at: a switch over the job, with no look at a weapon or at how far
//! the monster reaches; and from `CArrowEffect::SendMsg` (`Source/Contents/Effects/Skills/ArrowEffect.cpp`),
//! which names the sprite by the job. A character's arrow is another rule (`CPc::SetState`: the weapon is asked
//! whether it uses arrows).
//!
//! Only the jobs of the classic game are here. Those of later content in the same two switches, and the two
//! other things the first switch launches (`SpawnMandraAttack` for Mandragora, Hydra, Drosera and their like,
//! `LaunchBulletSpriteEffect` for the Disguiser), are not.

/// The arrow's sprite, among the client's sprites (`몬스터\skel_archer_arrow`): what `CArrowEffect` shows for
/// every job it has no other sprite for, and for a character.
pub const ARROW: &str = "몬스터/skel_archer_arrow";

/// The sprite of what a monster of a job sends at the one it strikes, among the client's sprites; nothing for
/// a job that sends nothing.
pub fn shot(job: u16) -> Option<&'static str> {
    Some(match job {
        // JT_ARCHER_SKELETON, JT_ORC_ARCHER, JT_GARGOYLE, JT_GOBLIN_ARCHER, JT_RAYDRIC_ARCHER, JT_KOBOLD_ARCHER,
        // JT_ARCHER_GUARDIAN; JT_G_ARCHER_SKELETON, JT_G_RAYDRIC_ARCHER, JT_G_KOBOLD_ARCHER, JT_G_ORC_ARCHER;
        // JT_MER_ARCHER01 to JT_MER_ARCHER10.
        1016 | 1189 | 1253 | 1258 | 1276 | 1282 | 1285 | 1420 | 1453 | 1455 | 1473 | 6017..=6026 => ARROW,
        // JT_LIVE_PEACH_TREE, JT_G_LIVE_PEACH_TREE.
        1410 | 1550 => "몬스터/PEACH_TREE_BULLET",
        // JT_EVIL_CLOUD_HERMIT, JT_G_EVIL_CLOUD_HERMIT.
        1412 | 1531 => "몬스터/GINSENG_BULLET",
        // JT_STONE_SHOOTER, JT_WOOTAN_SHOOTER.
        1495 => "몬스터/STONE_SHOOTER_BULLET",
        1498 => "몬스터/WOOTAN_SHOOTER_BULLET",
        // JT_PARASITE, JT_G_PARASITE.
        1500 | 1555 => "몬스터/PARASITE_BULLET",
        // JT_POTON_CANON and its three others.
        1664 => "몬스터/CANON_BULLET",
        1665 => "몬스터/CANON_BULLET_1",
        1666 => "몬스터/CANON_BULLET_2",
        1667 => "몬스터/CANON_BULLET_3",
        // JT_LADY_TANEE, JT_RAGGED_ZOMBIE.
        1688 => "몬스터/LADY_TANEE_BULLET",
        1865 => "몬스터/RAGGED_ZOMBIE_BULLET",
        _ => return None,
    })
}

/// How long after a monster's blow is told of its shot leaves (milliseconds), the blow landing after `lands`:
/// the shot leaves on the frame of the swing the blow is on, which is when the blow lands but for the few
/// jobs the client lands later (`super::lands_later`: the Archer Skeleton by the 8 frames its arrow flies).
pub fn leaves_after(lands: f64, job: u16) -> f64 {
    (lands - super::lands_later(job)).max(0.0)
}

/// How long a shot flies (milliseconds) to a target so far from where it leaves, in the world's units (a cell
/// is 5): 8 of the client's frames of 24 milliseconds from 40 away or farther; nearer, the client makes it
/// faster by 40 over the distance (`CArrowEffect::OnProcess`, `speedScale`), so that it flies 5 a frame, a
/// cell, and is there sooner: from the next cell in one frame.
pub fn flies(distance: f32) -> f64 {
    192.0 * f64::from((distance / 40.0).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shot_from_near_is_there_sooner() {
        assert_eq!((flies(40.0), flies(90.0)), (192.0, 192.0));
        // From four cells away, half the time; from the next cell, a frame.
        assert_eq!((flies(20.0), flies(5.0), flies(0.0)), (96.0, 24.0, 0.0));
    }

    #[test]
    fn the_archers_shoot_arrows_and_a_few_others_things_of_their_own() {
        assert!([1016, 1189, 1253, 1258, 1276, 1282, 1285, 1420, 1453, 1455, 1473, 6017, 6026].into_iter().all(|job| shot(job) == Some(ARROW)));
        assert_eq!((shot(1410), shot(1550)), (Some("몬스터/PEACH_TREE_BULLET"), Some("몬스터/PEACH_TREE_BULLET")));
        assert_eq!((shot(1412), shot(1495)), (Some("몬스터/GINSENG_BULLET"), Some("몬스터/STONE_SHOOTER_BULLET")));
        assert_eq!(shot(1498), Some("몬스터/WOOTAN_SHOOTER_BULLET"));
        assert_eq!((shot(1664), shot(1667)), (Some("몬스터/CANON_BULLET"), Some("몬스터/CANON_BULLET_3")));
        assert_eq!((shot(1688), shot(1865)), (Some("몬스터/LADY_TANEE_BULLET"), Some("몬스터/RAGGED_ZOMBIE_BULLET")));
    }

    #[test]
    fn a_monster_that_strikes_by_hand_or_with_a_spell_sends_nothing() {
        // A Poring, a Zombie, an Orc Warrior, a Mandragora (whose root is another effect), a character's job.
        assert!([1002, 1015, 1023, 1020, 11, 0, 6016, 6027].into_iter().all(|job| shot(job).is_none()));
    }

    #[test]
    fn the_shot_leaves_as_the_swing_reaches_its_blow() {
        // An Orc Archer's blow lands on that frame: the arrow leaves as the number rises.
        assert_eq!(leaves_after(288.0, 1189), 288.0);
        // An Archer Skeleton's lands 8 frames later, as its arrow arrives.
        assert_eq!(leaves_after(480.0, 1016), 288.0);
        // An Archer Guardian's lands much later than its arrow.
        assert_eq!((leaves_after(1200.0, 1285), leaves_after(100.0, 1285)), (288.0, 0.0));
    }
}
