//! The Status window as rules (the client's `UIStatusWnd`): the number it shows for the speed of the
//! character's blows.

/// The attack speed the window shows for the delay between blows the server keeps: `(2000 - delay) / 10`, cut
/// towards nought.
pub fn attack_speed(delay: i64) -> i64 {
    (2000 - delay) / 10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_speed_of_blows_is_counted_from_their_delay() {
        assert_eq!([attack_speed(500), attack_speed(2000), attack_speed(0), attack_speed(195)], [150, 0, 200, 180]);
        // A delay over 2000, and one under nought: cut towards nought both ways.
        assert_eq!([attack_speed(2005), attack_speed(2015), attack_speed(-5)], [0, -1, 200]);
    }
}
