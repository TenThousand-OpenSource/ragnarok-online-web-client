//! What the app keeps of a deal between two players.

use super::*;

/// A deal between two players (the client's `CUIExchangeUI` and what `CSession` keeps for it).
#[derive(Default)]
pub(in crate::web) struct Exchange {
    /// Who asks for a deal, and their level, until the player has answered.
    pub asked: Option<(String, u16)>,
    /// The other of the deal asked for or under way (the client's `exchangeName`), and their level.
    pub partner: String,
    pub level: u16,
    /// The deal's window is open.
    pub open: bool,
    /// What each side has put in, the player's own first: the things, and the zeny.
    pub mine: Vec<Item>,
    pub theirs: Vec<Item>,
    pub zeny: (u32, u32),
    /// Which sides are closed.
    pub closed: (bool, bool),
    /// What the player put in last and the server has not answered yet: where it is carried (0 for zeny),
    /// and how many.
    pub adding: Option<(u16, u32)>,
    /// OK, Trade or Cancel was pressed and waits for the server: the button cannot be pressed again.
    pub locking: bool,
    pub trading: bool,
    pub cancelling: bool,
    /// The box that asks for a screenshot of the deal is ticked.
    pub screenshot: bool,
    /// Trade was pressed with that box ticked: the trade is asked for once the picture is made.
    pub picturing: bool,
    /// The thing whose count is asked for before it goes in: where it is carried, and how many there are.
    pub counting: Option<(u16, u16)>,
}
