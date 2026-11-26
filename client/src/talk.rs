//! A talk with an NPC as rules: what the keyboard does to its windows, as the client's window manager hands
//! the keys out (`UIWindowMgr::ProcessEnterKey`, `ConfirmActiveDialog` and `SubmitActiveDialog` for the
//! arrows up and down), and what each window does with them (`UISayDialogWnd`, `UIChooseWnd`,
//! `UINpcEditDialogWnd`, `UINpcTextEditDialogWnd`, `UIListBox`). The packets are in `net/talk.rs`.
//!
//! Enter goes to one window only, and then nowhere else, not to the chat's line either: the box that asks
//! for a number or for words if there is one, else the menu, else what the NPC said. Each does what its own
//! button does: OK, OK of the picked line, "next" or "close"; what the NPC said does nothing while it has
//! no button (`UIFrameWnd`'s `UM_DEFPUSH` with the command 600). The arrows up and down go to the menu, and
//! with no menu to what the NPC said, whose text they move. Escape does nothing while an NPC is talked to
//! (`UIWindowMgr::ProcessKeyDown`, `VK_ESCAPE`: it leaves at `WID_SAYDIALOGWND`); Space, Home, End, Page Up
//! and Page Down are not the windows' keys.

use crate::press::talk::Press;

/// What Enter presses: of the box that asks (`asks`), the menu (`menu`: whether there is one) and the button
/// under what was said (`button`: "next" for true, "close" for false), the first there is, in that order.
pub fn entered(asks: Option<bool>, menu: bool, button: Option<bool>) -> Option<Press> {
    if asks.is_some() || menu {
        return Some(Press::Ok);
    }
    button.map(|next| if next { Press::Next } else { Press::Close })
}

/// The line of a menu of so many `rows` that is picked after an arrow, down or up, from the one that was: the
/// next or the one before, and at either end the same one, for the list does not go round
/// (`UIListBox::SendMsg`, `UM_DOWNPUSH` and `UM_UPPUSH`). A menu opens with its first line picked
/// (`UIListBox::UIListBox`).
pub fn stepped(picked: usize, rows: usize, down: bool) -> usize {
    if down { (picked + 1).min(rows.saturating_sub(1)) } else { picked.saturating_sub(1) }
}

/// How long after "next" or a menu's OK another of them is not taken, in milliseconds: the client keeps one
/// moment for both (`g_session.m_itemSendTick`, 300).
pub const APART: u32 = 300;

/// Whether "next" or a menu's OK at `now` comes too soon after the last one, at `last` (both by the client's
/// clock of milliseconds, which goes round).
pub fn too_soon(now: u32, last: u32) -> bool {
    now.wrapping_sub(last) < APART
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_presses_the_button_of_the_first_window_there_is() {
        // The box that asks comes before the menu, the menu before what was said.
        assert_eq!(entered(Some(false), true, Some(true)), Some(Press::Ok));
        assert_eq!(entered(Some(true), false, None), Some(Press::Ok));
        assert_eq!(entered(None, true, Some(false)), Some(Press::Ok));
        assert_eq!(entered(None, false, Some(true)), Some(Press::Next));
        assert_eq!(entered(None, false, Some(false)), Some(Press::Close));
        // What was said has no button yet: Enter does nothing.
        assert_eq!(entered(None, false, None), None);
    }

    #[test]
    fn the_arrows_stop_at_the_ends_of_a_menu() {
        assert_eq!(stepped(0, 3, true), 1);
        assert_eq!(stepped(1, 3, true), 2);
        assert_eq!(stepped(2, 3, true), 2);
        assert_eq!(stepped(2, 3, false), 1);
        assert_eq!(stepped(0, 3, false), 0);
        assert_eq!(stepped(0, 1, true), 0);
        assert_eq!(stepped(0, 0, true), 0);
    }

    #[test]
    fn an_answer_is_not_taken_twice_in_a_moment() {
        assert!(too_soon(1_299, 1_000));
        assert!(!too_soon(1_300, 1_000));
        // The clock went round between the two.
        assert!(too_soon(100, u32::MAX - 100));
        assert!(!too_soon(400, u32::MAX - 100));
    }
}
