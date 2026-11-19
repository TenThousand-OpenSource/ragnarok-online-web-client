//! What a hit shows over the one struck as it lands, and what it sounds like there.
//!
//! From the client's handler of the messages a hit becomes (`CGameActor::SendMsg`, `AM_ATTACKED` and its
//! like): which numbers and words it makes (`AM_MAKE_NUMBER`, `AM_MAKE_MISS`, `AM_MAKE_LUCKY`,
//! `AM_MAKE_CRITICAL_EFFECT`, `AM_MAKE_GUARD`) and which sound it plays (`CSession::GetHitWaveName`,
//! `CSession::GetWeaponHitWaveName`, `CSession::InitJobHitWaveNames`).

use super::flight::Flight;
use super::landing::How;

/// The colours the client makes its numbers in.
pub const WHITE: u32 = 0xffffff;
pub const RED: u32 = 0xff0000;
pub const YELLOW: u32 = 0xffff00;
/// Life recovered, and spirit.
pub const GREEN: u32 = 0x00ff00;
pub const BLUE: u32 = 0x0000ff;

/// The colour of the number the server has shown over someone with one of the client's effects
/// (ZC_NOTIFY_EFFECT3, the client's `ReceiveActorStatusValue`): life and spirit that come back in time
/// (`EF_HPTIME`, `EF_SPTIME`) and the eight effects that are a number in a colour (`EF_GREEN_NUMBER` to
/// `EF_PINK_NUMBER`); none for any other effect.
pub fn number_colour(effect: u32) -> Option<u32> {
    Some(match effect {
        331 | 657 => GREEN,
        332 | 658 => BLUE,
        659 => RED,
        660 => 0xff00ff,
        661 => 0x000000,
        662 => WHITE,
        663 => YELLOW,
        664 => 0xff55b1,
        _ => return None,
    })
}

/// One thing a hit shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sign {
    /// A number over the one struck, in a colour, flying as its kind does.
    Number(i32, u32, Flight),
    /// The mark of a critical blow.
    CriticalMark,
    /// The blue word over the total of a kick.
    Word,
    /// The word of a blow dodged by luck.
    Lucky,
    /// The word of a miss, which the client shows over the one who STRUCK: red when that is a player.
    Miss { red: bool },
    /// The client's effect of a blow warded off (`EF_GUARD`), on the one struck.
    Guard,
}

/// Who a hit is between, as far as what it shows goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Between {
    /// The one struck is a player's character (its numbers are red and it shows no totals).
    pub struck_is_character: bool,
    /// The one who struck is a player's character; none when that one is not in sight.
    pub striker_is_character: Option<bool>,
    /// The one struck is the player's own character under Kyrie Eleison or Parrying.
    pub guarded: bool,
}

/// What a hit shows as it lands.
pub fn signs(how: How, damage: i32, total: i32, between: Between) -> Vec<Sign> {
    if how == How::Lucky {
        return vec![Sign::Lucky];
    }
    if damage == 0 {
        // A blow that does nothing: said over the striker, if it is in sight; a monster's blow on the player's
        // own guarded character shows the guard instead.
        return match between.striker_is_character {
            Some(false) if between.guarded => vec![Sign::Guard],
            Some(is_character) => vec![Sign::Miss { red: is_character }],
            None => Vec::new(),
        };
    }
    if damage < 0 {
        return Vec::new();
    }
    match how {
        How::Critical => vec![Sign::CriticalMark, Sign::Number(damage, YELLOW, Flight::Critical)],
        _ if between.struck_is_character => vec![Sign::Number(damage, RED, Flight::Number)],
        How::Kick => vec![Sign::Word, Sign::Number(total, WHITE, Flight::Total)],
        How::Many => vec![Sign::Number(damage, WHITE, Flight::Number), Sign::Number(total, YELLOW, Flight::Total)],
        How::ManyCritical => vec![Sign::CriticalMark, Sign::Number(damage, YELLOW, Flight::Critical), Sign::Number(total, YELLOW, Flight::Total)],
        _ => vec![Sign::Number(damage, WHITE, Flight::Number)],
    }
}

/// Whether the client shows the other hand's damage of a blow for a striker of that job: an NPC or a monster,
/// an assassin, or a job over 4000 (`ProcessHitMessage`).
pub fn two_handed(job: u16, is_character: bool) -> bool {
    !is_character || job == 12 || job > 4000
}

/// The sound of a player's character being struck, by what its job wears (`InitJobHitWaveNames`: cloth
/// unless the job is listed as wearing leather or metal).
pub fn struck_sound(job: u16) -> &'static str {
    match job {
        3 | 6 | 11..=12 | 17 | 19..=20 | 24..=25 | 4004 | 4007 | 4012..=4013 | 4018 | 4020..=4021 | 4026 | 4029 | 4034..=4035 | 4040 | 4042..=4043 | 4046 | 4050 | 4056 | 4059 | 4062 | 4065 | 4068..=4069 | 4072 | 4075..=4076 | 4079 | 4211..=4212 | 4222..=4225 | 4228..=4229 | 4254 | 4257 | 4260 | 4263..=4264 | 4304..=4306 => {
            "player_wooden_male.wav"
        }
        1 | 7 | 14..=15 | 4002 | 4008 | 4015..=4016 | 4024 | 4030 | 4037..=4038 | 4047..=4048 | 4051 | 4054 | 4060 | 4066 | 4070 | 4073 | 4077 | 4226 | 4238..=4239 | 4241 | 4252 | 4255 | 4258 | 4262 | 4280..=4281 | 4302 | 4316 => "player_metal.wav",
        _ => "player_clothes.wav",
    }
}

