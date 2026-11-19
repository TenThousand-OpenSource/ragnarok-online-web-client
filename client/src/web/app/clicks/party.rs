//! The party: what its windows' buttons do, the commands typed for it, and the menu on a member.

use super::*;
use crate::net::party;

impl App {
    /// A press of this system (`crate::press::party`). What each button of the party window does is the
    /// client's `UIMessengerGroupWnd::SendMsg`; the setup window's, `UIPartySettingWnd::SendMsg`; the
    /// invitation window's, `UIPartyInvitationToWnd::SendMsg`.
    pub(super) fn party_pressed(&mut self, press: crate::press::party::Press) {
        use crate::press::party::Press;
        match press {
            Press::Window => {
                self.party.open = !self.party.open;
                (self.party.side, self.party.menu) = (None, None);
                self.friends_put_away();
                self.render();
            }
            Press::Size => {
                self.party.small = !self.party.small;
                self.render();
            }
            Press::Row(row) => {
                self.party.picked = Some(row);
                self.close_party_menu();
                self.redraw(&[&wnd::PARTY]);
            }
            // The Adventurer's Agency, the lock that keeps the window's buttons from being pressed and the
            // small windows of single members are not made here yet: these do nothing.
            Press::Agency | Press::Lock | Press::Sort => {}
            // The Friend tab: the window lists the friends (`clicks/friends.rs`).
            Press::Friends => self.messenger_turned(true),
            // The setup shows for a party there is (`ID_PARTY_SETTING`).
            Press::Setup => self.party_side(PartySide::Setup, !self.party.members.is_empty()),
            // Making one, for a player who has none (`ID_OPEN_PARTY_SETTINGS`).
            Press::Create => {
                self.party.making.clear();
                self.party_side(PartySide::Create, self.party.members.is_empty());
            }
            Press::Leave => self.party_asks(PartyAsk::Leave),
            // Only the leader invites; anyone else is told so in a message box (`ID_OPEN_PARTY_INVITATION`).
            Press::Invite => {
                if self.leads_party() || self.party.side == Some(PartySide::Invite) {
                    self.party_side(PartySide::Invite, true);
                } else {
                    self.message = Some(Self::words(2070, "Only the leader can invite."));
                    self.render();
                }
            }
            Press::Experience(_) | Press::Pickup(_) => {
                // Anyone sets up a party being made; one there is, its leader alone.
                if self.party.members.is_empty() || self.leads_party() {
                    match press {
                        Press::Experience(how) => self.party.draft.experience = how.min(1),
                        _ => self.party.draft.pickup = press == Press::Pickup(1),
                    }
                    // The name being typed is kept through the window's being drawn anew.
                    if self.party.side == Some(PartySide::Create) {
                        self.party.making = field("party-name");
                    }
                    self.redraw(&[&wnd::PARTY_SIDE]);
                } else {
                    self.chat_line(Self::words(294, "Only the Party Leader can change this setting."), PALE_CYAN);
                }
            }
            Press::SetupOk => self.party_setup_ok(),
            Press::SetupCancel | Press::InviteCancel => {
                self.party.side = None;
                self.redraw(&[&wnd::PARTY_SIDE]);
            }
            // The window stays for the next name, as the client's does.
            Press::InviteOk => {
                let name = field("party-invitee").trim().to_owned();
                if name.is_empty() {
                    self.chat_line(Self::words(2072, "You must enter a character name."), RED);
                } else {
                    self.party_invite(&name);
                }
            }
            Press::Yes | Press::No => self.party_answered(press == Press::Yes),
            Press::Do(entry) => self.party_menu_row(entry),
        }
    }

