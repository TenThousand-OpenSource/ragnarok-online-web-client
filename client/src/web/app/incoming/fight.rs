//! What the server says of a fight beside the blows (`net/fight.rs`): the MVP and its reward, the threads of
//! Devotion, what Sense tells, the Star Gladiator's places and missions, a skill changed or taken away, the
//! words of a talkie box and of Graffiti. (The blows themselves: `incoming/combat.rs`.)

use super::*;
use crate::net::fight::{self, StarLine};

/// The client's effects named here: the light over the MVP, the thread of Devotion, and the two a Star
/// Gladiator's skills show on the player's own character.
const EF_MVP: u32 = 68;
const EF_LINELINK: u32 = 232;
const EF_HATED: u32 = 543;
const EF_ELECTRIC: u32 = 432;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn fight_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        let own = self.own_account();
        match id {
            // The one who did the most to bring a boss down stands in the client's light (`AM_MVP`: `EF_MVP`).
            fight::ZC_MVP => {
                if let Some(who) = fight::mvp(packet) {
                    world::burst(EF_MVP, (who != own).then_some(who), None, 0.0);
                }
            }
            // The MVP's reward, said in the chat in yellow: the item by its name, or the experience (the
            // client names no amount).
            fight::ZC_MVP_GETTING_ITEM => {
                if let Some(item) = fight::mvp(packet) {
                    self.fight.mvp_item = Some(item);
                    data::want_items(std::iter::once(item), || dispatch(Action::ItemsKnown));
                    self.mvp_item_said();
                }
            }
            fight::ZC_MVP_GETTING_SPECIAL_EXP => {
                let words = format!("{}{}", Self::words(145, "Congratulations! You are the MVP! Your reward EXP Points are "), Self::words(146, "!!"));
                self.chat_line(words, YELLOW);
            }
            // The reward could not be carried: said in red.
            fight::ZC_THROW_MVPITEM => self.chat_line(
                Self::words(147, "You are the MVP, but you can't take the reward because you are over your weight limit."),
                RED
            ),
            fight::ZC_DEVOTIONLIST => self.devotion_shown(packet, own),
            // The window of a combo: the frames of the one who struck are held for as long (the client's
            // `SetSequenceDeadline`).
            fight::ZC_COMBODELAY => {
                if let Some((who, milliseconds)) = fight::combo_delay(packet) {
                    world::combo_window(who, own, f64::from(milliseconds));
                }
            }
            // A blade caught, or let go (`ReceiveActorPairFreeze`): the two are held still and grey.
            fight::ZC_BLADESTOP => {
                if let Some((who, whose, held)) = fight::blade_stop(packet) {
                    world::blade_caught(who, whose, own, held);
                }
            }
            fight::ZC_MONSTER_INFO => {
                if let Some(sensed) = fight::sensed(packet).filter(|sensed| fight::sense_shown(sensed.job)) {
                    // Named as the one the skill was used on is called (`UIMonsterInfoWnd::OnCreate`).
                    let name = self.fight.aimed.and_then(world::unit_name).unwrap_or_default();
                    self.fight.sensed = Some((sensed, name));
                    self.redraw(&[&wnd::SENSE_WINDOW]);
                }
            }
            fight::ZC_STARPLACE => {
                self.fight.star = fight::star_place(packet);
                self.redraw(&[&wnd::STAR_ASKS]);
            }
            fight::ZC_STARSKILL => self.star_told(packet),
            // A skill as it is now: one copied by Plagiarism comes so, and goes with the packet after.
            crate::net::inventory::ZC_SKILLINFO_UPDATE2 => {
                if let Some(changed) = crate::net::inventory::skill_changed(packet) {
                    self.skills.known.retain(|skill| skill.id != changed.id);
                    self.skills.known.push(changed);
                    self.refresh_items();
                }
            }
            crate::net::inventory::ZC_SKILLINFO_DELETE => {
                if let Some(taken) = crate::net::inventory::skill_taken(packet) {
                    self.skills.known.retain(|skill| skill.id != taken);
                    self.refresh_items();
                }
            }
            // A trap that was not to be seen is seen by everyone: its model shows (`world/skill_units/traps.rs`).
            crate::net::skill_units::ZC_SKILL_UPDATE => {
                if let Some(unit) = crate::net::skill_units::seen(packet) {
                    world::skill_unit_seen(unit);
                }
            }
            crate::net::skill_units::ZC_TALKBOX_CHATCONTENTS => {
                if let Some((unit, words)) = crate::net::skill_units::talkie(packet) {
                    world::spot_says(unit, &words, false);
                }
            }
            // Graffiti: a unit on the ground as any other, with its words.
            crate::net::skill_units::ZC_SKILL_ENTRY2 => {
                if let Some(written) = crate::net::skill_units::written(packet) {
                    world::skill_unit_shown(&crate::net::skill_units::Entry {
                        id: written.id,
                        creator: written.maker,
                        cell: written.cell,
                        unit: u32::from(written.kind),
                        range: 0,
                        visible: written.seen,
                        level: 1
                    });
                    if !written.words.is_empty() {
                        world::spot_says(written.id, &written.words, true);
                    }
                }
            }
            _ => return false,
        }
        true
    }

    /// The MVP's reward is said once its name is known (the client's line 143, the item's name, line 144).
    pub(in crate::web::app) fn mvp_item_said(&mut self) {
        let Some((name, _)) = self.fight.mvp_item.and_then(|item| data::item_as(item, true)) else { return };
        self.fight.mvp_item = None;
        let words = format!("{}{name}{}", Self::words(143, "Congratulations! You are the MVP! Your reward item is "), Self::words(144, "!!"));
        self.chat_line(words, YELLOW);
    }

    /// ZC_DEVOTIONLIST: the threads of a crusader's Devotion (the client's `ReceiveDevotionList`): those the
    /// crusader had are taken away, and one is drawn to each of those guarded now (`EF_LINELINK`, told the
    /// one it leads to).
    fn devotion_shown(&mut self, packet: &[u8], own: u32) {
        let Some((who, wards)) = fight::devotion(packet) else { return };
        let on = (who != own).then_some(who);
        world::effect_ends(EF_LINELINK, on);
        for ward in wards {
            world::burst(EF_LINELINK, on, Some(world::Blow { striker: on, place_of: (ward != own).then_some(ward), hits: 0 }), 0.0);
        }
    }

    /// ZC_STARSKILL: a line across the top of the screen, as the server's announcements are shown (the
    /// client's `ReceiveSpecialBroadcast`: its balloon, letters 14 high, in the colour 0xFFC8C8 counted from
    /// blue), with an effect on the player's own character for some.
    fn star_told(&mut self, packet: &[u8]) {
        let Some((told, (line, hated))) = fight::star_told(packet).and_then(|told| Some((told.clone(), told.line()?))) else { return };
        let own_name = self.game.as_ref().map(|game| game.name.clone()).unwrap_or_default();
        let two = |line: String, first: &str, second: &str| line.replacen("%s", first, 1).replacen("%s", second, 1);
        let words = match line {
            StarLine::Place(line) => two(Self::words(line, "%s: %s"), &own_name, &told.name),
            StarLine::Monster(line) => two(Self::words(line, "%s: %s"), &own_name, &data::job_name(told.value as u16)),
            StarLine::Mission => Self::words(927, "[Taekwon Mission] Target Monster: %s (%d%%)")
                .replacen("%s", &told.name, 1)
                .replacen("%d", &told.which.to_string(), 1)
                .replace("%%", "%"),
            StarLine::Taming => Self::words(1190, "[Taming Mission] Target Monster: %s").replacen("%s", &told.name, 1),
            StarLine::Collector => Self::words(1265, "[Collector Rank] Target Item: %s")
                .replacen("%s", &data::item_as(told.value, true).map(|(name, _)| name).unwrap_or_default(), 1),
            StarLine::Reset => Self::words(1033, "[Solar, Lunar and Stellar Angel] Designated places and monsters have been reset."),
            StarLine::Life => format!("Target HP : {}", told.value),
        };
        if hated {
            world::burst(EF_HATED, None, None, 0.0);
        } else if line == StarLine::Reset {
            world::burst(EF_ELECTRIC, None, None, 0.0);
        }
        self.shown_across(vec![words], "#c8c8ff".to_owned(), 14);
    }
}
