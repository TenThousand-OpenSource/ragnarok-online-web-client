//! The scripts of the homunculus and the mercenary at work (`crate::ai`): called every 140 milliseconds while
//! the one they move is out on the map, told what is in sight, and obeyed; the commands the player gives the
//! two (stand by, walk there, fight that one, use a skill), and their skills used as the character's own are.
//!
//! The client does this for each of the two in its actor's `OnProcess` (`Source/Contents/Sound/SoundMaker.cpp`:
//! the script loaded once, then `CMercenaryAI::Process`); the commands are its `SetMsg2AI` and
//! `SetResMsg2AI`, called from `CGameMode` and `UIWindowMgr`.

use super::*;
use crate::ai::{Ai, Command, Kind, Life, Order, Sight, command, motion};
use crate::net::{homunculus, mercenary};
use crate::press::skills::LevelStep;

/// How often the page looks whether a script is due: well under the 140 milliseconds between its calls.
const LOOKS_EVERY: i32 = 50;

impl App {
    /// Sets the timer that calls the scripts, while the player has a homunculus or a mercenary.
    pub(super) fn ai_wakes(&mut self) {
        if !self.ai.ticking && (self.homunculus.unit != 0 || self.mercenary.info.is_some()) {
            self.ai.ticking = true;
            after(LOOKS_EVERY, Action::AiTick);
        }
    }

    /// The timer: each of the two that is out has its script called if it is due, and the world is told
    /// which units they are and how much life each has.
    pub(super) fn ai_tick(&mut self) {
        self.ai.ticking = false;
        if !matches!(self.screen, Screen::Game) || self.server != Server::Map {
            return;
        }
        self.companion_runs(Kind::Homunculus);
        self.companion_runs(Kind::Mercenary);
        let homunculus = self.homunculus.info.as_ref().filter(|_| self.homunculus.out).map_or(
            (0, (0, 0)),
            |info| (self.homunculus.unit, (info.hp, info.max_hp))
        );
        let mercenary = self.mercenary.info.as_ref().filter(|_| self.mercenary.out).map_or(
            (0, (0, 0)),
            |info| (info.unit, (i64::from(info.hp), i64::from(info.max_hp)))
        );
        world::companions([homunculus.0, mercenary.0], [homunculus.1, mercenary.1]);
        self.ai_wakes();
    }

    /// The unit that is the player's homunculus or mercenary, 0 for none.
    fn companion_unit(&self, kind: Kind) -> u32 {
        match kind {
            Kind::Homunculus => self.homunculus.unit,
            Kind::Mercenary => self.mercenary.unit(),
        }
    }

    /// Looks whether one of the two is out on the map (the client's `homunAlive` and `mercenaryAlive`: set as
    /// its unit comes into sight, cleared as it dies or is put away) and notes the job it is drawn as. One
    /// that came or went has its windows follow (gone, they close: `AM_WILL_BE_DEAD`), and its script begins
    /// anew the next time. Gives whether it is out.
    fn companion_seen(&mut self, kind: Kind) -> bool {
        let unit = self.companion_unit(kind);
        let job = if unit == 0 { None } else { world::unit_job(unit) };
        let (out, drawn_as) = match kind {
            Kind::Homunculus => (&mut self.homunculus.out, &mut self.homunculus.job),
            Kind::Mercenary => (&mut self.mercenary.out, &mut self.mercenary.job),
        };
        let changed = *out != job.is_some();
        (*out, *drawn_as) = (job.is_some(), job.unwrap_or(*drawn_as));
        if changed {
            self.ai.of[kind as usize] = Default::default();
            match (kind, job.is_some()) {
                (Kind::Homunculus, false) =>
                    (self.homunculus.open, self.homunculus.skills_open, self.homunculus.asks, self.homunculus.marked) =
                        (false, false, None, None),
                (Kind::Mercenary, false) =>
                    (self.mercenary.open, self.mercenary.skills_open, self.mercenary.asks, self.mercenary.marked) =
                        (false, false, false, None),
                _ => {}
            }
            self.render();
        }
        job.is_some()
    }

