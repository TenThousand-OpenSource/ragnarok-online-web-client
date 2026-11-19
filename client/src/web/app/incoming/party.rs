//! The party: what the map server tells of it. What the chat says of each thing, and in which colour, is the
//! client's (`CGameMode::ReceivePartyRoster`, `UpdatePartyMemberInfo`, `ReceivePartyMemberLeft`,
//! `ReceiveNamedActionResult`, `ReceiveServerRateNotice`, `ReceivePartyChat`, `ReceiveToggleNotice` and the
//! `ID_ZC_ACK_MAKE_GROUP`, `ID_ZC_PARTY_JOIN_REQ` arms of its packet handler).

use super::*;
use crate::net::party;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn party_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            party::ZC_ACK_MAKE_GROUP => self.party_made(packet.get(2).copied().unwrap_or(u8::MAX)),
            party::ZC_GROUP_LIST => {
                if let Some((name, members)) = party::list(packet) {
                    (self.party.name, self.party.members, self.party.picked) = (name, members, None);
                    self.party_changed();
                }
            }
            party::ZC_ADD_MEMBER_TO_GROUP => self.party_member(packet),
            party::ZC_DELETE_MEMBER_FROM_GROUP => self.party_member_gone(packet),
            // A party invites the player: the question is asked in a box of its own (`UIJoinPartyAcceptWnd`).
            party::ZC_PARTY_JOIN_REQ => {
                if let Some((number, name)) = party::invited(packet) {
                    self.party.asks = Some(PartyAsk::Invited(number, name));
                    self.redraw(&[&wnd::PARTY_ASKS]);
                }
            }
            party::ZC_PARTY_JOIN_REQ_ACK => self.party_invite_answered(packet),
            party::ZC_REQ_GROUPINFO_CHANGE_V2 => self.party_shares(packet),
            party::ZC_NOTIFY_CHAT_PARTY => self.party_said(packet),
            // Where a member on this map is: a mark on the minimap.
            party::ZC_NOTIFY_POSITION_TO_GROUPM => {
                if let Some((account, cell)) = party::place(packet) {
                    self.party.places.insert(account, cell);
                    self.party_marks();
                }
            }
            // A member's life: the bar in the party window, and the bar under the member in the world.
            party::ZC_NOTIFY_HP_TO_GROUPM => {
                if let Some((account, left, most)) = party::life(packet) {
                    self.party.life.insert(account, (left, most));
                    world::unit_life(account, left, most);
                    self.redraw(&[&wnd::PARTY]);
                }
            }
            party::ZC_GROUP_ISALIVE => {
                if let Some((account, dead)) = party::dead(packet) {
                    if dead {
                        self.party.dead.insert(account);
                    } else {
                        self.party.dead.remove(&account);
                    }
                    self.redraw(&[&wnd::PARTY]);
                }
            }
            party::ZC_NOTIFY_MEMBERINFO_TO_GROUPM => {
                if let Some((account, job, level)) = party::job_level(packet) {
                    if let Some(member) = self.party.members.iter_mut().find(|member| member.account == account && !member.offline) {
                        (member.job, member.level) = (job, level);
                    }
                    self.redraw(&[&wnd::PARTY]);
                }
            }
            // The party has another leader. (What the client says of it in the chat was not found in its
            // code: nothing is said here.)
            party::ZC_CHANGE_GROUP_MASTER => {
                if let Some((before, now)) = party::leader(packet) {
                    for member in &mut self.party.members {
                        if member.account == before {
                            member.leader = false;
                        }
                        if member.account == now && !member.offline {
                            member.leader = true;
                        }
                    }
                    self.party_changed();
                }
            }
            // Whether the player turns invitations down: the client says so each time the server does, which
            // is on entering the game too.
            party::ZC_PARTY_CONFIG => {
                let refused = packet.get(2).copied().unwrap_or(0) != 0;
                self.chat_line(if refused { Self::words(1325, "Block all party invitations.") } else { Self::words(1326, "Allow all party invitations.") }, PALE_YELLOW);
            }
            _ => return false,
        }
        true
    }

    /// The player's own account, as the party's packets name everyone.
    pub(in crate::web::app) fn own_account(&self) -> u32 {
        self.account.as_ref().map_or(0, |account| account.account_id)
    }

    /// Whether the player leads the party (the client's `partyLeader`).
    pub(in crate::web) fn leads_party(&self) -> bool {
        let own = self.own_account();
        self.party.members.iter().any(|member| member.account == own && member.leader && !member.offline)
    }

    /// The list of members changed: its window, the window beside it and the marks on the minimap follow.
    pub(in crate::web::app) fn party_changed(&mut self) {
        let members = &self.party.members;
        self.party.places.retain(|account, _| members.iter().any(|member| member.account == *account && !member.offline));
        // The setup of a party there is not, or the making of one there is already, has nothing to show.
        let has = !self.party.members.is_empty();
        if matches!((self.party.side, has), (Some(PartySide::Setup | PartySide::Invite), false) | (Some(PartySide::Create | PartySide::Named), true)) {
            self.party.side = None;
        }
        self.redraw(&[&wnd::PARTY, &wnd::PARTY_SIDE, &wnd::PARTY_MENU]);
        self.party_marks();
        // Whom a press fights on a PVP map goes by the party (`incoming/pvp.rs`).
        self.sides_told();
    }

    /// ZC_ACK_MAKE_GROUP: made, the party window opens (the client's `EnsureWindow(WID_MESSENGERGROUPWND)`).
    fn party_made(&mut self, result: u8) {
        let (words, colour) = match result {
            party::made::DONE => (Self::words(77, "Party has successfully been organized."), CYAN),
            party::made::SAME_NAME => (Self::words(78, "That Party Name already exists."), RED),
            party::made::ALREADY_IN_ONE => (Self::words(79, "The Character is already in a party."), RED),
            party::made::NOT_HERE => (Self::words(1387, "Unable to organize a party in this map."), RED),
            _ => return,
        };
        self.chat_line(words, colour);
        if result == party::made::DONE {
            self.party.open = true;
            self.render();
        }
    }

    /// ZC_ADD_MEMBER_TO_GROUP: a member is new, or is somewhere else now, or went offline or came back.
    fn party_member(&mut self, packet: &[u8]) {
        let Some((member, name, cell, (pickup, items))) = party::joined(packet) else { return };
        let own = self.own_account();
        self.party.name = name;
        (self.party.sharing.pickup, self.party.sharing.items) = (pickup, items);
        let here = self.game.as_ref().is_some_and(|game| party::map_name(&game.map) == party::map_name(&member.map));
        if member.account == own {
            // The player is on another map now: the marks of the last one mean nothing there.
            self.party.places.clear();
        } else if member.offline || !here {
            self.party.places.remove(&member.account);
            self.party.life.remove(&member.account);
        } else {
            self.party.places.insert(member.account, cell);
        }
        // One account has one place in a party: the character that was there gives way to this one.
        match self.party.members.iter_mut().find(|known| known.account == member.account) {
            Some(known) => *known = member,
            None => self.party.members.push(member),
        }
        self.party_changed();
    }

    /// ZC_DELETE_MEMBER_FROM_GROUP: someone left or was expelled; when it is the player, there is no party any
    /// more.
    fn party_member_gone(&mut self, packet: &[u8]) {
        let Some((account, name, result)) = party::left(packet) else { return };
        match result {
            party::gone::LEFT | party::gone::EXPELLED => {
                self.party.members.retain(|member| member.name != name);
                self.party.places.remove(&account);
                self.party.life.remove(&account);
            }
            party::gone::CANNOT_LEAVE => return self.chat_line(Self::words(1872, "Cannot leave a party in this map."), RED),
            party::gone::CANNOT_EXPEL => return self.chat_line(Self::words(1873, "Cannot withdraw/break the party in this map."), RED),
            _ => {}
        }
        if self.game.as_ref().is_some_and(|game| game.name == name) {
            let (open, small) = (self.party.open, self.party.small);
            self.party = Party { open, small, ..Party::default() };
            self.chat_line(Self::words(84, "You left the party."), CYAN);
            self.redraw(&[&wnd::PARTY_ASKS]);
        }
        self.party.picked = None;
        self.party_changed();
    }

    /// ZC_PARTY_JOIN_REQ_ACK: what became of an invitation the player sent.
    fn party_invite_answered(&mut self, packet: &[u8]) {
        use party::invite::*;
        let Some((name, result)) = party::invite_answered(packet) else { return };
        let (words, colour) = match result {
            IN_OTHER_PARTY => (Self::words(80, "The Character already joined another party."), RED),
            REJECTED => (format!("{name}{}", Self::words(2060, " rejected your party invitation.")), RED),
            ACCEPTED => (format!("{name}{}", Self::words(2061, " accepted your party invitation.")), CYAN),
            FULL => (Self::words(83, "Party Capacity exceeded."), RED),
            SAME_ACCOUNT => (Self::words(608, "Character in the same account already joined."), RED),
            REFUSES_ALL => (Self::words(1324, "The character blocked the party invitation."), RED),
            OFFLINE => (Self::words(71, "The Character is not currently online or does not exist."), RED),
            THEIR_MAP_FORBIDS => (Self::words(1388, "(%s) are currently in a map which prevents joining a party.").replace("%s", &name), RED),
            THIS_MAP_FORBIDS => (Self::words(1871, "Cannot join a party in this map."), RED),
            MEMORIAL_DUNGEON => (Self::words(3027, "You can't invite or withdraw while in Memorial dungeon."), RED),
            LEVEL => (Self::words(3226, "Character with this level cannot join the party"), RED),
            _ => return,
        };
        self.chat_line(words, colour);
    }

    /// ZC_REQ_GROUPINFO_CHANGE_V2: how the party shares, told in three lines of chat as the client does.
    fn party_shares(&mut self, packet: &[u8]) {
        let Some(sharing) = party::sharing(packet) else { return };
        self.party.sharing = sharing;
        let (setup, each, even) = (Self::words(291, "Party Setup"), Self::words(287, "Each Take "), Self::words(288, "Even Share"));
        let experience = Self::words(292, "How to share EXP");
        let lines = [
            match sharing.experience {
                0 => Some(format!("{setup} - {experience} : {each}")),
                1 => Some(format!("{setup} - {experience} : {even}")),
                2 => Some(format!("{setup} - {experience} : {each}({})", Self::words(1535, "Can't be changed because of level gap"))),
                _ => None,
            },
            Some(format!("{setup} - {} : {}", Self::words(293, "How to share Items"), if sharing.pickup { Self::words(290, "Party Share") } else { each.clone() })),
            Some(format!("{setup} - {} : {}", Self::words(738, "Item sharing type"), if sharing.items { even.clone() } else { each.clone() })),
        ];
        for line in lines.into_iter().flatten() {
            self.chat_line(line, YELLOW);
        }
        if self.party.side == Some(PartySide::Setup) {
            self.party.draft = sharing;
        }
        self.redraw(&[&wnd::PARTY_SIDE]);
    }

    /// ZC_NOTIFY_CHAT_PARTY: a line of party chat, in the party's colours (the player's own orange, the others'
    /// pink), and over the head of whoever said it when they are in sight.
    fn party_said(&mut self, packet: &[u8]) {
        let Some((account, words)) = party::said(packet).filter(|(_, words)| !words.is_empty()) else { return };
        let own = self.own_account();
        world::said(account, own, &words);
        self.chat_line_of(said::PARTY, words, if account == own { "#ffc800" } else { "#ffc8c8" });
    }

    /// Writes the members' marks on the minimap anew.
    pub(in crate::web::app) fn party_marks(&self) {
        if let Some(marks) = element("party-marks") {
            marks.set_inner_html(&self.minimap_party());
        }
    }
}
