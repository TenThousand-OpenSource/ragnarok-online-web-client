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
            Action::Submit(form) => self.submitted(&form),
            Action::Click(what, arg) => self.click(&what, &arg),
            Action::Open(slot) => {
                self.chime();
                self.open(slot)
            }
            Action::Use(what) => self.used(&what),
            Action::Dropped(slot, dragged) => self.set_hotkey(slot, &dragged),
            // What was on the hotkey bar, dragged off it and let go over the world: the slot is emptied.
            Action::Thrown(dragged) if dragged.starts_with("slot:") => {
                if let Some(slot) = dragged.strip_prefix("slot:").and_then(|slot| slot.parse::<usize>().ok()) {
                    self.set_hotkey(slot, "");
                }
            }
            Action::Thrown(dragged) => self.thrown(&dragged),
            Action::SkillInfo(id) => {
                self.skills.info = Some(id);
                data::want_skill(id, || dispatch(Action::ItemsKnown));
                self.refresh_items();
            }
            Action::Info(id, identified) => {
                self.bag.info = Some((id, identified));
                data::want_items(std::iter::once(id), || dispatch(Action::ItemsKnown));
                self.refresh_items();
            }
            Action::ItemsKnown => {
                self.tell_obtained();
                self.tell_fed();
                self.refresh_items();
                self.refresh_trade();
                self.refresh_guide();
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
            Action::Socket(generation, event) => self.socket_event(generation, event),
            Action::KeepAlive => self.keep_alive(),
            // While a monster is being tamed nothing is struck: the slot machine comes first.
            Action::Pressed(_, true) if self.pets.taming.is_some() => world::stand_down(),
            Action::Pressed(id, monster) => {
                if let (Server::Map, Some(socket)) = (self.server, &self.socket) {
                    // A monster is fought until it or the character is down; anyone else is talked to.
                    socket.send(&if monster { units::attack(id) } else { npc::contact(id) });
                }
            }
            // A monster was picked to tame: the slot machine opens for it (not for players, as in the client).
            Action::Aimed(Some(target), _) if self.skills.aimed.is_some_and(|(skill, _)| skill == TAME) => self.taming_begins(target),
            Action::Aimed(None, _) if self.skills.aimed.is_some_and(|(skill, _)| skill == TAME) => self.skills.aimed = None,
            Action::GuideRead(id, picture) => {
                self.guide.pictures.insert(id, picture);
                self.refresh_guide();
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
            Action::TamingTick => self.taming_tick(),
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
            Action::Menu(id, at) => self.unit_menu(id, at),
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
                // No walk is asked for while the character's own blow is in motion, as in the original client; a
                // button held on the ground asks again once it is over.
                if world::striking() {
                    return;
                }
                if let (Server::Map, Some(socket)) = (self.server, &self.socket) {
                    socket.send(&packets::walk_to(x, y));
                }
            }
            Action::Moved(x, y) => self.moved(x, y),
            Action::WorldReady(outcome) => self.world_ready(outcome),
            Action::Arrive(map, x, y) => self.arrived(map, x, y),
            Action::Answer(request, answer) => self.answered(request, answer),
        }
    }
}
