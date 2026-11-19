//! What the app keeps of the party.

use super::*;
use crate::net::party;

/// What the party asks before it acts, in the client's message box.
#[derive(Clone, PartialEq)]
pub(in crate::web) enum PartyAsk {
    /// A party invites the player: its number and its name.
    Invited(u32, String),
    /// Leave the party?
    Leave,
    /// Leave it though nobody is there to lead it on: the party is no more then.
    Disband,
    /// Expel the member of that name?
    Expel(String),
    /// Hand the party to the member of that account?
    HandOver(u32),
}

/// The window that stands beside the party window.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum PartySide {
    /// The party's setup: the client's `UIPartySettingWnd`, 245 tall for a party there is.
    Setup,
    /// The same window making a party: 220 tall with a name to type (the window's Create button), or 180
    /// tall for a name already given (`/organize`).
    Create,
    Named,
    /// Whom to invite (`UIPartyInvitationToWnd`).
    Invite,
}

/// The party: who is in it, what is known of each, and its windows.
#[derive(Default)]
pub(in crate::web) struct Party {
    pub name: String,
    pub members: Vec<party::Member>,
    /// How it shares, as the server last said.
    pub sharing: party::Sharing,
    /// Each member's life (left and most), cell on this map, and who of them lies dead: by account.
    pub life: HashMap<u32, (i32, i32)>,
    pub places: HashMap<u32, (u16, u16)>,
    pub dead: std::collections::HashSet<u32>,
    /// The party window is open; in its small form; the member picked in it.
    pub open: bool,
    pub small: bool,
    pub picked: Option<usize>,
    /// The window beside it, and what is set in it until OK sends it.
    pub side: Option<PartySide>,
    pub draft: party::Sharing,
    /// The name of the party being made (the client's `partyRequestName`).
    pub making: String,
    pub asks: Option<PartyAsk>,
    /// The menu on a member, opened with the right button: where on the stage, and which member.
    pub menu: Option<((i32, i32), usize)>,
}
