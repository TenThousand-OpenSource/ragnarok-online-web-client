//! What the server tells the player in a line of the chat: an action that failed, what a map forbids, a line
//! of the message table with a number, a skill or a colour, experience gained, a rented item's time, a thing
//! damaged or bound. Each is said as the client's handler says it (`crate::net::notices` names them).

use super::*;
use crate::net::notices;
use crate::numbers::filled;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn notices_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            notices::ZC_ACTION_FAILURE => {
                let job = self.game.as_ref().map_or(0, |game| game.job);
                self.notice(notices::kind(packet).and_then(|kind| notices::action_failure(kind, job)));
            }
            notices::ZC_NOTIFY_MAPINFO => self.notice(notices::kind(packet).and_then(notices::map_info)),
            // A line with a number written into it, in yellow (`CGameMode::ReceiveFormattedMessage`); one
            // line takes the name of an item instead.
            notices::ZC_MSG_VALUE => {
                if let Some((line, value)) = notices::value(packet) {
                    let with = if line == notices::NAMES_AN_ITEM { Self::item_called(value as u32) } else { value.to_string() };
                    self.say_filled(said::GENERAL, line, &[&with], YELLOW);
                }
            }
            // A line about a skill, the skill's name before it in brackets (`case ID_ZC_MSG_SKILL`), among the
            // lines of skills that failed.
            notices::ZC_MSG_SKILL => {
                if let Some((skill, words)) = notices::skill(packet).and_then(|(skill, line)| Some((skill, data::message(line)?))) {
                    let name = data::skill(skill).map_or_else(|| skill.to_string(), |(_, name, _)| name);
                    self.chat_line_of(said::SKILL_FAILED, format!("[{name}] {}", crate::i18n::tr(&words)), notices::SKILL);
                }
            }
            notices::ZC_MSG_COLOR => {
                if let Some((line, colour)) = notices::coloured(packet) {
                    self.say_filled(said::GENERAL, line, &[], &colour);
                }
            }
            notices::ZC_SKILLMSG => {
                if let Some(line) = notices::skill_message(packet) {
                    self.say_filled(said::STATUS, line, &[], notices::GOSPEL);
                }
            }
            notices::ZC_NOTIFY_EXP2 => self.experience_told(packet),
            notices::ZC_RECOVER_PENALTY_OVERWEIGHT => {
                if let Some(percent) = notices::overweight(packet) {
                    self.notices.overweight = i64::from(percent);
                    self.redraw(&[&wnd::BASIC]);
                }
            }
            // Of the values of another the client reads only the manner points (`CPc::SetHonor`), by which it
            // shows the mark of one who may not talk over that player: a picture of its cash emotions, which
            // this client does not have. Nothing is shown.
            notices::ZC_PAR_CHANGE_USER => {}
            // A stat was raised: it is what the server says now (`CGameMode::ReceiveStatusChangeAck`).
            crate::net::packets::ZC_STATUS_CHANGE_ACK => {
                if let Some((stat, value)) = crate::net::packets::stat_raised(packet) {
                    self.hero.params.insert(stat, i64::from(value));
                    self.refresh_values();
                }
            }
            notices::ZC_EQUIPITEM_DAMAGED => self.damage_told(packet),
            notices::ZC_CASH_TIME_COUNTER => self.rental_told(packet),
            notices::ZC_CASH_ITEM_DELETE => self.rental_over(packet),
            // A thing put on is bound to the character (`case ID_ZC_NOTIFY_BIND_ON_EQUIP`).
            notices::ZC_NOTIFY_BIND_ON_EQUIP => {
                let bound = notices::kind(packet).and_then(|index| self.bag.at(index)).map(Self::item_name);
                if let Some(name) = bound {
                    self.say_filled(said::GENERAL, 1328, &[&name], notices::BOUND);
                }
            }
            // The client's `CGameMode` has no reader for it.
            notices::ZC_ACK_PVPPOINT => {}
            _ => return false,
        }
        true
    }

    /// A line of the message table said in the chat in its colour, if there is one to say.
    fn notice(&mut self, said: Option<(usize, &'static str)>) {
        if let Some((line, colour)) = said {
            self.say_filled(said::GENERAL, line, &[], colour);
        }
    }

    /// What an item is called by its number alone, or its number while its name is not known.
    fn item_called(id: u32) -> String {
        data::item(id).map_or_else(|| format!("{} {id}", crate::i18n::tr("Item")), |(name, _)| name)
    }

    /// Experience the player gained or lost (`CGameMode::ReceiveExpGainNotice`): "'1234' Base Experience
    /// acquired.", among the lines of experience, which a tab of the chat lists only when it is set to (the
    /// client's tabs do not list them as it starts: option 19 of a tab in `ChatWndInfo.lub`).
    fn experience_told(&mut self, packet: &[u8]) {
        let Some(gained) = notices::experience(packet).filter(|gained| gained.who == self.own_account()) else { return };
        let Some((named, ending)) = gained.lines() else { return };
        let words = |line: usize| data::message(line).map(|words| crate::i18n::tr(&words).to_owned()).unwrap_or_default();
        let named = named.map(|line| filled(&words(line), &[&gained.amount.to_string()])).unwrap_or_default();
        self.chat_line_of(said::EXPERIENCE, format!("{named}{}", words(ending)), gained.colour());
    }

    /// Something worn was damaged (`CGameMode::ReceiveEquipItemDamaged`), in red among the lines of
    /// damage: the player's own thing by its name (line 1300), a party member's by the member's name and the
    /// word for the place (line 1301). The client's words for the hands and the head are lines its table
    /// of this date does not reach; where the table has no such line, nothing is said.
    fn damage_told(&mut self, packet: &[u8]) {
        let Some((place, who)) = notices::damaged(packet) else { return };
        if who == self.own_account() {
            if let Some(name) = self.bag.items.iter().find(|item| item.worn & u32::from(place) != 0).map(Self::item_name) {
                self.say_filled(said::DAMAGED, 1300, &[&name], RED);
            }
        } else if let Some(member) = self.party.members.iter().find(|member| member.account == who).map(|member| member.name.clone()) {
            if let Some(part) = notices::damaged_part(place).and_then(data::message) {
                self.say_filled(said::DAMAGED, 1301, &[&member, crate::i18n::tr(&part)], RED);
            }
        }
    }

    /// How long a rented item still lasts (`CGameMode::ReceiveRentalItemTime`), in yellow.
    fn rental_told(&mut self, packet: &[u8]) {
        let Some((item, seconds)) = notices::rental_time(packet) else { return };
        let name = Self::item_called(item);
        let (line, left, last) = notices::rental_lines(seconds);
        self.say_filled(said::GENERAL, line, &[&name, &left.to_string()], YELLOW);
        if let Some(last) = last {
            self.say_filled(said::GENERAL, last, &[&name], YELLOW);
        }
    }

    /// A rented item's time is over (`CGameMode::ReceiveItemDeletionNotice`): it leaves the inventory, and
    /// the chat says so in yellow.
    fn rental_over(&mut self, packet: &[u8]) {
        let Some((index, item)) = notices::rental_over(packet) else { return };
        self.bag.items.retain(|carried| carried.index != index);
        self.refresh_items();
        self.say_filled(said::GENERAL, 1258, &[&Self::item_called(item)], YELLOW);
    }
}
