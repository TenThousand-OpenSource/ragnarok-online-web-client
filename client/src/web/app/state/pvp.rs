//! What the app keeps of fighting other players.

/// Fighting other players: what holds on the map the character is on, and the player's standing on a PVP map.
#[derive(Default)]
pub(in crate::web) struct Pvp {
    pub zone: crate::net::pvp::Zone,
    /// The player's rank and how many are on the map, as the server last said (0 until it has).
    pub rank: i32,
    pub total: i32,
    /// The pictures the standing is written in: the actions of the client's `RankFont`, each with its size
    /// and where its origin is; and whether they are being read.
    pub digits: Vec<(String, (usize, usize), (f32, f32))>,
    pub reading: bool,
}
