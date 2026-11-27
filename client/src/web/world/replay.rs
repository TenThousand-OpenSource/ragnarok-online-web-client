//! What a recording keeps of the map as it begins: everyone in sight as they stand now, the portals, the
//! things and the skills' units on the ground, the names told of the players, which way the player's own
//! character faces and what the server's clock shows. The Windows client writes its actors the same way, each
//! anew from what the actor is at that moment (`CGameMode::SaveActorStates`, `BuildActorEntryPacket`).

use super::*;

/// Someone in sight, as far as the world keeps what the packet that showed it told (`net::actors::Unit`).
/// Not kept, and so not here: the bits of the third option, a player's robe and character's number.
pub struct InSight {
    pub id: u32,
    pub kind: Kind,
    pub job: u16,
    /// The bits of the state it is in, and its body's and health's states.
    pub state: u32,
    pub body: u16,
    pub health: u16,
    /// Milliseconds a cell walked.
    pub speed: u16,
    /// A character's parts; nought for one drawn from a sprite of its own.
    pub sex: u8,
    pub hair_style: u16,
    pub hair_colour: u16,
    pub body_palette: u16,
    pub headgear: [u16; 3],
    pub weapon: u32,
    pub shield: u32,
    /// Which way its head is turned.
    pub head: u8,
    pub level: u16,
    pub boss: u8,
    /// A monster's life, left and most, where the server told it.
    pub life: Option<(i32, i32)>,
    pub name: String,
    /// The cell it is on now (one that walks: the cell it has come to), and which way it faces, as the
    /// server counts directions.
    pub cell: (u16, u16),
    pub direction: u8,
    /// The cell it walks to, when it walks.
    pub walks_to: Option<(u16, u16)>,
    pub sitting: bool,
    pub dead: bool,
    /// A player's guild and the version of its emblem.
    pub guild: (u32, u32),
    /// The names of a player's party and guild, where the server told them.
    pub party_name: String,
    pub guild_name: String,
}

/// What a thing lying on the ground is: its ID, the item, whether it is identified, how many, its cell and
/// where within the cell (in sixteenths).
pub type Lain = (u32, u32, bool, u16, (u16, u16), (u8, u8));

/// The map as a recording keeps it.
#[derive(Default)]
pub struct Snapshot {
    pub units: Vec<InSight>,
    /// The portals: each its ID and its cell.
    pub portals: Vec<(u32, (u16, u16))>,
    pub drops: Vec<Lain>,
    /// The skills' units: each its ID, its cell, its kind, and whether it is to be seen.
    pub skill_units: Vec<(u32, (u16, u16), u32, bool)>,
    /// Which way the player's own character faces, as the server counts directions.
    pub direction: u8,
    /// The server's time now, by the clock everyone's walk is shown by.
    pub server_time: u32,
}

fn cell_of((x, y): (i32, i32)) -> (u16, u16) {
    (x.clamp(0, 1023) as u16, y.clamp(0, 1023) as u16)
}

/// One in sight as the world has it now. One that is fading out of sight, or was struck down, is no more.
fn seen(state: &State, id: u32, mover: &Mover) -> Option<InSight> {
    if mover.pose.vanishes.is_some() || mover.dying.is_some() {
        return None;
    }
    let none = Figure { job: 0, sex: 0, hair_style: 0, hair_colour: 0, headgear: [0; 3], body_palette: 0, weapon: 0, shield: 0, action: 0, frame: 0 };
    let figure = if let Look::Character(figure) = &mover.look { figure } else { &none };
    let (card, states) = (state.cards.get(&id), mover.ail.states());
    let (party_name, guild_name) = state.guilds.names(id).map_or_else(Default::default, |(party, guild)| (party.to_owned(), guild.to_owned()));
    Some(InSight {
        id,
        kind: mover.kind,
        job: mover.job,
        state: mover.state,
        body: states.body,
        health: states.health,
        speed: mover.speed.clamp(0.0, f64::from(u16::MAX)) as u16,
        sex: figure.sex,
        hair_style: figure.hair_style,
        hair_colour: figure.hair_colour,
        body_palette: figure.body_palette,
        headgear: figure.headgear,
        weapon: figure.weapon,
        shield: figure.shield,
        head: mover.pose.head,
        level: card.map_or(0, |card| card.level),
        boss: card.map_or(0, |card| card.boss),
        life: mover.life,
        name: mover.name.clone(),
        cell: cell_of(mover.cell),
        direction: crate::pose::direction_of(mover.facing % 8),
        walks_to: mover.walk.as_ref().map(|route| cell_of(route.goal())).filter(|goal| *goal != cell_of(mover.cell)),
        sitting: mover.sitting,
        dead: mover.dead,
        guild: mover.guild,
        party_name,
        guild_name,
    })
}

/// The map as it is now; empty while none is shown.
pub fn snapshot() -> Snapshot {
    STATE.with_borrow(|state| {
        let Some(scene) = &state.scene else { return Snapshot::default() };
        let within = |lying: &Lying| {
            let sixteenths = |at: f32, cell: i32| ((at - cell as f32 + 0.5) * 16.0).round().clamp(0.0, 15.0) as u8;
            (sixteenths(lying.at.0, lying.cell.0), sixteenths(lying.at.1, lying.cell.1))
        };
        let lain = |(id, lying): (&u32, &Lying)| (*id, lying.item, lying.identified, lying.count, cell_of(lying.cell), within(lying));
        Snapshot {
            units: scene.units.iter().filter_map(|(id, mover)| seen(state, *id, mover)).collect(),
            portals: scene.portals.iter().map(|(id, cell)| (*id, cell_of(*cell))).collect(),
            drops: scene.drops.iter().map(lain).collect(),
            skill_units: state.grounds.standing(state.entered),
            direction: crate::pose::direction_of(scene.player.facing % 8),
            server_time: (state.clock.now(now()) + crate::server_time::BEHIND).max(0.0) as u64 as u32,
        }
    })
}
