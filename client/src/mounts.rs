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
    include_str!("job_mounts.txt")
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            Some((fields.next()?.parse::<u32>().ok()?, fields.next()?, fields.next()?.parse::<u16>().ok()?))
        })
        // What a character is dressed as (a line for any job) gives way to one in the air (bit 15), as in the
        // client.
        .find(|(bit, of, _)| set(*bit) && if *of == "*" { !set(15) } else { of.parse() == Ok(job) })
        .map_or(job, |(_, _, drawn)| drawn)
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
    use super::{drawn_job, off};

    #[test]
    fn the_off_button_takes_away_what_the_state_has() {
        assert_eq!(off(0), (0, None));
        assert_eq!(off(1 << 5), (4, Some(442)));
        assert_eq!(off(1 << 19), (1, Some(1463)));
        // A falcon goes before a bird.
        assert_eq!(off(1 << 4 | 1 << 5), (5, Some(441)));
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
