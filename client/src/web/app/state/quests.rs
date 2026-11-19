//! What the app keeps of the quests.

/// The quests: the journal as the server has it, its windows, and the marks over NPCs.
#[derive(Default)]
pub(in crate::web) struct Quests {
    /// The journal, by the quest's number (the client's `m_ongoingQuests`).
    pub log: std::collections::BTreeMap<u32, crate::net::quests::Quest>,
    /// The journal's window is open; which of its tabs (`crate::quests::tab`), which filter of its lower row
    /// (the client's number for it), and what was searched for.
    pub open: bool,
    pub tab: usize,
    pub filter: u32,
    pub search: String,
    /// The quest whose pane is open beside the journal: its number, and whether it is one of the recommended.
    pub pane: Option<(u32, bool)>,
    /// The recommended were looked at already: the first look opens the first of them.
    pub recommended_seen: bool,
    /// The button that tells of a new quest shows, until the journal is opened.
    pub notice: bool,
    /// What the browser keeps, once it was read (`remembered`): the list of quests on the screen is switched
    /// off; the quests left out of it; what of the recommended was put away.
    pub remembered: bool,
    pub display_off: bool,
    pub unlisted: std::collections::BTreeSet<u32>,
    pub passed: std::collections::BTreeSet<u32>,
    /// The marks over NPCs: the visit to a map they are of (`world::visit`), and each by its NPC.
    pub marks: (u32, std::collections::BTreeMap<u32, crate::net::quests::Mark>),
}
