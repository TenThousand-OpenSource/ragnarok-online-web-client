//! What a recording keeps of the session and of the map as it begins (`CGameMode::SendMsg` at
//! `MM_START_REPLAY_RECORDING`: the sections 3 to 24, then `Replay::RecordCachedPackets`).
//!
//! Left as a new session's, there being no such thing in this client or nothing kept of it: the memorial
//! dungeon and the battlegrounds' queue (10, 12: not kept past their windows), the cash shop's lists (14),
//! the windows open (19), the navigation (24), the flag `isOnQuest` (23, which the decompiled client does
//! not explain), the guilds' emblems as files (22), an elemental (9); of an actor, its robe, its third
//! option and a player's character number; of an item, what it looks like worn; of a status, its three
//! values; of a skill, its name in the client's files.

use super::*;
use crate::replay::sections::{self, Looks};
use crate::replay::snapshot::{self, actors, companions, ground, people, screen, session, skills, Snapshot};

/// What was gathered: the client's sections, the packets that stand at the head of the recorded ones, and
/// which way the character faces, for the header.
pub(in crate::web::app) struct Taken {
    pub client: Vec<(usize, Vec<u8>)>,
    pub head: Vec<(u32, Vec<u8>)>,
    pub direction: i32,
}

impl App {
    /// Everything a recording that begins now keeps besides its header.
    pub(in crate::web::app) fn replay_gathered(&self) -> Taken {
        let sight = world::snapshot();
        let figure = self.own_figure();
        let [lower, upper, middle] = figure.headgear.map(u32::from);
        let looks = Looks {
            account: self.own_account(),
            character: self.own_character(),
            job: figure.job.into(),
            head: figure.hair_style.into(),
            weapon: figure.weapon,
            shield: figure.shield,
            head_colour: figure.hair_colour.into(),
            worn: [lower, upper, middle],
            clothes_colour: figure.body_palette.into(),
            body_state: self.statuses.body.into(),
            effect_state: self.hero.state,
        };
        let mut client = vec![
            (sections::CHARACTER, sections::character(&looks, &self.hero.params)),
            (sections::STATS, sections::stats(&self.hero.stat_bonus, &self.hero.params)),
            (session::OWN, session::own(figure.sex)),
        ];
        client.extend(snapshot::sections(&self.replay_snapshot(&sight)));
        Taken { client, head: self.replay_head(&sight), direction: sight.direction.into() }
    }

    /// The sections 6 to 24.
    fn replay_snapshot(&self, sight: &world::Snapshot) -> Snapshot {
        let kept = |item: &Item| session::item(item, data::item_title(&item.into(), false).as_bytes());
        let list = |items: &[Item]| items.iter().map(kept).collect::<Vec<_>>();
        let stored = self.storage.open.as_ref().map_or(Vec::new(), |storage| list(&storage.items));
        let now = now();
        let icon = |active: &Active| {
            let left = active.end.map_or(0, |end| end.saturating_sub(now).max(1));
            screen::StatusIcon { id: active.state, expires: left, values: [0; 3], total: active.total, remaining: left }
        };
        let own = self.own_account();
        Snapshot {
            quests: self.quests.log.values().map(session::quest).collect(),
            messenger: people::Messenger {
                leader: self.party.members.iter().any(|member| member.leader && member.account == own).into(),
                party_name: self.party.name.as_bytes().to_vec(),
                party: self.party.members.iter().map(session::member).collect(),
                friends: self.friends.list.iter().map(session::friend).collect(),
            },
            items: session::lists(list(&self.bag.items), list(&self.cart.items), stored),
            companions: self.replay_companions(),
            shortcuts: snapshot::rest::Shortcuts { slots: session::slots(&self.skills.hotkeys), ..Default::default() },
            actors: self.replay_actors(sight),
            ground_items: sight.drops.iter().map(|&(id, item, identified, count, (x, y), within)| {
                ground::GroundItem { id, item, identified, x: x as i16, y: y as i16, count: count.min(i16::MAX as u16) as i16, within }
            }).collect(),
            ground_skills: ground::GroundSkills { units: sight.skill_units.iter().filter_map(skill_unit).collect(), worded: Vec::new() },
            icons: self.statuses.on.iter().map(icon).collect(),
            guild: self.replay_names(sight),
            ..Snapshot::default()
        }
    }

