//! What the server tells of the alchemist's homunculus: everything about it, what changed, how feeding it
//! went, its skills.

use super::*;
use crate::net::homunculus;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn homunculus_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            homunculus::ZC_PROPERTY_HOMUN | homunculus::ZC_PROPERTY_HOMUN_2 | homunculus::ZC_PROPERTY_HOMUN_4 => {
                if let Some(info) = homunculus::property(id, packet) {
                    self.homunculus_told(info);
                }
            }
            homunculus::ZC_CHANGESTATE_MER => self.homunculus_changed(packet),
            // Its life, its SP or its experience alone (the client's `ReceiveHomunculusProperty64`).
            homunculus::ZC_HO_PAR_CHANGE | homunculus::ZC_HO_PAR_CHANGE2 => {
                if let (Some((parameter, value)), Some(info)) = (homunculus::parameter_changed(id, packet), &mut self.homunculus.info) {
                    match parameter {
                        homunculus::parameter::EXP => info.exp = value,
                        homunculus::parameter::HP => info.hp = value,
                        homunculus::parameter::SP => info.sp = value,
                        _ => {}
                    }
                    self.redraw(&[&wnd::HOMUNCULUS]);
                }
            }
            homunculus::ZC_FEED_MER => {
                self.homunculus.fed = homunculus::fed(packet);
                if let Some((_, food)) = self.homunculus.fed {
                    data::want_items(std::iter::once(food), || dispatch(Action::ItemsKnown));
                }
                self.tell_homunculus_fed();
            }
            // Its skills, laid out as the character's own; and one of them raised.
            homunculus::ZC_HOSKILLINFO_LIST => {
                self.homunculus.skills = net::inventory::skills(packet);
                self.redraw(&[&wnd::HOMUNCULUS_SKILLS]);
            }
            homunculus::ZC_HOSKILLINFO_UPDATE => {
                if let Some((id, level, sp, range, raisable)) = net::inventory::skill_raised(packet) {
                    if let Some(skill) = self.homunculus.skills.iter_mut().find(|skill| skill.id == id) {
                        (skill.level, skill.sp, skill.range, skill.raisable) = (level, sp, range, raisable);
                    }
                    self.redraw(&[&wnd::HOMUNCULUS_SKILLS, &wnd::HOTKEYS]);
                }
            }
            // The server says whether the homunculus is fed by itself; the other settings of this packet are
            // read where they belong.
            crate::net::config::ZC_CONFIG => match crate::net::config::config(packet) {
                Some((homunculus::AUTOFEED, on)) => {
                    self.homunculus.autofeed = on;
                    self.redraw(&[&wnd::HOMUNCULUS]);
                }
                _ => return false,
            },
            _ => return false,
        }
        true
    }

    /// ZC_PROPERTY_HOMUN: everything about it (the client's `ReceiveHomunculusInfo3`). At a hunger of 10, 5
    /// and 1 the chat says, in red, that it starves (line 1018), once for each.
    fn homunculus_told(&mut self, info: homunculus::Homunculus) {
        if let Some(told) = crate::homunculus::hungry_now(info.hunger, self.homunculus.hungry_told) {
            self.homunculus.hungry_told = told;
            self.chat_line(Self::words(1018, "Your Homunculus is starving. Please feed it, otherwise it will leave you."), RED);
        }
        self.homunculus.info = Some(info);
        self.ai_wakes();
        self.redraw(&[&wnd::HOMUNCULUS, &wnd::HOMUNCULUS_SKILLS]);
    }

    /// ZC_CHANGESTATE_MER: which unit is the player's homunculus, or its intimacy or hunger alone (the
    /// client's `ReceiveHomunculusChange`).
    fn homunculus_changed(&mut self, packet: &[u8]) {
        let Some((what, unit, value)) = homunculus::changed(packet) else { return };
        match (what, &mut self.homunculus.info) {
            (homunculus::change::OWN, _) => {
                (self.homunculus.unit, self.homunculus.marked) = (unit, None);
                self.ai_wakes();
            }
            (homunculus::change::INTIMACY, Some(info)) => info.intimacy = value,
            (homunculus::change::HUNGER, Some(info)) => info.hunger = value,
            _ => {}
        }
        self.redraw(&[&wnd::HOMUNCULUS]);
    }

    /// Tells in the chat how feeding the homunculus went, once the food's name is known (the client's
    /// `ReceiveFeedResult`, with its colours: line 3281 with the food and how much of it is left, or line
    /// 591 that there is none). The client also has the homunculus sweat when there was none: not here.
    pub(in crate::web::app) fn tell_homunculus_fed(&mut self) {
        let Some((fed, food)) = self.homunculus.fed else { return };
        let Some((name, _)) = data::item(food) else { return };
        self.homunculus.fed = None;
        if fed {
            let left: u32 = self.bag.items.iter().filter(|item| item.id == food).map(|item| u32::from(item.amount)).sum();
            self.chat_line(Self::words(3281, "Homunculus has been fed. [%s] - %d remaining").replacen("%s", &name, 1).replacen("%d", &left.to_string(), 1), PALE_CYAN);
        } else {
            self.chat_line(Self::words(591, "Pet food [%s] is not available.").replace("%s", &name), "#f51e1e");
        }
    }
}
