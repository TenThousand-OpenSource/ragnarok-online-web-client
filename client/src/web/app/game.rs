//! The game's own doings: using items and skills, the hotkey bar, chat lines, entering a map.

use super::*;

/// The effect an item shows on the one who uses it, and its sound: the client's own tables
/// (`CGameMode::PlayItemUseEffect` for the effect of each item, `CEZ2STRScreenEffect::InitParticle` for its file).
pub(super) fn item_effect(item: u32) -> Option<(&'static str, Option<&'static str>)> {
    let found = |table: &'static str, wanted: u32| table.lines().filter_map(|line| line.split_once('\t')).find(|(id, _)| id.parse() == Ok(wanted)).map(|(_, value)| value.trim());
    let effect: u32 = found(include_str!("../item_effects.txt"), item)?.parse().ok()?;
    let file = found(include_str!("../effect_files.txt"), effect)?;
    let sound = match effect {
        204..=207 | 210 => Some("_heal_effect.wav"),
        208 | 209 | 211 => Some("effect/흡기.wav"),
        _ => None,
    };
    Some((file, sound))
}

impl App {
    /// Uses what was double-clicked or what a key stands for: `item:<index>`, `skill:<id>` or `slot:<number>`.
    pub(super) fn use_it(&mut self, what: &str) {
        let (Some(socket), Some(account)) = (&self.socket, &self.account) else { return };
        let Some((kind, number)) = what.split_once(':') else { return };
        let Ok(number) = number.parse::<u32>() else { return };
        // A slot of the bar stands for its item or its skill.
        let (kind, number, level) = match kind {
            "slot" => match self.skills.hotkeys.get(number as usize).copied().filter(|hotkey| hotkey.id != 0) {
                Some(hotkey) if hotkey.skill => ("skill", hotkey.id, Some(hotkey.count)),
                // The first of that item the character carries without wearing it.
                Some(hotkey) => match self.bag.items.iter().find(|item| item.id == hotkey.id && item.worn == 0) {
                    Some(item) => ("item", u32::from(item.index), None),
                    None => return,
                },
                None => return,
            },
            other => (other, number, None),
        };
        match kind {
            "item" => {
                let Some(item) = self.bag.items.iter().find(|item| u32::from(item.index) == number) else { return };
                if item.usable() {
                    socket.send(&items::use_item(item.index, account.account_id));
                } else if item.worn != 0 {
                    socket.send(&items::take_off(item.index));
                } else if item.wearable != 0 {
                    socket.send(&items::wear(item.index, item.wearable));
                }
            }
            "skill" => {
                let Some(skill) = self.skills.known.iter().find(|skill| u32::from(skill.id) == number).cloned() else { return };
                if skill.level == 0 || skill.passive() {
                    return;
                }
                if skill.on_self() {
                    socket.send(&items::use_skill(level.unwrap_or(skill.level).min(skill.level).max(1), skill.id, account.account_id));
                } else {
                    // A skill for someone or for a place is aimed with the pointer: the next press on the map
                    // says on whom or where.
                    self.skills.aimed = Some((skill.id, level.unwrap_or(skill.level).min(skill.level).max(1)));
                    world::aim(Some(skill.target == 2), skill.range);
                }
            }
            _ => {}
        }
    }

    /// Puts what was dragged on a slot of the hotkey bar, or empties the slot (for nothing dragged), and tells
    /// the server, which keeps the bar.
    pub(super) fn set_hotkey(&mut self, slot: usize, dragged: &str) {
        let mut parts = dragged.split(':');
        let hotkey = match (parts.next(), parts.next().and_then(|id| id.parse::<u32>().ok()), parts.next().and_then(|level| level.parse().ok())) {
            (Some("item"), Some(id), _) => Hotkey { skill: false, id, count: 0 },
            (Some("skill"), Some(id), Some(level)) => Hotkey { skill: true, id, count: level },
            (Some(""), ..) => Hotkey::default(),
            _ => return,
        };
        if slot >= items::HOTKEYS {
            return;
        }
        self.skills.hotkeys.resize(items::HOTKEYS, Hotkey::default());
        self.skills.hotkeys[slot] = hotkey;
        if let Some(socket) = &self.socket {
            socket.send(&items::set_hotkey(slot as u16, hotkey));
        }
        self.refresh_items();
    }