    /// The homunculus, the mercenary and the pet (section 9).
    fn replay_companions(&self) -> companions::Companions {
        let mut kept = companions::Companions::default();
        if let Some(info) = &self.homunculus.info {
            let whole = |value: i64| value.clamp(0, i64::from(i32::MAX)) as i32;
            kept.homunculus = companions::Homunculus {
                id: self.homunculus.unit,
                name: info.name.as_bytes().to_vec(),
                stats: info.stats,
                job: self.homunculus.job.into(),
                level: info.level.into(),
                hp: whole(info.hp),
                max_hp: whole(info.max_hp),
                sp: whole(info.sp),
                max_sp: whole(info.max_sp),
                friendly: info.intimacy,
                accessory: 0,
                exp: info.exp,
                max_exp: info.next_exp,
                hunger: info.hunger,
                max_hunger: 100,
                renamed: (info.flags & 1).into(),
                alive: self.homunculus.out.into(),
                range: info.range.into(),
            };
        }
        if let Some(info) = &self.mercenary.info {
            kept.mercenary = companions::Mercenary {
                id: info.unit,
                name: info.name.as_bytes().to_vec(),
                stats: info.stats,
                hp: info.hp,
                max_hp: info.max_hp,
                job: self.mercenary.job.into(),
                level: info.level.into(),
                sp: info.sp,
                max_sp: info.max_sp,
                loyalty: info.faith,
                summons: info.calls,
                kills: info.kills,
                range: info.range.into(),
                expires: info.expires,
                alive: self.mercenary.out.into(),
            };
        }
        if let Some(pet) = &self.pets.pet {
            kept.pet = companions::Pet {
                id: self.pets.unit,
                renamed: pet.renamed.into(),
                name: pet.name.as_bytes().to_vec(),
                accessory: pet.accessory.into(),
                job: pet.job.into(),
                level: pet.level.into(),
                hunger: pet.hunger,
                friendly: pet.intimacy,
                egg: 0,
                hunger_before: self.pets.hunger_before,
            };
        }
        kept
    }

    /// Everyone in sight as one who stands where it is now, each with the signs over it (section 15: a
    /// chat room's, a stall's), and the portals.
    fn replay_actors(&self, sight: &world::Snapshot) -> Vec<actors::ActorChunk> {
        let rooms = Some(&self.chat_rooms.rooms.1).filter(|_| self.chat_rooms.rooms.0 == world::visit());
        let mut chunks = Vec::new();
        for seen in &sight.units {
            // One that walks goes on from the cell it has come to, setting out now (the client writes the
            // time its walk began with the place it is at by now).
            let standing = self.replay_standing(seen);
            chunks.push(seen.walks_to.map_or_else(|| actors::stands(&standing), |to| actors::walks(&standing, to, sight.server_time)));
            if let Some(room) = rooms.and_then(|rooms| rooms.values().find(|room| room.owner == seen.id)) {
                chunks.push(actors::room(room.owner, room.id, room.limit, room.users, room.kind, room.title.as_bytes()));
            }
            if let Some(title) = self.vending.signs.get(&seen.id) {
                chunks.push(actors::sign(false, seen.id, title.as_bytes()));
            }
        }
        // A portal is an NPC of the job 45 to the server, which shows it as any other.
        let portal = |&(id, (x, y)): &(u32, (u16, u16))| {
            actors::Standing { kind: 6, id, job: 45, speed: 200, x, y, size: [5, 5], max_hp: -1, hp: -1, ..Default::default() }
        };
        chunks.extend(sight.portals.iter().map(|one| actors::stands(&portal(one))));
        chunks
    }

