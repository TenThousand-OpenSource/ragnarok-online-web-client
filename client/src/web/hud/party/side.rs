//! The window beside the party window: the one that sets the party up or makes one (`UIPartySettingWnd`), and
//! the one that asks whom to invite (`UIPartyInvitationToWnd`).

use super::*;

impl App {
    /// How large the window beside the party window is now: the setup 130 by 245, the making of a party 130
    /// by 220 with a name to type or 130 by 180 without, the invitation 150 by 100 (`UIWindowMgr::
    /// EnsureWindow`).
    pub(in crate::web) fn party_side_size(&self) -> (i32, i32) {
        match self.party.side {
            Some(PartySide::Setup) => (130, 245),
            Some(PartySide::Create) => (130, 220),
            Some(PartySide::Invite) => (150, 100),
            Some(PartySide::Named) | None => (130, 180),
        }
    }

    /// The window beside the party window: the setup, or the invitation.
    pub(in crate::web) fn party_side_window(&self) -> String {
        let Some(side) = self.party.side else { return String::new() };
        let (width, height) = self.party_side_size();
        if side == PartySide::Invite {
            // `UIPartyInvitationToWnd`: the words 10,22, the name typed 20,42 on grey 230, OK and Cancel.
            return [
                format!(
                    r#"{}{}{}<input id="party-invitee" class="a in" style="{};background:#e6e6e6;box-sizing:border-box" "#,
                    title_bar(width, &Self::words(2056, "Party Invitation")),
                    panel(width, height),
                    text(10, 22, BLACK, &Self::words(2058, "Player Name:")),
                    at(20, 42, 120, 20)
                ),
                format!(
                    r#"type="text" maxlength="23" autocomplete="off">{}{}"#,
                    picture_press(width - 92, height - 24, "btn_ok", Press::InviteOk),
                    picture_press(width - 46, height - 24, "btn_cancel", Press::InviteCancel)
                )
            ].concat();
        }
        // `UIPartySettingWnd`: what its rows stand under is its `m_mode`, 70 for a party there is (with how
        // experience is shared on top), 50 for one being made (with its name on top), 5 for one whose name is
        // given already.
        let mode = match side {
            PartySide::Setup => 70,
            PartySide::Create => 50,
            _ => 5,
        };
        let draft = self.party.draft;
        let (leads, new) = (self.leads_party(), self.party.members.is_empty());
        let mut window = title_bar(width, &if side == PartySide::Create { Self::words(2054, "Create Party") } else { Self::words(291, "Party Setup") });
        window.push_str(&panel(width, height));
        window.push_str(&cross(width, Press::SetupCancel));
        if side == PartySide::Setup {
            window.push_str(&text(10, 22, BLACK, &Self::words(292, "How to share EXP")));
            window.push_str(&radio(20, 42, draft.experience == 0, leads, &Self::words(287, "Each Take "), Some(Press::Experience(0).into())));
            window.push_str(&radio(20, 62, draft.experience == 1, leads, &Self::words(288, "Even Share"), Some(Press::Experience(1).into())));
        } else if side == PartySide::Create {
            window.push_str(&text(10, 22, BLACK, &Self::words(2057, "Party Name:")));
            window.push_str(&field("party-name", (20, 38, 100, 20), &self.party.making, 23, "background:#e6e6e6;box-sizing:border-box"));
        }
        window.push_str(&text(10, mode + 17, BLACK, &Self::words(293, "How to share Items")));
        window.push_str(&radio(20, mode + 37, !draft.pickup, new || leads, &Self::words(289, "Each Take"), Some(Press::Pickup(0).into())));
        window.push_str(&radio(20, mode + 57, draft.pickup, new || leads, &Self::words(290, "Party Share"), Some(Press::Pickup(1).into())));
        // The one way items are handed out: the client has this single button, always on.
        window.push_str(&text(10, mode + 82, BLACK, &Self::words(738, "Item sharing type")));
        window.push_str(&radio(20, mode + 102, true, new || leads, &Self::words(739, "Individual"), None));
        window.push_str(&picture_press(width - 92, height - 24, "btn_ok", Press::SetupOk));
        window.push_str(&picture_press(width - 46, height - 24, "btn_cancel", Press::SetupCancel));
        window
    }
}
