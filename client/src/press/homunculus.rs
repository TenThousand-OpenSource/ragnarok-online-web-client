//! The presses of the homunculus: its window (the name, feeding, deleting, feeding by itself), what the
//! window asks, the list of its skills, and the menu on the homunculus itself.

use super::skills::LevelStep;

/// A row of the menu on the homunculus itself (the client's `MENUID_HOMUN_*`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Order {
    /// Its window is opened or closed.
    Info,
    Feed,
    /// It comes to its owner and stands by, or goes about its own again.
    StandBy,
}

impl Order {
    pub const ALL: [Order; 3] = [Order::Info, Order::Feed, Order::StandBy];

    fn text(self) -> &'static str {
        match self {
            Order::Info => "info",
            Order::Feed => "feed",
            Order::StandBy => "stand-by",
        }
    }
}

worded!(Order);

presses! {
    /// What can be pressed here.
    Press {
        /// The window is opened or closed: the cross in its corner.
        Window = "homunculus",
        /// The button by its name: the name typed is to be given to it.
        Rename = "homunculus-rename",
        /// The Feed button.
        Feed = "homunculus-feed",
        /// The Delete button.
        Delete = "homunculus-delete",
        /// The Skill button: the list of its skills is opened or closed; and the list's own cross and Close.
        Skills = "homunculus-skills",
        /// The box of feeding by itself.
        AutoFeed = "homunculus-autofeed",
        /// What the window asks: OK.
        Yes = "homunculus-yes",
        /// What the window asks: Cancel.
        No = "homunculus-no",
        /// A row of the list of skills, counted from the top: it is the one the Use button uses.
        SkillPick(usize) = "homunculus-skill",
        /// The list's Use button.
        SkillUse = "homunculus-skill-use",
        /// The button by a skill that can be raised: a skill point of the homunculus goes into it.
        SkillRaise(u16) = "homunculus-skill-raise",
        /// An arrow by a skill's level: the level it is used at, one down or up.
        SkillLevel(LevelStep) = "homunculus-skill-level",
        /// A row of the menu on the homunculus itself.
        Do(Order) = "homunculus-do",
    }
}

#[cfg(test)]
mod tests {
    use super::{Order, Press};
    use crate::press::skills::LevelStep;
    use crate::press::{Named, Step};

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let mut all = vec![
            Press::Window,
            Press::Rename,
            Press::Feed,
            Press::Delete,
            Press::Skills,
            Press::AutoFeed,
            Press::Yes,
            Press::No,
            Press::SkillPick(2),
            Press::SkillUse,
            Press::SkillRaise(8001),
            Press::SkillLevel(LevelStep { skill: 8001, step: Step::Less }),
        ];
        all.extend(Order::ALL.map(Press::Do));
        crate::press::testing::read_back(all);
        assert_eq!(Press::parse("homunculus-do", "sit"), None);
    }
}
