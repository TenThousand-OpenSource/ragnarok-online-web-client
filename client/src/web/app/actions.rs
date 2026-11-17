//! What the app does about each action: the one place every event of the page and of the world arrives at.

use super::*;

impl App {
    pub(super) fn handle(&mut self, action: Action) {
        match action {
            Action::Submit(form) => match form.as_str() {
                "login" => {
                    self.chime();
                    self.login()
                }
                "register" => self.register(),
                "recover" => self.recover(),
                "reset" => self.reset(),
                "create" => self.create_character(),
                "throw" => {
                    if let (Some((index, most)), Some(socket)) = (self.bag.throwing.take(), &self.socket) {
                        let count = field("throw-count").trim().parse::<u32>().unwrap_or(0).min(u32::from(most)) as u16;
                        if count > 0 {
                            socket.send(&items::throw(index, count));
                        }
                    }
                    self.refresh_throw();
                }
                "chat" => {
                    let words = field("chat-input");
                    let words = words.trim();
                    // The client's own commands come first; everything else is said (the server reads its
                    // @commands).
                    if !words.is_empty() && self.command(words) {
                        set_field("chat-input", "");
                        return;
                    }
                    if let (false, Some(socket), Some(game)) = (words.is_empty(), &self.socket, &self.game) {
                        // A name in the small box at the left sends the words to that one alone.
                        let to = field("chat-to");
                        let to = to.trim();
                        if !to.is_empty() {
                            socket.send(&npc::whisper(to, words));
                            let line = format!("( To {to} ) : {words}");
                            set_field("chat-input", "");
                            return self.chat_line(line, "#ffff00");
                        }
                        socket.send(&packets::say(&game.name, words));
                        set_field("chat-input", "");
                    }
                }
                _ => {}
            },
            Action::Click(what, arg) => self.click(&what, &arg),
            Action::Open(slot) => {
                self.chime();
                self.open(slot)
            }
            Action::Use(what) => match (what.split_once(':'), &self.socket) {
                // With the storage open, a double click moves the whole of a stack into it or out of it.
                // A double click in the shop's list puts a thing aside; one on what was put aside takes it back.
                (Some(("shop-add", row)), _) => {
                    let Ok(row) = row.parse::<usize>() else { return };
                    // How many: what the line under the list says, or one bought and all of a stack sold.
                    let typed = field("shop-amount").trim().parse::<u32>().ok().filter(|amount| *amount > 0);
                    let most = match &self.deal.shop {
                        Some(Shop::Buy(offers)) if row < offers.len() => 30000,
                        Some(Shop::Sell(bids)) => bids.get(row).and_then(|(index, _)| self.bag.items.iter().find(|item| item.index == *index)).map_or(0, |item| u32::from(item.amount)),
                        _ => 0,
                    };
                    let buying = matches!(self.deal.shop, Some(Shop::Buy(_)));
                    let aside = self.deal.basket.iter().find(|(kept, _)| *kept == row).map_or(0, |(_, amount)| u32::from(*amount));
                    let amount = typed.unwrap_or(if buying { 1 } else { most }).min(most - aside.min(most)) as u16;
                    if amount > 0 {
                        match self.deal.basket.iter_mut().find(|(kept, _)| *kept == row) {
                            Some((_, kept)) => *kept += amount,
                            None => self.deal.basket.push((row, amount)),
                        }
                    }
                    self.refresh_trade();
                }
                (Some(("shop-remove", row)), _) => {
                    let row = row.parse::<usize>().ok();
                    self.deal.basket.retain(|(kept, _)| Some(*kept) != row);
                    self.refresh_trade();
                }
                (Some(("put", index)), Some(socket)) => {
                    if let Some(item) = index.parse().ok().and_then(|index: u16| self.bag.items.iter().find(|item| item.index == index && item.worn == 0)) {
                        socket.send(&trade::to_storage(item.index, u32::from(item.amount)));
                    }
                }
                (Some(("take", index)), Some(socket)) => {
                    if let Some(item) = index.parse().ok().and_then(|index: u16| self.deal.storage.as_ref()?.items.iter().find(|item| item.index == index)) {
                        socket.send(&trade::from_storage(item.index, u32::from(item.amount)));
                    }
                }
                _ => self.use_it(&what),
            },
            Action::Dropped(slot, dragged) => self.set_hotkey(slot, &dragged),
            // What was on the hotkey bar, dragged off it and let go over the world: the slot is emptied.
            Action::Thrown(dragged) if dragged.starts_with("slot:") => {
                if let Some(slot) = dragged.strip_prefix("slot:").and_then(|slot| slot.parse::<usize>().ok()) {
                    self.set_hotkey(slot, "");
                }
            }
            Action::Thrown(dragged) => {
                // The first of that item carried without being worn. One is dropped at once; of more, the
                // original asks how many.
                let Some(id) = dragged.strip_prefix("item:").and_then(|id| id.parse::<u32>().ok()) else { return };
                let Some(item) = self.bag.items.iter().find(|item| item.id == id && item.worn == 0) else { return };
                if item.amount <= 1 {
                    if let Some(socket) = &self.socket {
                        socket.send(&items::throw(item.index, 1));
                    }
                } else {
                    self.bag.throwing = Some((item.index, item.amount));
                    self.refresh_throw();
                }
            }
            Action::SkillInfo(id) => {
                self.skills.info = Some(id);
                data::want_skill(id, || dispatch(Action::ItemsKnown));
                self.refresh_items();
            }
            Action::Info(id) => {
                self.bag.info = Some(id);
                data::want_items(std::iter::once(id), || dispatch(Action::ItemsKnown));
                self.refresh_items();
            }
            Action::ItemsKnown => {
                self.tell_obtained();
                self.tell_fed();
                self.refresh_items();
                self.refresh_trade();
                self.paint_figures();
            }
            Action::Resized => self.render(),
            Action::CheckMusic => {
                let hidden = web_sys::window().and_then(|window| window.document()).is_some_and(|document| document.hidden());
                let held = self.sound && !hidden && !matches!(self.screen, Screen::Game) && !sound::music_playing();
                if held != self.music_held {
                    self.music_held = held;
                    self.render();
                }
            }
            Action::RetryLogin(generation) => {
                if let Some(credentials) = self.relogin.clone().filter(|_| generation == self.generation) {
                    self.login_tries += 1;
                    self.credentials = Some(credentials);
                    self.connect(Server::Login, LOGIN_PORT);
                }
            }
            Action::Socket(generation, _) if generation != self.generation => {}
            Action::Socket(generation, event) => match event {
                SocketEvent::Open => self.connected(),
                SocketEvent::Bytes(bytes) => self.received(generation, &bytes),
                SocketEvent::Closed(TOO_MANY_ATTEMPTS, reason) => {
                    let wait = minutes(reason.parse().unwrap_or(60));
                    self.leave(Some(fill(Key::TooManyLogins.text(self.lang), &[&wait])));
                }
                SocketEvent::Closed(..) => {
                    let text = if self.server == Server::Login && self.account.is_none() { Key::ConnectFailed } else { Key::Disconnected };
                    self.leave(Some(text.text(self.lang).to_owned()));
                }
            },
            Action::KeepAlive => match (&self.server, &self.socket, &self.account) {
                (Server::Char, Some(socket), Some(account)) => socket.send(&packets::ping(account.account_id)),
                (Server::Map, Some(socket), _) if self.game.is_some() => {
                    socket.send(&packets::tick(now()));
                    // Every half a minute, and at once on coming in: how many play (CZ_REQ_USER_COUNT, 0x00c1,
                    // what the client's `/who` sends).
                    if self.ticks % 3 == 0 {
                        socket.send(&0x00c1u16.to_le_bytes());
                    }
                    self.ticks = self.ticks.wrapping_add(1);
                }
                _ => {}
            },
            // While a monster is being tamed nothing is struck: the slot machine comes first.
            Action::Pressed(_, true) if self.pets.taming.is_some() => world::stand_down(),
            Action::Pressed(id, monster) => {
                if let (Server::Map, Some(socket)) = (self.server, &self.socket) {
                    // A monster is fought until it or the character is down; anyone else is talked to.
                    socket.send(&if monster { units::attack(id) } else { npc::contact(id) });
                }
            }
            // A monster was picked to tame: the slot machine opens for it (not for players, as in the client).
            Action::Aimed(Some(target), _) if self.skills.aimed.is_some_and(|(skill, _)| skill == TAME) => {
                self.skills.aimed = None;
                self.close_taming();
                // The fight, if there was one, stops for the taming.
                world::stand_down();
                if let Some(socket) = &self.socket {
                    // CZ_CANCEL_LOCKON (0x0118): the server stops the blows it was dealing on its own.
                    socket.send(&0x0118u16.to_le_bytes());
                }
                let tick = Closure::<dyn FnMut()>::new(|| dispatch(Action::TamingTick));
                let timer = web_sys::window().and_then(|window| window.set_interval_with_callback_and_timeout_and_arguments_0(tick.as_ref().unchecked_ref(), 16).ok()).unwrap_or(0);
                tick.forget();
                self.pets.taming = Some(Taming { target, state: 0, tick: 0, delay: 1, longest: 10000, wanted: -1, started: false, rested: None, frames: [Vec::new(), Vec::new()], shown: (usize::MAX, 0), timer });
                wasm_bindgen_futures::spawn_local(async {
                    let mut read = [Vec::new(), Vec::new()];
                    for (action, frames) in read.iter_mut().enumerate() {
                        for frame in 0..64 {
                            if frame > 0 && figure::timing("slotmachine", action).is_some_and(|(count, _)| frame >= count) {
                                break;
                            }
                            match figure::frame_picture("slotmachine", action, frame).await {
                                Ok(picture) => frames.push(picture),
                                Err(_) => break,
                            }
                        }
                    }
                    let [waiting, turning] = read;
                    dispatch(Action::TamingRead(waiting, turning));
                });
                self.render();
            }
            Action::Aimed(None, _) if self.skills.aimed.is_some_and(|(skill, _)| skill == TAME) => self.skills.aimed = None,
            Action::TamingRead(waiting, turning) => {
                if let Some(taming) = &mut self.pets.taming {
                    taming.frames = [waiting, turning];
                }
            }
            // Three seconds after its trick the pet, one time in four, shows how it feels standing about (the
            // client's `CGameActor::ProcessState`).
            Action::PetIdle => {
                if js_sys::Math::random() < 0.25 {
                    self.pet_feels(pets::moment::IDLE, None);
                }
            }
            // The client's `Paint` and `OnProcess` of the window, once a moment.
            Action::TamingTick => {
                let Some(taming) = &mut self.pets.taming else { return };
                let count = taming.frames[taming.state].len() as i32;
                if count == 0 {
                    return;
                }
                let frame = taming.tick / taming.delay.max(1);
                let shown = (taming.state, frame.clamp(0, count - 1) as usize);
                if taming.wanted != frame || taming.delay > taming.longest {
                    taming.tick += 1;
                }
                if frame >= count {
                    taming.tick = 0;
                    if taming.wanted != -1 {
                        taming.delay += 1;
                    }
                }
                let resting = taming.wanted == frame && taming.delay <= taming.longest;
                let over = match taming.rested {
                    None => {
                        if resting {
                            taming.rested = Some(js_sys::Date::now());
                        }
                        false
                    }
                    Some(since) => resting && js_sys::Date::now() - since >= 2000.0,
                };
                // (The page may have been written anew since, and the picture with it.)
                if shown != taming.shown || element("taming-picture").is_some_and(|picture| picture.get_attribute("src").is_none()) {
                    taming.shown = shown;
                    if let Some(picture) = element("taming-picture") {
                        let (url, _, origin) = &taming.frames[shown.0][shown.1];
                        let _ = picture.set_attribute("src", url);
                        let _ = picture.set_attribute("style", &format!("position:absolute;left:{}px;top:{}px", 98.0 - origin.0, 128.0 - origin.1));
                    }
                }
                if over {
                    self.close_taming();
                    self.render();
                }
            }
            Action::Aimed(whom, cell) => {
                if let (Server::Map, Some(socket), Some((skill, level))) = (self.server, &self.socket, self.skills.aimed.take()) {
                    socket.send(&match whom {
                        Some(id) => items::use_skill(level, skill, id),
                        None => crate::net::skills::use_on_ground(level, skill, cell),
                    });
                }
            }
            Action::Slid(name, level) => self.settings_slide(&name, level),
            Action::StrikeOnce(id) => {
                if let (Server::Map, Some(socket)) = (self.server, &self.socket) {
                    socket.send(&units::attack_once(id));
                }
            }
            // CZ_CANCEL_LOCKON (0x0118): the server stops the blows it was dealing on its own.
            Action::StopFight => {
                if let (Server::Map, Some(socket)) = (self.server, &self.socket) {
                    socket.send(&0x0118u16.to_le_bytes());
                }
            }
            Action::Quick(slot) => self.use_it(&format!("slot:{slot}")),
            // The level the aimed skill is used at, one up or down, between 1 and the level it is known at
            // (the client's `MM_USESKILL_LEVEL_PLUS`).
            Action::AimLevel(step) => {
                if let Some((skill, level)) = self.skills.aimed.filter(|(skill, _)| *skill != TAME) {
                    let most = self.skills.known.iter().find(|known| known.id == skill).map_or(1, |known| known.level.max(1));
                    self.skills.aimed = Some((skill, (i32::from(level) + step).clamp(1, i32::from(most)) as u16));
                    self.aim_words();
                }
            }
            // The right button on someone. The client has a menu for the player's own pet (its status, feeding,
            // a trick, the accessory off, back into the egg), for a homunculus, a mercenary and other players;
            // for a monster or an NPC it has none. Here: the pet's.
            Action::Menu(id, at) => {
                if id == self.pets.unit && self.pets.pet.is_some() {
                    // The page's place, in the stage's own measure: the stage may be drawn larger or smaller.
                    let stage = web_sys::window().and_then(|window| window.document()).and_then(|document| document.query_selector(".stage").ok().flatten());
                    let scale = stage.as_ref().and_then(|stage| Some(stage.get_bounding_client_rect().width() / f64::from(stage.clone().dyn_into::<web_sys::HtmlElement>().ok()?.offset_width().max(1)))).filter(|scale| *scale > 0.0).unwrap_or(1.0);
                    self.pets.menu_at = Some(((f64::from(at.0) / scale) as i32 + 1, (f64::from(at.1) / scale) as i32 + 1));
                    self.render();
                }
            }
            Action::Take(id) => {
                if let (Server::Map, Some(socket)) = (self.server, &self.socket) {
                    socket.send(&units::pick_up(id));
                }
            }
            Action::WalkTo(x, y) => {
                // A press elsewhere puts the pet's menu away.
                if self.pets.menu_at.take().is_some() {
                    self.render();
                }
                if let (Server::Map, Some(socket)) = (self.server, &self.socket) {
                    socket.send(&packets::walk_to(x, y));
                }
            }
            Action::Moved(x, y) => {
                if let Some(game) = &mut self.game {
                    (game.x, game.y) = (x, y);
                    if let Some(position) = element("position") {
                        position.set_text_content(Some(&format!(" {x}  {y}")));
                    }
                    if let Some((pictured, marked)) = super::super::view::minimap_place(game, self.minimap_zoom) {
                        for (id, style) in [("minimap", pictured), ("marker", marked)] {
                            if let Some(shown) = element(id) {
                                let _ = shown.set_attribute("style", &style);
                            }
                        }
                    }
                }
            }
            Action::WorldReady(outcome) => {
                if !matches!(self.screen, Screen::Game) {
                    return;
                }
                // The world goes by some of the player's options from its first frame.
                self.apply_options();
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
            Action::Arrive(map, x, y) => {
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
                self.refresh_values();
                self.refresh_dialog();
                if same {
                    // The map is already drawn: the character is only put elsewhere on it.
                    world::jump((x, y));
                    // (While the map is read, its loading screen stays; the dark goes when it shows.)
                    if !reading {
                        fade(false);
                    }
                } else {
                    self.enter_world();
                }
                if let Some(socket) = &self.socket {
                    socket.send(&packets::map_loaded());
                }
            }
            Action::Answer(request, answer) => self.answered(request, answer),
        }
    }
}