    /// One in sight as the packet that shows it standing tells of it (`BuildActorEntryPacket`).
    fn replay_standing(&self, seen: &world::InSight) -> actors::Standing {
        use crate::net::actors::Kind;
        let kind = match seen.kind {
            Kind::Player => 0,
            Kind::Monster => 5,
            Kind::Npc => 6,
            // (The server's 7 to 10: a pet, a homunculus, a mercenary, an elemental. The world keeps them as
            // one kind; the player's own two are told apart here, anyone else's is written as a pet.)
            Kind::Companion if seen.id == self.homunculus.unit => 8,
            Kind::Companion if seen.id == self.mercenary.unit() => 9,
            Kind::Companion => 7,
        };
        let (hp, max_hp) = seen.life.unwrap_or((-1, -1));
        actors::Standing {
            kind,
            id: seen.id,
            speed: seen.speed,
            body_state: seen.body,
            health_state: seen.health,
            effect_state: seen.state,
            job: seen.job,
            hair: seen.hair_style,
            weapon: seen.weapon,
            shield: seen.shield,
            headgear: seen.headgear,
            hair_colour: seen.hair_colour,
            clothes_colour: seen.body_palette,
            head_direction: seen.head.into(),
            guild: seen.guild.0,
            emblem: seen.guild.1 as u16,
            sex: seen.sex,
            x: seen.cell.0,
            y: seen.cell.1,
            direction: seen.direction,
            size: [5, 5],
            state: if seen.dead { 1 } else if seen.sitting { 2 } else { 0 },
            level: seen.level,
            max_hp,
            hp,
            boss: seen.boss,
            name: seen.name.as_bytes().to_vec(),
            ..actors::Standing::default()
        }
    }

    /// The names over the players in sight and the character's own guild (section 22).
    fn replay_names(&self, sight: &world::Snapshot) -> people::Guild {
        let players = sight.units.iter().filter(|seen| seen.kind == crate::net::actors::Kind::Player && !seen.name.is_empty());
        let card = |seen: &world::InSight| people::NameCard {
            id: seen.id,
            name: seen.name.as_bytes().to_vec(),
            party: seen.party_name.as_bytes().to_vec(),
            guild: seen.guild_name.as_bytes().to_vec(),
            clan: Vec::new(),
        };
        let own = people::NameCard {
            id: self.own_account(),
            name: self.game.as_ref().map_or(Vec::new(), |game| game.name.as_bytes().to_vec()),
            party: self.party.name.as_bytes().to_vec(),
            guild: self.guild.name.as_bytes().to_vec(),
            clan: Vec::new(),
        };
        people::Guild {
            cards: players.map(card).chain([own]).collect(),
            emblems: Vec::new(),
            guild: self.guild.id as i32,
            emblem_version: self.guild.emblem as i32,
        }
    }

    /// What stands at the head of the recorded packets (`Replay::RecordCachedPackets`): the map's kind as it
    /// was last told, and the three lists of skills. After them, this project's own: the server's time
    /// (ZC_NOTIFY_TIME), so that the clock everyone's walk is shown by is set as the replay begins and not at
    /// the first walk told (the client's files have no such packet there).
    fn replay_head(&self, sight: &world::Snapshot) -> Vec<(u32, Vec<u8>)> {
        let listed = |known: &[Skill]| -> Vec<skills::Listed> {
            let one = |skill: &Skill| skills::Listed {
                id: skill.id,
                target: skill.target,
                level: skill.level,
                sp: skill.sp,
                range: skill.range,
                name: Vec::new(),
                raisable: skill.raisable,
            };
            known.iter().map(one).collect()
        };
        let [property, kind] = &self.replay.map_told;
        let mut head = skills::head(
            [property.as_deref(), kind.as_deref()],
            &listed(&self.skills.known),
            &listed(&self.homunculus.skills),
            &listed(&self.mercenary.skills)
        );
        head.push((0, [&packets::ZC_NOTIFY_TIME.to_le_bytes()[..], &sight.server_time.to_le_bytes()].concat()));
        head
    }
}

/// A skill's unit as section 17 keeps one: its kind in a byte, as the packet of the first clients has it.
/// Who made it the world does not keep.
fn skill_unit(&(id, (x, y), unit, visible): &(u32, (u16, u16), u32, bool)) -> Option<ground::GroundSkill> {
    Some(ground::GroundSkill { id, creator: 0, x: x as i16, y: y as i16, kind: u8::try_from(unit).ok()?, visible })
}
