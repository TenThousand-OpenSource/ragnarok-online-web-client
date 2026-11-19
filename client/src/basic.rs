//! The Basic Information window as rules (the client's `UIBasicInfoWnd`): how tall it is, the hundredths it
//! writes beside a gauge, and what it shows of the weight carried.

/// How tall the window is: in its small form or open, and taller for a job with Activity Points (149 for 134
/// open and 71 for 55 closed).
pub fn height(small: bool, ap: bool) -> i32 {
    match (small, ap) {
        (true, false) => 55,
        (true, true) => 71,
        (false, false) => 134,
        (false, true) => 149,
    }
}

/// How many hundredths of `most` there are, as the window writes it beside a gauge: nothing while there is no
/// most.
pub fn percent(now: i64, most: i64) -> String {
    if most > 0 { format!("{}%", now * 100 / most) } else { String::new() }
}

/// How many hundredths of what can be carried are carried, as the window's foot tells under the pointer
/// (`UIBasicInfoWnd::OnMouseHover`): nought while nothing can be. (`inventory::weight::percent`, which the chat's
/// lines are counted with, takes a most of nought for one.)
pub fn weight_share(weight: i64, most: i64) -> i64 {
    if most > 0 { weight * 100 / most } else { 0 }
}

/// Whether the weight is written in red: carrying the `share` of what can be carried from which nothing is
/// recovered by resting (the client's `m_overweightThresholdPercent`), or more.
pub fn heavy(weight: i64, most: i64, share: i64) -> bool {
    most > 0 && weight * 100 / most >= share
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_is_taller_open_and_with_activity_points() {
        assert_eq!([height(true, false), height(true, true), height(false, false), height(false, true)], [55, 71, 134, 149]);
    }

    #[test]
    fn a_gauge_has_its_hundredths_while_it_has_a_most() {
        assert_eq!([percent(50, 200), percent(199, 200), percent(200, 200), percent(0, 7)], ["25%", "99%", "100%", "0%"]);
        // More than the most and less than nought are written as counted.
        assert_eq!([percent(300, 200), percent(-50, 200)], ["150%", "-25%"]);
        assert_eq!([percent(5, 0), percent(5, -1), percent(0, 0)], ["", "", ""]);
    }

    #[test]
    fn the_weight_is_a_share_and_red_from_the_share_told() {
        assert_eq!([weight_share(450, 1000), weight_share(999, 1000), weight_share(5, 0), weight_share(5, -3)], [45, 99, 0, 0]);
        assert!(heavy(500, 1000, 50) && heavy(900, 1000, 50) && !heavy(499, 1000, 50));
        assert!(heavy(700, 1000, 70) && !heavy(699, 1000, 70));
        // Nothing can be carried: never red, whatever the share.
        assert!(!heavy(5, 0, 50) && !heavy(5, 0, 0) && !heavy(5, -3, 0));
        assert!(heavy(0, 10, 0));
    }
}
