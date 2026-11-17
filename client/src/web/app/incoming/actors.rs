//! Everyone else on the map and the things lying on it: coming into sight, walking, changing their look, going.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn unit_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // Everyone else on the map: coming into sight, walking, and leaving it.
            units::ZC_NOTIFY_STANDENTRY | units::ZC_NOTIFY_NEWENTRY | units::ZC_NOTIFY_MOVEENTRY => {
                if let Some(unit) = units::unit(id, packet) {
                    world::unit_shown(&unit);
                }
            }
            units::ZC_HP_INFO => {
                if let Some((id, left, most)) = units::life(packet) {
                    world::unit_life(id, left, most);
                }
            }
            units::ZC_ITEM_FALL_ENTRY | units::ZC_ITEM_ENTRY => {
                if let Some((drop, item, cell, within)) = units::dropped(id, packet) {
                    // Its picture is drawn once the item's name is known. One that falls now (not one that
                    // already lay there) is seen falling.
                    data::want_items(std::iter::once(item), || {});
                    world::drop_shown(drop, item, cell, within, id == units::ZC_ITEM_FALL_ENTRY);
                }
            }
            units::ZC_ITEM_DISAPPEAR => {
                if let Some(id) = npc::npc(id, packet) {
                    world::drop_gone(id);
                }
            }
            units::ZC_NOTIFY_MOVE => {
                if let Some((id, from, to)) = units::moved(packet) {
                    world::unit_walk(id, from, to);
                }
            }
            units::ZC_NOTIFY_VANISH => {
                if let Some((id, died)) = units::vanished(packet) {
                    // The player's own character does not go away when it dies: it lies there until the player
                    // has it returned to its save point.
                    if Some(id) == self.account.as_ref().map(|account| account.account_id) {
                        if died {
                            self.dead = true;
                            // The client's data has one voice for dying, a man's (`die_male.wav`); where the
                            // client starts it was not found in its code.
                            if self.own_figure().sex == 1 {
                                sound::effect("die_male.wav", 1.0);
                            }
                            // The client opens its window of options (or closes it, if it was open).
                            self.esc = u8::from(self.esc == 0);
                            self.dialog = None;
                            world::own_dead(true);
                            self.refresh_dialog();
                            self.refresh_values();
                        }
                    } else {
                        world::unit_gone(id, died);
                        // The player's pet is out of sight (not teleported with its master: type 0): it is back
                        // in its egg, or it ran away. The server (rAthena) tells it no other way: its word of
                        // the egg, ZC_CHANGESTATE_PET type 6, says 0 only of a pet about to run away.
                        if id == self.pet_unit && packet.get(6) == Some(&0) {
                            (self.pet, self.pet_unit, self.pet_open, self.pet_menu, self.pet_asks) = (None, 0, false, false, 0);
                            self.render();
                        }
                    }
                }
            }
            // ZC_NPCSPRITE_CHANGE (0x01b0): `u32 who, u8 kind, u32 class`. A monster becomes another (the server has
            // a Fabre turn into a Pupa and a Pupa into a Creamy by itself, as a skill of theirs).
            0x01b0 => {
                if packet.len() >= 11 {
                    let who = u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]]);
                    let class = u32::from_le_bytes([packet[7], packet[8], packet[9], packet[10]]);
                    world::unit_becomes(who, class as u16);
                }
            }
            // ZC_ACK_REQNAMEALL_NPC (0x0adf): `u32 who, u32 group, char[24] name, char[24] title`.
            0x0adf => {
                if packet.len() >= 34 {
                    let who = u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]]);
                    let name = &packet[10..34];
                    let end = name.iter().position(|byte| *byte == 0).unwrap_or(name.len());
                    world::unit_named(who, &String::from_utf8_lossy(&name[..end]));
                }
            }
            // ZC_SPRITE_CHANGE (0x01d7): `u32 who, u8 kind, u32 value, u32 second`. What someone wears or holds
            // changes: it shows on them at once.
            0x01d7 => {
                if packet.len() >= 11 {
                    let who = u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]]);
                    let (kind, value) = (packet[6], u32::from_le_bytes([packet[7], packet[8], packet[9], packet[10]]));
                    let own = Some(who) == self.account.as_ref().map(|account| account.account_id);
                    world::looked((!own).then_some(who), kind, value);
                    // The windows that show the character follow.
                    let char_id = self.zone.as_ref().map(|zone| zone.char_id);
                    if let Some(character) = self.characters.iter_mut().find(|character| own && Some(character.char_id) == char_id) {
                        match kind {
                            1 => character.hair_style = value as _,
                            3 => character.headgear[0] = value as u16,
                            4 => character.headgear[1] = value as u16,
                            5 => character.headgear[2] = value as u16,
                            6 => character.hair_color = value as _,
                            7 => character.body_palette = value as _,
                            _ => {}
                        }
                        self.refresh_items();
                    }
                }
            }
            // ZC_RESURRECTION (0x0148): `u32 who, u16 how`. The player's own character stands up again, and the
            // window that offered the save point goes.
            0x0148 => {
                if packet.len() >= 6 && Some(u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]])) == self.account.as_ref().map(|account| account.account_id) {
                    self.dead = false;
                    self.esc = 0;
                    world::own_dead(false);
                    self.refresh_dialog();
                    self.refresh_values();
                }
            }
            _ => return false,
        }
        true
    }
}
