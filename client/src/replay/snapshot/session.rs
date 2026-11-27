//! Between what this client keeps of a session, as its packets told it (`crate::net`), and what the Windows
//! client keeps of the same in a replay file: an item and where a worn one stands among the ten of
//! `equippedItems`, a place of the shortcut bar, a quest and its time, one of the party and a friend; who the
//! character of a file is (sections 3, 4 and this project's one record in 11) and the packet that puts it
//! into the game; a status as the packet that tells of one.
//!
//! Texts: this client keeps the names the server sent as it read them (UTF-8, which on this server is
//! ASCII), and writes those bytes; nothing is turned into the client's code page 949.

use std::collections::HashMap;

use super::{items, people, quests, rest, screen};
use crate::net::inventory::{Hotkey, Item};
use crate::net::{friends, party, quests as told};
use crate::replay::{sections, tagged, File, Who};

/// An item of this client as the Windows client's lists keep one. `name` is what it is shown as; what it
/// looks like when worn (`sprite`) this client does not keep with the item.
pub fn item(item: &Item, name: &[u8]) -> items::Item {
    items::Item {
        kind: item.kind.into(),
        index: item.index.into(),
        location: item.wearable,
        worn: item.worn,
        quantity: item.amount.into(),
        cards: item.cards,
        id: item.id as i32,
        name: name.to_vec(),
        identified: item.identified,
        refine: item.refine.into(),
        ..items::Item::default()
    }
}

/// The same back; none for an empty place of a row of ten (no item's number).
pub fn item_back(item: &items::Item) -> Option<Item> {
    (item.id > 0).then(|| Item {
        index: item.index as u16,
        id: item.id as u32,
        kind: item.kind as u8,
        amount: item.quantity.clamp(0, i32::from(u16::MAX)) as u16,
        wearable: item.location,
        worn: item.worn,
        refine: item.refine.clamp(0, 255) as u8,
        identified: item.identified,
        cards: item.cards,
    })
}

/// Where among the ten of `equippedItems` a thing worn at those places stands (the lookups of `CSession`,
/// `Client/Session/Session.cpp`: 0 the lower headgear, 1 the right hand or both, 2 the garment, 3 an
/// accessory, 4 the armour, 5 the left hand, 6 the shoes, 7 the other accessory, 8 the upper headgear and
/// what covers more with it, 9 the middle one alone or with the lower). Which of the two accessories is
/// which was not found in the client: the right one (8) is put at 3 here, the left (128) at 7. None for what
/// is worn elsewhere (ammunition, a costume, a shadow thing) or not at all.
pub fn place_worn(worn: u32) -> Option<usize> {
    let at = |bit: u32| worn & bit != 0;
    Some(match worn {
        0 | 0x400.. => return None,
        _ if at(256) => 8,
        _ if at(512) => 9,
        _ if at(1) => 0,
        _ if at(2) => 1,
        _ if at(32) => 5,
        _ if at(4) => 2,
        _ if at(8) => 3,
        _ if at(128) => 7,
        _ if at(16) => 4,
        _ => 6,
    })
}

/// The lists of section 8 out of what this client carries, has in its cart and in an open storage: what is
/// worn at one of the ten places stands there (`equippedItems`) and not in the inventory's list, as the
/// client keeps it; what is worn elsewhere (ammunition, a costume) stays in the list with where it is worn.
pub fn lists(inventory: Vec<items::Item>, cart: Vec<items::Item>, storage: Vec<items::Item>) -> items::Items {
    let mut lists = items::Items { cart, storage, ..items::Items::default() };
    for item in inventory {
        match place_worn(item.worn).filter(|place| lists.equipped[*place].id == 0) {
            Some(place) => lists.equipped[place] = item,
            None => lists.inventory.push(item),
        }
    }
    lists
}

/// What the character carries, out of section 8: the inventory's list and, after it, what is worn (the ten
/// and the costumes' ten, 4603), each with where it is worn as its record says.
pub fn carried(lists: &items::Items) -> Vec<Item> {
    let costumes = lists.rows.iter().filter(|(tag, _)| *tag == 4603).flat_map(|(_, row)| row);
    lists.inventory.iter().chain(&lists.equipped).chain(costumes).filter_map(item_back).collect()
}

/// The 36 places of the shortcut bar (`m_shortcutSlots`) out of this client's first bar.
pub fn slots(hotkeys: &[Hotkey]) -> [rest::Slot; 36] {
    let mut slots = [rest::Slot::default(); 36];
    for (slot, hotkey) in slots.iter_mut().zip(hotkeys) {
        *slot = rest::Slot { kind: hotkey.skill.into(), id: hotkey.id, value: hotkey.count as i16 };
    }
    slots
}

