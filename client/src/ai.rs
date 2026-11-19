//! The scripts that move a homunculus and a mercenary: the client's own `AI\AI.lua` and `AI\AI_M.lua`, run as
//! the Windows client runs them (`CMercenaryAI`, `Source/Contents/Actors/MercenaryAI.cpp`).
//!
//! The server does not move a homunculus: its owner's client calls the script's `AI(id)` every 140
//! milliseconds, and the script asks, through functions the client provides, where everyone is and tells the
//! homunculus to walk, to strike, to use a skill. Here the script is run by a Lua interpreter written in Rust
//! (`lunita`, in its Lua 5.1 mode: the client embeds Lua 5.1), which builds for the browser. The script sees
//! the map as a `Sight` taken before each call, and what it asks for comes back as `Order`s: nothing here
//! touches the page or the socket, so all of it is tested natively (`ai/tests.rs`).
//!
//! `ai/host.rs` has the functions the client provides and `ai/sight.rs` what each answers, after its function
//! of `MercenaryAI.cpp`;
//! `ai/prelude.lua` what the players' own scripts (`AI\USER_AI`) expect of the client's Lua besides.

mod host;
mod sight;
#[cfg(test)]
mod tests;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use lunita::{Closure, Compat, Executor, Fuel, Function, Limits, Lua};

/// Whose script it is (`CMercenaryAI::kind`: 0 and 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Homunculus,
    Mercenary,
}

/// The client calls the script again once this many milliseconds have passed (`CMercenaryAI::Process`).
pub const EVERY: f64 = 140.0;

/// What someone on the map is doing, as the script is told (`V_MOTION`: the client's `render.state`; the
/// names are those of `AI\Const.lua` and of `USER_AI\Const_.lua`).
pub mod motion {
    pub const STAND: i32 = 0;
    pub const MOVE: i32 = 1;
    pub const ATTACK: i32 = 2;
    pub const DEAD: i32 = 3;
    pub const DAMAGE: i32 = 4;
    pub const SIT: i32 = 6;
    pub const SKILL: i32 = 7;
    pub const CASTING: i32 = 8;
}

/// The commands a player gives (`AI\Const.lua`; the client's `AiMessage::type`).
pub mod command {
    pub const NONE: i32 = 0;
    pub const MOVE: i32 = 1;
    pub const ATTACK_OBJECT: i32 = 3;
    pub const SKILL_OBJECT: i32 = 7;
    pub const SKILL_AREA: i32 = 8;
    pub const FOLLOW: i32 = 9;
}

/// Someone on the map, as far as the script can ask about them.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Seen {
    pub id: u32,
    pub cell: (i32, i32),
    /// One of `motion`.
    pub motion: i32,
    /// Whom it last struck or aimed a skill at (the client's `hitTargetAid`); 0 for no one.
    pub target: u32,
    pub job: u16,
    /// Hidden, cloaked or unseen (bits 1, 2 and 6 of its state): the script is not told where it is.
    pub hidden: bool,
    /// Whether the script may fight it: a monster, or a player who is fought on this map (`AiIsMonster`).
    pub enemy: bool,
}

/// The hidden states of someone, of the bits of the state the server tells (`StateFlag01`, `02` and `06`).
pub fn hidden(state: u32) -> bool {
    state & 0x46 != 0
}

/// Life and SP, each with its most.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Life {
    pub hp: i64,
    pub max_hp: i64,
    pub sp: i64,
    pub max_sp: i64,
}

/// What the script can ask about at one call: taken from the world and the session just before it.
#[derive(Clone, Default)]
pub struct Sight {
    /// The homunculus or mercenary the script moves.
    pub myself: u32,
    /// The player: its account, where it is and what it does, and its life.
    pub owner: Seen,
    pub owner_life: Life,
    /// Everyone else in sight, the one the script moves among them.
    pub actors: Vec<Seen>,
    pub life: Life,
    /// How far its blows reach, and the milliseconds between them.
    pub range: i32,
    pub attack_delay: i32,
    /// Its skills: number, level, and how far each reaches at that level.
    pub skills: Vec<(u32, i32, i32)>,
    /// The player's other companions (the mercenary to a homunculus, the homunculus to a mercenary, the pet):
    /// never struck, and their places told as they are.
    pub friends: Vec<u32>,
    /// Milliseconds, as the script's `GetTick` tells them.
    pub tick: f64,
    /// Whether a cell can be walked on.
    pub walkable: Option<Rc<dyn Fn(i32, i32) -> bool>>,
}

/// A command for the script (the client's `AiMessage`): which of `command`, and what goes with it.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Command {
    pub kind: i32,
    pub cell: (i32, i32),
    pub level: i32,
    pub skill: i32,
    pub target: u32,
}

/// What the script asked for in one call, in the order it asked.
#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    /// CZ_REQUEST_MOVENPC: walk to that cell.
    Move(u32, (u16, u16)),
    /// CZ_REQUEST_MOVETOOWNER.
    ToOwner(u32),
    /// CZ_REQUEST_ACTNPC: one blow at that one.
    Attack(u32, u32),
    /// The client's `MM_USE_SKILL`: a skill at that level on someone.
    SkillOn { level: u16, skill: u16, target: u32 },
    /// The client's `MM_USE_SKILL_TOGROUND`: a skill at that level on a cell.
    SkillAt { level: u16, skill: u16, cell: (u16, u16) },
    /// `TraceValue`: a line for the chat.
    Say(String),
}