/// The sound of a weapon's blow landing on a monster. `kind` is the client's number for the kind of weapon
/// the striker holds (0: bare hands; none: the hit of a skill), `named` what the client's weapon table has for
/// a kind (`ReqWeaponHitWaveName`), and `lot` a number under 4 drawn for the sounds the client has four of.
pub fn weapon_sound(kind: Option<u16>, named: impl Fn(u16) -> Option<String>, lot: u32) -> String {
    match kind {
        None => format!("_enemy_hit_normal{}.wav", 1 + lot % 4),
        Some(0) => format!("_hit_fist{}.wav", 1 + lot % 4),
        // A kind the table does not name sounds as a mace does (what the client fills its list with).
        Some(kind) => named(kind).filter(|name| !name.is_empty()).unwrap_or_else(|| "_hit_mace.wav".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ON_MONSTER: Between = Between { struck_is_character: false, striker_is_character: Some(true), guarded: false };
    const ON_PLAYER: Between = Between { struck_is_character: true, striker_is_character: Some(false), guarded: false };

    #[test]
    fn a_monster_struck_shows_white_and_a_player_red() {
        assert_eq!(signs(How::Plain, 50, 0, ON_MONSTER), [Sign::Number(50, WHITE, Flight::Number)]);
        assert_eq!(signs(How::Skill, 50, 0, ON_PLAYER), [Sign::Number(50, RED, Flight::Number)]);
        // A critical blow is yellow whoever takes it.
        assert_eq!(signs(How::Critical, 50, 0, ON_PLAYER), [Sign::CriticalMark, Sign::Number(50, YELLOW, Flight::Critical)]);
    }

    #[test]
    fn several_hits_show_the_total_in_yellow_over_a_monster_only() {
        assert_eq!(signs(How::Many, 100, 300, ON_MONSTER), [Sign::Number(100, WHITE, Flight::Number), Sign::Number(300, YELLOW, Flight::Total)]);
        assert_eq!(signs(How::Many, 100, 300, ON_PLAYER), [Sign::Number(100, RED, Flight::Number)]);
        assert_eq!(signs(How::Kick, 100, 100, ON_MONSTER), [Sign::Word, Sign::Number(100, WHITE, Flight::Total)]);
        assert_eq!(signs(How::ManyCritical, 100, 200, ON_MONSTER).len(), 3);
    }

    #[test]
    fn a_miss_is_said_over_the_striker_and_luck_over_the_one_struck() {
        assert_eq!(signs(How::Plain, 0, 0, ON_MONSTER), [Sign::Miss { red: true }]);
        assert_eq!(signs(How::Plain, 0, 0, ON_PLAYER), [Sign::Miss { red: false }]);
        assert_eq!(signs(How::Plain, 0, 0, Between { guarded: true, ..ON_PLAYER }), [Sign::Guard]);
        // A player's blow is not warded off so.
        assert_eq!(signs(How::Plain, 0, 0, Between { guarded: true, striker_is_character: Some(true), ..ON_PLAYER }), [Sign::Miss { red: true }]);
        assert_eq!(signs(How::Plain, 0, 0, Between { striker_is_character: None, ..ON_PLAYER }), []);
        assert_eq!(signs(How::Lucky, 0, 0, ON_PLAYER), [Sign::Lucky]);
    }

    #[test]
    fn the_numbers_the_server_shows_with_an_effect() {
        assert_eq!(number_colour(331), Some(GREEN));
        assert_eq!(number_colour(658), Some(BLUE));
        assert_eq!(number_colour(664), Some(0xff55b1));
        assert_eq!(number_colour(7), None);
    }

    #[test]
    fn what_a_struck_job_sounds_like() {
        assert_eq!(struck_sound(0), "player_clothes.wav");
        assert_eq!(struck_sound(7), "player_metal.wav");
        assert_eq!(struck_sound(4008), "player_metal.wav");
        assert_eq!(struck_sound(11), "player_wooden_male.wav");
        assert_eq!(struck_sound(8), "player_clothes.wav");
    }

    #[test]
    fn what_a_weapon_sounds_like() {
        let table = |kind: u16| (kind == 2).then(|| "_hit_sword.wav".to_owned());
        assert_eq!(weapon_sound(Some(2), table, 0), "_hit_sword.wav");
        assert_eq!(weapon_sound(Some(30), table, 0), "_hit_mace.wav");
        assert_eq!(weapon_sound(Some(0), table, 6), "_hit_fist3.wav");
        assert_eq!(weapon_sound(None, table, 3), "_enemy_hit_normal4.wav");
        assert!(two_handed(1002, false) && two_handed(12, true) && two_handed(4013, true) && !two_handed(7, true));
    }
}
