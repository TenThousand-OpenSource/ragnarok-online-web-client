//! What the servers say: each packet is read and the app's state, the windows and the world follow.

use super::*;

mod way_in;
mod ai;
mod homunculus;
mod mercenary;
mod choosing;
mod travel;
mod actors;
mod combat;
mod talk;
mod status;
mod inventory;
mod shops;
mod storage;
mod pets;
mod quests;
mod emotions;
mod party;
mod guild;
mod pvp;
mod exchange;
mod mail;
mod cart;
mod falcon;
mod vending;
mod fight;
mod friends;
mod whispers;
mod viewequip;
mod unread;
mod scripted;
mod chat_rooms;
mod family;
mod notices;
mod link;
mod broadcast;
mod marks;
mod statuses;
mod cooldowns;
mod commands;
mod skill_units;
mod spirits;
mod books;

/// The readers of the map server's packets, a system each (a file each in `incoming/`), in the order they are
/// asked: each gives back whether the packet was its own, and the first that says so has read it.
const MAP_SERVER: [fn(&mut App, u16, &[u8]) -> bool; 39] = [
    App::cart_packet,
    App::falcon_packet,
    App::spirits_packet,
    App::skill_unit_packet,
    App::vending_packet,
    App::chat_room_packet,
    App::scripted_packet,
    App::family_packet,
    App::ai_packet,
    App::choosing_packet,
    App::mail_packet,
    App::statuses_packet,
    App::cooldown_packet,
    App::command_packet,
    App::travel_packet,
    App::unit_packet,
    App::combat_packet,
    App::fight_packet,
    App::talk_packet,
    App::status_packet,
    App::item_packet,
    App::trade_packet,
    App::storage_packet,
    App::pet_packet,
    App::homunculus_packet,
    App::mercenary_packet,
    App::quest_packet,
    App::emotion_packet,
    App::party_packet,
    App::exchange_packet,
    App::friend_packet,
    App::viewequip_packet,
    App::pvp_packet,
    App::guild_packet,
    App::notice_packet,
    App::link_packet,
    App::broadcast_packet,
    App::mark_packet,
    App::book_packet,
];

impl App {
    /// A packet of the server the app is talking to: handed to what reads that server's (`incoming/way_in.rs`
    /// for the two servers of the way in, `MAP_SERVER` for the map server).
    pub(super) fn packet(&mut self, id: u16, packet: &[u8]) {
        match self.server {
            _ if id == packets::SC_NOTIFY_BAN => self.banned(packet),
            Server::Login => self.login_packet(id, packet),
            Server::Char => self.char_packet(id, packet),
            Server::Map => {
                let known = MAP_SERVER.iter().any(|read| read(self, id, packet));
                // Whatever is not read yet is at least counted, so that it shows there is more to do
                // (`incoming/unread.rs`).
                if !known {
                    self.unread_packet(id, packet);
                }
            }
            Server::None => {}
        }
    }
}
