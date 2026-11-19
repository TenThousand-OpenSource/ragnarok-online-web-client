//! The functions the client provides to the script of a homunculus or a mercenary
//! (`CMercenaryAI::RegisterFunctions`): `GetV`, `GetActors`, `GetTick`, `GetMsg`, `GetResMsg`, `IsMonster`,
//! `Move`, `MoveToOwner`, `Attack`, `SkillObject`, `SkillGround`, `Trace`, `TraceAI`, `TraceValue`; and
//! `require` and `dofile`, which the client's Lua has of itself and which here read the files the gateway gave.
//!
//! What each answers is a rule over the `Sight` (the functions of `impl Sight` below, each after its
//! `Ai…` function of `MercenaryAI.cpp`); the rest is handing numbers to and from the interpreter.

use super::*;
use lunita::{Callback, CallbackReturn, Context, Table, Value};

/// What `GetV` is asked for (`AI\Const.lua`'s `V_*`).
mod v {
    pub const OWNER: i32 = 0;
    pub const POSITION: i32 = 1;
    pub const MOTION: i32 = 3;
    pub const ATTACK_RANGE: i32 = 4;
    pub const TARGET: i32 = 5;
    pub const SKILL_RANGE: i32 = 6;
    pub const HOMUNCULUS_TYPE: i32 = 7;
    pub const HP: i32 = 8;
    pub const SP: i32 = 9;
    pub const MAX_HP: i32 = 10;
    pub const MAX_SP: i32 = 11;
    pub const MERCENARY_TYPE: i32 = 12;
    pub const POSITION_IN_SKILL_RANGE: i32 = 13;
    pub const SKILL_RANGE_AT_LEVEL: i32 = 14;
}

/// The skills of help a companion has: used on someone, `SkillObject` answers 1 (the client's
/// `IsSupportSkill`, of which these are the ones a homunculus or a mercenary can have: Pain Killer and the
/// mercenaries' Regain to Compress, Kyrie, Blessing and Increase Agility. Its table of skill actions, asked
/// for any other skill, is not read here).
fn helps(skill: u32) -> bool {
    matches!(skill, 8021 | 8226..=8231 | 8238..=8240)
}

impl Sight {
    /// The one of that ID: the player, or someone in sight.
    fn find(&self, id: u32) -> Option<&Seen> {
        if id == self.owner.id { Some(&self.owner) } else { self.actors.iter().find(|seen| seen.id == id) }
    }

    fn walkable(&self, (x, y): (i32, i32)) -> bool {
        self.walkable.as_ref().is_none_or(|walkable| walkable(x, y))
    }

    /// The walkable cell nearest a cell that is not, looked for along the line from it to the player
    /// (`CGameMode::GetClosestValidCell`): at each step of the line the cell and its eight neighbours, in the
    /// client's order. The client also wants a way from the player to the cell found, and no shop standing
    /// on it; neither is asked here.
    pub(super) fn closest_walkable(&self, from: (i32, i32)) -> Option<(i32, i32)> {
        const AROUND: [(i32, i32); 9] = [(0, 0), (1, 0), (1, 1), (1, -1), (-1, 0), (-1, 1), (-1, -1), (0, 1), (0, -1)];
        let (dx, dy) = (self.owner.cell.0 - from.0, self.owner.cell.1 - from.1);
        let along_x = dx.abs() >= dy.abs();
        let (steps, step) = if along_x { (dx.abs(), dx.signum()) } else { (dy.abs(), dy.signum()) };
        (0..steps).find_map(|walked| {
            let on = if along_x { (from.0 + walked * step, walked * step * dy / dx + from.1) } else { (walked * step * dx / dy + from.0, from.1 + walked * step) };
            AROUND.iter().map(|(x, y)| (on.0 + x, on.1 + y)).find(|cell| self.walkable(*cell))
        })
    }

