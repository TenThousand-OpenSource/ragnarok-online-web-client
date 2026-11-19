//! The tests of the server's clock (`server_time.rs`).

use super::*;

fn set(local: f64, told: u32) -> Clock {
    let mut clock = Clock::default();
    clock.set(local, told);
    clock
}

#[test]
fn the_clock_is_read_72_behind_what_the_server_told() {
    let clock = set(5_000.0, 100_000);
    assert_eq!(clock.now(5_000.0), 99_928.0);
    assert_eq!(clock.now(5_250.0), 100_178.0);
}

#[test]
fn a_time_ahead_of_the_clock_sets_it_at_once() {
    // Set from a packet that took long to come: the next comes quicker, and tells a later time than the
    // clock has.
    let mut clock = set(5_000.0, 100_000);
    clock.sync(6_000.0, 101_090);
    assert_eq!(clock.now(6_000.0), 101_018.0);
    // One that tells just the clock's time, or a little less, changes nothing.
    let was = clock;
    clock.sync(7_000.0, 102_090);
    clock.sync(8_000.0, 103_000);
    assert_eq!(clock, was);
}

#[test]
fn the_fourth_late_time_in_a_row_takes_the_clock_back_by_144() {
    let mut clock = set(5_000.0, 100_000);
    // Each of these was 200 on its way longer than the one the clock was set from.
    for at in [6_000.0, 7_000.0, 8_000.0] {
        clock.sync(at + 200.0, 100_000 + at as u32 - 5_000);
        assert_eq!(clock.now(9_000.0), 103_928.0);
    }
    clock.sync(9_200.0, 104_000);
    assert_eq!(clock.now(9_200.0), 104_128.0 - 144.0);
}

#[test]
fn a_time_that_is_not_late_starts_the_count_anew() {
    let mut clock = set(5_000.0, 100_000);
    for at in [6_000.0, 7_000.0, 8_000.0] {
        clock.sync(at + 200.0, 100_000 + at as u32 - 5_000);
    }
    // 144 behind is not yet late.
    clock.sync(9_144.0, 104_000);
    for at in [10_000.0, 11_000.0, 12_000.0] {
        clock.sync(at + 200.0, 100_000 + at as u32 - 5_000);
    }
    assert_eq!(clock.now(13_000.0), 107_928.0);
}

#[test]
fn a_time_told_after_the_servers_count_ran_round_is_still_the_nearest() {
    let mut clock = set(1_000.0, u32::MAX - 100);
    let before = clock.now(1_000.0);
    // 300 later the server's count has run round to 199.
    assert_eq!(clock.of(1_300.0, 199), before + 72.0 + 300.0);
    clock.sync(1_300.0, 250);
    assert_eq!(clock.now(1_300.0), before + 300.0 + 51.0);
}
