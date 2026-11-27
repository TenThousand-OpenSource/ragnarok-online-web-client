//! The skill window, a skill's description, and the hotkey bar.

use super::*;

impl App {
    /// A press of this system (`crate::press::skills`).
    pub(super) fn skills_pressed(&mut self, press: crate::press::skills::Press) {
        use crate::press::Step;
        use crate::press::skills::{LevelStep, Press};
        match press {
            // Opened, the window puts away the button that tells of a job level gained and takes the mark off
            // its icon (`UIWindowMgr::EnsureWindow`, `WID_SKILLLISTWND`).
            Press::Window => {
                self.skills.open = !self.skills.open;
                self.skills.level_told &= !self.skills.open;
                self.render();
            }
            // The button that tells of a job level gained (`UINotifyJobLevelUpWnd::SendMsg`): the skill window
            // is shown, which puts the button away, and comes to the front.
            Press::JobLevelGained => {
                (self.skills.open, self.skills.level_told) = (true, false);
                self.render();
                self.window_pressed(wnd::SKILLS.id);
            }
            Press::WordsOk => self.skill_words_typed(),
            // Cancel puts the box away and the skill is not used (`ID_CANCEL`).
            Press::WordsCancel => {
                self.skills.writing = None;
                self.redraw(&[&wnd::SKILL_WORDS]);
            }
            // A skill pressed to be raised: a level of it is planned, and before that of what it asks for and
            // lacks, as far as the points left go (`crate::skills::raises`; the client's `IncreaseSkillLevel`).
            // Nothing is sent before Apply.
            Press::Raise(id) => {
                if !self.skill_opens(id) {
                    return;
                }
                let job = self.own_figure().job;
                let more = crate::skills::raises(id, self.skill_points_left(), |skill| self.skill_planned_level(skill), |skill| data::skill_asks(job, skill).1);
                self.skills.plan.extend(more);
                self.redraw(&[&wnd::SKILLS]);
            }
            // Apply asks first (`ID_APPLY`: the client's line 1377 in a box of Yes and No), and Yes sends a
            // level for each one planned (`ApplySkillPoints`, `MM_UPGRADE_SKILLLEVEL`); Reset forgets them.
            Press::Apply => {
                self.skills.asks = !self.skills.plan.is_empty();
                self.redraw(&[&wnd::SKILL_ASKS]);
            }
            Press::ApplyYes => {
                for skill in std::mem::take(&mut self.skills.plan) {
                    self.send_on_map(&net::inventory::raise_skill(skill));
                }
                self.skills.asks = false;
                self.redraw(&[&wnd::SKILL_ASKS, &wnd::SKILLS]);
            }
            Press::ApplyNo => {
                self.skills.asks = false;
                self.redraw(&[&wnd::SKILL_ASKS]);
            }
            Press::Reset => {
                self.skills.plan.clear();
                self.redraw(&[&wnd::SKILLS]);
            }
            // The small button turns the tree into the list of rows and back (`ID_MINI`).
            Press::Mini => {
                (self.skills.mini, self.skills.picked) = (!self.skills.mini, None);
                self.render();
            }
            Press::Pick(row) => {
                self.skills.picked = Some(row);
                self.redraw(&[&wnd::SKILLS]);
            }
            Press::InfoClose => {
                self.skills.info = None;
                self.refresh_items();
            }
            Press::HotkeyRows(step) => {
                self.skills.hotkey_rows = if step == Step::More { (self.skills.hotkey_rows + 1).min(3) } else { self.skills.hotkey_rows.saturating_sub(1) };
                self.render();
            }
            Press::Tab(tab) => {
                self.skills.tab = tab;
                self.render();
            }
            // An arrow by a skill's level: the level it is used at, one down or up, between 1 and what is known
            // (the client's `UINewSkillListWnd`). It is the level it goes onto the hotkey bar with.
            Press::Level(LevelStep { skill: id, step }) => {
                let Some(known) = self.skills.known.iter().find(|skill| skill.id == id).map(|skill| skill.level.max(1)) else { return };
                self.use_level_stepped(id, step, known);
                // Kept in the browser for this character, so the levels are the same next time. (Where the
                // client keeps them was not looked for: this is this client's own way.)
                if let Some(zone) = &self.zone {
                    store(&format!("roweb.skill-levels.{}", zone.char_id), crate::kept::paired(&self.skills.use_level, ':').as_deref());
                }
                self.refresh_items();
            }
        }
    }

    /// The skill points not planned away yet (the client's `m_skillPoints` beside the session's).
    pub(in crate::web) fn skill_points_left(&self) -> u16 {
        let all = crate::skills::points(self.hero.params.get(&packets::parameter::SKILL_POINTS).copied().unwrap_or(0));
        all.saturating_sub(self.skills.plan.len() as u16)
    }

