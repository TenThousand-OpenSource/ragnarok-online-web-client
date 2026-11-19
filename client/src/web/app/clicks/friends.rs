//! The friends list: what the buttons of its side of the window do, its key, the menu on a friend and its
//! questions (the client's `UIMessengerGroupWnd::SendMsg` with `mode` 0, `UIFriendOptionWnd::SendMsg`,
//! `UIJoinFriendAcceptWnd::SendMsg`, and `HK_FRIENDWND_ONOFF` of `UIWindowMgr`).

use super::*;
use crate::net::friends;

impl App {
    /// A press of this system (`crate::press::friends`).
    pub(super) fn friend_pressed(&mut self, press: crate::press::friends::Press) {
        use crate::press::friends::Press;
        match press {
            Press::Party => self.messenger_turned(false),
            // A press on the friend picked already changes nothing: the row stays as it is on the page, for
            // the second press of a double click to land on.
            Press::Row(row) => {
                if self.friends.menu.take().is_some() || self.friends.picked != Some(row) {
                    self.friends.picked = Some(row);
                    self.redraw(&[&wnd::PARTY, &wnd::FRIEND_MENU]);
                }
            }
            // The talk with the friend picked (`ID_TALK`).
            Press::Talk => {
                if let Some(row) = self.friends.picked {
                    self.friend_talk(row);
                }
            }
            // The Friend Setup opens beside the list, or closes; the party's window that stood there goes
            // (`ID_PARTY_SETTING` with `mode` 0).
            Press::Setup => {
                self.friends.setup = !self.friends.setup;
                self.party.side = None;
                self.render();
            }
            Press::SetupClose => {
                self.friends.setup = false;
                self.redraw(&[&wnd::FRIEND_SETUP]);
            }
            Press::Option(which) => {
                self.whisper_option_switched(which);
                self.redraw(&[&wnd::FRIEND_SETUP]);
            }
            // Removing the friend picked is asked about first (`ID_DELETE_FRIEND`).
            Press::Remove => {
                if let Some(row) = self.friends.picked {
                    self.friend_removal_asked(row);
                }
            }
            Press::Yes | Press::No => self.friend_asked_answered(press == Press::Yes),
            Press::Do(entry) => self.friend_menu_row(entry),
        }
    }

    /// The key of the friends list or of the party (`HK_FRIENDWND_ONOFF`, `HK_PARTYWND_ONOFF`): the window
    /// closes if it is open, and opens on that side if it was closed or showed the other. Gives back that
    /// the key was used.
    pub(in crate::web::app) fn messenger_key(&mut self, friends: bool) -> bool {
        let other = self.friends.shown != friends;
        self.party.open = !self.party.open || other;
        self.messenger_turned(friends);
        true
    }

    /// The window turns to its Friend side or its Party side (`ID_RADIOBTN`). The small windows beside it and
    /// the menus are put away: each belongs to one side. (The client leaves the one beside it open until its
    /// own button is pressed again.)
    pub(in crate::web::app) fn messenger_turned(&mut self, friends: bool) {
        self.friends.shown = friends;
        (self.party.side, self.party.menu) = (None, None);
        self.friends_put_away();
        self.render();
    }

    /// What stands beside the list of friends and over it is put away: the window closes or turns to the party.
    pub(in crate::web::app) fn friends_put_away(&mut self) {
        (self.friends.setup, self.friends.menu) = (false, None);
    }

    /// A window opens for a talk with the friend of that row (`ID_TALK`; a double click on the row does it
    /// too, `OnLBtnDblClk`): not with the player's own character.
    pub(in crate::web::app) fn friend_talk(&mut self, row: usize) {
        let Some(name) = self.friends.list.get(row).map(|friend| friend.name.clone()) else { return };
        if self.game.as_ref().is_none_or(|game| game.name != name) {
            self.whisper_to(&name);
        }
    }

    /// The player of that name is asked to be a friend (`UIWindowMgr::RequestAddFriend`).
    pub(in crate::web::app) fn friend_wanted(&mut self, name: &str) {
        self.send_on_map(&friends::add(name));
    }

    fn friend_removal_asked(&mut self, row: usize) {
        if let Some(friend) = self.friends.list.get(row) {
            self.friends.asks = Some(FriendAsk::Remove(friend.account, friend.character));
            self.redraw(&[&wnd::FRIEND_ASKS]);
        }
    }

    /// The answer to what the friends list asked. Taking someone as a friend opens the window on its Friend
    /// side (the client's `MM_JOIN_FRIEND`).
    fn friend_asked_answered(&mut self, yes: bool) {
        let Some(ask) = self.friends.asks.take() else { return };
        self.redraw(&[&wnd::FRIEND_ASKS]);
        match ask {
            FriendAsk::Wanted(account, character, _) => {
                self.send_on_map(&friends::answer(account, character, yes));
                if yes {
                    self.party.open = true;
                    self.messenger_turned(true);
                }
            }
            FriendAsk::Remove(account, character) if yes => self.send_on_map(&friends::remove(account, character)),
            FriendAsk::Remove(..) => {}
        }
    }

    /// A row of the menu on a friend was pressed (`UM_MENU_SELECTED`).
    fn friend_menu_row(&mut self, entry: crate::press::friends::Entry) {
        use crate::press::friends::Entry;
        let Some((_, row)) = self.friends.menu.take() else { return };
        self.redraw(&[&wnd::FRIEND_MENU]);
        let Some(name) = self.friends.list.get(row).map(|friend| friend.name.clone()) else { return };
        match entry {
            Entry::Chat => self.friend_talk(row),
            // A letter is written to the friend (the client's `MM_OPEN_WRITE_RODEX`: `clicks/mail/writing.rs`).
            Entry::Mail => self.mail_write_to(&name),
            Entry::Remove => self.friend_removal_asked(row),
            Entry::Invite => self.party_invite(&name),
        }
    }

    /// Puts the menu on a friend away, if it is open.
    pub(in crate::web::app) fn close_friend_menu(&mut self) {
        if self.friends.menu.take().is_some() {
            self.redraw(&[&wnd::FRIEND_MENU]);
        }
    }

    /// The right button on a friend in the list: the friend is picked and its menu opens there (`OnRBtnUp`).
    pub(in crate::web::app) fn friend_menu_opens(&mut self, row: usize, at: (f32, f32)) {
        if row < self.friends.list.len() {
            let at = Self::on_stage(at);
            (self.friends.picked, self.friends.menu) = (Some(row), Some((at, row)));
            self.redraw(&[&wnd::PARTY]);
            self.open_at(&wnd::FRIEND_MENU, &wnd::menu_style(at));
        }
    }
}