    /// One of the two: if it is out, its script is loaded if it is not yet (the client's own, or the
    /// player's with the client's `/hoai` or `/merai` on), called if it is due, and obeyed. A script that
    /// does not load or fails is told of in the chat in red with what the client writes in its "AI.lua
    /// error" box, and is left alone after: the homunculus then stands, as in the client.
    fn companion_runs(&mut self, kind: Kind) {
        if !self.companion_seen(kind) {
            return;
        }
        let user = self.option(if kind == Kind::Homunculus { "TT_HOMUN_USERAI_ON_OFF" } else { "TT_MER_USERAI_ON_OFF" });
        let script = &mut self.ai.of[kind as usize];
        if script.user != user {
            *script = Script { user, ..Default::default() };
        }
        if script.failed {
            return;
        }
        let mut ai = match script.running.take() {
            Some(ai) => ai,
            None => {
                let Some(files) = data::ai_scripts(user) else { return };
                match Ai::load(kind, user, files) {
                    Ok(ai) => ai,
                    Err(error) => return self.script_failed(kind, &error),
                }
            }
        };
        let called = if ai.due(f64::from(now())) { self.companion_sight(kind).map(|sight| ai.call(sight)) } else { None };
        self.ai.of[kind as usize].running = Some(ai);
        match called {
            Some(Ok(orders)) => self.obeys(orders),
            Some(Err(error)) => self.script_failed(kind, &error),
            None => {}
        }
    }

    fn script_failed(&mut self, kind: Kind, error: &str) {
        let script = &mut self.ai.of[kind as usize];
        (script.running, script.failed) = (None, true);
        self.chat_line(format!("AI.lua error: {error}"), RED);
    }

    /// What the script of one of the two may see now (`crate::ai::Sight`); none while it is in the middle of
    /// a blow (the client does not call the script until the blow's motion is over) or the world has no map.
    fn companion_sight(&self, kind: Kind) -> Option<Sight> {
        let unit = self.companion_unit(kind);
        let (owner, actors) = world::sight(self.account.as_ref()?.account_id, &self.ai.targets)?;
        if actors.iter().any(|seen| seen.id == unit && seen.motion == motion::ATTACK) {
            return None;
        }
        let hero = |name: u16| self.hero.params.get(&name).copied().unwrap_or(0);
        let owner_life = Life {
            hp: hero(packets::parameter::HP),
            max_hp: hero(packets::parameter::MAX_HP),
            sp: hero(packets::parameter::SP),
            max_sp: hero(packets::parameter::MAX_SP)
        };
        let (life, range, attack_delay, skills) = match kind {
            Kind::Homunculus => {
                let info = self.homunculus.info.as_ref()?;
                (Life { hp: info.hp, max_hp: info.max_hp, sp: info.sp, max_sp: info.max_sp }, i32::from(info.range), info.stats[7], &self.homunculus.skills)
            }
            Kind::Mercenary => {
                let info = self.mercenary.info.as_ref()?;
                (
                    Life { hp: i64::from(info.hp), max_hp: i64::from(info.max_hp), sp: i64::from(info.sp), max_sp: i64::from(info.max_sp) },
                    i32::from(info.range),
                    info.stats[7],
                    &self.mercenary.skills
                )
            }
        };
        Some(Sight {
            myself: unit,
            owner,
            owner_life,
            actors,
            life,
            range,
            attack_delay,
            skills: skills.iter().map(|skill| (u32::from(skill.id), i32::from(skill.level), i32::from(skill.range))).collect(),
            friends: [self.homunculus.unit, self.mercenary.unit(), self.pets.unit].into_iter().filter(|friend| *friend != 0 && *friend != unit).collect(),
            tick: f64::from(now()),
            walkable: Some(std::rc::Rc::new(world::walkable)),
        })
    }

