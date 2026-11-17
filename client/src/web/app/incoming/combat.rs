//! Blows and skills: who strikes whom, what is being cast, what it does when it lands.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn combat_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // 0x0147: an item used has the character cast a skill (a Fly Wing casts Teleport): the client casts it,
            // on the character itself, as the original does. `u16 skill, u32 kind, u16 level, …`.
            0x0147 => {
                if let (Some(skill), Some(level), Some(socket), Some(account)) = (packet.get(2..4), packet.get(8..10), &self.socket, &self.account) {
                    socket.send(&items::use_skill(u16::from_le_bytes([level[0], level[1]]), u16::from_le_bytes([skill[0], skill[1]]), account.account_id));
                }
            }
            // 0x013d and 0x0a27: the character recovers life or spirit (a potion, food): `u16 which, amount`.
            // The client shows the amount as a number over the character, green for life and blue for spirit.
            0x013d | 0x0a27 => {
                let which = packet.get(2..4).map_or(0, |which| u16::from_le_bytes([which[0], which[1]]));
                let amount = if id == 0x013d { packet.get(4..6).map_or(0, |amount| i32::from(i16::from_le_bytes([amount[0], amount[1]]))) } else { packet.get(4..8).map_or(0, |amount| i32::from_le_bytes([amount[0], amount[1], amount[2], amount[3]])) };
                if amount > 0 && matches!(which, packets::parameter::HP | packets::parameter::SP) {
                    world::recovered(amount, which == packets::parameter::SP);
                    sound::effect("_heal_effect.wav", 1.0);
                }
            }
            units::ZC_NOTIFY_ACT => {
                let own = self.account.as_ref().map_or(0, |account| account.account_id);
                // Hits of every kind; sitting down, picking up and the like are not shown yet.
                match units::act(packet) {
                    Some(act) if matches!(act.kind, 0 | 4 | 8 | 9 | 10 | 11 | 13) => {
                        // The sound of the blow, heard when it lands: the weapon's when the character strikes,
                        // the common one when it is struck. A monster's own sounds are in its sprite.
                        let heard = if act.source == own {
                            Some(self.weapon_sound().to_owned())
                        } else if act.target == own {
                            Some(format!("_enemy_hit_normal{}.wav", 1 + packet.get(10).copied().unwrap_or(0) % 4))
                        } else {
                            None
                        };
                        world::struck(act.source, act.target, own, f64::from(act.speed), f64::from(act.delay), act.damage, heard);
                        // One who holds a bow sends an arrow, which arrives as the blow lands.
                        world::arrow(act.source, act.target, own, f64::from(act.delay));
                    }
                    // Someone bends down for a thing on the ground.
                    Some(act) if act.kind == 1 => world::picks_up(act.source, own, act.target),
                    Some(act) if matches!(act.kind, 2 | 3) => {
                        if act.source == own {
                            self.hero.sitting = act.kind == 2;
                        }
                        world::sat(act.source, own, act.kind == 2);
                    }
                    _ => {}
                }
            }
            // Someone starts to cast (the client's `NotifySkillCasting`). A skill with no casting time shows
            // nothing here. The rings are those of the cast's element (`GetSkillActionInfo`).
            skills::ZC_USESKILL_ACK => {
                if let Some(cast) = skills::casting(packet).filter(|cast| cast.milliseconds > 0) {
                    let own = self.account.as_ref().map_or(0, |account| account.account_id);
                    let effect = match cast.element {
                        1 => 54,
                        2 => 56,
                        3 => 55,
                        4 => 57,
                        5 => 59,
                        6 => 58,
                        _ => 12,
                    };
                    world::cast(cast.source, own, cast.target, cast.cell, f64::from(cast.milliseconds), effect);
                }
            }
            // ZC_DISPEL (0x01b9): `u32 who`. A cast is broken off (the caster was struck, or moved).
            0x01b9 => {
                if packet.len() >= 6 {
                    let own = self.account.as_ref().map_or(0, |account| account.account_id);
                    world::cast_over(u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]]), own, true);
                }
            }
            // A skill lands: the caster strikes, and each of its hits shows its share of the damage, one after
            // another.
            skills::ZC_NOTIFY_SKILL => {
                if let Some(landed) = skills::landing(packet) {
                    let own = self.account.as_ref().map_or(0, |account| account.account_id);
                    world::cast_over(landed.source, own, false);
                    let hits = i32::from(landed.hits.max(1));
                    let each = landed.damage / hits;
                    let delay = f64::from(landed.attack_motion.clamp(0, 1500));
                    // Each hit of a skill sounds as one of the client's four plain blows (its
                    // `GetWeaponHitWaveName(-1)` for the hits of skills).
                    let blow = || Some(format!("_enemy_hit_normal{}.wav", 1 + (js_sys::Math::random() * 4.0) as u32));
                    world::struck(landed.source, landed.target, own, f64::from(landed.attack_motion), delay, each, blow());
                    // The hits follow one another 200 milliseconds apart (the client's `g_multiHitInterval`).
                    for hit in 1..hits {
                        world::lands(landed.target, own, delay + f64::from(hit) * 200.0, each, blow());
                    }
                    // What the skill looks and sounds like on the one struck (the client's `PlaySkillEffect`
                    // for the bolts themselves, and the effect of each hit by the skill's element).
                    let on = (landed.target != own).then_some(landed.target);
                    let pick = |most: f64| 1 + (js_sys::Math::random() * most) as u32;
                    match landed.skill {
                        // Fire Bolt and Cold Bolt: a bolt for each hit.
                        19 => world::bolts(landed.target, hits as u32, true),
                        14 => world::bolts(landed.target, hits as u32, false),
                        // Lightning Bolt: the client's lightning, once.
                        // (Its sound is the client's `EF_LightBolt.wav`; where the client starts it was not
                        // found in its code.)
                        20 => world::effect_later("lightning.str", on, 0.0, Some("effect/ef_lightbolt.wav")),
                        _ => {}
                    }
                    for hit in 0..hits {
                        let when = delay + f64::from(hit) * 200.0;
                        match landed.skill {
                            19 => world::effect_later(&format!("firehit{}.str", pick(3.0)), on, when, Some(&format!("_enemy_hit_fire{}.wav", pick(2.0)))),
                            20 => world::effect_later(&format!("windhit{}.str", pick(3.0)), on, when, None),
                            _ => {}
                        }
                    }
                }
            }
            skills::ZC_USE_SKILL => {
                if let Some((skill, _, target, who, _)) = skills::used(packet) {
                    let own = self.account.as_ref().map_or(0, |account| account.account_id);
                    world::cast_over(who, own, false);
                    // Resurrection (skill 54) shows the client's effect on the one raised, with its sound
                    // (`EF_RESURRECTION`: `Resurrection.str` and `priest_resurrection.wav`).
                    if skill == 54 {
                        world::effect("resurrection.str", (target != own).then_some(target));
                        sound::effect("effect/priest_resurrection.wav", 1.0);
                    }
                }
            }
            // A skill could not be used: the client says why in the chat, in red, with the line of its message
            // table for the cause (`ReceiveSkillFailure`). A cause this has no line for says the plain one.
            skills::ZC_ACK_TOUSESKILL => {
                if let Some((skill, value, worked, cause)) = skills::failed(packet).filter(|(_, _, worked, _)| !worked) {
                    let _ = worked;
                    let line = skills::failure(skill, value, cause).and_then(data::message).unwrap_or_else(|| "Skill has failed.".to_owned());
                    let named = data::skill(skill).map(|(_, name, _)| name).unwrap_or_default();
                    self.chat_line(crate::i18n::tr(&line).replace("%s", &named), "#ff0000");
                }
            }
            // ZC_ATTACK_RANGE (0x013a): `u16 range`: how far the weapon reaches.
            0x013a => {
                if packet.len() >= 4 {
                    world::weapon_reaches(i32::from(u16::from_le_bytes([packet[2], packet[3]])));
                }
            }
            // ZC_ATTACK_FAILURE_FOR_DISTANCE (0x0139): `u32 target, i16 x, i16 y, i16 own x, i16 own y, i16 range`.
            // The blow was asked for from too far: the character goes nearer and strikes from there.
            0x0139 => {
                if packet.len() >= 16 {
                    let target = u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]]);
                    world::too_far(target, i32::from(i16::from_le_bytes([packet[14], packet[15]])));
                }
            }
            _ => return false,
        }
        true
    }
}
