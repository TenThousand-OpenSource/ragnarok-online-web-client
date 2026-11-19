//! The skill window, a skill's description, and the hotkey bar.

use super::*;

impl App {
    /// A press of this system (`crate::press::skills`).
    pub(super) fn skills_pressed(&mut self, press: crate::press::skills::Press) {
        use crate::press::Step;
        use crate::press::skills::{LevelStep, Press};
        match press {
            Press::Window => {
                self.skills.open = !self.skills.open;
                self.render();
            }
            Press::Raise(id) => {
                if let Some(socket) = &self.socket {
                    socket.send(&net::inventory::raise_skill(id));
                }
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
            world::aim_guarded(self.guarded(skill.id));
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
            socket.send(&match whom {
                Some(id) => net::inventory::use_skill(level, skill, id),
                None => crate::net::skills::use_on_ground(level, skill, cell),
            });
        }
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