    /// Opens the window beside the party window, or closes it when it is the one open; `may` is whether it
    /// has anything to show now. What is set in it starts from how the party shares (`UIPartySettingWnd::
    /// OnCreate`: experience that could not be changed counts as each its own).
    fn party_side(&mut self, side: PartySide, may: bool) {
        self.party.side = (self.party.side != Some(side) && may).then_some(side);
        let sharing = self.party.sharing;
        self.party.draft = party::Sharing { experience: if sharing.experience == 2 { 0 } else { sharing.experience }, pickup: sharing.pickup, items: false };
        self.render();
    }

    fn party_asks(&mut self, ask: PartyAsk) {
        self.party.asks = Some(ask);
        self.redraw(&[&wnd::PARTY_ASKS]);
    }

    /// OK of the setup window: a party is made with the name typed or given, or the leader's changes are sent.
    fn party_setup_ok(&mut self) {
        let draft = self.party.draft;
        match self.party.side {
            Some(PartySide::Create) => {
                let name = field("party-name").trim().to_owned();
                if name.contains('#') {
                    return self.chat_line(Self::words(2839, "Special symbol can't be used in name"), RED);
                }
                if name.is_empty() {
                    return self.chat_line(Self::words(2073, "You must enter the name of the party."), RED);
                }
                self.send_on_map(&party::make(&name, draft.pickup, draft.items));
            }
            Some(PartySide::Named) => {
                let name = std::mem::take(&mut self.party.making);
                self.send_on_map(&party::make(&name, draft.pickup, draft.items));
            }
            Some(PartySide::Setup) if self.leads_party() && draft != self.party.sharing => {
                self.send_on_map(&party::share(draft));
            }
            _ => {}
        }
        self.party.side = None;
        self.redraw(&[&wnd::PARTY_SIDE]);
    }

    /// The answer to what the party asked.
    fn party_answered(&mut self, yes: bool) {
        let Some(ask) = self.party.asks.take() else { return };
        match ask {
            // Joining opens the party window (the client's `MM_JOIN_GROUP`).
            PartyAsk::Invited(number, _) => {
                self.send_on_map(&party::answer(number, yes));
                if yes {
                    self.party.open = true;
                    self.render();
                }
            }
            PartyAsk::Leave if yes => self.party_leave(),
            PartyAsk::Disband if yes => {
                self.send_on_map(&party::leave());
            }
            PartyAsk::Expel(name) if yes => self.party_expel(&name),
            PartyAsk::HandOver(account) if yes => {
                self.send_on_map(&party::hand_over(account));
            }
            _ => {}
        }
        self.redraw(&[&wnd::PARTY_ASKS]);
    }

    /// The player leaves the party (the client's `MM_REQ_LEAVE_GROUP`). A leader with others in the party
    /// first hands it to a member who is online on the same map; with members online only elsewhere it is
    /// told to hand the party over first; with nobody online it is asked whether the party is to be no more.
    fn party_leave(&mut self) {
        let own = self.own_account();
        if self.leads_party() && self.party.members.len() > 1 {
            let map = self.game.as_ref().map(|game| crate::party::map_name(&game.map).to_owned()).unwrap_or_default();
            let mut online = self.party.members.iter().filter(|member| member.account != own && !member.offline);
            match online.clone().find(|member| crate::party::map_name(&member.map) == map) {
                Some(heir) => {
                    let heir = heir.account;
                    self.send_on_map(&party::hand_over(heir));
                }
                None if online.next().is_some() => return self.chat_line(Self::words(3273, "You can leave a party after delegation."), RED),
                None => return self.party_asks(PartyAsk::Disband),
            }
        }
        if self.party.side == Some(PartySide::Invite) {
            self.party.side = None;
            self.redraw(&[&wnd::PARTY_SIDE]);
        }
        self.send_on_map(&party::leave());
    }

    /// The leader expels the member of that name (`MM_REQ_EXPEL_GROUP_MEMBER`).
    fn party_expel(&mut self, name: &str) {
        if let (true, Some(member)) = (self.leads_party(), self.party.members.iter().find(|member| member.name == name)) {
            self.send_on_map(&party::expel(member.account, name));
        }
    }