    /// Does what a script asked for: the requests the client sends for each of its functions (`AiMove`,
    /// `AiMoveToOwner`, `AiHomunAttack`, and `MM_USE_SKILL` and `MM_USE_SKILL_TOGROUND` for a skill).
    fn obeys(&mut self, orders: Vec<Order>) {
        for order in orders {
            match order {
                Order::Move(unit, cell) => self.send_on_map(&homunculus::move_to(unit, cell)),
                Order::ToOwner(unit) => self.send_on_map(&homunculus::to_owner(unit)),
                Order::Attack(unit, target) => self.send_on_map(&homunculus::attack(unit, target)),
                Order::SkillOn { level, skill, target } => self.send_on_map(&net::inventory::use_skill(level, skill, target)),
                Order::SkillAt { level, skill, cell } => self.send_on_map(&skills::use_on_ground(level, skill, cell)),
                // `TraceValue`: the client's `0xFFFF` of the kind its fourth tab lists.
                Order::Say(words) => self.chat_line(words, YELLOW),
            }
        }
    }

    /// A command for the script of one of the two, if it is out and its script runs (the client's
    /// `HasActiveHomunculusAI`); with `later`, one to be done after the others.
    fn companion_told(&mut self, kind: Kind, told: Command, later: bool) {
        let out = if kind == Kind::Homunculus { self.homunculus.out } else { self.mercenary.out };
        if let Some(ai) = self.ai.of[kind as usize].running.as_ref().filter(|_| out) {
            ai.tell(told, later);
        }
    }

    /// Stand by: it comes to its owner and stays, or goes about its own again (the client's Alt+T and
    /// `/hw`, `MM_HOMUN_WAIT`; the mercenary's key; the menus on the two).
    pub(super) fn companion_stands_by(&mut self, kind: Kind) {
        match kind {
            Kind::Homunculus => self.homunculus.marked = None,
            Kind::Mercenary => self.mercenary.marked = None,
        }
        self.companion_told(kind, Command { kind: command::FOLLOW, ..Command::default() }, false);
    }

    /// The mouse commanded one of the two (Alt and the right button for the homunculus, Alt and the left
    /// one for the mercenary; `CGameMode`): on the ground, it walks there. On someone who can be fought, the
    /// first press picks that one and a second on the same sends it to fight; with Shift either is kept to be
    /// done after its other commands, at once.
    pub(super) fn commanded(&mut self, mercenary: bool, whom: Option<u32>, cell: (u16, u16), later: bool) {
        let kind = if mercenary { Kind::Mercenary } else { Kind::Homunculus };
        let marked = if mercenary { &mut self.mercenary.marked } else { &mut self.homunculus.marked };
        let told = match whom {
            Some(target) if later || *marked == Some(target) => {
                if !later {
                    *marked = None;
                }
                Command { kind: command::ATTACK_OBJECT, target, ..Command::default() }
            }
            // Picked: the next press on it is the command.
            Some(target) => return *marked = Some(target),
            None => {
                if !later {
                    *marked = None;
                }
                Command { kind: command::MOVE, cell: (i32::from(cell.0), i32::from(cell.1)), ..Command::default() }
            }
        };
        self.companion_told(kind, told, later);
    }

    /// A skill of the homunculus or of the mercenary by its number, with the unit whose it is and which of
    /// the two that is; none for any other skill.
    pub(super) fn companion_skill(&self, id: u16) -> Option<(Skill, u32, Kind)> {
        let of = |skills: &[Skill]| skills.iter().find(|skill| skill.id == id).cloned();
        if homunculus::is_skill(u32::from(id)) {
            of(&self.homunculus.skills).filter(|_| self.homunculus.out).map(|skill| (skill, self.homunculus.unit, Kind::Homunculus))
        } else if mercenary::is_skill(u32::from(id)) {
            of(&self.mercenary.skills).filter(|_| self.mercenary.out).map(|skill| (skill, self.mercenary.unit(), Kind::Mercenary))
        } else {
            None
        }
    }

