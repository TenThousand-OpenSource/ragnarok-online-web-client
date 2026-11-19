//! What the app keeps of the family: who the character is married to, and who asks to adopt it.

use crate::net::family::Parent;

/// The family.
#[derive(Default)]
pub(in crate::web) struct Family {
    /// What the player's partner is called, as the server last told (the client's `coupleName`); empty for
    /// one who has none.
    pub partner: String,
    /// Who asks to adopt the character, while the question stands; and whether the first question was
    /// answered with OK, so that the second one stands (the client's `UIBabyAcceptWnd`, then
    /// `UIBabyAcceptWnd2`).
    pub asks: Option<(Parent, bool)>,
}
