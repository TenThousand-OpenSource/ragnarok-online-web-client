//! The friends list: what the map server tells of it. What the chat says, and in which colour, is the client's
//! (`CGameMode::ReceiveFriendList`, `ReceiveFriendAddResult`, `CSession::SetFriendOffline` and the
//! `ID_ZC_*FRIENDS*` arms of its packet handler).

use super::*;
use crate::net::friends;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn friends_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // The whole list, told on entering: everyone is out of the game until the server says otherwise.
            friends::ZC_FRIENDS_LIST => {
                self.friends.list = friends::list(packet);
                self.friends_changed();
            }
            friends::ZC_FRIENDS_STATE => {
                if let Some(friend) = friends::state(packet) {
                    self.friend_came_or_went(friend);
                }
            }
            // Someone asks to be a friend. With the client's `/notrade` on they are turned down at once;
            // otherwise the question is asked in a box of its own (`UIJoinFriendAcceptWnd`).
            friends::ZC_REQ_ADD_FRIENDS => {
                if let Some((account, character, name)) = friends::asked(packet) {
                    if self.option("TT_NOTRADE_ON_OFF") {
                        self.send_on_map(&friends::answer(account, character, false));
                    } else {
                        self.friends.asks = Some(FriendAsk::Wanted(account, character, name));
                        self.redraw(&[&wnd::FRIEND_ASKS]);
                    }
                }
            }
            friends::ZC_ADD_FRIENDS_LIST => {
                if let Some((result, friend)) = friends::answered(packet) {
                    self.friend_answered(result, friend);
                }
            }
            friends::ZC_DELETE_FRIENDS => {
                if let Some((_, character)) = friends::removed(packet) {
                    self.friends.list.retain(|friend| friend.character != character);
                    self.friends_changed();
                }
            }
            _ => return false,
        }
        true
    }

    /// The list changed: the pick and the menu went by the places in it, so they are let go, and the window
    /// that shows the list follows.
    fn friends_changed(&mut self) {
        (self.friends.picked, self.friends.menu) = (None, None);
        self.redraw(&[&wnd::PARTY, &wnd::FRIEND_MENU]);
    }

    /// A friend came into the game or left it (`ID_ZC_FRIENDS_STATE`): the name is taken as the server has
    /// it now (a friend told of with no name is removed, as the client does: `CSession::UpdateFriendName`),
    /// and with the client's `/loginout` on the chat tells of it.
    fn friend_came_or_went(&mut self, told: friends::Friend) {
        if told.name.is_empty() {
            if let Some(gone) = self.friends.list.iter().position(|friend| friend.character == told.character) {
                let gone = self.friends.list.remove(gone);
                self.send_on_map(&friends::remove(gone.account, gone.character));
                self.friends_changed();
            }
            return;
        }
        let Some(friend) = self.friends.list.iter_mut().find(|friend| friend.account == told.account && friend.character == told.character) else { return };
        (friend.name, friend.offline) = (told.name.clone(), told.offline);
        if self.option("TT_LOGINOUT_ON_OFF") {
            let line = if told.offline { Self::words(1042, "%s has logged out.") } else { Self::words(1041, "%s has logged in.") };
            self.chat_line(line.replace("%s", &told.name), CYAN);
        }
        self.redraw(&[&wnd::PARTY]);
    }

    /// What became of asking someone to be a friend (`ReceiveFriendAddResult`): the chat tells it, and a
    /// friend made joins the list, in the game. (The client adds one already in the list a second time; here
    /// the one there is kept.)
    fn friend_answered(&mut self, result: u16, friend: friends::Friend) {
        let named = |line: usize, otherwise: &str| Self::words(line, otherwise).replace("%s", &friend.name);
        let (words, colour) = match result {
            friends::added::DONE => (named(821, "You have become friends with (%s)."), CYAN),
            friends::added::REFUSED => (named(822, "(%s) does not want to be friends with you."), RED),
            friends::added::OWN_LIST_FULL => (named(819, "Your Friend List is full."), RED),
            friends::added::THEIR_LIST_FULL => (named(820, "(%s)'s Friend List is full."), RED),
            4 => (named(2737, "The player has already requested you to be a friend."), RED),
            6 => (named(3448, "You're already friends"), RED),
            7 => (named(3449, "Can't find other party"), RED),
            8 => (named(3450, "You're already a friend with another character."), RED),
            _ => return,
        };
        self.chat_line(words, colour);
        if result == friends::added::DONE {
            match self.friends.list.iter_mut().find(|known| known.name == friend.name) {
                Some(known) => *known = friend,
                None => self.friends.list.push(friend),
            }
            self.friends_changed();
        }
    }
}
