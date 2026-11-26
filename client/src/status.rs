//! The Status window as rules (the client's `UIStatusWnd`): the number it shows for the speed of the
//! character's blows; and what a level told to the player's own character shows on it.

/// The attack speed the window shows for the delay between blows the server keeps: `(2000 - delay) / 10`, cut
/// towards nought.
pub fn attack_speed(delay: i64) -> i64 {
    (2000 - delay) / 10
}

/// The effect on the player's own character when its base level is told (`CGameMode::ReceiveParChange`,
/// `SP_BASELEVEL`, `GameModePackets.cpp:5576`), by the job it is drawn as: `EF_ANGEL2` (338) for a Super
/// Novice and for the Gangsi, the Death Knight and the Collector (4050 to 4052), `EF_ANGEL3` (582) for the
/// Taekwon line, `EF_ANGEL` (371) for everyone else. The client launches it with `levelup.wav` whenever the
/// packet comes, not asking whether the level went up. ZC_NOTIFY_EFFECT, which tells the others, does nothing
/// for the player's own character there: `ReceivePartyMemberEffect` looks it up with `GetGameActorByAID`,
/// which goes through the others only.
pub fn level_effect(job: u16) -> u32 {
    if crate::jobs::super_novice(job) || matches!(job, 4050..=4052) {
        338
    } else if crate::jobs::taekwon(job) {
        582
    } else {
        371
    }
}

/// The effect on the player's own character when its job level is told (the same function, `SP_JOBLEVEL`):
/// `EF_JOBLVUP50` (337) past level 50, `EF_JOBLVUP` (158) up to it. No sound is played by the handler.
pub fn job_level_effect(level: i32) -> u32 {
    if level > 50 { 337 } else { 158 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_level_told_shows_the_effect_of_the_job() {
        assert_eq!([level_effect(0), level_effect(12), level_effect(4008)], [371, 371, 371]);
        assert_eq!([level_effect(23), level_effect(4045), level_effect(4050), level_effect(4052)], [338; 4]);
        assert_eq!([level_effect(4046), level_effect(4047), level_effect(4049)], [582; 3]);
        assert_eq!([job_level_effect(1), job_level_effect(50), job_level_effect(51), job_level_effect(70)], [158, 158, 337, 337]);
    }

    #[test]
    fn the_speed_of_blows_is_counted_from_their_delay() {
        assert_eq!([attack_speed(500), attack_speed(2000), attack_speed(0), attack_speed(195)], [150, 0, 200, 180]);
        // A delay over 2000, and one under nought: cut towards nought both ways.
        assert_eq!([attack_speed(2005), attack_speed(2015), attack_speed(-5)], [0, -1, 200]);
    }
}
