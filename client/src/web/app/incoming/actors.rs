//! Everyone else on the map and the things lying on it: coming into sight, walking, changing their look, going.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn actors_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // Everyone else on the map: coming into sight, walking, and leaving it.
            net::actors::ZC_NOTIFY_STANDENTRY | net::actors::ZC_NOTIFY_NEWENTRY | net::actors::ZC_NOTIFY_MOVEENTRY => self.unit_entered(id, packet),
            net::actors::ZC_HP_INFO => {
                if let Some((id, left, most)) = net::actors::life(packet) {
                    world::unit_life(id, left, most);
                }
            }
            net::actors::ZC_ITEM_FALL_ENTRY | net::actors::ZC_ITEM_ENTRY => {
                if let Some((drop, item, cell, within, identified, count)) = net::actors::dropped(id, packet) {
                    // Its picture is drawn once the item's name is known. One that falls now (not one that
                    // already lay there) is seen falling.
                    data::want_items(std::iter::once(item), || {});
                    world::drop_shown(drop, (item, identified, count), cell, within, id == net::actors::ZC_ITEM_FALL_ENTRY);
                }
            }
            net::actors::ZC_ITEM_DISAPPEAR => {
                if let Some(id) = net::talk::npc(id, packet) {
                    world::drop_gone(id);
                }
            }
            net::actors::ZC_NOTIFY_MOVE => {
                if let Some((id, walk)) = net::actors::moved(packet) {
                    world::unit_walk(id, &walk);
                }
            }
            net::actors::ZC_HIGHJUMP | net::actors::ZC_FASTMOVE => {
                if let Some((id, cell)) = net::actors::slid(packet) {
                    let own = Some(id) == self.account.as_ref().map(|account| account.account_id);
                    world::slid((!own).then_some(id), cell);
                }
            }
            net::actors::ZC_NOTIFY_VANISH => self.unit_vanished(packet),
            // Someone turns on the spot, or turns its head.
            net::actors::ZC_CHANGE_DIRECTION => {
                if let Some((who, head, direction)) = net::actors::turned(packet) {
                    let own = Some(who) == self.account.as_ref().map(|account| account.account_id);
                    world::turned((!own).then_some(who), head, direction);
                }
            }
            // The states that show on someone in sight are told anew (`crate::state_effects::seen`).
            net::actors::ZC_NPC_SHOWEFST_UPDATE => {
                if let Some((who, bits, third)) = net::actors::third_option(packet) {
                    world::third_option_told(who, bits, third);
                }
            }
            net::actors::ZC_STATE_CHANGE => self.unit_stated(packet),
            // A monster becomes another (the server has a Fabre turn into a Pupa and a Pupa into a Creamy by
            // itself, as a skill of theirs).
            net::actors::ZC_NPCSPRITE_CHANGE => {
                if let Some((who, class)) = net::actors::became(packet) {
                    world::unit_becomes(who, class as u16);
                }
            }
            net::actors::ZC_ACK_REQNAMEALL_NPC => Self::unit_named(packet),
            net::actors::ZC_SPRITE_CHANGE => self.look_changed(packet),
            net::actors::ZC_RESURRECTION => self.unit_risen(packet),
            _ => return false,
        }
        true
    }

    /// Someone comes into sight, standing, walking or coming into being there.
    fn unit_entered(&mut self, id: u16, packet: &[u8]) {
        if let Some(unit) = net::actors::unit(id, packet) {
            world::unit_shown(&unit);
            // A player that comes into being in sight (logged in, teleported in) arrives in the walls
            // of light of `EF_ENTRY2`, unless it is invisible (the client's reader of the packet).
            if id == net::actors::ZC_NOTIFY_NEWENTRY && unit.kind == net::actors::Kind::Player && unit.state & 0x40 == 0 {
                world::burst(344, Some(unit.id), None, 0.0);
            }
            // A player of a guild: its name and emblem are asked for (`incoming/guild.rs`).
            if unit.kind == net::actors::Kind::Player {
                self.guild_looked_up(unit.id, unit.guild, u32::from(unit.emblem));
            }
        }
    }

    /// ZC_RESURRECTION: the player's own character stands up again, and the window that offered the save point
    /// goes; another who lay dead stands up where it lay.
    fn unit_risen(&mut self, packet: &[u8]) {
        match net::actors::risen(packet) {
            Some(who) if Some(who) == self.account.as_ref().map(|account| account.account_id) => {
                self.hero.dead = false;
                self.esc = 0;
                world::own_dead(false);
                self.redraw(&[&wnd::DIALOG]);
                self.refresh_values();
            }
            Some(who) => world::risen(who),
            None => {}
        }
    }

    /// ZC_STATE_CHANGE: the state someone is in changed: what they ride, whether they hide, how they ail.
    fn unit_stated(&mut self, packet: &[u8]) {
        if let Some((who, bits)) = net::actors::state_changed(packet) {
            let own = Some(who) == self.account.as_ref().map(|account| account.account_id);
            world::stated((!own).then_some(who), bits, None);
            // Its body and health states colour it, mark it and sound (`world/ailments.rs`).
            if let Some((_, body, health)) = crate::net::statuses::body_and_health(packet) {
                world::ailing((!own).then_some(who), body, health);
            }
            if own {
                self.hero.state = bits;
                self.render();
                // Its body and health states are listed among its states (`incoming/statuses.rs`).
                self.body_stated(packet);
            }
        }
    }

    /// ZC_ACK_REQNAMEALL_NPC (0x0adf): `u32 who, u32 group, char[24] name, char[24] title`.
    fn unit_named(packet: &[u8]) {
        if let Some((who, group, name, title)) = net::actors::named(packet) {
            world::unit_named(who, &name);
            // The title and the group are written with the name (`world/names.rs`).
            world::unit_titled(who, group, &title);
        }
    }

    fn unit_vanished(&mut self, packet: &[u8]) {
        if let Some((id, died)) = net::actors::vanished(packet) {
            // The player's own character does not go away when it dies: it lies there until the player
            // has it returned to its save point.
            if Some(id) == self.account.as_ref().map(|account| account.account_id) {
                if died {
                    self.hero.dead = true;
                    // This client's own choice, asked for by its owner: the voice of dying. The
                    // client's data has the files (`die_male.wav` for men, `die_<first job>_female.wav`
                    // for women), but the 2026 client names none of them, in its code or in the bodies'
                    // sprites: there a player dies without a sound. A woman's file is that of the
                    // first job hers grew out of; a Doram, who grew out of none of them, has none.
                    const WOMEN: [&str; 7] = ["novice", "swordman", "magician", "archer", "acolyte", "merchant", "thief"];
                    let figure = self.own_figure();
                    if figure.sex == 1 {
                        sound::effect("die_male.wav", 1.0);
                    } else if let Some(first) = data::first_job(figure.job) {
                        sound::effect(&format!("die_{}_female.wav", WOMEN[usize::from(first)]), 1.0);
                    }
                    // The client opens its window of options (or closes it, if it was open).
                    self.esc = u8::from(self.esc == 0);
                    self.dialog = None;
                    world::own_dead(true);
                    self.redraw(&[&wnd::DIALOG]);
                    self.refresh_values();
                }
            } else {
                world::unit_gone(id, died);
                // The player's pet is out of sight (not teleported with its master: type 0): it is back
                // in its egg, or it ran away. The server (rAthena) tells it no other way: its word of
                // the egg, ZC_CHANGESTATE_PET type 6, says 0 only of a pet about to run away.
                if id == self.pets.unit && net::actors::vanished_how(packet) == Some(0) {
                    (self.pets.pet, self.pets.unit, self.pets.open, self.pets.menu, self.pets.asks) = (None, 0, false, false, 0);
                    self.render();
                }
            }
        }
    }

    /// What someone wears or holds changes (ZC_SPRITE_CHANGE): it shows on them at once.
    fn look_changed(&mut self, packet: &[u8]) {
        if let Some((who, kind, value, second)) = net::actors::look(packet) {
            let own = Some(who) == self.account.as_ref().map(|account| account.account_id);
            world::looked((!own).then_some(who), kind, value, second);
            // A unit of a skill on the ground changes its kind in the same packet (`world/skill_units.rs`).
            if kind == 0 && !own {
                world::skill_unit_becomes(who, value);
            }
            // The windows that show the character follow.
            let char_id = self.zone.as_ref().map(|zone| zone.char_id);
            if let Some(character) = self.characters.iter_mut().find(|character| own && Some(character.char_id) == char_id) {
                match kind {
                    // The job changed: the character, its name in the Basic Information window and its
                    // pages of skills are those of the new one.
                    0 => {
                        character.job = value as u16;
                        if let Some(game) = &mut self.game {
                            game.job = value as u16;
                        }
                    }
                    1 => character.hair_style = value as _,
                    3 => character.headgear[0] = value as u16,
                    4 => character.headgear[1] = value as u16,
                    5 => character.headgear[2] = value as u16,
                    6 => character.hair_color = value as _,
                    7 => character.body_palette = value as _,
                    _ => {}
                }
                self.refresh_items();
                self.refresh_values();
            }
        }
    }
}
