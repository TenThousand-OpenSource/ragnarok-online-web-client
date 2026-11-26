//! What the app keeps of the bank.

/// The bank (`UIBank_NewWnd`, and `CSession::m_bankBalance`).
#[derive(Default)]
pub(in crate::web) struct Bank {
    pub open: bool,
    /// What the bank holds, as the server last said.
    pub balance: i64,
    /// What stands in the field, as the last button left it: the window is drawn with it.
    pub typed: String,
    /// The short line at the window's foot after a sum the client would not send: its line of the message
    /// table (`UIBank_NewWnd::m_status`).
    pub status: Option<usize>,
}