    /// That player is invited into the party (`MM_REQ_JOIN_GROUP`): a name too long is refused in a message
    /// box, and a leader is told in the chat that the invitation went.
    pub(in crate::web::app) fn party_invite(&mut self, name: &str) {
        if name.len() >= 24 {
            self.message = Some(Self::words(531, "Name is too long. Please enter a name up to 23 english characters."));
            return self.render();
        }
        self.send_on_map(&party::invite(name));
        if self.leads_party() {
            self.chat_line(format!("{name}{}", Self::words(2059, " has received an invitation to join your party.")), CYAN);
        }
    }

    /// A row of the menu on a member was pressed (`UM_MENU_SELECTED`).
    fn party_menu_row(&mut self, entry: crate::press::party::Entry) {
        use crate::press::party::Entry;
        let Some((_, row)) = self.party.menu.take() else { return };
        self.render();
        let Some(member) = self.party.members.get(row).cloned() else { return };
        match entry {
            // Letters and the small windows of single members are not made here yet.
            Entry::Mail | Entry::AddSmall | Entry::RemoveSmall => {}
            Entry::Leave => self.party_asks(PartyAsk::Leave),
            Entry::Chat => self.whisper_to(&member.name),
            Entry::Expel => self.party_asks(PartyAsk::Expel(member.name)),
            Entry::HandOver if member.offline => self.chat_line_of(said::PARTY, Self::words(1534, "Can't re-assign party leader"), RED),
            Entry::HandOver => self.party_asks(PartyAsk::HandOver(member.account)),
        }
    }

    /// Puts the menu on a member away, if it is open.
    pub(in crate::web::app) fn close_party_menu(&mut self) {
        if self.party.menu.take().is_some() {
            self.render();
        }
    }

    /// The right button on a member in the party window: the member is picked and its menu opens there.
    pub(in crate::web::app) fn party_menu_opens(&mut self, row: usize, at: (f32, f32)) {
        if row < self.party.members.len() {
            (self.party.picked, self.party.menu) = (Some(row), Some((Self::on_stage(at), row)));
            self.render();
        }
    }

    /// What was typed into the chat, when it is the party's: a line for the party (`%` before it, the client's
    /// `TT_NORMAL`), or one of its commands (`/organize name`, `/invite name`, `/leave`, `/expel name`,
    /// `/accept` and `/refuse` for invitations: `TT_REQ_MAKE_GROUP` and the rest). `false` for anything else.
    pub(in crate::web::app) fn party_typed(&mut self, words: &str) -> bool {
        if let Some(line) = words.strip_prefix('%') {
            if let (false, Some(name)) = (line.is_empty(), self.game.as_ref().map(|game| game.name.clone())) {
                self.send_on_map(&party::say(&name, line));
            }
            return true;
        }
        let (command, name) = words.split_once(' ').map_or((words, ""), |(command, name)| (command, name.trim()));
        match (command, name.is_empty()) {
            ("/organize", false) => self.party_organize(name),
            ("/invite", false) => self.party_invite(name),
            ("/expel", false) => self.party_expel(name),
            ("/leave", true) => self.party_leave(),
            ("/accept" | "/refuse", true) => {
                self.send_on_map(&party::refuse_invitations(command == "/refuse"));
            }
            _ => return false,
        }
        true
    }

    /// `/organize name` (the client's `MM_MAKE_GROUP`): the name is looked at, and the small setup window asks
    /// how the new party is to share items before it is made.
    fn party_organize(&mut self, name: &str) {
        if !self.party.members.is_empty() {
            return;
        }
        if name.contains('#') || name.len() >= 24 {
            self.message = Some(if name.contains('#') { Self::words(2839, "Special symbol can't be used in name") } else { Self::words(
                531,
                "Name is too long. Please enter a name up to 23 english characters."
            ) });
            return self.render();
        }
        self.party.making = name.to_owned();
        self.party.side = None;
        self.party_side(PartySide::Named, true);
    }
}
