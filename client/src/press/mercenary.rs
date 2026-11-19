//! The presses of the mercenary: its window (dismissing it), what the window asks, the list of its skills, and
//! the menu on the mercenary itself.

use super::skills::LevelStep;

/// A row of the menu on the mercenary itself (the client's `MENUID_MER_*`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Order {
    /// Its window is opened or closed.
    Info,
    /// It comes to its owner and stands by, or goes about its own again.
    StandBy,
}

impl Order {
    pub const ALL: [Order; 2] = [Order::Info, Order::StandBy];

    fn text(self) -> &'static str {
        match self {
            Order::Info => "info",
            Order::StandBy => "stand-by",
        }
    }
}

impl std::fmt::Display for Order {
    fn fmt(&self, to: &mut std::fmt::Formatter) -> std::fmt::Result {
        to.write_str(self.text())
    }
}

impl std::str::FromStr for Order {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        Order::ALL.into_iter().find(|order| order.text() == text).ok_or(())
    }
}

presses! {
    /// What can be pressed here.
    Press {
        /// The window is opened or closed: the cross in its corner.
        Window = "mercenary",
        /// The button that dismisses it.
        Fire = "mercenary-fire",
        /// The Skill button: the list of its skills is opened or closed; and the list's own cross and Close.
        Skills = "mercenary-skills",
        /// Whether to dismiss it: OK.
        Yes = "mercenary-yes",
        /// Whether to dismiss it: Cancel.
        No = "mercenary-no",
        /// A row of the list of skills, counted from the top: it is the one the Use button uses.
        SkillPick(usize) = "mercenary-skill",
        /// The list's Use button.
        SkillUse = "mercenary-skill-use",
        /// An arrow by a skill's level: the level it is used at, one down or up.
        SkillLevel(LevelStep) = "mercenary-skill-level",
        /// A row of the menu on the mercenary itself.
        Do(Order) = "mercenary-do",
    }
}

#[cfg(test)]
mod tests {
    use super::{Order, Press};
    use crate::press::skills::LevelStep;
    use crate::press::{Named, Step};

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let mut all = vec![Press::Window, Press::Fire, Press::Skills, Press::Yes, Press::No, Press::SkillPick(1), Press::SkillUse, Press::SkillLevel(LevelStep { skill: 8201, step: Step::More })];
        all.extend(Order::ALL.map(Press::Do));
        for name in Press::NAMES {
            assert!(all.iter().any(|press| press.name().0 == *name), "{name} is not tried");
        }
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
        assert_eq!(Press::parse("mercenary-do", "feed"), None);
    }
}
