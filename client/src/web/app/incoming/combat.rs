//! Blows and skills: who strikes whom, what is being cast, what it does when it lands.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn combat_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        let own = self.own_account();
        match id {
            // The server has one of the client's effects shown on someone: an NPC's script, an item, some
            // skills, a GM's `@effect` (the client's `ReceiveNotifyEffect`: `LaunchEffect` on that actor).
            skills::ZC_NOTIFY_EFFECT2 | skills::ZC_NOTIFY_EFFECT3 => {
                if let Some((who, effect)) = skills::effect_shown(packet) {
                    // An effect that is a number shows the number sent with it over that one, in its colour
                    // (the client's `ReceiveActorStatusValue`).
                    if let (Some(colour), Some(number)) = (crate::blow::signs::number_colour(effect), skills::effect_number(packet)) {
                        world::number_over(who, own, number, colour);
                    }
                    self.effect_shown(effect, who, own, 0.0, None);
                }
            }
            // An item used has the character cast a skill (a Fly Wing casts Teleport): the client casts it, on
            // the character itself, as the original does.
            skills::ZC_AUTORUN_SKILL => {
                if let (Some((skill, level)), Some(socket), Some(account)) = (skills::item_skill(packet), &self.socket, &self.account) {
                    socket.send(&net::inventory::use_skill(level, skill, account.account_id));
                }
            }
            // The character recovers life or spirit (a potion, food): the client shows the amount as a number
            // over the character, green for life and blue for spirit.
            skills::ZC_RECOVERY | skills::ZC_RECOVERY2 => {
                if let Some((which, amount)) = skills::recovered(packet).filter(
                    |(which, amount)| *amount > 0 && matches!(*which, packets::parameter::HP | packets::parameter::SP)
                ) {
                    world::number_over(own, own, amount, if which == packets::parameter::SP { crate::blow::signs::BLUE } else { crate::blow::signs::GREEN });
                    sound::effect("_heal_effect.wav", 1.0);
                }
            }
            net::actors::ZC_NOTIFY_ACT => self.act_seen(packet),
            skills::ZC_USESKILL_ACK => self.cast_begun(packet),
            // A skill that warns of the area it will strike is being cast: red planes lie on its cells meanwhile
            // (`CGameMode::SpawnSkillWarningPlanes`: the area is the skill's `SkillScale` at that level).
            skills::ZC_SKILL_SCALE => {
                let warned = skills::warned(packet).and_then(|warned| Some((warned, data::skill_scale(warned.skill, warned.level)?)));
                if let Some((warned, across)) = warned {
                    world::area_warned(warned.source, own, warned.cell, across, f64::from(warned.milliseconds));
                }
            }
            // A cast is broken off (the caster was struck, or moved).
            skills::ZC_DISPEL => {
                if let Some(who) = skills::dispelled(packet) {
                    world::cast_over(who, own, true);
                }
            }
            skills::ZC_NOTIFY_SKILL => self.skill_landed(packet),
            skills::ZC_USE_SKILL => self.skill_used(packet),
            // A skill could not be used: the client says why in the chat, in red, with the line of its message
            // table for the cause (`ReceiveSkillFailure`). A cause this has no line for says the plain one.
            skills::ZC_ACK_TOUSESKILL => {
                let failed = skills::failed(packet).filter(|failed| !failed.worked);
                if let Some(skills::Failed { skill, value, cause, .. }) = failed {
                    let line = crate::skills::failure(skill, value, cause).and_then(data::message);
                    // With the player's `/skillfail` off the plain failure is not told (the client's option
                    // 113 hides its lines of no particular cause); one with a cause of its own still is.
                    if (line.is_none() || cause == 0) && !self.option("TT_SKILLFAIL_ON_OFF") {
                        return true;
                    }
                    let line = line.unwrap_or_else(|| "Skill has failed.".to_owned());
                    let named = data::skill(skill).map(|(_, name, _)| name).unwrap_or_default();
                    self.chat_line_of(said::SKILL_FAILED, crate::i18n::tr(&line).replace("%s", &named), RED);
                }
            }
            // How far the weapon reaches.
            skills::ZC_ATTACK_RANGE => {
                if let Some(range) = skills::attack_range(packet) {
                    world::weapon_reaches(i32::from(range));
                }
            }
            // The blow was asked for from too far: the character goes nearer and strikes from there.
            skills::ZC_ATTACK_FAILURE_FOR_DISTANCE => {
                if let Some((target, range)) = skills::out_of_reach(packet) {
                    world::too_far(target, i32::from(range));
                }
            }
            _ => return false,
        }
        true
    }

    /// Shows a skill's effects of the client's tables, each on the one who used the skill or on its target. The
    /// effects of a skill that strikes are told of the blow (`hits`: of how many hits it is): who strikes, and
    /// the place the client's table has for the effect, the striker's or that of the one struck. Those of a
    /// skill that strikes nobody are told the other of the two (the client's `AM_SETPOS` in its handler of
    /// ZC_USE_SKILL: the place, or the ID, of the one the effect is not on).
    fn skill_shown(&self, effects: &[(bool, u32)], who: u32, target: u32, own: u32, hits: Option<u32>) {
        for &(on_target, effect) in effects {
            let whom = if on_target { target } else { who };
            let other = if on_target { who } else { target };
            let blow = match hits {
                Some(hits) => world::Blow { striker: (who != own).then_some(who), place_of: (whom != own).then_some(whom), hits },
                None => world::Blow { striker: (whom != own).then_some(whom), place_of: (other != own).then_some(other), hits: 0 },
            };
            self.effect_shown(effect, whom, own, 0.0, Some(blow));
        }
    }

    /// The sounds the client's handler of a skill starts beside its effects (`effect_tables::skill_sounds`),
    /// each heard from where the one its arm asks for stands. (The client plays them whole, wherever in sight
    /// the skill is used: here they are heard from their place, as every other sound of the world is.)
    fn skill_sounded(&self, skill: u16, when: effect_tables::Sounds, who: u32, target: u32, own: u32) {
        for (on_target, sound) in effect_tables::skill_sounds(skill, when) {
            let whom = if on_target { target } else { who };
            world::sound_on(sound, (whom != own).then_some(whom));
        }
    }

    /// What the client's handler of a skill that strikes does by itself for a monk's combo
    /// (`crate::blow::combos`), as the skill's packet is read: its effects on the striker or on the one struck
    /// (one of the striker's that goes by a place is told where the one struck stands), its shake of the
    /// screen and its sound, heard from where the striker stands (the client plays it whole).
    fn combo_shown(&self, skill: u16, who: u32, target: u32, own: u32) {
        let Some(combo) = crate::blow::combos::of_skill(skill) else { return };
        if combo.quake {
            world::quake(0);
        }
        if !combo.sound.is_empty() {
            world::sound_on(combo.sound, (who != own).then_some(who));
        }
        for &(on_struck, effect) in combo.effects {
            let told = world::Blow { striker: (who != own).then_some(who), place_of: (target != own).then_some(target), hits: 0 };
            self.effect_shown(effect, if on_struck { target } else { who }, own, 0.0, (!on_struck).then_some(told));
        }
    }

    /// Shows one of the client's effects on someone after `delay` milliseconds, with the sound it starts with.
    /// An effect that is played from files is played so; of those the client draws by code, only the ones
    /// drawn here too (`world::burst` sees to both). `blow` is what an effect launched for a blow is told: who strikes whom,
    /// and with how many hits.
    fn effect_shown(&self, effect: u32, whom: u32, own: u32, delay: f64, blow: Option<world::Blow>) {
        let on = (whom != own).then_some(whom);
        match effect {
            // `EF_TALK_FROSTJOKE`, `EF_TALK_SCREAM`: the player's own character says a line of the client's table
            // (`crate::song_talk`); the client launches them on no one else.
            crate::song_talk::EF_TALK_FROSTJOKE | crate::song_talk::EF_TALK_SCREAM => {
                if let (true, Some(line), Some(game)) = (whom == own, data::song_line(effect), &self.game) {
                    self.send_on_map(&packets::say(&game.name, &line));
                }
            }
            _ => {
                world::burst(effect, on, blow, delay);
            }
        }
    }

    /// Whether the player's own character is under a state that wards blows off: Kyrie Eleison or Parrying
    /// (the two the client asks for, `HasActiveSkillEffect(19)` and `(104)`, before it shows a monster's blow
    /// that did nothing as the guard and not as a miss).
    fn wards_blows(&self) -> bool {
        const EFST_KYRIE: u32 = 19;
        const EFST_PARRYING: u32 = 104;
        self.statuses.on.iter().any(|active| matches!(active.state, EFST_KYRIE | EFST_PARRYING))
    }

    /// ZC_NOTIFY_ACT: someone strikes someone, bends down for a thing, sits or stands (the client's
    /// `CGameActor::ProcessHitMessage`; it reads the kinds 0, 4, 8, 9, 10, 11 and 13 as blows and shows no
    /// other).
    fn act_seen(&mut self, packet: &[u8]) {
        let own = self.own_account();
        match net::actors::act(packet) {
            Some(act) if crate::blow::landing::is_blow(act.kind) => {
                // The hits it is made of, each with what it shows and sounds like as it lands (a monster's
                // own sounds are in its sprite).
                let hits = crate::blow::landing::of_act(act.kind, act.damage, i32::from(act.hits), act.extra, world::two_handed(act.source, own));
                let lands = world::struck(world::Strike {
                    source: act.source,
                    target: act.target,
                    own,
                    lasts: f64::from(act.speed),
                    flinch: f64::from(act.delay),
                    hits,
                    ring: true,
                    guarded: self.wards_blows(),
                    skill: None,
                    follow: None
                });
                // One who holds a bow sends an arrow, which arrives as the blow lands.
                world::arrow(act.source, act.target, own, lands);
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

    /// Someone starts to cast (the client's `NotifySkillCasting`). A skill with no casting time shows
    /// nothing here. The rings are those of the cast's element (`GetSkillActionInfo`).
    fn cast_begun(&mut self, packet: &[u8]) {
        if let Some(cast) = skills::casting(packet).filter(|cast| cast.milliseconds > 0) {
            let own = self.own_account();
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
            // A cast on a place, at no one: the skill's circle lies on the ground there meanwhile.
            if cast.target == 0 {
                world::scope_shown(cast.cell, cast.skill, f64::from(cast.milliseconds));
            }
        }
    }

    /// A skill lands: the caster strikes, and each of its hits shows its share of the damage, one after
    /// another.
    fn skill_landed(&mut self, packet: &[u8]) {
        // (A kind of blow the client does not show gives no hits, and nothing is shown of the skill.)
        if let Some((landed, blows)) = skills::landing(packet)
            .map(|landed| (landed, crate::blow::landing::of_skill(landed.skill, landed.action, landed.damage, i32::from(landed.hits))))
            .filter(|(_, blows)| !blows.is_empty())
        {
            let own = self.own_account();
            let from_unit = world::skill_unit_stands(landed.source);
            world::cast_over(landed.source, own, false);
            // The hits as the client counts them, and when each lands after the first.
            let hits = blows.len() as i32;
            let (each, later): (i32, Vec<f64>) = (blows[0].damage, blows.iter().map(|blow| blow.after).collect());
            let delay = f64::from(landed.attack_motion.clamp(0, 1500));
            let lands = world::struck(world::Strike {
                source: landed.source,
                target: landed.target,
                own,
                lasts: f64::from(landed.attack_motion),
                flinch: delay,
                hits: blows,
                ring: false,
                guarded: false,
                skill: Some(landed.skill),
                follow: effect_tables::skill_follow(landed.skill).filter(|_| !from_unit)
            });
            // A unit on the ground strikes (`CSkill::SendMsg`, `AM_SKILL`): the hits show the unit's own effect
            // on the one struck, and nothing of what a striker launches or what follows it.
            if from_unit {
                let struck = (landed.target != own).then_some(landed.target);
                let told = world::Blow { striker: struck, place_of: struck, hits: 0 };
                for after in later.iter().filter(|_| each > 0) {
                    if let Some(effect) = crate::skill_units::struck_by_unit(landed.skill) {
                        self.effect_shown(effect, landed.target, own, lands + after, Some(told));
                    }
                }
                return;
            }
            // What the skill looks and sounds like on the one struck (the client's `PlaySkillEffect`
            // for the bolts themselves, and the effect of each hit by the skill's element).
            let on = (landed.target != own).then_some(landed.target);
            // Lightning Bolt: the client's lightning, once.
            // (Its sound is the client's `EF_LightBolt.wav`; where the client starts it was not
            // found in its code.)
            if landed.skill == 20 {
                world::effect_later("lightning.str", on, 0.0, Some("effect/ef_lightbolt.wav"));
            } else {
                // Any other skill: what the client's table has for it (`CSession::PlaySkillEffect`), shown as
                // the skill lands, as the client does, and told who strikes whom with how many hits (Cold Bolt
                // and Fire Bolt send a bolt for each).
                self.skill_shown(&effect_tables::skill_hits(landed.skill), landed.source, landed.target, own, Some(hits as u32));
            }
            self.skill_sounded(landed.skill, effect_tables::Sounds::Strikes, landed.source, landed.target, own);
            // What the striker launches as it strikes, and what follows on the one struck as the blow
            // lands (the client's two tables, `CGameActor::ProcessSkillMessage`). The bolts of fire and
            // of lightning have their hits shown above.
            // The striker's effect is told how many hits the blow is and where the one struck stands
            // (`AM_SETCOUNT`, `AM_SETPOS`): Soul Strike sends a bolt for each hit there.
            if let Some(effect) = effect_tables::skill_launch(landed.skill) {
                let blow = world::Blow {
                    striker: (landed.source != own).then_some(landed.source),
                    place_of: (landed.target != own).then_some(landed.target),
                    hits: hits as u32
                };
                self.effect_shown(effect, landed.source, own, 0.0, Some(blow));
            }
            self.combo_shown(landed.skill, landed.source, landed.target, own);
            // What follows begins at the moment the blow lands, with its number (the client hands both to the one
            // struck in one message, `AM_WILL_BE_ATTACKED`, for one time): so Frost Diver's ice round the one
            // struck, `EF_FROSTDIVER2`, and its sound `EF_FrostDiver2.wav` come then. (The client plays
            // `EF_FrostDiver.wav` for other skills only; for a mage's Frost Diver nowhere.)
            if let Some(effect) = effect_tables::skill_follow(landed.skill).filter(|_| !matches!(landed.skill, 19 | 20)) {
                // The client tells what follows that it is of one hit (`AM_SETCOUNT`), and `EF_ENERGYDRAIN` where
                // the striker stands (`AM_SETPOS`): told here to the two effects drawn that go by either, Blitz
                // Beat's and the drain.
                let from = if effect == 217 { landed.source } else { landed.target };
                let told = world::Blow { striker: (landed.target != own).then_some(landed.target), place_of: (from != own).then_some(from), hits: 1 };
                // (Any other is told the striker's place and no count.)
                let back = world::Blow { place_of: (landed.source != own).then_some(landed.source), hits: 0, ..told };
                self.effect_shown(effect, landed.target, own, lands, Some(if matches!(effect, 115 | 217) { told } else { back }));
            }
            // The effect of each hit that hurts, which the client picks by the skill (the `hitEffect` of that
            // message, which every hit of the skill repeats 200 milliseconds after the one before): the ring of
            // a plain blow for most, the hit of fire, cold or lightning for the bolts, none for some.
            for after in later {
                let when = lands + after;
                if let Some(effect) = effect_tables::skill_struck(landed.skill).filter(|_| each > 0) {
                    // It is on the one struck and told where the striker stands.
                    let told = world::Blow {
                        striker: (landed.target != own).then_some(landed.target),
                        place_of: (landed.source != own).then_some(landed.source),
                        hits: 0
                    };
                    self.effect_shown(effect, landed.target, own, when, Some(told));
                }
            }
        }
    }

    fn skill_used(&mut self, packet: &[u8]) {
        if let Some(skills::Used { skill, level, target, who, worked }) = skills::used(packet) {
            let own = self.own_account();
            // Chase Walk (`ST_CHASEWALK`, `GameModePackets.cpp:18990`): the one who used it puffs smoke, as one
            // that hides does; the client does so only for a skill that worked.
            if skill == 389 && worked {
                world::puffed((who != own).then_some(who));
            }
            world::cast_over(who, own, false);
            // The one who used it goes through the motion the client has for that skill.
            world::uses_skill(who, own, effect_tables::skill_state(skill));
            // The effects the client shows for that skill (`ReceiveSkillUse`), on the one who used it or
            // on its target, each with the sound it starts with. An effect that is a `.str` file is
            // played from the file; of those the client draws by code, the ones drawn here too.
            self.skill_shown(&effect_tables::skill_effects(skill), who, target, own, None);
            self.skill_sounded(skill, effect_tables::Sounds::Used, who, target, own);
            // Heal and its like (`AL_HEAL`, `WE_MALE`, `NPC_CHEAL`, `AB_CHEAL`, `AB_HIGHNESSHEAL`, `HLIF_HEAL`):
            // the packet's level is how much was healed, and the client picks the effect on the one healed by
            // it, in its handler of the packet (`Client/Mode/GameModePackets.cpp`): `EF_HEAL4` from 4000,
            // `EF_HEAL2` from 2000, `EF_HEAL` from 200, `EF_EXIT` (7) under that.
            if matches!(skill, 28 | 334 | 729 | 2043 | 2051 | 8001) {
                let effect = match level {
                    4000.. => 325,
                    2000.. => 313,
                    200.. => 312,
                    _ => 7,
                };
                self.effect_shown(effect, target, own, 0.0, None);
                // How much was healed rises over the one healed, in green.
                world::number_over(target, own, level, crate::blow::signs::GREEN);
            }
            // Potion Pitcher (`AM_POTIONPITCHER`: `LaunchTargetEffect(0x12B, source, target, effectLevel)`): the
            // one who used it throws a potion at the target (`EF_THROWITEM2`), in the colour of the packet's
            // level.
            if skill == 231 {
                let thrown = world::Blow { striker: (who != own).then_some(who), place_of: (target != own).then_some(target), hits: level.max(0) as u32 };
                self.effect_shown(299, who, own, 0.0, Some(thrown));
            }
            // Spirit Recovery and the wife's gift of spirit (`MG_SRECOVERY`, `WE_FEMALE`) show the spirit
            // given, in blue, the same way.
            if matches!(skill, 9 | 335) {
                world::number_over(target, own, level, crate::blow::signs::BLUE);
            }
        }
    }
}