    /// How far a skill of its own reaches (`V_SKILLATTACKRANGE`): 1 for one it does not have. The client
    /// reads the reach at another level than the skill's own from its Lua tables (`GetSkillAttackRange`);
    /// here it is what the server told of the skill at the level it is known at.
    fn skill_range(&self, skill: u32) -> i32 {
        self.skills.iter().find(|known| known.0 == skill).map_or(1, |known| known.2)
    }

    /// Where someone is (`V_POSITION`, and `V_POSITION_APPLY_SKILLATTACKRANGE` with a skill): the cell, or
    /// -1, -1 for one not in sight or hidden. For anyone but the player and its companions, and with a skill
    /// that reaches further than the next cell, the cell told is moved by that reach as the client moves it
    /// (`AiHomunGetV`: along the angle between the two, whose sine it always takes away from the row); a cell
    /// that cannot be walked on is put right, or the script's own cell given. (The client also widens the
    /// reach by the size of the one it was told to fight, read from its Lua tables: not here.)
    pub(super) fn position(&self, id: u32, skill: u32) -> (i32, i32) {
        let Some(seen) = self.find(id).filter(|seen| id == self.owner.id || !seen.hidden) else { return (-1, -1) };
        let (mut x, mut y) = seen.cell;
        let reach = if skill != 0 { self.skill_range(skill) } else { 0 };
        let apart = id != self.owner.id && id != self.myself && !self.friends.contains(&id);
        if let (true, true, Some(me)) = (apart, reach > 0, self.find(self.myself)) {
            let (dx, dy) = ((me.cell.0 - x) as f32, (y - me.cell.1) as f32);
            let distance = (dy * dy + dx * dx).sqrt();
            if distance > 0.0 {
                let angle = (dx / distance).acos();
                x = (x as f32 + angle.cos() * reach as f32) as i32;
                y = (y as f32 - angle.sin() * reach as f32) as i32;
                if !self.walkable((x, y)) {
                    (x, y) = self.closest_walkable((x, y)).unwrap_or(me.cell);
                }
            }
        }
        (x, y)
    }

    /// Whether the script may fight that one (`AiIsMonster`): an enemy that is not hidden.
    pub(super) fn is_monster(&self, id: u32) -> bool {
        self.find(id).is_some_and(|seen| seen.enemy && !seen.hidden)
    }

    /// `GetV`: what it answers, number by number (`AiHomunGetV`, `AiMercenaryGetV`). `-1` is the client's
    /// answer for what it does not know.
    pub(super) fn value(&self, kind: Kind, what: i32, id: u32, skill: u32) -> Vec<f64> {
        let seen = self.find(id);
        let of_life = |pick: fn(&Life) -> i64| {
            if id == self.owner.id {
                pick(&self.owner_life) as f64
            } else if id == self.myself {
                pick(&self.life) as f64
            } else {
                -1.0
            }
        };
        match what {
            v::OWNER => vec![f64::from(self.owner.id)],
            v::POSITION | v::POSITION_IN_SKILL_RANGE => {
                let (x, y) = self.position(id, if what == v::POSITION_IN_SKILL_RANGE { skill } else { 0 });
                vec![f64::from(x), f64::from(y)]
            }
            v::MOTION => vec![seen.map_or(-1.0, |seen| f64::from(seen.motion))],
            v::ATTACK_RANGE => vec![if seen.is_some() && self.range != 0 { f64::from(self.range) } else { -1.0 }],
            // A mercenary's script is told of no target of one who lies dead.
            v::TARGET => vec![seen.filter(|seen| kind == Kind::Homunculus || seen.motion != motion::DEAD).map_or(-1.0, |seen| f64::from(seen.target))],
            v::SKILL_RANGE | v::SKILL_RANGE_AT_LEVEL => vec![f64::from(self.skill_range(skill))],
            // Lif is 1: the homunculi count from 6001.
            v::HOMUNCULUS_TYPE if kind == Kind::Homunculus => vec![seen.map_or(-1.0, |seen| f64::from(seen.job % 6000))],
            v::MERCENARY_TYPE if kind == Kind::Mercenary => vec![seen.map_or(-1.0, |seen| f64::from(crate::net::mercenary::kind(seen.job)))],
            v::HP => vec![of_life(|life| life.hp)],
            v::SP => vec![of_life(|life| life.sp)],
            v::MAX_HP => vec![of_life(|life| life.max_hp)],
            v::MAX_SP => vec![of_life(|life| life.max_sp)],
            _ => Vec::new(),
        }
    }

