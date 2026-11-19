//! What the app keeps of a fight beside the blows: the two windows it opens.

/// The windows a fight opens.
#[derive(Default)]
pub(in crate::web) struct Fight {
    /// What Sense told of a monster, with the monster's name: the Monster Info window shows it while this
    /// is there (the client's `UIMonsterInfoWnd`, which keeps the packet).
    pub sensed: Option<(crate::net::fight::Sensed, String)>,
    /// The place the server asks about: whether this map is to be the place of the sun (0), the moon (1) or
    /// the stars (2) (`UIStarPlaceAcceptWnd::m_place`).
    pub star: Option<u8>,
    /// Whom the character last used a skill on: Sense names the monster it tells of by this (the client's
    /// `m_monsterInfoActorId`).
    pub aimed: Option<u32>,
    /// The MVP's reward, until its name is known and it is said in the chat.
    pub mvp_item: Option<u32>,
}
