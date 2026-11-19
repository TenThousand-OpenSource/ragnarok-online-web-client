//! What a job is drawn as in a state the server tells of: on its bird, its dragon or its wolf, in its machine,
//! in the air, dressed for a wedding.
//!
//! The server tells a character's job and, apart, a set of bits for the state it is in (its `option`, the
//! client's "effect state"). The client draws another job's body for some of those (`CGameActor::
//! GetEffectiveJob`); `job_mounts.txt` is its table, written out by `tools/client-tables/job_mounts.py`.

/// The job whose body is drawn for a character of that job in that state: the job itself where the client's
/// table has nothing for it.
pub fn drawn_job(job: u16, state: u32) -> u16 {
    let set = |bit: u32| state >> bit & 1 == 1;
    crate::tables::tabbed(include_str!("job_mounts.txt"))
        .filter_map(|(bit, rest)| {
            let mut fields = rest.split('\t');
            Some((bit.parse::<u32>().ok()?, fields.next()?, fields.next()?.parse::<u16>().ok()?))
        })
        // What a character is dressed as (a line for any job) gives way to one in the air (bit 15), as in the
        // client.
        .find(|(bit, of, _)| set(*bit) && if *of == "*" { !set(15) } else { of.parse() == Ok(job) })
        .map_or(job, |(_, _, drawn)| drawn)
}

/// How high over where its name would stand the name of a character stands, by the job it is drawn as
/// (`drawn_job`) and whether it is a Doram (`CGameActor::GetNameOffset`): what the signs of Fury and its like
/// are lifted by (`crate::prim::Prim::lifted`). 6 for one on a bird, a dragon or a wolf or in a machine, 4 for
/// a baby so mounted; else -1 for a baby and -2 for a Doram; nothing for anyone else. A monster or an NPC has
/// none.
///
/// Left out of the client's rule, as this client does not know of them: one riding a rented mount
/// (`isRiding`: 6, 2 for a Novice's kind, 0 for a Gunslinger's, 4 for a baby, -3 for a Doram), and one shrunk
/// by an effect (its `render.flags & 0x10`, which makes all of these smaller).
pub fn name_offset(drawn: u16, doram: bool) -> f32 {
    // `IsBabyJob`, the ranges of the client's `JT_…_B`.
    let baby = crate::jobs::baby(drawn);
    match drawn {
        // `JT_CHICKEN`, `JT_CHICKEN2` and theirs of the high jobs; the third jobs' mounts from `JT_RUNE_CHICKEN`
        // to `JT_RUNE_CHICKEN5_H`; `JT_WOLF_WINDHAWK` to `JT_IMPERIAL_GUARD_CHICKEN`.
        13 | 21 | 4014 | 4022 | 4080..=4095 | 4278..=4281 => 6.0,
        // The babies' mounts: `JT_CHICKEN_B`, `JT_CHICKEN2_B`, `JT_RUNE_CHICKEN_B` to `JT_MADOGEAR_B`.
        4036 | 4044 | 4109..=4112 => 4.0,
        _ if baby => -1.0,
        _ if doram => -2.0,
        _ => 0.0,
    }
}

/// What the Equipment window's "off" button asks the server to take away from a character in that state, and
/// the line of the client's message table that the button says (none: nothing to take away but a cart). As the
/// client's `UIEquipWnd` has them: a falcon first, then a bird, a machine, a dragon; the number is what the
/// request (`CZ_UNINSTALLATION`, 0x0b35) carries.
pub fn off(state: u32) -> (u8, Option<usize>) {
    let set = |bit: u32| state >> bit & 1 == 1;
    if set(4) {
        (5, Some(441))
    } else if set(5) {
        (4, Some(442))
    } else if set(22) {
        (3, Some(1464))
    } else if [19, 23, 24, 25, 26].into_iter().any(set) {
        (1, Some(1463))
    } else {
        (0, None)
    }
}

#[cfg(test)]
mod tests {
    use super::{drawn_job, name_offset, off};

    #[test]
    fn the_off_button_takes_away_what_the_state_has() {
        assert_eq!(off(0), (0, None));
        assert_eq!(off(1 << 5), (4, Some(442)));
        assert_eq!(off(1 << 19), (1, Some(1463)));
        // A falcon goes before a bird.
        assert_eq!(off(1 << 4 | 1 << 5), (5, Some(441)));
    }

    #[test]
    fn a_name_stands_higher_over_one_who_is_mounted_and_lower_over_a_baby_or_a_doram() {
        // A Lord Knight on a Peco Peco, a Rune Knight on a dragon, a baby Knight on a Peco Peco.
        assert_eq!((name_offset(4014, false), name_offset(4080, false), name_offset(4036, false)), (6.0, 6.0, 4.0));
        // A baby Swordman, a Summoner, a baby Summoner (a baby first), a Knight on foot.
        assert_eq!((name_offset(4024, false), name_offset(4218, true), name_offset(4220, true), name_offset(7, false)), (-1.0, -2.0, -1.0, 0.0));
    }

    #[test]
    fn a_job_in_no_state_is_itself() {
        assert_eq!(drawn_job(4252, 0), 4252);
        // A state that means nothing to that job, too: a Novice told it rides a dragon.
        assert_eq!(drawn_job(0, 1 << 19), 0);
    }

    #[test]
    fn knights_ride() {
        // A Knight on a Peco Peco, a Rune Knight and a Dragon Knight on their dragons, a Ranger on a wolf.
        assert_eq!(drawn_job(7, 1 << 5), 13);
        assert_eq!(drawn_job(4054, 1 << 19), 4080);
        assert_eq!(drawn_job(4252, 1 << 19), 4280);
        assert_eq!(drawn_job(4056, 1 << 21), 4084);
        // The dragons of other colours are other bits.
        assert_eq!(drawn_job(4054, 1 << 23), 4088);
    }

    #[test]
    fn a_wedding_dress_is_for_any_job_but_not_in_the_air() {
        assert_eq!(drawn_job(4252, 1 << 12), 22);
        assert_eq!(drawn_job(4047, 1 << 12 | 1 << 15), 4048);
    }
}
