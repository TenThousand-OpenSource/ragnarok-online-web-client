//! The presses of the skills: the skill window with its tabs and the buttons by each skill, a skill's
//! description, the hotkey bar's buttons for its rows, the button that tells of a job level gained, and the box
//! that asks the words of a Talkie Box or of Graffiti.

use super::Step;

/// An arrow by a skill's level: the skill, and which way the level it is used at goes. Written as `id:+` or
/// `id:-`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelStep {
    pub skill: u16,
    pub step: Step,
}

impl std::fmt::Display for LevelStep {
    fn fmt(&self, to: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(to, "{}:{}", self.skill, self.step)
    }
}

impl std::str::FromStr for LevelStep {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        let (skill, step) = text.split_once(':').ok_or(())?;
        Ok(LevelStep { skill: skill.parse().map_err(|_| ())?, step: step.parse()? })
    }
}

presses! {
    /// What can be pressed here.
    Press {
        /// The skill window is opened or closed: its icon in the menu, and the cross in its corner.
        Window = "skills",
        /// A skill that can be raised, pressed in the tree, or the button by it in the list: a level of the
        /// skill of that number is planned (`UINewSkillListWnd::OnLBtnUp`, `ID_LVLUP1`…).
        Raise(u16) = "raise-skill",
        /// Apply and Reset under the skills (`ID_APPLY`, `ID_RESET`): the levels planned are asked about and
        /// sent, or forgotten; and the two answers to what Apply asks.
        Apply = "skills-apply",
        Reset = "skills-reset",
        ApplyYes = "skills-apply-yes",
        ApplyNo = "skills-apply-no",
        /// The small button of the title bar (`ID_MINI`): the tree becomes the list of rows and back.
        Mini = "skills-mini",
        /// A row of the list, counted from the first skill of the page: it is marked.
        Pick(usize) = "skill-row",
        /// The cross of a skill's description.
        InfoClose = "skill-info-close",
        /// The hotkey bar's plus and minus: a row more or a row less.
        HotkeyRows(Step) = "hotkey-rows",
        /// A tab of the skill window, counted from the top: one for each job the character grew through.
        Tab(usize) = "skill-tab",
        /// An arrow by a skill's level: the level it is used at, one down or up.
        Level(LevelStep) = "skill-level",
        /// The button that tells of a job level gained (the client's `UINotifyJobLevelUpWnd`): the skill window
        /// opens.
        JobLevelGained = "job-level-gained",
        /// OK and Cancel of the box that asks the words of a Talkie Box or of Graffiti (the client's
        /// `UITalkboxTrapInputWnd`, `UIGraffiStrboxWnd`).
        WordsOk = "skill-words-ok",
        WordsCancel = "skill-words-cancel",
    }
}

#[cfg(test)]
mod tests {
    use super::{LevelStep, Press};
    use crate::press::Step;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [
            Press::Window,
            Press::Raise(28),
            Press::Apply,
            Press::Reset,
            Press::ApplyYes,
            Press::ApplyNo,
            Press::Mini,
            Press::Pick(3),
            Press::InfoClose,
            Press::HotkeyRows(Step::More),
            Press::HotkeyRows(Step::Less),
            Press::Tab(1),
            Press::Level(LevelStep { skill: 5, step: Step::More }),
            Press::Level(LevelStep { skill: 5, step: Step::Less }),
            Press::JobLevelGained,
            Press::WordsOk,
            Press::WordsCancel,
        ];
        crate::press::testing::read_back(all);
    }

    #[test]
    fn an_arrow_is_written_as_the_skill_and_the_way() {
        assert_eq!(LevelStep { skill: 19, step: Step::Less }.to_string(), "19:-");
        assert_eq!("19:up".parse::<LevelStep>(), Err(()));
        assert_eq!("bolt:+".parse::<LevelStep>(), Err(()));
    }
}
