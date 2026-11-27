//! A replay begins from what its file keeps of the session and of the map, as the Windows client's does
//! (`LoginMode.cpp` at the arm that begins a replay: the header's name, map and cell and section 3; then
//! `CGameMode::Initialize` with the replay's state 6: the sections 18, 6, 4, 8, 9, 7, 10 to 14, 22, 15, 16,
//! 17, 23, 24, 20, 19, 21). The session's part is set as the game is entered; the map's part (those in
//! sight, the ground, the signs, the states and their effects) once the map shows, and the recording's
//! clock begins then.
//!
//! Read and not shown, there being nothing here to show it in: the storage's list (8, without its window),
//! the other lists and rows of items, the memorial dungeon, the queue, the shops (10, 12, 14), the windows
//! (19: none is opened), the setting and the navigation (20, 24), the emblems' files and the flag of 23, an
//! elemental, the two quick places of the bar, a bar that fills over someone and a Madogear.

use super::*;
use crate::replay::snapshot::{self, actors, companions, ground, session, Snapshot};
use crate::replay::File;

impl App {
    /// The game is entered as the character of a file that keeps it as the Windows client does, with no
    /// server behind it, and the session is set from the file. Whether the game was entered.
    pub(in crate::web::app) fn replay_entered(&mut self, file: &File, character: &session::Character) -> bool {
        self.packet(packets::ZC_ACCEPT_ENTER, &session::entering(&file.header, session::server_time(file)));
        if !matches!(self.screen, Screen::Game) {
            return false;
        }
        let snapshot = snapshot::of_file(file);
        // Sections 3 and 4: the character's values, what it holds and the state it is in.
        self.hero.params.extend(character.values.iter().map(|(number, value)| (*number, *value)));
        (self.hero.stat_bonus, self.hero.state) = (character.bonus, character.looks.effect_state);
        world::stated(None, character.looks.effect_state, None);
        world::looked(None, 2, character.looks.weapon, character.looks.shield);
        // Section 8: what is carried and worn, and the cart.
        self.bag.items = session::carried(&snapshot.items);
        self.cart.items = snapshot.items.cart.iter().filter_map(session::item_back).collect();
        // Section 13: the bar (this client's has two places more, which the server keeps and nobody sees).
        self.skills.hotkeys = session::hotkeys(&snapshot.shortcuts.slots);
        self.skills.hotkeys.resize(net::inventory::HOTKEYS, Hotkey::default());
        for skill in self.skills.hotkeys.iter().filter(|hotkey| hotkey.skill && hotkey.id != 0).map(|hotkey| hotkey.id as u16) {
            data::want_skill(skill, || dispatch(Action::ItemsKnown));
        }
        // Sections 7 and 6: the party, the friends, the journal.
        self.party.name = net::bytes::text(&snapshot.messenger.party_name);
        self.party.members = snapshot.messenger.party.iter().filter(|member| member.valid != 0).map(session::member_back).collect();
        // (The client keeps neither the job nor the level of those of the party in a file; the character's
        // own are known all the same.)
        let (own, who) = (self.own_account(), &character.who);
        if let Some(member) = self.party.members.iter_mut().find(|member| member.account == own) {
            (member.job, member.level) = (who.job, who.level);
        }
        self.friends.list = snapshot.messenger.friends.iter().filter(|member| member.valid != 0).map(session::friend_back).collect();
        self.quests.log = snapshot.quests.iter().map(|quest| (quest.id, session::quest_back(quest))).collect();
        self.replay_companions_given(&snapshot.companions);
        // Section 22: the character's own guild (`SetGuildInfo`).
        let (guild, emblem) = (snapshot.guild.guild.max(0) as u32, snapshot.guild.emblem_version.max(0) as u32);
        world::own_guild(self.own_account(), &file.header.name, guild, emblem);
        self.items_changed();
        self.refresh_values();
        true
    }