    /// Asks what the items carried and on the bar are called, and redraws their windows.
    pub(super) fn items_changed(&self) {
        let stored = self.deal.storage.iter().flat_map(|storage| storage.items.iter().map(|item| item.id));
        let ids = self.bag.items.iter().map(|item| item.id).chain(self.skills.hotkeys.iter().filter(|hotkey| !hotkey.skill).map(|hotkey| hotkey.id)).chain(stored);
        data::want_items(ids, || dispatch(Action::ItemsKnown));
        // The weapon in hand (bit 2 of where a thing is worn) is drawn on the character.
        world::own_weapon(self.bag.items.iter().find(|item| item.worn & 2 != 0).map_or(0, |item| item.id));
        self.refresh_items();
        self.paint_figures();
    }

    /// The sound a blow of the weapon in hand makes when it lands (the client's `_hit_*.wav`): by the kind of
    /// weapon its number tells, bare fists if nothing is held.
    pub(super) fn weapon_sound(&self) -> &'static str {
        // Bit 2 of where a thing is worn is the hand that holds the weapon.
        match self.bag.items.iter().find(|item| item.worn & 2 != 0).map_or(0, |item| item.id) {
            1100..=1199 | 13400..=13499 => "_hit_sword.wav",
            1200..=1249 | 13000..=13099 => "_hit_dagger.wav",
            1300..=1399 => "_hit_axe.wav",
            1400..=1499 => "_hit_spear.wav",
            1500..=1549 | 16000..=16099 => "_hit_mace.wav",
            1550..=1699 | 2000..=2099 => "_hit_rod.wav",
            1700..=1749 | 18100..=18199 => "_hit_arrow.wav",
            _ => "_hit_fist1.wav",
        }
    }

    /// The chat's next size, as the client steps it (`UM_TOGGLEPUSH`): one row, then three more each time up to
    /// thirteen, then none.
    pub(super) fn chat_resize(&mut self) {
        let rows = if self.chat.rows == 0 { 1 } else { self.chat.rows + 3 };
        self.chat.rows = if rows > 15 { 0 } else { rows / 3 * 3 + 1 };
    }

    /// Tells of the thing just picked up, once its name is known: a line in the chat, and a note on the screen.
    pub(super) fn tell_obtained(&mut self) {
        let Some((id, amount)) = self.bag.obtained.filter(|_| !self.bag.told_obtained) else { return };
        let Some((name, _)) = data::item(id) else { return };
        self.bag.told_obtained = true;
        self.chat_line(format!("{name} - {amount} {}", crate::i18n::tr("obtained.")), "#ffff00");
        if let Some(note) = element("obtained") {
            note.set_inner_html(&self.obtained_note());
        }
    }

    /// Adds a line to the chat.
    pub(super) fn chat_line(&mut self, words: String, colour: &str) {
        self.chat.lines.push((words, colour.to_owned()));
        if self.chat.lines.len() > super::super::hud::CHAT_KEPT {
            self.chat.lines.remove(0);
        }
        self.refresh_chat();
    }

    /// The client's chime (`버튼소리.wav`). The original plays it in a few places only: on logging in, on choosing
    /// a server, on entering the game with a character (and for a notice, a whisper window, someone joining a
    /// chat room). Its buttons, tabs and items make no sound.
    pub(super) fn chime(&self) {
        if self.sound {
            sound::button();
        }
    }

    /// Takes the slot machine of taming away, and stops its timer.
    pub(super) fn close_taming(&mut self) {
        if let Some(taming) = self.pets.taming.take() {
            if let Some(window) = web_sys::window() {
                window.clear_interval_with_handle(taming.timer);
            }
        }
    }

    pub(super) fn entered_map(&mut self, packet: &[u8]) {
        let (Some(accepted), Some(zone), Some(socket)) = (packets::map_accepted(packet), &self.zone, &self.socket) else {
            return self.leave(Some(Key::Rejected.text(self.lang).to_owned()));
        };
        let name = self.characters.iter().find(|character| character.char_id == zone.char_id).map(|character| character.name.clone()).unwrap_or_default();
        let map = zone.map.clone();
        // The map counts as loaded: the server now puts the character into the world, where other players see it.
        socket.send(&packets::map_loaded());
        // Until the map server sends the character's values, the ones of the character list stand in.
        let character = self.characters.iter().find(|character| character.char_id == zone.char_id);
        let job = character.map_or(0, |character| character.job);
        self.hero.params.clear();
        self.bag.items.clear();
        self.skills.known.clear();
        self.skills.hotkeys.clear();
        self.hero.stat_bonus = [0; 6];
        self.chat.lines.clear();
        if let Some(character) = character {
            use packets::parameter::*;
            self.hero.params.extend([
                (HP, character.hp as i64),
                (MAX_HP, character.max_hp as i64),
                (SP, character.sp as i64),
                (MAX_SP, character.max_sp as i64),
                (BASE_LEVEL, i64::from(character.base_level)),
                (JOB_LEVEL, i64::from(character.job_level)),
                (BASE_EXP, character.base_exp as i64),
                (ZENY, i64::from(character.zeny)),
            ]);
            for (stat, value) in character.stats.iter().enumerate() {
                self.hero.params.insert(STR + stat as u16, i64::from(*value));
            }
        }
        // The chat starts four rows tall; hidden by the player, it stays hidden.
        if self.chat.rows == 0 && self.chat.rows_kept == 0 {
            self.chat.rows = 4;
        }
        self.game = Some(Game { name, job, map, x: accepted.x, y: accepted.y, packets: 0, map_size: None });
        self.screen = Screen::Game;
        self.tune();
        self.enter_world();
    }

    /// The player's own character as a figure, standing and facing the screen, with the weapon it holds.
    pub(in super::super) fn own_figure(&self) -> Figure {
        let character = self.zone.as_ref().and_then(|zone| self.characters.iter().find(|character| character.char_id == zone.char_id));
        Figure {
            job: character.map_or(0, |character| character.job),
            sex: character.map_or(1, |character| character.sex),
            hair_style: character.map_or(1, |character| character.hair_style),
            hair_colour: character.map_or(0, |character| u16::from(character.hair_color)),
            headgear: character.map_or([0; 3], |character| character.headgear),
            body_palette: character.map_or(0, |character| character.body_palette),
            weapon: self.bag.items.iter().find(|item| item.worn & 2 != 0).map_or(0, |item| item.id),
            action: 0,
            frame: 0,
        }
    }

    /// Draws the map the character is on. The wait box stays until it shows.
    pub(super) fn enter_world(&mut self) {
        let (Some(game), Some(zone)) = (&self.game, &self.zone) else { return };
        let _ = zone;
        let figure = self.own_figure();
        let hooks = world::Hooks {
            walk_to: |x, y| dispatch(Action::WalkTo(x, y)),
            press: |id, monster| dispatch(Action::Pressed(id, monster)),
            take: |id| dispatch(Action::Take(id)),
            moved: |x, y| dispatch(Action::Moved(x, y)),
            ready: |outcome| dispatch(Action::WorldReady(outcome)),
            aimed: |whom, cell| dispatch(Action::Aimed(whom, cell)),
        };
        sound::map_track(data::music(&game.map));
        world::enter(game.map.clone(), (game.x, game.y), figure, hooks);
        // The client's loading screen stands over everything until the map shows.
        loading(Some(self.loadings));
        self.loadings += 1;
        loading_ahead(self.loadings);
        self.wait = None;
        self.ask = None;
        self.render();
    }
}
