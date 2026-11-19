//! The menu on another player: opening it where the right button was pressed, and what its rows do (the
//! client's `CGameMode::OpenActorMenu` and the `MM_MENUSELECTED` arm of `CGameMode::SendMsg`).

use super::*;

impl App {
    /// The right button on someone who is not the player's pet: another player whose name is known has the
    /// menu; nobody else has one. (The client has none either while its item shop is open.)
    pub(in crate::web::app) fn player_menu_opens(&mut self, id: u32, at: (f32, f32)) {
        let Some(name) = world::player_name(id).filter(|_| id != self.own_account()) else { return };
        self.player_menu = Some(PlayerMenu { at: Self::on_stage(at), who: id, name, guild: world::player_guild(id).map_or(0, |(guild, _)| guild) });
        self.render();
    }

    /// Puts the menu away, if it is open.
    pub(in crate::web::app) fn close_player_menu(&mut self) {
        if self.player_menu.take().is_some() {
            self.render();
        }
    }

    /// A row of the menu was pressed (`crate::press::player_menu`).
    pub(super) fn player_menu_pressed(&mut self, press: crate::press::player_menu::Press) {
        use crate::press::player_menu::{Entry, Press};
        let Press::Do(entry) = press;
        let Some(menu) = self.player_menu.take() else { return };
        self.render();
        match entry {
            Entry::Deal => self.exchange_request(menu.who),
            Entry::Invite => self.party_invite(&menu.name),
            Entry::Guild => self.guild_invite_seen(menu.who),
            // The master's two rows (`MENUID_ALLY_GUILD`, `MENUID_HOSTILE_GUILD`: `clicks/guild/manage.rs`).
            Entry::Ally => self.guild_ally_asked(menu.who),
            Entry::Oppose => self.guild_opposed(menu.who),
            // A window opens for the talk with that player (`MENUID_MAKE_WHISPER_WINDOW`: `clicks/whispers.rs`).
            Entry::Chat => self.whisper_to(&menu.name),
            // The server is asked what the player wears (`MENUID_VIEW_OTHERUSER_EQUIPWND`: `clicks/viewequip.rs`).
            Entry::Equipment => self.viewequip_asked(menu.who),
            // The player is asked to be a friend (`MENUID_ADD_TO_FRIEND_CHAR_LIST`: `clicks/friends.rs`).
            Entry::Friend => self.friend_wanted(&menu.name),
            // The player's whispers are turned away, or let in again if they were (`MENUID_REQ_EX_LIST`,
            // `MENUID_REQ_IN`: what `/ex name` and `/in name` do).
            Entry::Block => self.whisper_from(&menu.name, self.commands.blocked.contains(&menu.name)),
            // A letter is written to the player (`MM_OPEN_WRITE_RODEX` with the name: `clicks/mail/writing.rs`).
            Entry::Mail => self.mail_write_to(&menu.name),
        }
    }
}