    /// `Move(id, x, y)`: the cell asked for, put right if it cannot be walked on, and only if it is less than
    /// 25 cells away counted along both ways (`AiMove`).
    pub(super) fn moves(&self, id: u32, to: (i32, i32)) -> Option<Order> {
        let me = self.find(id).filter(|_| id == self.myself)?;
        let to = if self.walkable(to) { to } else { self.closest_walkable(to)? };
        ((to.0 - me.cell.0).abs() + (to.1 - me.cell.1).abs() < 25).then_some(Order::Move(id, (to.0.max(0) as u16, to.1.max(0) as u16)))
    }

    /// `Attack(id, target)`: one blow, at someone in sight who is not hidden and not another companion of the
    /// player's, and not sooner after the last than the delay between its blows (`AiHomunAttack`,
    /// `AiMercenaryAttack`). An option of the session's (its seventeenth) stops a homunculus's blows in the
    /// client; the decompilation does not say which, and it is not asked here.
    pub(super) fn attacks(&self, id: u32, target: u32, last: f64) -> Option<Order> {
        let found = id == self.myself && self.find(id).is_some() && !self.friends.contains(&target) && self.find(target).is_some_and(|seen| !seen.hidden);
        (found && f64::from(self.attack_delay) + last < self.tick).then_some(Order::Attack(id, target))
    }

    /// Whether it stands, walks or flinches: the client lets it use a skill only then.
    fn free(&self) -> bool {
        self.find(self.myself).is_some_and(|me| matches!(me.motion, motion::STAND | motion::MOVE | motion::DAMAGE))
    }

    /// `SkillObject(id, level, skill, target)`: a skill of its own on someone in sight (`AiSkillObject`), and
    /// whether the script is answered 1: the skill is one of help.
    pub(super) fn skill_on(&self, id: u32, level: i32, skill: u32, target: u32) -> (Option<Order>, bool) {
        let own = id == self.myself && self.skills.iter().any(|known| known.0 == skill);
        let there = self.find(target).is_some_and(|seen| target == self.owner.id || !seen.hidden);
        if !own || (!there && self.find(target).is_some()) {
            return (None, false);
        }
        ((there && self.free()).then_some(Order::SkillOn { level: level.max(0) as u16, skill: skill as u16, target }), helps(skill))
    }

    /// `SkillGround(id, level, skill, x, y)`: a skill of its own on a cell (`AiSkillGround`).
    pub(super) fn skill_at(&self, id: u32, level: i32, skill: u32, cell: (i32, i32)) -> Option<Order> {
        let own = id == self.myself && self.skills.iter().any(|known| known.0 == skill);
        (own && self.free()).then_some(Order::SkillAt { level: level.max(0) as u16, skill: skill as u16, cell: (cell.0.max(0) as u16, cell.1.max(0) as u16) })
    }
}

impl Command {
    /// The command as the table the script reads (`AiGetMsg`): its kind first, then what goes with that kind.
    pub(super) fn row(&self) -> Vec<f64> {
        let (kind, cell, target) = (f64::from(self.kind), [f64::from(self.cell.0), f64::from(self.cell.1)], f64::from(self.target));
        match self.kind {
            1 | 4 | 5 => vec![kind, cell[0], cell[1]],
            command::ATTACK_OBJECT => vec![kind, target],
            command::SKILL_OBJECT => vec![kind, f64::from(self.level), f64::from(self.skill), target],
            command::SKILL_AREA => vec![kind, f64::from(self.level), f64::from(self.skill), cell[0], cell[1]],
            _ => vec![kind],
        }
    }
}

/// What a function gives back to the script.
enum Gives {
    Nothing,
    Numbers(Vec<f64>),
    /// A table of numbers, counted from 1.
    Row(Vec<f64>),
}