    /// The level a skill will be at once what is planned is sent.
    pub(in crate::web) fn skill_planned_level(&self, id: u16) -> u16 {
        self.skill_level(id) + crate::skills::planned(&self.skills.plan, id)
    }

    /// Whether a press on a skill plans a level of it: with points left, under its last level, one the server
    /// lists as raisable, or one not known yet whose needs are lacking and can be planned with it
    /// (`crate::skills::plus_by_table`).
    pub(in crate::web) fn skill_opens(&self, id: u16) -> bool {
        let left = i64::from(self.skill_points_left());
        let learned = self.skills.known.iter().find(|skill| skill.id == id);
        let level = learned.map_or(0, |skill| skill.level);
        let most = data::skill(id).map_or(level, |(.., most)| most);
        let raisable = learned.is_some_and(|skill| skill.raisable) && left > 0;
        let (job_level, needs) = data::skill_asks(self.own_figure().job, id);
        let job = self.hero.params.get(&packets::parameter::JOB_LEVEL).copied().unwrap_or(0);
        let by_table = crate::skills::plus_by_table(raisable, left, level, job, (job_level, &needs), |skill| self.skill_level(skill));
        (raisable || by_table) && self.skill_planned_level(id) < most
    }

    /// A skill is used, from its window, by a key or from the hotkey bar (then at the level it was put there
    /// with): at once on the character itself, or aimed with the pointer.
    pub(in crate::web::app) fn use_skill(&mut self, id: u16, level: Option<u16>) {
        // A skill of the homunculus or of the mercenary is used through the one whose it is (`ai.rs`).
        if self.uses_companion_skill(id, level) {
            return;
        }
        // Change Cart opens the window of the looks, or is not used at all (`clicks/cart.rs`).
        if id == CHANGE_CART && self.skills.known.iter().any(|skill| skill.id == CHANGE_CART && skill.level > 0) && !self.cart_change_used() {
            return;
        }
        let (Some(socket), Some(account)) = (&self.socket, &self.account) else { return };
        // A skill of the guild's is used as one of the character's own (the client's `MM_USING_SKILL` for both).
        let Some(skill) = self.skills.known.iter().find(|skill| skill.id == id).cloned().or_else(|| self.guild_skill(id)) else { return };
        if skill.level == 0 || skill.passive() {
            return;
        }
        // One that still waits is not used, and the chat says so (`cooldowns`).
        if !self.cooldowns.waits.ready(skill.id, now()) {
            return self.skill_waits(skill.id);
        }
        // From the bar it is used at the level it was put there with; from the skill window, at the
        // level picked there.
        let level = level.or_else(|| self.skills.use_level.get(&skill.id).copied());
        if skill.on_self() {
            socket.send(&net::inventory::use_skill(level.unwrap_or(skill.level).min(skill.level).max(1), skill.id, account.account_id));
        } else {
            // A skill for someone or for a place is aimed with the pointer: the next press on the map
            // says on whom or where.
            self.skills.aimed = Some((skill.id, level.unwrap_or(skill.level).min(skill.level).max(1)));
            world::aim(Some(skill.target == 2), skill.range);
            world::aim_support(crate::skills::aim::support(skill.id));
            // A skill for a trap (the server's kind 32: Remove Trap, Spring Trap) is pointed at a trap.
            world::aim_traps(skill.target == 32);
            self.aim_words();
        }
    }

    /// Writes the skill being aimed and the level it will be used at by the pointer.
    pub(in crate::web::app) fn aim_words(&self) {
        let words = self.skills.aimed
            .filter(|(skill, _)| *skill != TAME)
            .map(|(skill, level)| format!("{} Lv {level}", data::skill(skill).map(|(_, name, _)| name).unwrap_or_default()))
            .unwrap_or_default();
        world::aim_words(&words);
    }

    /// A skill was pressed with the right button: its description shows.
    pub(in crate::web::app) fn skill_info(&mut self, id: u16) {
        self.skills.info = Some(id);
        data::want_skill(id, || dispatch(Action::ItemsKnown));
        self.refresh_items();
    }

