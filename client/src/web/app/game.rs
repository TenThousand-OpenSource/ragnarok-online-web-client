//! The game's own doings: using items and skills, the hotkey bar, chat lines, entering a map.

use super::*;

impl App {
    /// Uses what was double-clicked or what a key stands for: an item carried, a skill, a slot of the bar.
    pub(super) fn use_it(&mut self, what: Use) {
        // A double click on a row of a list to pick from is picking it and pressing OK (the client's
        // `UIItemIdentifyWnd::OnLBtnDblClk`).
        match what {
            Use::Egg(row) => {
                self.pressed(crate::press::pets::Press::EggPick(row));
                return self.pressed(crate::press::pets::Press::EggOk);
            }
            Use::Identify(row) => {
                self.pressed(crate::press::inventory::Press::IdentifyPick(row));
                return self.pressed(crate::press::inventory::Press::IdentifyOk);
            }
            Use::Choice(row) => return self.choice_used(row),
            Use::Room(id) => return self.room_used(id),
            Use::Emotion(emotion) => return self.emotion_shown(emotion),
            // A card asks what it can be set into (`clicks/choosing.rs`).
            Use::Item(index) if self.card_used(index) => return,
            _ => {}
        }
        // A slot of the bar stands for its item or its skill.
        let (what, level) = match what {
            Use::Slot(slot) => match self.skills.hotkeys.get(slot).copied().filter(|hotkey| hotkey.id != 0) {
                Some(hotkey) if hotkey.skill => (Use::Skill(hotkey.id as u16), Some(hotkey.count)),
                // The first of that item the character carries without wearing it.
                Some(hotkey) => match self.bag.items.iter().find(|item| item.id == hotkey.id && item.worn == 0) {
                    Some(item) => (Use::Item(item.index), None),
                    None => return,
                },
                None => return,
            },
            other => (other, None),
        };
        // A skill of the homunculus or of the mercenary is used through the one whose it is (`ai.rs`).
        if let Use::Skill(id) = what {
            if self.uses_companion_skill(id, level) {
                return;
            }
        }
        // Change Cart opens the window of the looks, or is not used at all (`clicks/cart.rs`).
        if what == Use::Skill(CHANGE_CART) && self.skills.known.iter().any(|skill| skill.id == CHANGE_CART && skill.level > 0) && !self.cart_change_used() {
            return;
        }
        let (Some(socket), Some(account)) = (&self.socket, &self.account) else { return };
        match what {
            Use::Item(index) => {
                let Some(item) = self.bag.items.iter().find(|item| item.index == index) else { return };
                if item.usable() {
                    socket.send(&items::use_item(item.index, account.account_id));
                } else if item.worn != 0 {
                    socket.send(&items::take_off(item.index));
                // A pet's accessory (the server's item kind 8) goes nowhere on the character, and is asked to be
                // worn all the same: the server puts it on the pet. The client tells such a thing by its number
                // (its Lua `IsPetAccessory`, not read here) and sends the same request (`MM_WEAR_EQUIP`).
                } else if item.wearable != 0 || item.kind == 8 {
                    socket.send(&items::wear(item.index, item.wearable));
                }
            }
            Use::Skill(id) => {
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
                    socket.send(&items::use_skill(level.unwrap_or(skill.level).min(skill.level).max(1), skill.id, account.account_id));
                } else {
                    // A skill for someone or for a place is aimed with the pointer: the next press on the map
                    // says on whom or where.
                    self.skills.aimed = Some((skill.id, level.unwrap_or(skill.level).min(skill.level).max(1)));
                    world::aim(Some(skill.target == 2), skill.range);
                    world::aim_guarded(self.guarded(skill.id));
                    self.aim_words();
                }
            }
            _ => {}
        }
    }

    /// Puts what was dragged on a slot of the hotkey bar, or empties the slot (for nothing dragged), and tells
    /// the server, which keeps the bar.
    pub(super) fn set_hotkey(&mut self, slot: usize, dragged: Option<Drag>) {
        let hotkey = match dragged {
            // Not every item may go on the bar (`Item::fits_hotkeys`): anything else is let go of and nothing
            // happens.
            Some(Drag::Item(id)) => {
                let fits = self.bag.items.iter().find(|item| item.id == id).is_some_and(Item::fits_hotkeys);
                if !fits {
                    return;
                }
                Hotkey { skill: false, id, count: 0 }
            }
            // A thing worn goes on the bar as the item it is, by the same rule.
            Some(Drag::Worn(index)) => match self.bag.items.iter().find(|item| item.index == index).filter(|item| item.fits_hotkeys()) {
                Some(item) => Hotkey { skill: false, id: item.id, count: 0 },
                None => return,
            },
            Some(Drag::Skill(skill)) => Hotkey { skill: true, id: skill.id, count: skill.level },
            // A thing of a letter being written does not go on the bar.
            Some(Drag::Attached(_)) => return,
            // Nor does a row of a shop, or a thing that lies in the storage.
            Some(Drag::Offer(_) | Drag::Basket(_) | Drag::Stored(_)) => return,
            // Nor a thing of the cart, or of a merchant's shop (the client lets go of what was taken up in the
            // cart there and nothing happens).
            Some(Drag::Carted(_) | Drag::Stock(_) | Drag::Stalled(_) | Drag::Vended(_) | Drag::Wanted(_)) => return,
            None => Hotkey::default(),
            // From another slot of the bar: what was there moves here, and that slot is left empty.
            Some(Drag::Slot(from)) => {
                let Some(moved) = self.skills.hotkeys.get(from).copied().filter(|moved| moved.id != 0 && from != slot && slot < items::HOTKEYS) else { return };
                self.skills.hotkeys.resize(items::HOTKEYS, Hotkey::default());
                self.skills.hotkeys[from] = Hotkey::default();
                self.send(&items::set_hotkey(from as u16, Hotkey::default()));
                moved
            }
        };
        if slot >= items::HOTKEYS {
            return;
        }
        self.skills.hotkeys.resize(items::HOTKEYS, Hotkey::default());
        self.skills.hotkeys[slot] = hotkey;
        self.send(&items::set_hotkey(slot as u16, hotkey));
        self.refresh_items();
    }

    /// Asks what the items carried and on the bar are called, and redraws their windows.
    pub(super) fn items_changed(&self) {
        let stored = self.storage.open.iter().flat_map(|storage| storage.items.iter().map(|item| item.id));
        let ids = self.bag.items.iter().map(|item| item.id).chain(self.skills.hotkeys.iter().filter(|hotkey| !hotkey.skill).map(|hotkey| hotkey.id)).chain(stored);
        data::want_items(ids, || dispatch(Action::ItemsKnown));
        // The weapon in hand (bit 2 of where a thing is worn) is drawn on the character.
        world::own_weapon(self.bag.items.iter().find(|item| item.worn & 2 != 0).map_or(0, |item| item.id));
        self.refresh_items();
        self.paint_figures();
    }

    /// The chat's next size, as the client steps it (`UM_TOGGLEPUSH`): one row, then three more each time up to
    /// thirteen, then none.
    pub(super) fn chat_resize(&mut self) {
        let rows = if self.chat.rows == 0 { 1 } else { self.chat.rows + 3 };
        self.chat.rows = if rows > 15 { 0 } else { rows / 3 * 3 + 1 };
    }

    /// Tells of the thing just picked up, once its name is known: a line in the chat, and a note on the screen.
    pub(super) fn tell_obtained(&mut self) {
        let Some((id, amount, identified)) = self.bag.obtained.filter(|_| !self.bag.told_obtained) else { return };
        let Some((name, _)) = data::item_as(id, identified) else { return };
        self.bag.told_obtained = true;
        self.chat_line_of(said::ITEM, format!("{name} - {amount} {}", crate::i18n::tr("obtained.")), YELLOW);
        if let Some(note) = element("obtained") {
            note.set_inner_html(&self.obtained_note());
        }
    }

    /// Adds a line to the chat.
    pub(super) fn chat_line(&mut self, words: String, colour: &str) {
        self.chat_line_of(said::GENERAL, words, colour);
    }

    /// A line for the chat window, of one of the client's kinds (`said`): the window's tabs list it or not by that.
    pub(super) fn chat_line_of(&mut self, kind: u8, words: String, colour: &str) {
        // The items a line links to are asked about, so that the line can name them.
        self.chat_links_wanted(&words);
        self.chat.lines.push((words, colour.to_owned(), kind));
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

    pub(super) fn entered_map(&mut self, packet: &[u8]) {
        let (Some(accepted), Some(zone), Some(socket)) = (packets::map_accepted(packet), &self.zone, &self.socket) else {
            return self.leave(Some(Key::Rejected.text(self.lang).to_owned()));
        };
        let name = self.characters.iter().find(|character| character.char_id == zone.char_id).map(|character| character.name.clone()).unwrap_or_default();
        let map = zone.map.clone();
        // The clock the world shows everyone's walk by is set from the server's time (`CSession::SetServerTime`).
        world::clock_set(accepted.server_tick);
        // The map counts as loaded: the server now puts the character into the world, where other players see it.
        socket.send(&packets::map_loaded());
        // Until the map server sends the character's values, the ones of the character list stand in.
        let character = self.characters.iter().find(|character| character.char_id == zone.char_id);
        let job = character.map_or(0, |character| character.job);
        self.hero.params.clear();
        self.bag.items.clear();
        self.skills.known.clear();
        self.skills.hotkeys.clear();
        // Another map server tells the states and the waits anew (the client's `ChangeZoneServer`).
        (self.statuses, self.cooldowns) = Default::default();
        // The guild is told anew too, if the character is of one; the standing on a PVP map starts over.
        (self.guild, self.pvp.rank, self.pvp.total) = Default::default();
        // The world is told who the player's own character is: its name shows under it, and its guild's once
        // the server has said (`world/guild.rs`).
        world::own_guild(self.account.as_ref().map_or(0, |account| account.account_id), &name, 0, 0);
        self.hero.stat_bonus = [0; 6];
        // The journal is the character's that enters: the server tells it anew, if it has any quest.
        (self.quests.log, self.quests.pane, self.quests.notice) = Default::default();
        // The talks with one player each, and what was open of the friends list and of another's equipment,
        // were the last character's. The friends themselves the server tells anew.
        (self.whispers.talks, self.whispers.menu, self.viewequip.seen, self.friends.asks, self.friends.menu) = Default::default();
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
        self.game = Some(Game { name, job, map, x: accepted.x, y: accepted.y, map_size: None });
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
            // What is worn on the left hand alone: a weapon of two hands is not a shield.
            shield: self.bag.items.iter().find(|item| item.worn & 0x22 == 0x20).map_or(0, |item| item.id),
            action: 0,
            frame: 0,
        }
    }

    /// The same as the windows show it: on its mount, in its machine, by the state it is in.
    pub(in super::super) fn shown_figure(&self) -> Figure {
        let figure = self.own_figure();
        Figure { job: crate::mounts::drawn_job(figure.job, self.hero.state), ..figure }
    }

    /// Draws the map the character is on. The wait box stays until it shows.
    pub(super) fn enter_world(&mut self) {
        let (Some(game), Some(zone)) = (&self.game, &self.zone) else { return };
        let _ = zone;
        let figure = self.own_figure();
        let hooks = world::Hooks {
            walk_to: |x, y| dispatch(Action::WalkTo(x, y)),
            face: |head, direction| dispatch(Action::Face(head, direction)),
            press: |id, monster| dispatch(Action::Pressed(id, monster)),
            take: |id| dispatch(Action::Take(id)),
            moved: |x, y| dispatch(Action::Moved(x, y)),
            ready: |outcome| dispatch(Action::WorldReady(outcome)),
            aimed: |whom, cell| dispatch(Action::Aimed(whom, cell)),
            menu: |id, at| dispatch(Action::Menu(id, at)),
            aim_level: |step| dispatch(Action::AimLevel(step)),
            strike_once: |id| dispatch(Action::StrikeOnce(id)),
            stop: || dispatch(Action::StopFight),
            quick: |slot| dispatch(Action::Quick(slot)),
            commanded: |mercenary, whom, cell, later| dispatch(Action::Commanded(mercenary, whom, cell, later)),
        };
        sound::map_track(data::music(&game.map));
        // The server is asked its time as each map begins (the client's `SendSyncRequest(true)` as its game
        // mode starts), and then every twelve seconds (`keep_alive`).
        self.send_on_map(&packets::tick(now()));
        world::enter(game.map.clone(), (game.x, game.y), figure, hooks);
        // The client's loading screen stands over everything until the map shows.
        loading(Some(self.loadings));
        self.loadings += 1;
        loading_ahead(self.loadings);
        self.wait = None;
        self.ask = None;
        self.render();
    }

    /// Writes the skill being aimed and the level it will be used at by the pointer.
    pub(super) fn aim_words(&self) {
        let words = self.skills.aimed.filter(|(skill, _)| *skill != TAME).map(|(skill, level)| format!("{} Lv {level}", data::skill(skill).map(|(_, name, _)| name).unwrap_or_default())).unwrap_or_default();
        world::aim_words(&words);
    }

    /// Shows the sign of the map come into, when the player has it so (the client's `ShowCurrentMapName`):
    /// the small line, the large one, on the map's picture. A map with neither line says its name.
    pub(super) fn map_sign(&self) {
        let Some(sign) = self.game.as_ref().and_then(|game| data::map_sign(&game.map)).filter(|sign| sign.shown && self.option("TT_SHOW_MAP_NAME_ON_OFF")) else { return };
        let Some(shown) = element("mapsign") else { return };
        let picture = if sign.picture.is_empty() { "noname" } else { sign.picture.as_str() };
        let title = if sign.title.is_empty() && sign.over.is_empty() { sign.name.as_str() } else { sign.title.as_str() };
        shown.set_inner_html(&format!(
            r#"<img alt="" draggable="false" src="{}" onerror="this.remove()"><div class="over">{}</div><div class="title">{}</div>"#,
            crate::web::asset_url(&format!("{}display_mapname/{picture}.png", crate::web::parts::UI)),
            escape(&sign.over),
            escape(title)
        ));
        // Taken off and put on again, so that it starts over on a map come into while the last still shows.
        shown.set_class_name("");
        let _ = shown.dyn_ref::<web_sys::HtmlElement>().map(web_sys::HtmlElement::offset_width);
        shown.set_class_name("shown");
    }
}
