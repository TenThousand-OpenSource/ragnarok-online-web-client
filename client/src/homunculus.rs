//! The homunculus as rules: the word for its intimacy, when the chat says it is hungry, the number its window
//! shows for the speed of its blows. No packet is read here: those are in `net/homunculus.rs`.

/// How close a homunculus is to its owner, as the line of the client's message table that says it and the
/// English of it (`UIHomunInfoWnd::Paint`: seven stages, and "Unknown" outside them).
pub fn intimacy_word(intimacy: i32) -> (usize, &'static str) {
    match intimacy {
        0..=3 => (1022, "Hate with a Passion"),
        4..=10 => (1021, "Hate"),
        11..=100 => (672, "Awkward"),
        101..=250 => (673, "Shy"),
        251..=750 => (669, "Neutral"),
        751..=910 => (674, "Cordial"),
        911..=1000 => (675, "Loyal"),
        _ => (676, "Unknown"),
    }
}

/// Whether the chat is to say that the homunculus is hungry: at a hunger of 10, 5 and 1, once for each as it
/// is reached (the client's `ReceiveHomunculusInfo3` and its `m_homunHungerNotified`). Gives the hunger to
/// remember as told of.
pub fn hungry_now(hunger: i32, told: i32) -> Option<i32> {
    (matches!(hunger, 1 | 5 | 10) && told != hunger).then_some(hunger)
}

/// The number its window shows for the speed of its blows (`UIHomunInfoWnd::Paint`: `(2000 - delay) / 10`).
pub fn attack_speed(delay: i32) -> i32 {
    (2000 - delay) / 10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_it_as_its_window_does() {
        assert_eq!(
            (
                intimacy_word(0).0,
                intimacy_word(10).0,
                intimacy_word(100).0,
                intimacy_word(250).0,
                intimacy_word(750).0,
                intimacy_word(910).0,
                intimacy_word(1000).0,
                intimacy_word(1001).0
            ),
            (1022, 1021, 672, 673, 669, 674, 675, 676)
        );
        assert_eq!((hungry_now(10, 0), hungry_now(10, 10), hungry_now(5, 10), hungry_now(11, 10)), (Some(10), None, Some(5), None));
        assert_eq!(attack_speed(1400), 60);
    }
}
