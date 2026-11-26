//! What the server tells of the achievements and the titles (`crate::net::achievements`).

use super::*;
use crate::net::achievements::{self, Record, Standing};

/// The client's effect of an achievement completed (`EF_ACH_COMPLETE`).
const COMPLETED: u32 = 1094;
/// How long the line across the screen that tells of it stays, in milliseconds (`CGameMode::OnProcess`: ten
/// seconds after `achievementBalloonTick`).
const TOLD_FOR: f64 = 10_000.0;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn achievements_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // Another character of the account: what the one before had is forgotten (`CSession::ClearAchievements`).
            packets::ZC_ACCEPT_ENTER => {
                self.achievements = Achievements::default();
                return false;
            }
            // `CGameMode::ReceiveAchievementList`.
            achievements::ZC_ALL_ACH_LIST => {
                self.own_title_asked();
                if let Some((standing, records)) = achievements::list(packet) {
                    self.achievements.standing = standing;
                    self.achievements.records.extend(records.into_iter().map(|record| (record.id, record)));
                    self.achievements_redrawn();
                }
            }
            achievements::ZC_ACH_UPDATE => {
                self.own_title_asked();
                if let Some((standing, record)) = achievements::update(packet) {
                    self.achievement_changed(standing, record);
                }
            }
            // `CGameMode::ReceiveAchievementResult`: the reward was given; a title it gave is one more to wear.
            achievements::ZC_REQ_ACH_REWARD_ACK => {
                if let Some((false, achievement)) = achievements::answered(packet) {
                    let title = data::achievements().iter().find(|info| info.id == achievement).map_or(0, |info| info.title_id);
                    if title > 0 && !self.achievements.new_titles.contains(&title) {
                        self.achievements.new_titles.push(title);
                    }
                    self.achievements_redrawn();
                }
            }
            // `CGameMode::ReceiveJobTitle`: the title is worn, and written before the character's name.
            achievements::ZC_ACK_CHANGE_TITLE => {
                if let Some((false, title)) = achievements::answered(packet) {
                    self.achievements.title = title;
                    world::player_titled(self.own_account(), &data::title_name(title));
                    self.redraw(&[&wnd::EQUIPMENT]);
                }
            }
            // A player's names: the title is told with them (`CGameMode::SetActorNameCard`), the player's own
            // too. The guild reads the rest.
            net::guild::ZC_ACK_REQNAMEALL => {
                if let (Some(who), Some(title)) = (net::guild::names(packet).map(|(who, _)| who), achievements::title_named(packet)) {
                    if who == self.own_account() && self.achievements.title != title {
                        self.achievements.title = title;
                        self.redraw(&[&wnd::EQUIPMENT]);
                    }
                    world::player_titled(who, &data::title_name(title));
                }
                return false;
            }
            _ => return false,
        }
        true
    }

    /// The title the character wears is told with its names, which the client asks for when the pointer first
    /// comes over the character (so that its page of titles shows none until then). Here they are asked for
    /// once, as the server first tells of the achievements after the game is entered.
    fn own_title_asked(&mut self) {
        if !std::mem::replace(&mut self.achievements.own_asked, true) {
            self.send_on_map(&net::guild::ask_names(self.own_account()));
        }
    }

    /// A player comes into sight: one of no guild is asked for its names, once, for the title told with them
    /// (one of a guild is asked by the guild: `incoming/guild.rs`). The client asks when the pointer first
    /// comes over someone, with the same packet.
    pub(in crate::web::app) fn title_looked_up(&mut self, who: u32, guild: u32) {
        if guild == 0 && who != self.own_account() && self.guild.asked.insert(who) {
            self.send_on_map(&net::guild::ask_names(who));
        }
    }

    /// `CGameMode::ReceiveAchievementUpdate`: one achievement moved on. One that is completed now and was not
    /// is told of: the line across the screen for ten seconds, the effect on the character, the mark on the
    /// menu's icon until the window is opened.
    fn achievement_changed(&mut self, standing: Standing, record: Record) {
        let before = self.achievements.records.get(&record.id).is_some_and(|known| known.completed);
        if !before && record.completed {
            let title = data::achievements().iter().find(|info| info.id == record.id).map(|info| info.title.clone()).unwrap_or_default();
            let words = crate::numbers::filled(&Self::words(2681, "< %s > achieved."), &[&title]);
            self.achievements.balloon = Some((words, crate::web::now() + TOLD_FOR));
            world::burst(COMPLETED, None, None, 0.0);
            self.achievements.told = !self.achievements.open;
        }
        self.achievements.standing = standing;
        // (The server tells the standing alone with an achievement numbered 0, which is none.)
        if record.id != 0 {
            self.achievements.records.insert(record.id, record);
        }
        self.achievements_redrawn();
        self.redraw(&[&wnd::ACHIEVEMENT_TOLD]);
    }

    /// The line that told of an achievement completed has stood its time: looked at with the watch on the
    /// connection (`actions/link.rs`).
    pub(in crate::web::app) fn achievement_told(&mut self) {
        if self.achievements.balloon.as_ref().is_some_and(|(_, until)| crate::web::now() >= *until) {
            self.achievements.balloon = None;
            self.redraw(&[&wnd::ACHIEVEMENT_TOLD]);
        }
    }

    /// The windows that show the achievements are drawn anew.
    pub(in crate::web::app) fn achievements_redrawn(&self) {
        self.redraw(&[&wnd::ACHIEVEMENTS, &wnd::ACHIEVEMENTS_TRACED, &wnd::MENU, &wnd::EQUIPMENT]);
    }
}
