//! What the app keeps for the chat commands.

/// What the chat commands wait to hear of.
#[derive(Default)]
pub(in crate::web) struct Commands {
    /// `/who` was typed: the next count of players the server tells is told in the chat.
    pub(in crate::web::app) who_asked: bool,
    /// The character whose whispers were last asked to be blocked or let in (the client's `refuseWhisperName`).
    pub(in crate::web::app) whisper_name: String,
    /// The characters whose whispers the player has turned away (the client's `g_whisperBlockMgr`).
    pub blocked: std::collections::BTreeSet<String>,
}
