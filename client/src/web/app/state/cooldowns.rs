//! What the app keeps of the waits before skills may be used again.

/// The waits before skills may be used again (the client's `m_requestTimers`).
#[derive(Default)]
pub(in crate::web) struct Cooldowns {
    pub waits: crate::cooldowns::Waits,
}
