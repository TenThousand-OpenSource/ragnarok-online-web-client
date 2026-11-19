//! What the app keeps of looking at another player's equipment.

/// Another player's equipment as the server last showed it, and the player's own switch for it.
#[derive(Default)]
pub(in crate::web) struct ViewEquip {
    /// What is shown in the window, while it is open (the client's `m_equipViewName`, `m_equipViewLook` and
    /// the things it keeps apart for the window).
    pub seen: Option<crate::net::viewequip::Seen>,
    /// Others may look at the player's own equipment (the client's `m_allowEquipView`).
    pub allowed: bool,
}