/// A number as the script gets it: a whole one as an integer, so that it is the same table key and compares
/// the same as one the script counts itself.
fn number<'gc>(value: f64) -> Value<'gc> {
    if value.fract() == 0.0 && value.abs() < 9.0e15 { Value::Integer(value as i64) } else { Value::Number(value) }
}

/// Makes one of the client's functions known to the script: called with the numbers the script passed
/// (anything that is no number counts as 0, as `lua_tonumber` has it).
fn function<'gc>(ctx: Context<'gc>, name: &'static str, shared: &Rc<Shared>, call: fn(&Shared, &[f64]) -> Gives) {
    let shared = shared.clone();
    let _ = ctx.set_global(
        name,
        Callback::from_fn(&ctx, move |ctx, _, mut stack| {
            let mut passed: Vec<f64> = stack.as_slice().iter().map(|value| value.to_number().unwrap_or(0.0)).collect();
            passed.resize(passed.len().max(5), 0.0);
            stack.clear();
            match call(&shared, &passed) {
                Gives::Nothing => {}
                Gives::Numbers(numbers) => numbers.into_iter().for_each(|value| stack.push_back(number(value))),
                Gives::Row(numbers) => {
                    let row = Table::new(&ctx);
                    for (place, value) in numbers.into_iter().enumerate() {
                        row.set(ctx, place as i64 + 1, number(value))?;
                    }
                    stack.push_back(row.into());
                }
            }
            Ok(CallbackReturn::Return)
        }),
    );
}

/// The path of a file as the files are kept: small letters, `/` between, no `./` before it, and `.lua` after
/// a name that has none (`require "AI\\Const"` is `AI\Const.lua`).
pub(super) fn path(name: &[u8]) -> String {
    let path = String::from_utf8_lossy(name).replace('\\', "/").to_lowercase();
    let path = path.trim_start_matches("./").to_owned();
    if path.ends_with(".lua") { path } else { format!("{path}.lua") }
}

/// `require` (a file is run once) and `dofile` (every time): the file is compiled and called in the script's
/// place.
fn loader<'gc>(ctx: Context<'gc>, name: &'static str, shared: &Rc<Shared>, once: bool) {
    let shared = shared.clone();
    let _ = ctx.set_global(
        name,
        Callback::from_fn(&ctx, move |ctx, _, mut stack| {
            let asked: lunita::String = stack.consume(ctx)?;
            let file = path(asked.as_bytes());
            if once && !shared.required.borrow_mut().insert(file.clone()) {
                return Ok(CallbackReturn::Return);
            }
            let Some(source) = shared.files.get(&file) else { return Err(format!("cannot open {file}").into_value(ctx).into()) };
            let closure = Closure::load(ctx, Some(file.as_str()), source)?;
            Ok(CallbackReturn::Call { function: closure.into(), then: None })
        }),
    );
}

/// A function that takes a text: `Trace` and `TraceAI` (which write to the debugger and to `TraceAI.txt`
/// beside the client: a page has neither, so they do nothing) and `TraceValue`, whose text goes to the chat.
fn texts<'gc>(ctx: Context<'gc>, name: &'static str, shared: &Rc<Shared>, said: bool) {
    let shared = shared.clone();
    let _ = ctx.set_global(
        name,
        Callback::from_fn(&ctx, move |_, _, mut stack| {
            if let (true, Value::String(text)) = (said, stack.get(0)) {
                shared.orders.borrow_mut().push(Order::Say(String::from_utf8_lossy(text.as_bytes()).into_owned()));
            }
            stack.clear();
            Ok(CallbackReturn::Return)
        }),
    );
}