/// The same back: the 36, an empty one for a place that holds nothing.
pub fn hotkeys(slots: &[rest::Slot; 36]) -> Vec<Hotkey> {
    slots.iter().map(|slot| Hotkey { skill: slot.kind != 0 && slot.id != 0, id: slot.id, count: slot.value.max(0) as u16 }).collect()
}

/// The days from 1970 of a date, and the date of so many days (the usual count of the civil calendar).
fn days_of(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year - era * 400;
    let of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    era * 146_097 + of_era * 365 + of_era / 4 - of_era / 100 + of_year - 719_468
}

fn date_of(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let of_era = shifted - era * 146_097;
    let year = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year + year / 4 - year / 100);
    let shifted_month = (5 * of_year + 2) / 153;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    (year + era * 400 + i64::from(month <= 2), month, of_year - (153 * shifted_month + 2) / 5 + 1)
}

/// A quest of this client's journal as the Windows client keeps one: its time as a date (the year, the
/// month, the day, the hour, the minute; zeros for one that does not run out). The client writes the date by
/// its own clock; here it is the date in UTC, which was not found to matter to anything this client shows.
pub fn quest(quest: &told::Quest) -> quests::Quest {
    let when = match i64::from(quest.deadline) {
        0 => [0; 5],
        seconds => {
            let (year, month, day) = date_of(seconds / 86_400);
            [year, month, day, seconds % 86_400 / 3600, seconds % 3600 / 60].map(|part| part as i16)
        }
    };
    let hunt = |hunt: &told::Hunt| quests::Hunt {
        id: hunt.id as i32,
        condition: hunt.condition as i32,
        monster: hunt.mob as i32,
        min_level: hunt.levels.0,
        max_level: hunt.levels.1,
        killed: hunt.killed,
        goal: hunt.goal,
        name: hunt.name.as_bytes().to_vec(),
    };
    quests::Quest { id: quest.id, state: quest.active.into(), when, hunts: quest.hunts.iter().map(hunt).collect() }
}

/// The same back: the seconds are those of the date's minute.
pub fn quest_back(quest: &quests::Quest) -> told::Quest {
    let [year, month, day, hour, minute] = quest.when.map(i64::from);
    let deadline = if quest.when == [0; 5] || !(1..=12).contains(&month) {
        0
    } else {
        (days_of(year, month, day) * 86_400 + hour * 3600 + minute * 60).clamp(0, i64::from(u32::MAX)) as u32
    };
    let hunt = |hunt: &quests::Hunt| told::Hunt {
        id: hunt.id as u32,
        condition: hunt.condition as u32,
        mob: hunt.monster as u32,
        levels: (hunt.min_level, hunt.max_level),
        killed: hunt.killed,
        goal: hunt.goal,
        name: crate::net::bytes::text(&hunt.name),
    };
    told::Quest { id: quest.id, active: quest.state != 0, deadline, hunts: quest.hunts.iter().map(hunt).collect() }
}

/// One of the party as the Windows client keeps one (its job and its level are not kept by it).
pub fn member(member: &party::Member) -> people::Member {
    people::Member {
        valid: 1,
        account: member.account,
        character: member.character,
        leader: member.leader.into(),
        offline: member.offline.into(),
        colour: 0,
        name: member.name.as_bytes().to_vec(),
        map: member.map.as_bytes().to_vec(),
    }
}

pub fn member_back(member: &people::Member) -> party::Member {
    party::Member {
        account: member.account,
        character: member.character,
        name: crate::net::bytes::text(&member.name),
        map: crate::net::bytes::text(&member.map),
        leader: member.leader != 0,
        offline: member.offline != 0,
        ..party::Member::default()
    }
}

/// A friend likewise.
pub fn friend(friend: &friends::Friend) -> people::Member {
    let (account, character, name) = (friend.account, friend.character, friend.name.as_bytes().to_vec());
    people::Member { valid: 1, account, character, offline: friend.offline.into(), name, ..people::Member::default() }
}

pub fn friend_back(member: &people::Member) -> friends::Friend {
    let name = crate::net::bytes::text(&member.name);
    friends::Friend { account: member.account, character: member.character, name, offline: member.offline != 0 }
}

/// The number of the section the Windows client leaves empty (`CSession::BuildReservedReplayRecords` writes
/// nothing into 11, `ReadReplayReserved` reads nothing of it), in which this client keeps the one thing of
/// the character it needs that no section of the client's has: its sex (the client's session has it from
/// the login, which a replay does not go through).
pub const OWN: usize = 11;

