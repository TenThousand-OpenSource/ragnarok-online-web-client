//! What the app does about each action: the one place every event of the page and of the world arrives at.
//!
//! `handle` only says who sees to each action. What takes more than a few lines is a method in one of the files
//! here, by what it is about.

use super::*;

mod forms;
mod bag;
mod link;
mod pointer;
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
            Action::WindowPressed(window) => {
                self.window_pressed(&window);
                self.party_mini_pressed(&window);
            }
            Action::WindowDropped(window) => self.window_dropped(&window),
            Action::ChatTabTorn(tab, pointer) => self.chat_tab_torn(tab, pointer),
            Action::ChatOwnSized(tab, width, height) => self.chat_own_sized(tab, width, height),
            // What was on the hotkey bar, dragged off it and let go over the world: the slot is emptied.
            Action::Thrown(Drag::Slot(slot)) => self.set_hotkey(slot, None),
            Action::Thrown(Drag::Member(row)) => self.party_mini_taken(row),
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
            Action::SkillInfo(id) => self.skill_info(id),
            Action::Info(id, identified) => self.item_info(id, identified),
            Action::ItemsKnown => self.items_known(),
            Action::Navigation(news) => self.navigation_news(news),
            Action::Resized => self.render(),
            Action::CheckMusic => self.music_checked(),
            Action::RetryLogin(generation) => self.login_retried(generation),
            Action::Socket(generation, _) if generation != self.generation => {}
            Action::Socket(generation, event) => self.socket_event(generation, event),
            Action::KeepAlive => self.keep_alive(),
            Action::Watch => self.watch(),
            // While a monster is being tamed nothing is struck: the slot machine comes first.
            Action::Pressed(_, true) if self.pets.taming.is_some() => world::stand_down(),
            Action::Pressed(id, monster) => self.unit_pressed(id, monster),
            // A monster was picked to tame: the slot machine opens for it (not for players, as in the client).
            Action::Aimed(Some(target), _) if self.skills.aimed.is_some_and(|(skill, _)| skill == TAME) => self.taming_begins(target),
            Action::Aimed(None, _) if self.skills.aimed.is_some_and(|(skill, _)| skill == TAME) => self.skills.aimed = None,
            Action::GuideRead(id, picture) => self.guide_picture_read(id, picture),
            Action::TamingRead(waiting, turning) => self.taming_read(waiting, turning),
            Action::PetIdle => self.pet_idles(),
            Action::PvpRead(digits) => self.pvp_digits_read(digits),
            Action::BattlegroundTick => self.battleground_tick(),
            Action::ReplayTick => self.replay_tick(),
            Action::ReplaysRead(files) => self.replays_read(files),
            Action::ReplayPicked(name, bytes) => self.replay_taken(name, bytes),
            Action::AgencyAnswered(request, answer) => self.agency_answered(request, answer),
            Action::EmotionsRead(pictures) => self.emotions_read(pictures),
            Action::Pictured(saved) => self.screenshot_saved(saved),
            Action::TamingTick => self.taming_tick(),
            Action::StatusTick => self.status_tick(),
            Action::RankingLooked => self.ranking_looked(),
            Action::Scripted => self.scripted_due(),
            Action::Book(news) => self.book_news(news),
            Action::AiTick => self.ai_tick(),
            Action::Commanded(mercenary, whom, cell, later) => self.commanded(mercenary, whom, cell, later),
            // A skill of the homunculus or of the mercenary is used by its script, which is told where.
            Action::Aimed(whom, cell) if self.companion_aimed(whom, cell) => {}
            Action::Aimed(whom, cell) => self.skill_aimed(whom, cell),
            Action::AimedOwn => self.own_aimed(),
            Action::Slid(name, level) => self.settings_slide(&name, level),
            Action::StrikeOnce(id) => self.send_on_map(&net::actors::attack_once(id)),
            Action::StopFight => self.send_on_map(&net::actors::stop_attack()),
            Action::Quick(slot) => self.used(Use::Slot(slot as usize)),
            Action::AimLevel(step) => self.aim_level_stepped(step),
            Action::Menu(id, at) => self.unit_menu(id, at),
            Action::MenuOn(what, at) => self.menu_on(what, at),
            Action::Take(id) => self.send_on_map(&net::actors::pick_up(id)),
            Action::WalkTo(x, y) => self.walk_asked(x, y),
            Action::Face(head, direction) => self.send_on_map(&net::actors::turn(head, direction)),
            Action::Moved(x, y) => self.moved(x, y),
            Action::WorldReady(outcome) => self.world_ready(outcome),
            Action::Arrive(map, x, y) => self.arrived(map, x, y),
            Action::Guild(event) => self.guild_event(event),
            Action::Answer(request, answer) => self.answered(request, answer),
        }
    }
}