    /// The skill being aimed was pointed at someone or at a cell: it is used there.
    pub(in crate::web::app) fn skill_aimed(&mut self, whom: Option<u32>, cell: (u16, u16)) {
        // Sense names the monster it tells of by whom it was used on (`incoming/fight.rs`).
        if self.skills.aimed.is_some_and(|(skill, _)| skill == 93) {
            self.fight.aimed = whom;
        }
        if let (Server::Map, Some(socket), Some((skill, level))) = (self.server, &self.socket, self.skills.aimed.take()) {
            // A Talkie Box and Graffiti ask for their words first, and are placed when those are given
            // (`CPlayer::SendMsg`, `AM_REQ_GROUND_SKILL`). The client asks as its character comes within the
            // skill's reach of the place; this client leaves the way there to the server, and asks at once.
            if whom.is_none() && crate::skills::words_asked(skill).is_some() {
                self.skills.writing = Some((skill, level, cell));
                return self.redraw(&[&wnd::SKILL_WORDS]);
            }
            socket.send(&match whom {
                Some(id) => net::inventory::use_skill(level, skill, id),
                None => crate::net::skills::use_on_ground(level, skill, cell),
            });
        }
    }

    /// The skill being aimed at someone was pointed at the player's own character (the client's pick has its
    /// own character among the others: `CGameMode::HandleActorPick`, `Source/Client/Mode/GameMode.cpp:10499`):
    /// a skill of the homunculus or of the mercenary is told to its script, as for anyone pointed at; of the
    /// character's own skills only one for a friend is used (`Skill::on_friend`), and any other is let go.
    pub(in crate::web::app) fn own_aimed(&mut self) {
        let own = Some(self.own_account());
        if self.companion_aimed(own, (0, 0)) {
            return;
        }
        let friendly = self.skills.aimed.is_some_and(|(skill, _)| self.skills.known.iter().any(|known| known.id == skill && known.on_friend()));
        if friendly {
            self.skill_aimed(own, (0, 0));
        } else {
            self.skills.aimed = None;
        }
    }

    /// OK of the box that asks the words of a Talkie Box or of Graffiti, and Enter in its line (`ID_OK`, then
    /// `CGameMode::SendMsg`, `MM_USE_SKILL_TOGROUND_WITH_TALKBOX`): the box closes; a skill that still waits
    /// is not used, and the chat says so; else the skill is placed with what was typed.
    pub(in crate::web::app) fn skill_words_typed(&mut self) {
        let words = field(&wnd::SKILL_WORDS.field());
        let Some((skill, level, cell)) = self.skills.writing.take() else { return };
        self.redraw(&[&wnd::SKILL_WORDS]);
        if !self.cooldowns.waits.ready(skill, now()) {
            return self.skill_waits(skill);
        }
        // The client's filter of words (`g_forbiddenWords`, `manner.txt`; `GameMode.cpp:3543`): with a word of
        // it in the text nothing is sent, and the chat says the line 50 of the message table in red.
        if data::foul(&words) {
            return self.chat_line(Self::words(50, "Chat Filter: Yeah, uh, I don't think so buddy..."), RED);
        }
        self.send_on_map(&crate::net::skills::use_on_ground_with_words(level, skill, cell, &words));
    }

    /// The level the aimed skill is used at, one up or down, between 1 and the level it is known at
    /// (the client's `MM_USESKILL_LEVEL_PLUS`).
    pub(in crate::web::app) fn aim_level_stepped(&mut self, step: i32) {
        if let Some((skill, level)) = self.skills.aimed.filter(|(skill, _)| *skill != TAME) {
            let most = self.skills.known
                .iter()
                .find(|known| known.id == skill)
                .cloned()
                .or_else(|| self.companion_skill(skill).map(|(known, ..)| known))
                .map_or(1, |known| known.level.max(1));
            self.skills.aimed = Some((skill, (i32::from(level) + step).clamp(1, i32::from(most)) as u16));
            self.aim_words();
        }
    }

    /// The key of the hotkey bar: the bar runs through its sizes, as the client's does: a row more each time,
    /// then away, then one row again. (The key's own arm writes the screen anew.)
    pub(in crate::web::app) fn hotkey_bar_stepped(&mut self) {
        (self.skills.hotkey_rows, self.skills.hotkeys_hidden) = crate::hotkeys::bar_stepped(self.skills.hotkey_rows, self.skills.hotkeys_hidden);
    }

    /// The level a skill is used at goes one down or up, between 1 and the level it is known at; at the level
    /// it is known at nothing is kept of it. (For the character's own skills and for those of a homunculus
    /// and a mercenary: `ai.rs`.)
    pub(in crate::web::app) fn use_level_stepped(&mut self, id: u16, step: crate::press::Step, known: u16) {
        let used = self.skills.use_level.get(&id).copied().unwrap_or(known);
        match crate::skills::use_level_stepped(used, known, step == crate::press::Step::More) {
            Some(wanted) => self.skills.use_level.insert(id, wanted),
            None => self.skills.use_level.remove(&id),
        };
    }
}
