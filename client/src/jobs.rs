//! What the client knows of a job by its number alone.

/// Whether a job has Activity Points, the third thing after life and spirit: the fourth jobs and the jobs that
/// stand beside them (the client's `CSession::IsTreeGroupThree`: from the Dragon Knight to the end of the fourth
/// jobs, and from the Sky Emperor to the end of the upper extended ones). The job is the one the character is
/// drawn as: its mounted forms are numbered among them.
pub fn has_ap(job: u16) -> bool {
    matches!(job, 4252..=4299 | 4302..=4329)
}

#[cfg(test)]
mod tests {
    use super::has_ap;

    #[test]
    fn fourth_jobs_have_activity_points() {
        // A Dragon Knight, on foot and on its dragon; a Hyper Novice.
        assert!(has_ap(4252) && has_ap(4280) && has_ap(4307));
        // Not a Rune Knight, a Novice, or the gap between the two ranges.
        assert!(!has_ap(4054) && !has_ap(0) && !has_ap(4300));
    }
}
