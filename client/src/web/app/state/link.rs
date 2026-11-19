//! What the app keeps of the connection for the watch on it.

/// The connection as the watch sees it (`crate::net::link`).
#[derive(Default)]
pub(in crate::web) struct Link {
    /// When something last came from the server, in milliseconds (the client's `m_lastReceiveTick`).
    pub heard_at: f64,
}
