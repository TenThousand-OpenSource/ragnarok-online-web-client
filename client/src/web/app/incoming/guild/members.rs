//! The guild's lists as the server tells them: its members, its positions and what each may do.

use super::*;

impl App {
    /// The packets that tell of the guild's lists: the members, the positions, the guilds allied and opposed.
    /// `false` for any other.
    pub(super) fn guild_lists_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            guild::ZC_MYGUILD_BASIC_INFO => (self.guild.related, self.guild.menu) = (guild::related(packet), None),
            guild::ZC_DELETE_RELATED_GUILD => {
                if let Some((number, opposed)) = guild::unrelated(packet) {
                    self.guild.related.retain(|(guild, _, against)| (*guild, *against) != (number, opposed));
                    self.guild.menu = None;
                }
            }
            guild::ZC_ADD_RELATED_GUILD => self.guild.related.extend(guild::related_added(packet)),
            guild::ZC_MEMBERMGR_INFO => {
                (self.guild.members, self.guild.picked, self.guild.menu, self.guild.combo) = (guild::members(packet), None, None, None);
                self.guild.member_moves.clear();
            }
            // A member came into the game or left it. One the list does not have has just joined: the server
            // tells of nobody new by itself, so the members are asked for again.
            guild::ZC_UPDATE_CHARSTAT => {
                if let Some((character, online)) = guild::member_online(packet) {
                    match self.guild.members.iter_mut().find(|member| member.character == character) {
                        Some(member) => member.online = online,
                        None => self.send_on_map(&guild::ask(guild::menu::MEMBERS)),
                    }
                }
            }
            guild::ZC_POSITION_ID_NAME_INFO => {
                for (id, name) in guild::position_names(packet) {
                    self.guild_position(id).name = name;
                }
            }
            guild::ZC_POSITION_INFO => {
                for told in guild::position_rights(packet) {
                    self.guild.rights.insert(told.id);
                    let position = self.guild_position(told.id);
                    (position.invites, position.expels, position.storage, position.tax) = (told.invites, told.expels, told.storage, told.tax);
                }
            }
            guild::ZC_ACK_CHANGE_GUILD_POSITIONINFO => {
                for changed in guild::positions_changed(packet) {
                    let id = changed.id;
                    self.guild.rights.insert(id);
                    self.guild.position_edits.remove(&id);
                    *self.guild_position(id) = changed;
                }
            }
            guild::ZC_ACK_REQ_CHANGE_MEMBERS => {
                for (character, position) in guild::members_moved(packet) {
                    if let Some(member) = self.guild.members.iter_mut().find(|member| member.character == character) {
                        member.position = position;
                    }
                    // The first position is the master's (the client's `SetMasterCharacterId`).
                    if position == 0 {
                        self.guild.master_character = character;
                    }
                    self.guild.member_moves.remove(&character);
                }
            }
            _ => return false,
        }
        // What the player may do, who the player's friends are and whose heads the list shows follow from these.
        self.guild_rights_known();
        self.sides_told();
        self.guild_heads_wanted();
        true
    }

    /// What the player may do in the guild, from what is known of it. The server says so itself with
    /// ZC_UPDATE_GDID, which is all the client goes by; but rAthena does not send that to one who has just
    /// joined, nor when the master changes what a position may do. So, decided here: the master is whoever the
    /// guild's figures name; and once the player's own position is known (ZC_MEMBERMGR_INFO) and what that
    /// position may do (ZC_POSITION_INFO, ZC_ACK_CHANGE_GUILD_POSITIONINFO), inviting and expelling follow
    /// from them. One who becomes the master, or is it no more, asks anew which tabs open.
    pub(super) fn guild_rights_known(&mut self) {
        let own = self.own_character();
        let guild = &mut self.guild;
        let was = guild.master;
        if guild.master_character != 0 {
            guild.master = guild.master_character == own;
        }
        let position = guild.members.iter().find(|member| member.character == own).map(|member| member.position);
        if let Some(position) = position.and_then(|id| guild.positions.iter().find(|position| position.id == id && guild.rights.contains(&id))) {
            (guild.invites, guild.expels) = (position.invites, position.expels);
        }
        if was != guild.master && guild.tabs.take().is_some() {
            self.send_on_map(&guild::ask_tabs());
        }
    }

    /// The position of that number, made if it is not known yet.
    pub(super) fn guild_position(&mut self, id: u32) -> &mut guild::Position {
        let place = match self.guild.positions.iter().position(|position| position.id == id) {
            Some(place) => place,
            None => {
                self.guild.positions.push(guild::Position { id, ..Default::default() });
                self.guild.positions.sort_by_key(|position| position.id);
                self.guild.positions.iter().position(|position| position.id == id).unwrap_or(0)
            }
        };
        &mut self.guild.positions[place]
    }
}
