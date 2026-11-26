//! The presses of the achievements (the client's `UIAchievementWnd` and `UIAchTracingWnd`) and of the titles
//! (the third page of its `UIEquipWnd`).

/// A tab of the window: the group, and the tab under it if it is one of those.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tab {
    pub group: u32,
    pub sub: Option<u32>,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, to: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.sub {
            Some(sub) => write!(to, "{}:{sub}", self.group),
            None => write!(to, "{}", self.group),
        }
    }
}

impl std::str::FromStr for Tab {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        let (group, sub) = match text.split_once(':') {
            Some((group, sub)) => (group, Some(sub.parse().map_err(|_| ())?)),
            None => (text, None),
        };
        Ok(Tab { group: group.parse().map_err(|_| ())?, sub })
    }
}

presses! {
    /// What can be pressed here.
    Press {
        /// The menu's icon, the window's key and its cross: the window opens, or closes if it was open.
        Window = "achievements",
        /// A tab down the left, or one under the open tab (`UIAchTab::OnLBtnDown`).
        Tab(Tab) = "achievements-tab",
        /// The arrows beside the tabs under the open one, where there are more than three: one tab on or back.
        Subs(super::Step) = "achievements-subs",
        /// One of the two round buttons over the list: 0 the completed first, 1 what is left to do.
        Shown(usize) = "achievements-shown",
        /// A row of the list: the achievement is picked (`UIAchListWnd::OnLBtnDown`).
        Row(u32) = "achievements-row",
        /// The picture of a row: the achievement is kept in sight, or no longer (the list's buttons).
        Trace(u32) = "achievements-trace",
        /// The button that takes the reward of the achievement picked (474).
        Reward = "achievements-reward",
        /// The window opens on an achievement: the name of one completed lately, a goal that leads to
        /// another, the arrow of one kept in sight (`UIAchievementWnd::SelectAchievement`).
        Show(u32) = "achievements-show",
        /// The name of an achievement kept in sight: its goals are listed under it, or put away (478).
        Unfold(u32) = "achievements-unfold",
        /// A line of the list of titles is picked: a title, or 0 for the line that takes the title off.
        Title(u32) = "achievements-title",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        crate::press::testing::read_back([
            Press::Window,
            Press::Tab(Tab { group: 2, sub: Some(5) }),
            Press::Tab(Tab { group: 0, sub: None }),
            Press::Subs(crate::press::Step::More),
            Press::Subs(crate::press::Step::Less),
            Press::Shown(1),
            Press::Row(127001),
            Press::Trace(127001),
            Press::Reward,
            Press::Show(200005),
            Press::Unfold(200005),
            Press::Title(1013),
        ]);
    }

    #[test]
    fn a_tab_is_its_group_and_what_is_under_it() {
        assert_eq!("3".parse(), Ok(Tab { group: 3, sub: None }));
        assert_eq!("3:1".parse(), Ok(Tab { group: 3, sub: Some(1) }));
        assert_eq!("3:".parse::<Tab>(), Err(()));
        assert_eq!("x".parse::<Tab>(), Err(()));
    }
}
