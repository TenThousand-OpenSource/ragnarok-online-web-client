//! The character moving: over the cells of a map, and from map to map.

use super::*;

impl App {
    /// The character stands on another cell: the place under the minimap and the minimap itself follow.
    pub(super) fn moved(&mut self, x: u16, y: u16) {
        if let Some(game) = &mut self.game {
            (game.x, game.y) = (x, y);
            if let Some(position) = element("position") {
                position.set_text_content(Some(&format!(" {x}  {y}")));
            }
            if let Some((pictured, marked)) = crate::web::view::minimap_place(game, self.minimap_zoom) {
                for (id, style) in [("minimap", pictured), ("marker", marked)] {
                    if let Some(shown) = element(id) {
                        let _ = shown.set_attribute("style", &style);
                    }
                }
            }
            // Brought nearer, the minimap moves under its marks as the character walks.
            if self.minimap_zoom > 1.0 {
                if let Some(marks) = element("map-marks") {
                    marks.set_inner_html(&crate::web::view::minimap_marks(game, self.minimap_zoom, self.minimap_marks_off));
                    self.party_marks();
                    self.quest_marks();
                    self.compass_marks();
                }
            }
        }
    }

    /// The map is drawn, or could not be.
    pub(super) fn world_ready(&mut self, outcome: Result<(), String>) {
        if !matches!(self.screen, Screen::Game) {
            return;
        }
        // The world goes by some of the player's options from its first frame.
        self.apply_options();
        if outcome.is_ok() {
            self.map_sign();
            // The Tipbox opens as the game is first entered, unless the player has it not.
            self.tips_at_start();
            // The effects of the states the character is in begin anew on the map it has come to.
            world::own_states_again(self.statuses.on.iter().map(|active| active.state));
        }
        if let Err(error) = outcome {
            web_sys::console::error_1(&format!("world: {error}").into());
            self.message = Some(error);
        }
        if let Some(game) = &mut self.game {
            game.map_size = world::map_size();
        }
        self.wait = None;
        self.render();
        // The loading screen goes, and the game comes out of the dark.
        loading(None);
        fade(false);
    }

    /// The server put the character somewhere, on this map or on another.
    pub(super) fn arrived(&mut self, map: String, x: u16, y: u16) {
        if !matches!(self.screen, Screen::Game) {
            return fade(false);
        }
        // Right after the character comes in the server may name its place once more, while the map is
        // still being read: the reading goes on, and the character is put there when the map shows.
        let reading = world::reading(&map);
        let same = self.game.as_ref().is_some_and(|game| game.map == map) && (world::map_size().is_some() || reading);
        if let Some(game) = &mut self.game {
            (game.x, game.y) = (x, y);
            if !same {
                (game.map, game.map_size) = (map, None);
            }
        }
        // A dead character arrives alive.
        self.hero.dead = false;
        self.esc = 0;
        self.dialog = None;
        // What an NPC's script showed and the chat room sat in are of the place left.
        self.scripted_moved(same);
        self.chat_rooms_moved();
        self.refresh_values();
        self.redraw(&[&wnd::DIALOG]);
        if same {
            // The map is already drawn: the character is only put elsewhere on it.
            world::jump((x, y));
            world::own_states_again(self.statuses.on.iter().map(|active| active.state));
            // (While the map is read, its loading screen stays; the dark goes when it shows.)
            if !reading {
                fade(false);
            }
        } else {
            self.enter_world();
        }
        self.send(&packets::map_loaded());
    }

    pub(in crate::web::app) fn entered_map(&mut self, packet: &[u8]) {
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
        world::own_guild(self.own_account(), &name, 0, 0);
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

    /// Draws the map the character is on. The wait box stays until it shows.
    pub(in crate::web::app) fn enter_world(&mut self) {
        let (Some(game), Some(_)) = (&self.game, &self.zone) else { return };
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

    /// Shows the sign of the map come into, when the player has it so (the client's `ShowCurrentMapName`):
    /// the small line, the large one, on the map's picture. A map with neither line says its name.
    pub(in crate::web::app) fn map_sign(&self) {
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