    /// Uses a skill of the homunculus or of the mercenary, from its list or from the hotkey bar (the client's
    /// `MM_USING_SKILL`): one for itself is asked of the server at once, on the unit whose skill it is; one
    /// for someone or for a place is aimed with the pointer, and the character does not walk for it: the
    /// script brings its own one into reach. `level` is the level a slot of the bar has it at. Gives whether
    /// the skill was one of theirs.
    pub(super) fn uses_companion_skill(&mut self, id: u16, level: Option<u16>) -> bool {
        let Some((skill, unit, _)) = self.companion_skill(id) else { return false };
        if skill.level == 0 || skill.passive() {
            return true;
        }
        if !self.cooldowns.waits.ready(skill.id, now()) {
            self.skill_waits(skill.id);
            return true;
        }
        let level = level.or_else(|| self.skills.use_level.get(&skill.id).copied()).unwrap_or(skill.level).min(skill.level).max(1);
        if skill.on_self() {
            self.send_on_map(&net::inventory::use_skill(level, skill.id, unit));
        } else {
            self.skills.aimed = Some((skill.id, level));
            world::aim(Some(skill.target == 2), u16::MAX);
            world::aim_guarded(false);
            self.aim_words();
        }
        true
    }

    /// The skill being aimed was pointed at someone or at a cell: if it is a skill of the homunculus or of
    /// the mercenary, its script is told to use it there (the client's `SKILL_OBJECT_CMD` and
    /// `SKILL_AREA_CMD`), and brings its own one into reach first. Gives whether it was one of theirs.
    pub(super) fn companion_aimed(&mut self, whom: Option<u32>, cell: (u16, u16)) -> bool {
        let Some((skill, level)) = self.skills.aimed else { return false };
        let Some((_, _, kind)) = self.companion_skill(skill) else { return false };
        self.skills.aimed = None;
        let (level, skill) = (i32::from(level), i32::from(skill));
        let told = match whom {
            Some(target) => Command { kind: command::SKILL_OBJECT, level, skill, target, ..Command::default() },
            None => Command { kind: command::SKILL_AREA, level, skill, cell: (i32::from(cell.0), i32::from(cell.1)), ..Command::default() },
        };
        self.companion_told(kind, told, false);
        true
    }

    /// An arrow by a skill of one of the two: the level it is used at, one down or up, between 1 and what
    /// is known (`most`; `UISkillListWnd`, its `ID_LVLSELECTDN1` and on). The levels are kept with the
    /// character's own skills' (`Skills::use_level`): the numbers of skills are not shared.
    pub(super) fn companion_skill_level(&mut self, step: LevelStep, most: Option<u16>) {
        let Some(known) = most.map(|most| most.max(1)) else { return };
        self.use_level_stepped(step.skill, step.step, known);
    }

    /// The right button was pressed on someone: if it is the player's homunculus or mercenary, its menu opens
    /// (the client's `OpenActorMenu`). Gives whether it was.
    pub(super) fn companion_menu(&mut self, id: u32, at: (f32, f32)) -> bool {
        if id != 0 && id == self.homunculus.unit && self.homunculus.shown() {
            self.homunculus.menu_at = Some(Self::on_stage(at));
        } else if id != 0 && id == self.mercenary.unit() && self.mercenary.shown() {
            self.mercenary.menu_at = Some(Self::on_stage(at));
        } else {
            return false;
        }
        self.render();
        true
    }

    /// Puts the menus on the two away.
    pub(super) fn close_companion_menus(&mut self) {
        if self.homunculus.menu_at.take().is_some() | self.mercenary.menu_at.take().is_some() {
            self.render();
        }
    }
}
