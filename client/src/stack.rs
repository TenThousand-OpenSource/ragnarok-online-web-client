//! The order the game's windows lie in, one over another.
//!
//! From the client's `UIWindowMgr`: it keeps a list of its windows (`windowHistory`), the one in front first. A
//! window opened or pressed goes to the front (`BringToFront`, from `UIFrameWnd::OnLBtnDown`); Escape closes the
//! one in front (`CloseWindow` of the list's first), and only with none left does it open the game's menu.
//! Here the last of the list is the one in front.

/// The windows that are open, by name, the one in front last.
#[derive(Default)]
pub struct Stack(Vec<&'static str>);

impl Stack {
    /// Takes the windows open now: those closed leave the list, those newly open go to the front, in the order
    /// given. Gives back whether anything changed.
    pub fn settle(&mut self, open: &[&'static str]) -> bool {
        let before = self.0.clone();
        self.0.retain(|window| open.contains(window));
        for window in open {
            if !self.0.contains(window) {
                self.0.push(window);
            }
        }
        self.0 != before
    }

    /// Brings a window to the front, if it is one of the list. Gives back whether anything changed.
    pub fn raise(&mut self, name: &str) -> bool {
        match self.0.iter().position(|window| *window == name) {
            Some(at) if at + 1 < self.0.len() => {
                let window = self.0.remove(at);
                self.0.push(window);
                true
            }
            _ => false,
        }
    }

    /// The window in front.
    pub fn front(&self) -> Option<&'static str> {
        self.0.last().copied()
    }

    /// The windows from the one at the back to the one in front.
    pub fn windows(&self) -> &[&'static str] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_opened_lies_in_front() {
        let mut stack = Stack::default();
        assert!(stack.settle(&["inventory"]));
        assert!(stack.settle(&["inventory", "skills"]));
        assert_eq!(stack.front(), Some("skills"));
        // Opened later, though named earlier: it is the new one that goes to the front.
        assert!(stack.settle(&["status", "inventory", "skills"]));
        assert_eq!(stack.windows(), ["inventory", "skills", "status"]);
        assert!(!stack.settle(&["status", "inventory", "skills"]));
    }

    #[test]
    fn a_window_pressed_comes_to_the_front() {
        let mut stack = Stack::default();
        stack.settle(&["inventory", "skills", "status"]);
        assert!(stack.raise("inventory"));
        assert_eq!(stack.windows(), ["skills", "status", "inventory"]);
        assert!(!stack.raise("inventory"));
        assert!(!stack.raise("chat"));
    }

    #[test]
    fn closing_the_one_in_front_leaves_the_one_behind_it_in_front() {
        let mut stack = Stack::default();
        stack.settle(&["inventory", "skills", "status"]);
        stack.raise("skills");
        stack.settle(&["inventory", "status"]);
        assert_eq!(stack.front(), Some("status"));
        stack.settle(&[]);
        assert_eq!(stack.front(), None);
    }
}