/// The script's files, by their path under the client's folder in small letters with `/` between
/// (`ai/ai.lua`, `ai/user_ai/const_.lua`), as the gateway gives them.
pub type Files = HashMap<String, Vec<u8>>;

/// What the script's functions share with the one who calls it.
#[derive(Default)]
struct Shared {
    kind: Cell<Option<Kind>>,
    sight: RefCell<Sight>,
    /// The command given and the command given with Shift, to be done after the others (`GetMsg`,
    /// `GetResMsg`): read once, then gone.
    command: Cell<Command>,
    reserved: Cell<Command>,
    orders: RefCell<Vec<Order>>,
    /// When it last asked for a blow (`AiHomunAttack`'s `lastAttackTime`).
    struck: Cell<f64>,
    files: Files,
    /// The files `require` has run.
    required: RefCell<HashSet<String>>,
}

/// A script loaded and ready to be called (`CMercenaryAI` with its `lua_State`).
pub struct Ai {
    lua: Lua,
    shared: Rc<Shared>,
    /// When it was last called.
    called: f64,
}

/// How much one call may compute before it is given up as a script that does not end: steps of the
/// interpreter, each of `FUEL`. The client has no such bound (a script that loops hangs it); a page must not
/// hang. This client's own number: the client's own script takes a few hundredths of one step.
const STEPS: u32 = 400;
const FUEL: i32 = 4096;
/// The memory a script may hold: this client's own bound, for the same reason.
const MEMORY: usize = 32 << 20;
/// What is run before a script: the names of Lua 5.1 the interpreter lacks, and files and a clock as far as
/// a page has them (`ai/prelude.lua` says what and why). The players' own scripts use them.
const PRELUDE: &[u8] = include_bytes!("ai/prelude.lua");

impl Ai {
    /// Loads a script as the client does (`CMercenaryAI::Load`): the standard library, the client's
    /// functions, then the file run once. `user` picks the player's own script (`AI\USER_AI\AI.lua`, the
    /// client's `/hoai` and `/merai`) over the client's. An error is the text the client shows in its
    /// "AI.lua error" box.
    pub fn load(kind: Kind, user: bool, files: Files) -> Result<Ai, String> {
        let name = match (kind, user) {
            (Kind::Homunculus, false) => "ai/ai.lua",
            (Kind::Homunculus, true) => "ai/user_ai/ai.lua",
            (Kind::Mercenary, false) => "ai/ai_m.lua",
            (Kind::Mercenary, true) => "ai/user_ai/ai_m.lua",
        };
        let shared = Rc::new(Shared { kind: Cell::new(Some(kind)), files, ..Shared::default() });
        let mut lua = Lua::with_hash_seed(0x726f_7765_62);
        lua.set_compat(Compat::Lua51);
        lua.set_limits(Limits { memory: Some(MEMORY), string_len: Some(1 << 20) });
        lua.load_core().map_err(|error| error.to_string())?;
        let started = lua.try_enter(|ctx| {
            host::provide(ctx, &shared);
            let source = shared.files.get(name).ok_or_else(|| lunita::Error::from(format!("cannot open {name}").into_value(ctx)))?;
            let prelude = Closure::load(ctx, Some("prelude.lua"), PRELUDE)?;
            let script = Closure::load(ctx, Some(name), source)?;
            Ok(ctx.stash(Executor::start(ctx, Function::compose(&ctx, [prelude.into(), script.into()]), ())))
        });
        let mut ai = Ai { lua, shared, called: 0.0 };
        let executor = started.map_err(|error| error.to_string())?;
        ai.run(&executor)?;
        Ok(ai)
    }

    /// Whether it is time to call the script again (`CMercenaryAI::Process`: not within 140 milliseconds of
    /// the last call); if so the time is noted.
    pub fn due(&mut self, now: f64) -> bool {
        if now - self.called < EVERY {
            return false;
        }
        self.called = now;
        true
    }

    /// A command for the script: the next call reads it (`SetMsg2AI`), or keeps it to do after the others
    /// when it was given with Shift held (`SetResMsg2AI`).
    pub fn tell(&self, command: Command, reserved: bool) {
        if reserved { self.shared.reserved.set(command) } else { self.shared.command.set(command) }
    }

    /// Calls the script's `AI(id)` with what it may see, and gives what it asked for. An error is the text
    /// the client shows before it shuts the script down (`CMercenaryAI::Process`).
    pub fn call(&mut self, sight: Sight) -> Result<Vec<Order>, String> {
        let myself = sight.myself;
        *self.shared.sight.borrow_mut() = sight;
        self.shared.orders.borrow_mut().clear();
        let executor = self
            .lua
            .try_enter(|ctx| {
                let ai: Function = ctx.get_global("AI")?;
                Ok(ctx.stash(Executor::start(ctx, ai, i64::from(myself))))
            })
            .map_err(|error| error.to_string())?;
        self.run(&executor)?;
        Ok(std::mem::take(&mut *self.shared.orders.borrow_mut()))
    }

    /// Runs what was started to its end, or gives it up once it has taken too long.
    fn run(&mut self, executor: &lunita::StashedExecutor) -> Result<(), String> {
        for _ in 0..STEPS {
            let mut fuel = Fuel::with(FUEL);
            let done = self.lua.enter(|ctx| ctx.fetch(executor).step(ctx, &mut fuel)).map_err(|error| error.to_string())?;
            if done {
                return self.lua.try_enter(|ctx| ctx.fetch(executor).take_result::<()>(ctx)?).map_err(|error| error.to_string());
            }
        }
        Err("the script does not come to an end".to_owned())
    }
}

use lunita::IntoValue;
