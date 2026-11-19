//! The functions the client provides to the script of a homunculus or a mercenary
//! (`CMercenaryAI::RegisterFunctions`): `GetV`, `GetActors`, `GetTick`, `GetMsg`, `GetResMsg`, `IsMonster`,
//! `Move`, `MoveToOwner`, `Attack`, `SkillObject`, `SkillGround`, `Trace`, `TraceAI`, `TraceValue`; and
//! `require` and `dofile`, which the client's Lua has of itself and which here read the files the gateway gave.
//!
//! What each answers is a rule over the `Sight` (the functions of `impl Sight` in `sight.rs`, each after its
//! `Ai…` function of `MercenaryAI.cpp`); here is the rest: handing numbers to and from the interpreter.

use super::*;
use lunita::{Callback, CallbackReturn, Context, Table, Value};

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
