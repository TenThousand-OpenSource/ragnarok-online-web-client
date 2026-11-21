//! What passes between the world and the page: the player's options the world goes by (`Options`), and what
//! the world tells the app (`Hooks`), which it never reaches into.

/// The options of the client's Game Settings window the world goes by.
#[derive(Clone, Copy)]
pub struct Options {
    /// A press on a monster fights it until it is down (`/noctrl`); off, that takes Ctrl, and a plain press
    /// fights only while the button is held.
    pub no_ctrl: bool,
    /// A skill aimed at someone goes on whoever is pressed, Shift held or not (`/noshift`).
    pub no_shift: bool,
    /// The right button uses the bar's ninth slot (`/q1`); the wheel its eighth and seventh (`/q2`).
    pub quick_right: bool,
    pub quick_wheel: bool,
    /// Effects of skills and items are drawn (`/effect`); a miss is shown (`/miss`); a monster being fought
    /// shows its life (`/monsterhp`).
    pub effects: bool,
    pub misses: bool,
    pub monster_life: bool,
    /// Other players are not drawn.
    pub hide_players: bool,
    /// The camera may go out to 730 instead of 400 (`/zoom`).
    pub zoom_out: bool,
    /// This project's own, on unless the player switches it off (`/farwalk`): a press farther than the
    /// server walks in one go sends the character there in stretches (`pursuit::journey`). Off, such a press
    /// is asked once and does nothing, as in the client.
    pub far_walk: bool,
    /// The screen shakes when an effect asks for it (`/quake`: the client's `MM_QUAKE` does nothing with it off).
    pub quake: bool,
    /// The name of a character the pointer is on stands over its head, not under its feet (`/font`).
    pub name_above: bool,
    /// The emblem of a player's guild is drawn by the name (`/emblem`).
    pub emblems: bool,
    /// The aura of the highest level shows (`/aura2`), and it is the full one (`/aura`): `crate::aura::Options`.
    pub auras: bool,
    pub aura_full: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            no_ctrl: true,
            no_shift: false,
            quick_right: false,
            quick_wheel: false,
            effects: true,
            misses: true,
            monster_life: true,
            hide_players: false,
            zoom_out: true,
            far_walk: true,
            quake: true,
            name_above: false,
            emblems: true,
            auras: true,
            aura_full: false
        }
    }
}

/// What the page is told, and what it is asked for.
pub struct Hooks {
    /// The player pressed a cell to walk to.
    pub walk_to: fn(u16, u16),
    /// The character turned, or turned its head, towards a place pressed: how the head is turned and the
    /// direction, as the server is told them.
    pub face: fn(u8, u8),
    /// The character stands on, or passes, another cell.
    pub moved: fn(u16, u16),
    /// The player pressed someone: their ID, and whether it is a monster.
    pub press: fn(u32, bool),
    /// The character stands by a thing on the ground the player pressed: pick it up.
    pub take: fn(u32),
    /// The map is drawn (or could not be: then with the reason).
    pub ready: fn(Result<(), String>),
    /// The skill being aimed was pointed at someone (their ID) or at a cell.
    pub aimed: fn(Option<u32>, (u16, u16)),
    /// The skill being aimed at someone was pointed at the player's own character, whose ID the world is not
    /// told.
    pub aimed_own: fn(),
    /// The right button was pressed and let go on someone without moving: their ID, and where on the page.
    pub menu: fn(u32, (f32, f32)),
    /// The wheel was turned while a skill is aimed: up (1) or down (-1) a level.
    pub aim_level: fn(i32),
    /// One blow at a monster, not a fight until it is down (the client without Ctrl, when its No Ctrl option
    /// is off).
    pub strike_once: fn(u32),
    /// The fight is called off: the server is told to stop striking.
    pub stop: fn(),
    /// A slot of the hotkey bar is used by the mouse (the client's Quick Skill options).
    pub quick: fn(usize),
    /// The mouse commands the homunculus, or the mercenary (true): to fight someone (their ID) or to walk to
    /// a cell, at once or, with Shift (true), after its other commands (`ai.rs`).
    pub commanded: fn(bool, Option<u32>, (u16, u16), bool),
}
