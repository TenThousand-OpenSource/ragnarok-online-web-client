//! What the app keeps of the instances (`crate::instances`).

/// The party's instance and its window (`CSession`'s `memorialDungeonName`, `memorialDungeonState` and the
/// times; `UIMemorialDunWnd`).
#[derive(Default)]
pub(in crate::web) struct Instances {
    /// The instance the server told of; none, and there is no window (the client's empty name).
    pub known: Option<crate::instances::Instance>,
    /// Whether the window shows.
    pub shown: bool,
    /// Whether the player has the window open (`memorialWindowOpen`): off once its cross was pressed while
    /// the instance runs, and then the server telling of the instance again does not bring it back.
    pub wanted: bool,
}
