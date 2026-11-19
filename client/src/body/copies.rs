//! The copies of a character that an effect leaves standing: the client's `CHalluPC`, an actor of its own
//! made as its source is at that moment (its action and frame, what it wears), which neither moves nor
//! changes its picture. It is drawn in the colour it is told, at 180 of 255, and after 105 passes loses 8 of
//! that at every fourth pass, until less than 8 is left.
//!
//! At its 110th pass the client launches `EF_HALLUTIME` on it, which is not drawn here.

/// How much of a copy shows after so many passes of its life (of 255); nothing once it is gone.
pub fn alpha(passes: i32) -> Option<u8> {
    let lost = (passes / 4 - 26).max(0) * 8;
    (passes <= 192).then_some((180 - lost) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_copy_stands_for_a_while_and_then_fades_by_eight_every_fourth_pass() {
        assert_eq!((alpha(0), alpha(107)), (Some(180), Some(180)));
        assert_eq!((alpha(108), alpha(111), alpha(112)), (Some(172), Some(172), Some(164)));
        // The last it shows is 4; with less than 8 left it is gone at its next pass.
        assert_eq!((alpha(191), alpha(192), alpha(193)), (Some(12), Some(4), None));
    }
}
