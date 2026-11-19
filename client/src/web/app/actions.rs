//! What the app does about each action: the one place every event of the page and of the world arrives at.
//!
//! `handle` only says who sees to each action. What takes more than a few lines is a method in one of the files
//! here, by what it is about.

use super::*;

mod forms;
mod bag;
mod link;
mod taming;
mod travel;

impl App {
    pub(super) fn handle(&mut self, action: Action) {
        match action {
            Action::Submit(form) => self.submitted(form),
            Action::Click(what, arg) => self.click(&what, &arg),
            Action::Open(slot) => {
                self.chime();
                self.open(slot)
            }
            Action::Use(what) => self.used(what),
            Action::Dropped(slot, dragged) => self.set_hotkey(slot, dragged),
            Action::WindowPressed(window) => self.window_pressed(&window),
            // What was on the hotkey bar, dragged off it and let go over the world: the slot is emptied.
            Action::Thrown(Drag::Slot(slot)) => self.set_hotkey(slot, None),
            Action::Thrown(dragged) => self.thrown(dragged),
            Action::Offered(dragged) => self.exchange_offered(dragged),
            Action::Mailed(dragged) => self.mail_dropped(dragged),
            Action::Uncarted(index) => self.cart_let_out(index, true),
            Action::CartLookRead(kind, picture, origin) => self.cart_look_read(kind, picture, origin),
            Action::LetGo(dragged, onto) => self.let_go(dragged, onto),
            Action::Unstored(index) => self.storage_let_out(index),
            Action::Stowed(what) => self.storage_stowed(what),
            Action::Forged(dragged) => self.forge_dropped(dragged),
            Action::Unmailed(index) => self.mail_taken_out(index),
            Action::SkillInfo(id) => {
                self.skills.info = Some(id);
                data::want_skill(id, || dispatch(Action::ItemsKnown));
                self.refresh_items();
            }
            Action::Info(id, identified) => {
                self.bag.info = Some((id, identified));
                // With the item, the cards set into it where it is carried: its window draws them in its slots.
                let cards: Vec<u32> = self.bag.items.iter().filter(|item| item.id == id).flat_map(|item| crate::item_names::Named::from(item).cards_set()).collect();
                data::want_items(std::iter::once(id).chain(cards), || dispatch(Action::ItemsKnown));
                // A book has its two buttons there, whose pictures are read.
                if data::is_book(id) {
                    self.book_pictures_wanted();
                }
                self.refresh_items();
            }
            Action::ItemsKnown => {
                self.chat_links_named();
                self.tell_obtained();
                self.mvp_item_said();
                self.tell_fed();
                self.tell_homunculus_fed();
                self.refresh_items();
                self.redraw(&[&wnd::TRADE]);
                self.redraw(&[&wnd::GUIDE]);
                self.redraw(&[&wnd::QUEST_PANE]);
                self.redraw(&[&wnd::MAIL_READ, &wnd::MAIL_WRITE]);
                self.vending_redrawn();
                self.redraw(&[&wnd::VIEW_EQUIP]);
                self.redraw(&[&wnd::CHOOSE_PICTURED, &wnd::CHOOSE_LINE]);
                self.paint_figures();
            }
            Action::Resized => self.render(),
            Action::CheckMusic => {
                let hidden = crate::web::document().is_some_and(|document| document.hidden());
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
            Action::Socket(generation, event) => self.socket_event(generation, event),
            Action::KeepAlive => self.keep_alive(),
            Action::Watch => self.watch(),
            // While a monster is being tamed nothing is struck: the slot machine comes first.
            Action::Pressed(_, true) if self.pets.taming.is_some() => world::stand_down(),
            Action::Pressed(id, monster) => {
                // A monster is fought until it or the character is down; anyone else is talked to.
                self.send_on_map(&if monster { units::attack(id) } else { npc::contact(id) });
            }
            // A monster was picked to tame: the slot machine opens for it (not for players, as in the client).
            Action::Aimed(Some(target), _) if self.skills.aimed.is_some_and(|(skill, _)| skill == TAME) => self.taming_begins(target),
            Action::Aimed(None, _) if self.skills.aimed.is_some_and(|(skill, _)| skill == TAME) => self.skills.aimed = None,
            Action::GuideRead(id, picture) => {
                self.guide.pictures.insert(id, picture);
                self.redraw(&[&wnd::GUIDE]);
            }
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
            Action::PvpRead(digits) => {
                self.pvp.digits = digits;
                self.rank_shown();
            }
            Action::EmotionsRead(pictures) => self.emotions_read(pictures),
            Action::Pictured(saved) => self.screenshot_saved(saved),
            Action::TamingTick => self.taming_tick(),
            Action::StatusTick => self.status_tick(),
            Action::Scripted => self.scripted_due(),
            Action::Book(news) => self.book_news(news),
            Action::AiTick => self.ai_tick(),
            Action::Commanded(mercenary, whom, cell, later) => self.commanded(mercenary, whom, cell, later),
            // A skill of the homunculus or of the mercenary is used by its script, which is told where.
            Action::Aimed(whom, cell) if self.companion_aimed(whom, cell) => {}
            Action::Aimed(whom, cell) => {
                // Sense names the monster it tells of by whom it was used on (`incoming/fight.rs`).
                if self.skills.aimed.is_some_and(|(skill, _)| skill == 93) {
                    self.fight.aimed = whom;
                }
                if let (Server::Map, Some(socket), Some((skill, level))) = (self.server, &self.socket, self.skills.aimed.take()) {
                    socket.send(&match whom {
                        Some(id) => items::use_skill(level, skill, id),
                        None => crate::net::skills::use_on_ground(level, skill, cell),
                    });
                }
            }
            Action::Slid(name, level) => self.settings_slide(&name, level),
            Action::StrikeOnce(id) => {
                self.send_on_map(&units::attack_once(id));
            }
            Action::StopFight => {
                self.send_on_map(&units::stop_attack());
            }
            Action::Quick(slot) => self.use_it(Use::Slot(slot as usize)),
            // The level the aimed skill is used at, one up or down, between 1 and the level it is known at
            // (the client's `MM_USESKILL_LEVEL_PLUS`).
            Action::AimLevel(step) => {
                if let Some((skill, level)) = self.skills.aimed.filter(|(skill, _)| *skill != TAME) {
                    let most = self.skills.known.iter().find(|known| known.id == skill).cloned().or_else(|| self.companion_skill(skill).map(|(known, ..)| known)).map_or(1, |known| known.level.max(1));
                    self.skills.aimed = Some((skill, (i32::from(level) + step).clamp(1, i32::from(most)) as u16));
                    self.aim_words();
                }
            }
            Action::Menu(id, at) => self.unit_menu(id, at),
            Action::MenuOn(things::Menu::Member(row), at) => self.party_menu_opens(row, at),
            Action::MenuOn(things::Menu::GuildMember(row), at) => self.guild_menu_opens(GuildMenu::Member(row), at),
            Action::MenuOn(things::Menu::GuildRelated(row), at) => self.guild_menu_opens(GuildMenu::Related(row), at),
            Action::MenuOn(things::Menu::Friend(row), at) => self.friend_menu_opens(row, at),
            Action::MenuOn(things::Menu::Whisper(place), at) => self.whisper_menu_opens(place, at),
            Action::MenuOn(things::Menu::Room, at) => self.room_menu_opens(None, at),
            Action::MenuOn(things::Menu::RoomMember(row), at) => self.room_menu_opens(Some(row), at),
            Action::Take(id) => {
                self.send_on_map(&units::pick_up(id));
            }
            Action::WalkTo(x, y) => {
                self.close_menus();
                // No walk is asked for while the character's own blow is in motion, as in the original client; a
                // button held on the ground asks again once it is over.
                if world::striking() {
                    return;
                }
                self.send_on_map(&packets::walk_to(x, y));
            }
            Action::Face(head, direction) => self.send_on_map(&units::turn(head, direction)),
            Action::Moved(x, y) => self.moved(x, y),
            Action::WorldReady(outcome) => self.world_ready(outcome),
            Action::Arrive(map, x, y) => self.arrived(map, x, y),
            Action::Guild(event) => self.guild_event(event),
            Action::Answer(request, answer) => self.answered(request, answer),
        }
    }
}
