//! What the client knows of a job by its number alone.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Whether a job has Activity Points, the third thing after life and spirit: the fourth jobs and the jobs that
/// stand beside them (the client's `CSession::IsTreeGroupThree`: from the Dragon Knight to the end of the fourth
/// jobs, and from the Sky Emperor to the end of the upper extended ones). The job is the one the character is
/// drawn as: its mounted forms are numbered among them.
pub fn has_ap(job: u16) -> bool {
    matches!(job, 4252..=4299 | 4302..=4329)
}

/// Whether a job is a baby's (the client's `IsBabyJob`: the ranges of its `JT_…_B`), by the job the character
/// is drawn as.
pub fn baby(job: u16) -> bool {
    matches!(
        job,
        4023..=4045 | 4096..=4112 | 4158..=4182 | 4191 | 4193..=4196 | 4205..=4210 | 4220 | 4222..=4238 | 4241 | 4242
        | 4244 | 4247 | 4248 | 4352 | 4354
    )
}

/// Whether a job is a Super Novice's (the client's `IsSuperNoviceClass`): the Super Novice, its baby, the
/// Expanded Super Novice and its baby. (The client names its later kinds too; they come with later content.)
pub fn super_novice(job: u16) -> bool {
    matches!(job, 23 | 4045 | 4190 | 4191)
}

/// Whether a job is of the Taekwon line, by the job the character is drawn as (the client's
/// `IsTaekwonDisplayJob`).
pub fn taekwon(job: u16) -> bool {
    matches!(job, 4046..=4049 | 4225 | 4226 | 4238 | 4227 | 4239..=4244 | 4302 | 4303 | 4316)
}

/// Whether what is shown under that number is a monster, as the colour of its name is decided
/// (`crate::names::colour`): 1001 up to, not reaching, 3999, and 20021 to 31999.
pub fn monster(job: u16) -> bool {
    (1001..3999).contains(&job) || (20021..=31999).contains(&job)
}

/// Whether a job is a hireling's, a mercenary that is no monster (the client's `IsHirelingJob`: 6018 to 6052).
pub fn hireling(job: u16) -> bool {
    (6018..=6052).contains(&job)
}

/// Whether a job is a homunculus's, or of the mercenaries numbered among them (the client's
/// `CSession::IsCompanionJob`: 6001 to 6052).
pub fn companion(job: u16) -> bool {
    (6001..=6052).contains(&job)
}

/// The job a new Doram character starts as.
pub const SUMMONER: u16 = 4218;
/// The Doram's jobs; every other player job is human. (rodecomp, `CSession::IsDoramJob`.)
const DORAM_JOBS: [u16; 7] = [4217, SUMMONER, 4219, 4220, 4221, 4308, 4315];

/// Whether a job is a Doram's.
pub fn doram(job: u16) -> bool {
    DORAM_JOBS.contains(&job)
}

/// The name of a job's body sprite files, from the list taken out of the original client
/// (`web/data/job_sprites.txt`). A job the client has no sprite for is drawn as a Novice.
pub fn body_sprite(job: u16) -> &'static str {
    static BODIES: OnceLock<HashMap<u16, &'static str>> = OnceLock::new();
    let bodies = BODIES.get_or_init(
        || crate::tables::tabbed(include_str!("web/data/job_sprites.txt"))
            .filter_map(|(job, name)| Some((job.parse().ok()?, name)))
            .collect()
    );
    // The sprites are numbered as the jobs, the later jobs moved down to follow the first ones.
    let sprite_job = if job > 3950 { job - 3950 } else { job };
    bodies.get(&sprite_job).or_else(|| bodies.get(&0)).copied().unwrap_or("초보자")
}

/// The plain job of a mounted or re-dressed job (the client's `JobNames::BaseJob`, written out by
/// `tools/client-tables/job_bases.py` into `web/data/job_bases.txt`); the job itself for any other.
pub fn base(job: u16) -> u16 {
    static BASES: OnceLock<HashMap<u16, u16>> = OnceLock::new();
    BASES
        .get_or_init(
            || crate::tables::tabbed(include_str!("web/data/job_bases.txt"))
                .filter_map(|(job, base)| Some((job.parse().ok()?, base.parse().ok()?)))
                .collect()
        )
        .get(&job)
        .copied()
        .unwrap_or(job)
}

/// The first job a job grew out of, among the classic ones (0 novice, 1 swordman, 2 magician, 3 archer,
/// 4 acolyte, 5 merchant, 6 thief), by the client's own tables: a mounted job is its plain job (`base`), and
/// each job grew out of the one `grew_out_of` names for it (the client's `jobinheritlist.lub`). The jobs that
/// table starts from a Novice (Taekwon, Ninja, Gunslinger, Super Novice) are of the Novice's line; a job it does
/// not name at all (a Doram) has none.
pub fn first_job(job: u16, grew_out_of: impl Fn(u16) -> Option<u16>) -> Option<u16> {
    let mut job = base(job);
    // No line of jobs is ten long: a table gone wrong does not hold this up for ever.
    for _ in 0..10 {
        if job <= 6 {
            return Some(job);
        }
        job = grew_out_of(job)?;
    }
    None
}

