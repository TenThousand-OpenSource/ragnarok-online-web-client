//! The guild: what the map server tells of it. What the chat says of each thing, and in which colour, is the
//! client's (the `ID_ZC_RESULT_MAKE_GUILD`, `ID_ZC_ACK_REQ_JOIN_GUILD`, `ID_ZC_REQ_JOIN_GUILD`,
//! `ID_ZC_UPDATE_GDID_02f7` and `ID_ZC_ACK_DISORGANIZE_GUILD_RESULT` arms of its packet handler,
//! `CGameMode::ReceiveGuildLeave2`, `ReceiveGuildExpulsion2`, `ReceiveGuildNotice`, `ReceiveGuildChat`).

use super::*;
use crate::net::guild;

mod castles;
mod members;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn guild_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        let byte = guild::result(packet).unwrap_or(u8::MAX);
        match id {
            guild::ZC_RESULT_MAKE_GUILD => self.guild_made(byte),
            guild::ZC_UPDATE_GDID => self.guild_belonging(packet),
            guild::ZC_ACK_GUILD_MENUINTERFACE => self.guild.tabs = guild::tabs(packet),
            guild::ZC_GUILD_INFO => self.guild_told(packet),
            guild::ZC_GUILD_SKILLINFO => {
                if let Some((points, skills)) = guild::skills(packet) {
                    (self.guild.skill_points, self.guild.skills) = (points, skills);
                }
            }
            guild::ZC_GUILD_NOTICE => self.guild_notice(packet),
            // A guild invites the player: the question is asked in a box of its own (`UIJoinGuildAcceptWnd`).
            guild::ZC_REQ_JOIN_GUILD => self.guild.asks = guild::invited(packet).map(|(number, name)| GuildAsk::Invited(number, name)),
            // Another guild's master asks for an alliance (`UIAllyGuildAcceptWnd`).
            guild::ZC_REQ_ALLY_GUILD => self.guild.asks = guild::ally_asked(packet).map(|(account, name)| GuildAsk::Ally(account, data::starred(&name))),
            guild::ZC_ACK_REQ_ALLY_GUILD => self.guild_ally_answered(byte),
            guild::ZC_ACK_REQ_HOSTILE_GUILD => self.guild_oppose_answered(byte),
            guild::ZC_ACK_REQ_JOIN_GUILD => self.guild_invite_answered(byte),
            guild::ZC_ACK_LEAVE_GUILD => self.guild_member_gone(guild::left(packet), false),
            guild::ZC_ACK_BAN_GUILD => self.guild_member_gone(guild::expelled(packet), true),
            guild::ZC_BAN_LIST => self.guild.expelled = guild::expelled_list(packet),
            guild::ZC_ACK_DISORGANIZE_GUILD_RESULT => self.guild_broken(guild::broken_how(packet).unwrap_or(u32::MAX)),
            guild::ZC_GUILD_EMBLEM_IMG => self.guild_emblem(packet),
            _ if self.guild_lists_packet(id, packet) => {}
            _ if self.guild_castles_packet(id, packet) => {}
            // What is told of those in sight changes nothing in the guild's windows.
            _ => return self.guild_sight_packet(id, packet),
        }
        self.redraw(&[&wnd::GUILD, &wnd::GUILD_ASKS, &wnd::GUILD_MENU]);
        true
    }

    /// The packets that tell of the guilds of those in sight, and the guild's chat. `false` for any other.
    fn guild_sight_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // A line of guild chat, in the client's 0xB4FFB4, unless the player has such lines off
            // (`/notalkmsg2`).
            guild::ZC_GUILD_CHAT => {
                if let Some(words) = guild::said(packet).filter(|words| !words.is_empty() && !self.option("TT_NOTALKMSG2_ON_OFF")) {
                    // (With stars for a word that is not let be written: `ReceiveGuildChat`, `crate::manner`.)
                    self.chat_line_of(said::GUILD, data::starred(&words), "#b4ffb4");
                }
            }
            guild::ZC_CHANGE_GUILD => {
                if let Some((who, number, emblem)) = guild::changed(packet) {
                    // Of the player's own character the world is told apart: it is not among those in sight.
                    if who == self.own_account() {
                        world::own_guild(who, &self.own_name(), number, emblem);
                        self.guild_emblem_wanted(number, emblem);
                    }
                    world::guild_changed(who, number, emblem);
                    self.guild.asked.remove(&who);
                    // A player's names are asked for again; a guild flag (told so when its castle changes
                    // hands: rAthena's `flagemblem`) has only an emblem to show.
                    if world::player_guild(who).is_some() {
                        self.guild_looked_up(who, number, emblem);
                    } else {
                        self.guild_emblem_wanted(number, emblem);
                    }
                }
            }
            guild::ZC_GUILD_POSITION => {
                if let Some((who, position)) = guild::position_of(packet) {
                    world::player_position(who, position);
                }
            }
            guild::ZC_ACK_REQNAMEALL => {
                if let Some((who, names)) = guild::names(packet) {
                    world::player_card(who, names.party, names.guild, names.position);
                }
            }
            _ => return false,
        }
        true
    }

    /// ZC_ACK_REQ_ALLY_GUILD: what the chat says of an alliance asked for, in the client's colours (yellow, and
    /// its 0xFFFF00 for one made).
    fn guild_ally_answered(&mut self, answer: u8) {
        let (words, colour) = match answer {
            guild::ally::ALREADY => (Self::words(394, "Your Guild is already in an Alliance with this Guild."), YELLOW),
            guild::ally::REJECTED => (Self::words(395, "You rejected the Alliance Request."), YELLOW),
            guild::ally::ACCEPTED => (Self::words(396, "You accepted the Alliance Request."), CYAN),
            guild::ally::THEIRS_FULL => (Self::words(397, "The other guild has too many Alliances."), YELLOW),
            guild::ally::OURS_FULL => (Self::words(398, "You have too many Alliances."), YELLOW),
            guild::ally::DISABLED => (Self::words(1717, "Alliances are disabled."), YELLOW),
            _ => return,
        };
        self.chat_line_of(said::GUILD, words, colour);
    }

    /// ZC_ACK_REQ_HOSTILE_GUILD: what the chat says of a guild set as opposed (the client's 0xFFFF00 when it
    /// is, red when it is not).
    fn guild_oppose_answered(&mut self, result: u8) {
        let (words, colour) = match result {
            guild::oppose::DONE => (Self::words(495, "Antagonist has been set."), CYAN),
            guild::oppose::FULL => (Self::words(496, "Guild has too many Antagonists."), RED),
            guild::oppose::ALREADY => (Self::words(497, "Already set as an Antagonist"), RED),
            guild::oppose::DISABLED => (Self::words(1718, "Antagonists are disabled."), RED),
            _ => return,
        };
        self.chat_line_of(said::GUILD, words, colour);
    }

    /// What the player's own character is called.
    fn own_name(&self) -> String {
        self.game.as_ref().map(|game| game.name.clone()).unwrap_or_default()
    }

    /// The player's own character, as the guild's packets name everyone.
    pub(in crate::web) fn own_character(&self) -> u32 {
        self.zone.as_ref().map_or(0, |zone| zone.char_id)
    }

    /// ZC_RESULT_MAKE_GUILD: what the chat says of it. Without an Emperium the server refuses, and the client
    /// says its line 405.
    fn guild_made(&mut self, result: u8) {
        let (words, colour) = match result {
            guild::made::DONE => (Self::words(374, "Guild has been created."), CYAN),
            guild::made::ALREADY_IN_ONE => (Self::words(375, "You are already in a Guild."), YELLOW),
            guild::made::SAME_NAME => (Self::words(376, "That Guild Name already exists."), YELLOW),
            guild::made::NO_ITEM => (Self::words(405, "You need an Emperium to create a Guild."), YELLOW),
            guild::made::NOT_HERE => (Self::words(2687, "Can't create a Guild in this area."), YELLOW),
            _ => return,
        };
        self.chat_line_of(said::GUILD, words, colour);
    }

    /// ZC_GUILD_INFO: the guild's figures. One who has just joined learns of the guild by this alone: rAthena
    /// sends ZC_UPDATE_GDID at that moment only once it knows the newcomer's position, which it does not yet
    /// (`guild_member_added` before `guild_recv_info`), so the guild is taken from here, and its members and
    /// skills asked for as on ZC_UPDATE_GDID; and its positions, from which what the newcomer may do is worked
    /// out (`guild_rights_known`).
    fn guild_told(&mut self, packet: &[u8]) {
        let Some(info) = guild::info(packet) else { return };
        if self.guild.id != info.guild && info.guild != 0 {
            self.guild.renewed();
            (self.guild.id, self.guild.name, self.guild.emblem) = (info.guild, info.name.clone(), info.emblem);
            for what in [guild::menu::SKILLS, guild::menu::MEMBERS, guild::menu::POSITIONS] {
                self.send_on_map(&guild::ask(what));
            }
            self.guild_own_shown();
        }
        self.guild.master_character = info.master_character;
        self.guild.info = Some(info);
        self.guild_rights_known();
    }

    /// The world is told which guild the player's own character is of, and the server asked what is written
    /// under it (the names of the party and the guild, the position); the emblem is asked for when it is not
    /// read yet; who the player's friends are is told anew.
    fn guild_own_shown(&mut self) {
        let (own, number, emblem) = (self.own_account(), self.guild.id, self.guild.emblem);
        world::own_guild(own, &self.own_name(), number, emblem);
        if number != 0 {
            self.send_on_map(&guild::ask_names(own));
        }
        self.guild_emblem_wanted(number, emblem);
        self.sides_told();
    }

    /// ZC_UPDATE_GDID: the guild the player is of. The client asks for its skills and its members at once,
    /// the first time; the emblem is asked for when it is not known yet.
    fn guild_belonging(&mut self, packet: &[u8]) {
        let Some(belonging) = guild::belonging(packet) else { return };
        let new = self.guild.id != belonging.guild;
        if new {
            self.guild.renewed();
        }
        let was = self.guild.master;
        let guild = &mut self.guild;
        (guild.id, guild.emblem, guild.name, guild.master_character) = (belonging.guild, belonging.emblem, belonging.name, belonging.master_character);
        (guild.invites, guild.expels, guild.master) = (belonging.invites, belonging.expels, belonging.master);
        if new && belonging.guild != 0 {
            self.send_on_map(&guild::ask(guild::menu::SKILLS));
            self.send_on_map(&guild::ask(guild::menu::MEMBERS));
        }
        // Which tabs open goes by whether the player is the master: asked anew when that changed.
        if !new && was != self.guild.master && self.guild.tabs.take().is_some() {
            self.send_on_map(&guild::ask_tabs());
        }
        self.guild_own_shown();
    }

    /// ZC_GUILD_NOTICE: kept for the window, and told in the chat in the client's 0x80FFFF, each line in
    /// square brackets, when the player has such news on (`/loginout`).
    fn guild_notice(&mut self, packet: &[u8]) {
        let Some((subject, notice)) = guild::notice(packet) else { return };
        // (Both with stars for a word that is not let be written: `ReceiveGuildNotice`, `crate::manner`.)
        let (subject, notice) = (data::starred(&subject), data::starred(&notice));
        if self.option("TT_LOGINOUT_ON_OFF") {
            for line in [&subject, &notice].into_iter().filter(|line| !line.is_empty()) {
                self.chat_line(format!("[ {line} ]"), "#ffff80");
            }
        }
        (self.guild.subject, self.guild.notice) = (subject, notice);
    }

    /// ZC_ACK_REQ_JOIN_GUILD: what became of an invitation the player sent.
    fn guild_invite_answered(&mut self, result: u8) {
        let (words, colour) = match result {
            guild::invite::IN_OTHER_GUILD => (Self::words(378, "He/She is already in a Guild."), RED),
            guild::invite::REJECTED => (Self::words(379, "You have refused the guild invitation."), RED),
            guild::invite::ACCEPTED => (Self::words(380, "You have accepted the guild invitation."), CYAN),
            guild::invite::FULL => (Self::words(381, "Your Guild is full."), RED),
            _ => return,
        };
        self.chat_line_of(said::GUILD, words, colour);
    }

    /// ZC_ACK_LEAVE_GUILD and ZC_ACK_BAN_GUILD: a member left or was expelled, with the reason given. When it
    /// is the player, there is no guild any more and its window closes; anyone else is told of in two lines.
    fn guild_member_gone(&mut self, gone: Option<(u32, String)>, expelled: bool) {
        let Some((character, reason)) = gone else { return };
        // (The reason with stars for a word that is not let be written: `ReceiveGuildMemberLeft`,
        // `ReceiveGuildMemberExpelled`.)
        let reason = data::starred(&reason);
        if character == self.own_character() {
            self.guild.left();
            self.chat_line_of(
                said::GUILD,
                if expelled { Self::words(490, "You have been expelled from the Guild.") } else { Self::words(489, "You left the guild.") },
                RED
            );
            self.guild_own_shown();
            return;
        }
        let place = self.guild.members.iter().position(|member| member.character == character);
        let name = place.map(|place| self.guild.members.remove(place).name).filter(|name| !name.is_empty()).unwrap_or_else(|| Self::words(581, "Nameless"));
        (self.guild.picked, self.guild.menu, self.guild.combo) = (None, None, None);
        self.guild.member_moves.remove(&character);
        let lines = if expelled {
            self.guild.expelled.push((name.clone(), reason.clone()));
            [
                Self::words(370, "%s has been expelled from the guild.").replacen("%s", &name, 1),
                Self::words(371, "Expulsion Reason: %s").replacen("%s", &reason, 1)
            ]
        } else {
            [Self::words(364, "%s has left the guild.").replacen("%s", &name, 1), Self::words(365, "Withdrawal Reason: %s").replacen("%s", &reason, 1)]
        };
        for line in lines {
            self.chat_line_of(said::GUILD, line, YELLOW);
        }
    }

    /// ZC_ACK_DISORGANIZE_GUILD_RESULT: broken up, the guild is no more and its window closes.
    fn guild_broken(&mut self, result: u32) {
        let words = match result {
            guild::broken::DONE => Self::words(400, "Guild was successfully disbanded."),
            guild::broken::WRONG_KEY => Self::words(401, "You have failed to disband the guild due to your incorrect SSN."),
            guild::broken::MEMBERS_LEFT => Self::words(402, "You have failed to disband the guild because there are guildsmen still present."),
            _ => return,
        };
        if result == guild::broken::DONE {
            // The server tells nothing of the flags of a castle the guild held (rAthena's `OnRecvCastle`
            // refreshes them only for a castle that has an owner), so they are bared here (`guild_gone`).
            world::guild_gone(self.guild.id);
            self.guild.left();
            self.guild_own_shown();
        }
        self.chat_line_of(said::GUILD, words, YELLOW);
    }

    /// ZC_GUILD_EMBLEM_IMG: the emblem is unpacked and made a picture for the page; the windows and the names
    /// under players show it from then.
    fn guild_emblem(&mut self, packet: &[u8]) {
        let Some((number, version, packed)) = guild::emblem(packet) else { return };
        let Some(pixels) = guild::emblem_picture(packed) else { return };
        let Some(picture) = page_picture(&pixels) else { return };
        world::emblem_known(number, version, picture.clone());
        world::emblem_pixels(number, version, pixels);
        self.guild.emblems.insert((number, version), picture);
        // The player's own guild has this emblem now (the client's `SetGuildInfo` on the player's character).
        if number == self.guild.id {
            self.guild.emblem = version;
            world::own_guild(self.own_account(), &self.own_name(), number, version);
        }
    }

    /// Asks for a guild's emblem when that version of it is not read yet (the client's `CGuildMgr`, which
    /// keeps them in files). A guild with no emblem has version 0.
    pub(in crate::web::app) fn guild_emblem_wanted(&mut self, number: u32, version: u32) {
        if number != 0 && version != 0 && !self.guild.emblems.contains_key(&(number, version)) {
            // Marked as asked for with an empty picture, which draws nothing.
            self.guild.emblems.insert((number, version), String::new());
            self.send_on_map(&guild::ask_emblem(number));
        }
    }

    /// A player of a guild is in sight: what the party and the guild are called and the emblem are asked for,
    /// once each. (The client asks for the names when the pointer first comes over someone, with the same
    /// packet; here it is asked as they come into sight, so the name is whole at once.)
    pub(in crate::web::app) fn guild_looked_up(&mut self, who: u32, number: u32, emblem: u32) {
        if number == 0 || who == self.own_account() {
            return;
        }
        if self.guild.asked.insert(who) {
            self.send_on_map(&guild::ask_names(who));
        }
        self.guild_emblem_wanted(number, emblem);
    }
}

/// A picture as the page takes one: a data URL of it.
fn page_picture(picture: &crate::bmp::Image) -> Option<String> {
    let canvas: web_sys::HtmlCanvasElement = crate::web::document()?.create_element("canvas").ok()?.dyn_into().ok()?;
    canvas.set_width(picture.width);
    canvas.set_height(picture.height);
    let context: web_sys::CanvasRenderingContext2d = canvas.get_context("2d").ok()??.unchecked_into();
    let data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(wasm_bindgen::Clamped(&picture.rgba), picture.width, picture.height).ok()?;
    context.put_image_data(&data, 0.0, 0.0).ok()?;
    canvas.to_data_url().ok()
}