/// That section: the sex (9014, a byte) between 9000 and 9001. This project's own.
pub fn own(sex: u8) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 9000, &[]);
    tagged(&mut section, 9014, &[sex]);
    tagged(&mut section, 9001, &[]);
    section
}

fn own_read(section: &[u8]) -> Option<u8> {
    super::opened(section, 9000)?.into_iter().find(|(tag, _)| *tag == 9014).map(|(_, bytes)| super::u8_of(bytes))
}

/// The character of a file that keeps it as the Windows client does.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Character {
    pub who: Who,
    pub looks: sections::Looks,
    /// Its values by the server's numbers, and the bonuses of its six stats.
    pub values: HashMap<u16, i64>,
    pub bonus: [i32; 6],
}

/// Who the character of a file is, by its sections 3 and 4 (`sections::told`); none for a file without the
/// third. The sex is this client's own record of section 11: male for a file that has none (one of the
/// Windows client).
pub fn character(file: &File) -> Option<Character> {
    let (looks, values, bonus) = sections::told(file.section(sections::CHARACTER), file.section(sections::STATS))?;
    let value = |number: u16| values.get(&number).copied().unwrap_or(0);
    let sex = own_read(file.section(OWN)).unwrap_or(1);
    let [lower, upper, middle] = looks.worn.map(|worn| worn as u16);
    let who = Who {
        character: looks.character,
        job: looks.job as u16,
        level: value(11) as u16,
        job_level: value(55) as u32,
        zeny: value(20).clamp(0, i64::from(u32::MAX)) as u32,
        look: [looks.head as u16, looks.head_colour as u16, looks.clothes_colour as u16, lower, upper, middle, sex.into()],
        stats: [13, 14, 15, 16, 17, 18].map(|stat| value(stat).clamp(0, 255) as u8),
    };
    Some(Character { who, looks, values, bonus })
}

/// The server's time as the head of a file's packets tells it (ZC_NOTIFY_TIME at the tick 0, which this
/// client writes there: `skills`); nought for a file that has none there (one of the Windows client).
pub fn server_time(file: &File) -> u32 {
    let told = |packet: &[u8]| (crate::replay::packet_id(packet) == Some(crate::net::packets::ZC_NOTIFY_TIME)).then(|| super::u32_of(&packet[2..]));
    file.packets.iter().take_while(|(tick, _)| *tick == 0).find_map(|(_, packet)| told(packet)).unwrap_or(0)
}

/// The packet that puts the character of a file into the game with no server behind it, as ZC_ACCEPT_ENTER
/// (0x02EB, 13 bytes) would: the server's time (`server_time`: the clock everyone's walk is shown by is set
/// from it), the cell and the way faced of the file's header, the character's size (5 by 5, as the server
/// sends it) and no font. The client itself has no such packet in a file: it sets its session's place from
/// the header (`LoginMode.cpp`, the arm that begins a replay).
pub fn entering(header: &crate::replay::Header, time: u32) -> Vec<u8> {
    let cell = |value: i32| value.clamp(0, 1023) as u16;
    let place = super::actors::place(cell(header.x), cell(header.y), header.direction.rem_euclid(8) as u8);
    [&crate::net::packets::ZC_ACCEPT_ENTER.to_le_bytes()[..], &time.to_le_bytes(), &place, &[5, 5], &[0, 0]].concat()
}

/// A status of section 18 as the packet that tells of one going on with its times (ZC_MSG_STATE_CHANGE3,
/// 0x0983, 29 bytes), on that one: the time left is what the section kept (`expires`, the milliseconds left
/// when the recording began), and one that does not end (0 or 999999 there) has the server's 9999.
pub fn status(who: u32, icon: &screen::StatusIcon) -> Vec<u8> {
    let endless = matches!(icon.expires, 0 | 999_999);
    let (total, remaining) = if endless { (icon.total, crate::net::statuses::NO_TIME) } else { (icon.total, icon.expires) };
    let numbers = [total, remaining, icon.values[0], icon.values[1], icon.values[2]];
    let begun = [&crate::net::statuses::ZC_MSG_STATE_CHANGE3.to_le_bytes()[..], &(icon.id as u16).to_le_bytes(), &who.to_le_bytes(), &[1]];
    [begun.concat(), numbers.iter().flat_map(|number| number.to_le_bytes()).collect()].concat()
}

#[cfg(test)]
mod tests;
