//! The homunculus window, what it asks, the list of the homunculus's skills, and the menu on the homunculus.

use super::*;
use crate::net::homunculus;

impl App {
    /// A press of this system (`crate::press::homunculus`).
    pub(super) fn homunculus_pressed(&mut self, press: crate::press::homunculus::Press) {
        use crate::press::homunculus::Press;
        match press {
            Press::Window => {
                (self.homunculus.open, self.homunculus.asks) = (!self.homunculus.open && self.homunculus.shown(), None);
                self.render();
            }
            // A new name: 23 letters at most (line 938 says so), and asked about before it is sent.
            Press::Rename => {
                let name = field(&wnd::HOMUNCULUS.field()).trim().to_owned();
                if name.len() >= 24 {
                    self.message = Some(Self::words(938, "Please give your Homunculus a name no longer than 23 letters."));
                    self.render();
                } else if !name.is_empty() {
                    self.homunculus_asks(HomunculusAsk::Rename(name));
                }
            }
            Press::Feed => self.homunculus_asks(HomunculusAsk::Feed),
            Press::Delete => self.homunculus_asks(HomunculusAsk::Delete),
            Press::Skills => {
                self.homunculus.skills_open = !self.homunculus.skills_open && self.homunculus.shown();
                self.render();
            }
            // The box of feeding by itself: the server is asked, and tells how it stands (`ZC_CONFIG`).
            Press::AutoFeed => self.send_on_map(&crate::net::viewequip::set(homunculus::AUTOFEED, !self.homunculus.autofeed)),
            Press::Yes | Press::No => self.homunculus_answered(press == Press::Yes),
            Press::SkillPick(row) => {
                self.homunculus.skill_picked = Some(row);
                self.redraw(&[&wnd::HOMUNCULUS_SKILLS]);
            }
            // Use: the picked skill, if it is learned and costs something (`UISkillListWnd`, its `ID_OK`).
            Press::SkillUse => {
                if let Some(skill) = self.homunculus.skill_picked.and_then(|row| self.homunculus.skills.get(row)).filter(|skill| skill.level != 0 && skill.sp != 0) {
                    self.used(Use::Skill(skill.id));
                }
            }
            // A skill point of the homunculus goes into the skill: the same request as for the character's
            // own (`MM_UPGRADE_SKILLLEVEL`), which the server tells apart by the skill.
            Press::SkillRaise(id) => {
                if self.homunculus.info.as_ref().is_some_and(|info| info.skill_points > 0) {
                    self.send_on_map(&items::raise_skill(id));
                }
            }
            Press::SkillLevel(step) => {
                let most = self.homunculus.skills.iter().find(|skill| skill.id == step.skill).map(|skill| skill.level);
                self.companion_skill_level(step, most);
                self.redraw(&[&wnd::HOMUNCULUS_SKILLS]);
            }
            Press::Do(order) => self.homunculus_menu_does(order),
        }
    }

    /// The window asks before it acts.
    fn homunculus_asks(&mut self, asks: HomunculusAsk) {
        self.homunculus.asks = Some(asks);
        self.redraw(&[&wnd::HOMUNCULUS_ASKS]);
    }

    /// What the window asked was answered (`UIHomunInfoWnd::SendMsg`): fed (not while a deal is open: line
    /// 3551 says so), named (the window shows the name at once), or deleted (its windows close at once).
    fn homunculus_answered(&mut self, yes: bool) {
        match self.homunculus.asks.take().filter(|_| yes) {
            Some(HomunculusAsk::Feed) if self.exchange.open => self.chat_line(Self::words(3551, "Failed to feed the homunculus. Homunculus in a state that can't be fed."), RED),
            Some(HomunculusAsk::Feed) => self.send_on_map(&homunculus::command(homunculus::order::FEED)),
            Some(HomunculusAsk::Rename(name)) => {
                self.send_on_map(&homunculus::rename(&name));
                if let Some(info) = &mut self.homunculus.info {
                    info.name = name;
                }
            }
            Some(HomunculusAsk::Delete) => {
                self.send_on_map(&homunculus::command(homunculus::order::DELETE));
                (self.homunculus.open, self.homunculus.skills_open, self.homunculus.out, self.homunculus.marked) = (false, false, false, None);
                return self.render();
            }
            None => {}
        }
        self.redraw(&[&wnd::HOMUNCULUS, &wnd::HOMUNCULUS_ASKS]);
    }

    /// A row of the menu on the homunculus itself (the client's `MENUID_HOMUN_SHOWINFO`, `_FEEDING`,
    /// `_FOLLOW`): its window with the server asked for what it shows, feeding asked about, standing by.
    fn homunculus_menu_does(&mut self, order: crate::press::homunculus::Order) {
        use crate::press::homunculus::Order;
        self.homunculus.menu_at = None;
        match order {
            Order::Info => {
                self.send_on_map(&homunculus::command(homunculus::order::INFO));
                self.homunculus.open = !self.homunculus.open;
            }
            Order::Feed => self.homunculus.asks = Some(HomunculusAsk::Feed),
            Order::StandBy => self.companion_stands_by(crate::ai::Kind::Homunculus),
        }
        self.render();
    }
}
