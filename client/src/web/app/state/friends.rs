//! What the app keeps of the friends list.

/// What the friends list asks before it acts, in the client's message box.
#[derive(Clone, PartialEq)]
pub(in crate::web) enum FriendAsk {
    /// Someone asks to be the player's friend: the account, the character and the name (the client's
    /// `UIJoinFriendAcceptWnd`).
    Wanted(u32, u32, String),
    /// Remove the friend of that account and character?
    Remove(u32, u32),
}

/// The friends list (the client's `CSession::m_friends`), and the Friend side of the window it shares with the
/// party (`UIMessengerGroupWnd` with `mode` 0).
#[derive(Default)]
pub(in crate::web) struct Friends {
    /// The friends, in the order the server told them: a friend's colour goes by its place.
    pub list: Vec<crate::net::friends::Friend>,
    /// The window shows the friends, not the party (the client's `messengerGroupWndInfo.mode`, turned round:
    /// the window opens on the party until the Friend tab or its key is pressed).
    pub shown: bool,
    /// The friend picked in the list.
    pub picked: Option<usize>,
    /// The menu on a friend, opened with the right button: where on the stage, and which friend.
    pub menu: Option<((i32, i32), usize)>,
    /// The Friend Setup window is open beside the list (`UIFriendOptionWnd`).
    pub setup: bool,
    pub asks: Option<FriendAsk>,
}

impl Friends {
    /// Whether someone of that name is a friend (the client's `CSession::IsFriendName`).
    pub fn has(&self, name: &str) -> bool {
        self.list.iter().any(|friend| friend.name == name)
    }
}