/// `ReadFile(name)`: the text of a file of the scripts' folder, or nothing; `ai/prelude.lua` makes the files
/// a script opens to read of it. The client has no such function: its Lua reads the disk.
fn reads<'gc>(ctx: Context<'gc>, shared: &Rc<Shared>) {
    let shared = shared.clone();
    let _ = ctx.set_global(
        "ReadFile",
        Callback::from_fn(&ctx, move |ctx, _, mut stack| {
            let found = match stack.get(0) {
                Value::String(name) => shared.files.get(path(name.as_bytes()).trim_end_matches(".lua")).or_else(|| shared.files.get(&path(name.as_bytes()))),
                _ => None,
            };
            match found {
                Some(text) => stack.replace(ctx, ctx.intern(text)),
                None => stack.clear(),
            }
            Ok(CallbackReturn::Return)
        }),
    );
}

/// Everything the client provides, by the names its scripts call.
pub(super) fn provide<'gc>(ctx: Context<'gc>, shared: &Rc<Shared>) {
    // Lua's own name for the table of globals, which the interpreter leaves out.
    let _ = ctx.set_global("_G", ctx.globals());
    reads(ctx, shared);
    loader(ctx, "require", shared, true);
    loader(ctx, "dofile", shared, false);
    texts(ctx, "Trace", shared, false);
    texts(ctx, "TraceAI", shared, false);
    texts(ctx, "TraceValue", shared, true);
    // Everyone in sight, the player last (`AiGetActors`).
    function(ctx, "GetActors", shared, |shared, _| {
        let sight = shared.sight.borrow();
        Gives::Row(sight.actors.iter().map(|seen| f64::from(seen.id)).chain([f64::from(sight.owner.id)]).collect())
    });
    function(ctx, "GetTick", shared, |shared, _| Gives::Numbers(vec![shared.sight.borrow().tick]));
    function(ctx, "IsMonster", shared, |shared, passed| Gives::Numbers(vec![f64::from(u8::from(shared.sight.borrow().is_monster(passed[0] as u32)))]));
    function(ctx, "GetV", shared, |shared, passed| {
        let Some(kind) = shared.kind.get() else { return Gives::Nothing };
        Gives::Numbers(shared.sight.borrow().value(kind, passed[0] as i32, passed[1] as u32, passed[2] as u32))
    });
    // The command is read once: after it there is none (`AiGetMsg`, `AiGetResMsg`).
    function(ctx, "GetMsg", shared, |shared, passed| if passed[0] as u32 == shared.sight.borrow().myself { Gives::Row(shared.command.take().row()) } else { Gives::Nothing });
    function(ctx, "GetResMsg", shared, |shared, passed| if passed[0] as u32 == shared.sight.borrow().myself { Gives::Row(shared.reserved.take().row()) } else { Gives::Nothing });
    function(ctx, "Move", shared, |shared, passed| ordered(shared, shared.sight.borrow().moves(passed[0] as u32, (passed[1] as i32, passed[2] as i32))));
    function(ctx, "MoveToOwner", shared, |shared, passed| {
        let sight = shared.sight.borrow();
        let id = passed[0] as u32;
        ordered(shared, (id == sight.myself && sight.find(id).is_some()).then_some(Order::ToOwner(id)))
    });
    function(ctx, "Attack", shared, |shared, passed| {
        let sight = shared.sight.borrow();
        let order = sight.attacks(passed[0] as u32, passed[1] as u32, shared.struck.get());
        if order.is_some() {
            shared.struck.set(sight.tick);
        }
        ordered(shared, order)
    });
    function(ctx, "SkillObject", shared, |shared, passed| {
        let (order, helps) = shared.sight.borrow().skill_on(passed[0] as u32, passed[1] as i32, passed[2] as u32, passed[3] as u32);
        ordered(shared, order);
        if helps { Gives::Numbers(vec![1.0]) } else { Gives::Nothing }
    });
    function(ctx, "SkillGround", shared, |shared, passed| ordered(shared, shared.sight.borrow().skill_at(passed[0] as u32, passed[1] as i32, passed[2] as u32, (passed[3] as i32, passed[4] as i32))));
}

/// Notes what the script asked for, if the rule let it through.
fn ordered(shared: &Shared, order: Option<Order>) -> Gives {
    shared.orders.borrow_mut().extend(order);
    Gives::Nothing
}