/// The client's number for a kind of weapon (its `Weapon_IDs`), by what the server's database calls the kind
/// (the item's `SubType`).
pub fn weapon_kind(kind: &str) -> Option<u16> {
    Some(match kind {
        "Dagger" => 1,
        "1hSword" => 2,
        "2hSword" => 3,
        "1hSpear" => 4,
        "2hSpear" => 5,
        "1hAxe" => 6,
        "2hAxe" => 7,
        "Mace" => 8,
        "2hMace" => 9,
        "Staff" => 10,
        "Bow" => 11,
        "Knuckle" => 12,
        "Musical" => 13,
        "Whip" => 14,
        "Book" => 15,
        "Katar" => 16,
        "Revolver" => 17,
        "Rifle" => 18,
        "Gatling" => 19,
        "Shotgun" => 20,
        "Grenade" => 21,
        "Huuma" => 22,
        "2hStaff" => 23,
        _ => return None,
    })
}

/// What a job is called until the client's own tables are there (`web/data/jobs.rs`, which then names every
/// job): the first jobs' names, and `Job <number>` for any other.
pub fn name(job: u16) -> String {
    let name = match job {
        0 => "Novice",
        1 => "Swordman",
        2 => "Magician",
        3 => "Archer",
        4 => "Acolyte",
        5 => "Merchant",
        6 => "Thief",
        7 => "Knight",
        8 => "Priest",
        9 => "Wizard",
        10 => "Blacksmith",
        11 => "Hunter",
        12 => "Assassin",
        14 => "Crusader",
        15 => "Monk",
        16 => "Sage",
        17 => "Rogue",
        18 => "Alchemist",
        19 => "Bard",
        20 => "Dancer",
        23 => "Super Novice",
        4218 => "Summoner",
        _ => return format!("Job {job}"),
    };
    name.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn babies_are_told_by_their_ranges() {
        // A Baby Novice, a Baby Super Novice, a Baby Rune Knight, a Baby Summoner.
        assert!(baby(4023) && baby(4045) && baby(4096) && baby(4220));
        // A Novice, a High Novice, a Rune Knight, the Expanded Super Novice (its baby is one).
        assert!(!baby(0) && !baby(4001) && !baby(4054) && !baby(4190) && baby(4191));
        assert!(!baby(4046) && !baby(4221) && !baby(4243) && !baby(4353));
    }

    #[test]
    fn the_other_kinds_are_told_by_their_numbers() {
        assert!(super_novice(23) && super_novice(4045) && super_novice(4190) && super_novice(4191) && !super_novice(0) && !super_novice(4023));
        assert!(taekwon(4046) && taekwon(4049) && taekwon(4227) && taekwon(4244) && taekwon(4316) && !taekwon(4050) && !taekwon(4245));
        assert!(monster(1001) && monster(1002) && monster(3998) && monster(20021) && monster(31999));
        assert!(!monster(1000) && !monster(3999) && !monster(20020) && !monster(32000) && !monster(45));
        assert!(hireling(6018) && hireling(6052) && !hireling(6017) && !hireling(6053));
        assert!(companion(6001) && companion(6052) && !companion(6000) && !companion(6053) && !companion(1002));
    }

    #[test]
    fn fourth_jobs_have_activity_points() {
        // A Dragon Knight, on foot and on its dragon; a Hyper Novice.
        assert!(has_ap(4252) && has_ap(4280) && has_ap(4307));
        // Not a Rune Knight, a Novice, or the gap between the two ranges.
        assert!(!has_ap(4054) && !has_ap(0) && !has_ap(4300));
    }

    #[test]
    fn a_doram_is_told_from_a_human() {
        assert!(doram(SUMMONER) && doram(4217) && doram(4315));
        assert!(!doram(0) && !doram(4216) && !doram(4222));
    }

    #[test]
    fn a_job_has_the_body_the_clients_list_names() {
        assert_eq!(body_sprite(0), "초보자");
        // The later jobs are numbered 3950 lower among the sprites; one the list does not have is a Novice.
        assert_eq!(body_sprite(4001), body_sprite(51));
        assert_eq!(body_sprite(60000), body_sprite(0));
    }

    #[test]
    fn a_job_is_of_the_line_of_its_first_job() {
        let grew = |job: u16| match job {
            7 => Some(1),
            4008 => Some(4002),
            4002 => Some(4001),
            4001 => Some(0),
            50 => Some(50),
            _ => None,
        };
        assert_eq!(first_job(3, grew), Some(3));
        assert_eq!(first_job(7, grew), Some(1));
        assert_eq!(first_job(4008, grew), Some(0));
        // One the table does not name has no line; a table that goes round ends.
        assert_eq!(first_job(SUMMONER, grew), None);
        assert_eq!(first_job(50, grew), None);
        assert_eq!(base(7), 7);
    }

    #[test]
    fn a_weapon_is_numbered_as_the_client_numbers_its_kind() {
        assert_eq!((weapon_kind("Dagger"), weapon_kind("Bow"), weapon_kind("2hStaff")), (Some(1), Some(11), Some(23)));
        assert_eq!(weapon_kind("Shield"), None);
    }

    #[test]
    fn a_job_has_a_name_before_the_tables_are_there() {
        assert_eq!((name(0), name(11), name(SUMMONER), name(4008)), ("Novice".to_owned(), "Hunter".to_owned(), "Summoner".to_owned(), "Job 4008".to_owned()));
    }
}
