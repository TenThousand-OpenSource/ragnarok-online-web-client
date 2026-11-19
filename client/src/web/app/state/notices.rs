//! What the app keeps of what the server's notices tell.

/// What a notice of the server set, for the windows to go by.
pub(in crate::web) struct Notices {
    /// From what share of the weight it may carry the character no longer recovers by itself, in hundredths
    /// (the client's `m_overweightThresholdPercent`: 50 until ZC_RECOVER_PENALTY_OVERWEIGHT tells another).
    pub overweight: i64,
}

impl Default for Notices {
    fn default() -> Self {
        Self { overweight: 50 }
    }
}
