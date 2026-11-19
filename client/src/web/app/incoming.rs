//! What the servers say: each packet is read and the app's state, the windows and the world follow.
//!
//! Each system has its own file here, named as its packets are in `net/` and its presses in `clicks/`; its
//! reader is `<file>_packet`, one name of `MAP_SERVER`.

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
mod hotkeys;
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
///
/// **The order carries meaning.** Some readers overhear a packet of another system: they do their part and
/// give back `false`, so that the packet goes on to its own reader, which must therefore stand after them.
/// Nothing but this list holds that. Who overhears what, and whose it is:
///
/// | The packet | Overheard by (in this order) | Read by |
/// |---|---|---|
/// | `ZC_ACCEPT_ENTER` (the game is entered) | cart, vending, ai, mail: each forgets the character before | travel |
/// | `ZC_NPCACK_MAPMOVE` (another map) | vending, ai | travel |
/// | `ZC_INVENTORY_START`, `ZC_INVENTORY_ITEMLIST_NORMAL`, `_EQUIP` | cart: the cart's list is one kind of these | inventory (which hands the storage's kind on) |
/// | `ZC_MSG_STATE_CHANGE`, `2`, `3`, `ZC_EFST_SET_ENTER2` | cart (who pushes which cart), mail (letters sent today; `2` and `3` only) | statuses |
/// | `ZC_DELETE_ITEM_FROM_BODY` | mail: a letter being written closes | inventory |
/// | `ZC_NOTIFY_ACT`, `ZC_USESKILL_ACK` | ai: who struck whom | combat |
/// | `ZC_NOTIFY_SKILL` | falcon (a skill that sends it), skill units (a trap going off), ai | combat |
/// | `ZC_USE_SKILL` | falcon (Spring Trap) | combat |
/// | `ZC_NOTIFY_GROUNDSKILL` | falcon (Detect) | skill units |
/// | `ZC_NOTIFY_VANISH` | spirits (the spheres go), vending (the sign goes), ai | actors |
/// | `ZC_USER_COUNT` | commands: the answer to `/who` | status |
///
/// Two readers take a packet only when it is theirs by what it says, and so stand before the reader that takes
/// it otherwise: chat rooms take `ZC_NOTIFY_CHAT` and `ZC_NOTIFY_PLAYERCHAT` while the character sits in a
/// room, before talk; the homunculus takes `ZC_CONFIG` for its own setting, before viewequip. And choosing
/// stands before notices: both name `ZC_ACK_REQNAME_BYGID`, and it is choosing that reads it.
const MAP_SERVER: [fn(&mut App, u16, &[u8]) -> bool; 41] = [
    // Overhears the game entered, the inventory's lists and the states: before travel, inventory, statuses.
    App::cart_packet,
    // Overhears the skills that send a falcon; reads nothing: before skill units and combat.
    App::falcon_packet,
    // Overhears who goes out of sight: before actors.
    App::spirits_packet,
    // Overhears a skill that strikes: before combat. Reads a skill used on a place: after the falcon.
    App::skill_units_packet,
    // Overhears the game entered, another map, who goes out of sight: before travel and actors.
    App::vending_packet,
    // Takes what is said near while the character sits in a room: before talk.
    App::chat_rooms_packet,
    App::scripted_packet,
    App::family_packet,
    // Overhears blows, skills, who goes out of sight, the game entered, another map; reads nothing: before
    // travel, actors and combat.
    App::ai_packet,
    // At every packet it asks for the names of makers a window looked up in vain. Reads the name of a character
    // asked by its number: before notices.
    App::choosing_packet,
    // Overhears the game entered, the states, a thing leaving the inventory: before travel, statuses, inventory.
    App::mail_packet,
    App::statuses_packet,
    App::cooldowns_packet,
    // Overhears how many play: before status.
    App::commands_packet,
    App::travel_packet,
    App::actors_packet,
    App::combat_packet,
    App::fight_packet,
    App::talk_packet,
    // (Whispers were read with what is said, and the hotkey bar's list with the inventory: each stands
    // where the reader it was taken out of stands.)
    App::whispers_packet,
    App::status_packet,
    App::inventory_packet,
    App::hotkeys_packet,
    App::shops_packet,
    App::storage_packet,
    App::pets_packet,
    // Takes its own setting of `ZC_CONFIG`: before viewequip.
    App::homunculus_packet,
    App::mercenary_packet,
    App::quests_packet,
    App::emotions_packet,
    App::party_packet,
    App::exchange_packet,
    App::friends_packet,
    App::viewequip_packet,
    App::pvp_packet,
    App::guild_packet,
    App::notices_packet,
    App::link_packet,
    App::broadcast_packet,
    App::marks_packet,
    App::books_packet,
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