    /// Section 9: the homunculus, the mercenary, the pet, each if the file has one.
    fn replay_companions_given(&mut self, kept: &companions::Companions) {
        let (homunculus, mercenary, pet) = (&kept.homunculus, &kept.mercenary, &kept.pet);
        if homunculus.id != 0 {
            let info = net::homunculus::Homunculus {
                name: net::bytes::text(&homunculus.name),
                flags: u8::from(homunculus.renamed != 0) | if homunculus.alive != 0 { 4 } else { 0 },
                level: homunculus.level.clamp(0, 0xffff) as u16,
                hunger: homunculus.hunger,
                intimacy: homunculus.friendly,
                stats: homunculus.stats,
                hp: homunculus.hp.into(),
                max_hp: homunculus.max_hp.into(),
                sp: homunculus.sp.into(),
                max_sp: homunculus.max_sp.into(),
                exp: homunculus.exp,
                next_exp: homunculus.max_exp,
                skill_points: 0,
                range: homunculus.range.clamp(0, 0xffff) as u16,
            };
            let job = homunculus.job.clamp(0, 0xffff) as u16;
            (self.homunculus.unit, self.homunculus.out, self.homunculus.job, self.homunculus.info) = (homunculus.id, homunculus.alive != 0, job, Some(info));
        }
        if mercenary.id != 0 {
            let info = net::mercenary::Mercenary {
                unit: mercenary.id,
                stats: mercenary.stats,
                name: net::bytes::text(&mercenary.name),
                level: mercenary.level.clamp(0, 0xffff) as u16,
                hp: mercenary.hp,
                max_hp: mercenary.max_hp,
                sp: mercenary.sp,
                max_sp: mercenary.max_sp,
                expires: mercenary.expires,
                faith: mercenary.loyalty,
                calls: mercenary.summons,
                kills: mercenary.kills,
                range: mercenary.range.clamp(0, 0xffff) as u16,
            };
            let job = mercenary.job.clamp(0, 0xffff) as u16;
            (self.mercenary.out, self.mercenary.job, self.mercenary.info) = (mercenary.alive != 0, job, Some(info));
        }
        if pet.id != 0 {
            let whole = |value: i32| value.clamp(0, 0xffff) as u16;
            let info = net::pets::Pet {
                name: net::bytes::text(&pet.name),
                renamed: pet.renamed != 0,
                level: whole(pet.level),
                hunger: pet.hunger,
                intimacy: pet.friendly,
                accessory: whole(pet.accessory),
                job: whole(pet.job),
            };
            (self.pets.unit, self.pets.hunger_before, self.pets.pet) = (pet.id, pet.hunger_before, Some(info));
        }
    }

    /// The map of a replay that begins from its file's sections shows: what the file keeps of the map is
    /// put on it, each thing as the packet it is kept as, handed to the reader of that packet (the client's
    /// `ReadReplayActors`, `ReadReplayGroundItems`, `ReadReplayGroundSkills`, and the states it registers
    /// before its world is made); then the recording's clock begins.
    pub(in crate::web::app) fn replay_map_shown(&mut self) {
        let Some(playing) = self.replay.playing.as_mut().filter(|playing| playing.waits) else { return };
        (playing.waits, playing.counted) = (false, crate::web::now());
        let file = playing.file.clone();
        let snapshot = snapshot::of_file(&file);
        // The server's clock is set anew to the time the recording began at: the map took its time to
        // show, and those written as walking set out at that time.
        if let Some(time) = Some(session::server_time(&file)).filter(|time| *time != 0) {
            world::clock_set(time);
        }
        self.replay_map_given(&snapshot);
        after(REPLAY_TICK, Action::ReplayTick);
    }

    fn replay_map_given(&mut self, snapshot: &Snapshot) {
        let own = self.own_account();
        // The names told of the players (`CActorNameCardHolder::Load`): those of the party and of the guild
        // are written under a player's name.
        for card in &snapshot.guild.cards {
            let (party, guild) = (net::bytes::text(&card.party), net::bytes::text(&card.guild));
            if !party.is_empty() || !guild.is_empty() {
                world::player_card(card.id, party, guild, String::new());
            }
        }
        for packet in snapshot.actors.iter().filter_map(actors::packet) {
            self.replay_given(&packet);
        }
        for item in &snapshot.ground_items {
            self.replay_given(&ground::item_entry(item));
        }
        for unit in &snapshot.ground_skills.units {
            self.replay_given(&ground::skill_entry(unit));
        }
        // (A unit that carries words is the packet as the file has it, if it is whole.)
        for unit in snapshot.ground_skills.worded.iter().filter(|unit| unit.len() >= 97) {
            let mut packet = unit.clone();
            packet[..2].copy_from_slice(&net::skill_units::ZC_SKILL_ENTRY2.to_le_bytes());
            self.replay_given(&packet);
        }
        for icon in &snapshot.icons {
            self.replay_given(&session::status(own, icon));
        }
    }

    /// A packet made of what a file keeps is handed to the game as the map server's.
    fn replay_given(&mut self, packet: &[u8]) {
        if let Some(id) = crate::replay::packet_id(packet) {
            self.packet(id, packet);
        }
    }
}
