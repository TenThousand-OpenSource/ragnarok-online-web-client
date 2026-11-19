//! What the client knows of a job by its number alone.

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
    matches!(job, 4023..=4045 | 4096..=4112 | 4158..=4182 | 4191 | 4193..=4196 | 4205..=4210 | 4220 | 4222..=4238 | 4241 | 4242 | 4244 | 4247 | 4248 | 4352 | 4354)
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
    }

    #[test]
    fn fourth_jobs_have_activity_points() {
        // A Dragon Knight, on foot and on its dragon; a Hyper Novice.
        assert!(has_ap(4252) && has_ap(4280) && has_ap(4307));
        // Not a Rune Knight, a Novice, or the gap between the two ranges.
        assert!(!has_ap(4054) && !has_ap(0) && !has_ap(4300));
    }
}
