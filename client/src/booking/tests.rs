//! The tests of the old party booking's rules.

use super::*;

fn regions() -> Vec<Region> {
    let maps = |names: &[&str]| names.iter().map(|name| ((*name).to_owned(), [0, 0, 0])).collect();
    vec![
        Region { name: "All Maps".to_owned(), maps: maps(&["All Maps"]) },
        Region { name: "Prontera".to_owned(), maps: maps(&["Fields", "Culvert F1", "Culvert F2"]) },
        Region { name: "Geffen".to_owned(), maps: maps(&["Fields", "Tower F1"]) },
    ]
}

fn ad(index: u32, level: i16, place: i16, jobs: [i16; JOBS]) -> Ad {
    Ad { index, name: format!("Leader{index}"), time: 0, level, place, jobs }
}

fn entry(index: u32) -> Option<Entry> {
    Entry::of(&ad(index, 50, 0x0100, [1, -1, -1, -1, -1, -1]), &regions())
}

#[test]
fn a_job_is_named_by_its_line_of_the_message_table() {
    // "Attack" to "Support" are 1712 to 1715, "Swordman" 1629, "Soul Linker" 1687.
    assert_eq!([job_line(1), job_line(4), job_line(5), job_line(63), job_line(-1)], [Some(1712), Some(1715), Some(1629), Some(1687), None]);
    assert_eq!(jobs_of(0).collect::<Vec<_>>(), [1, 2, 3, 4]);
    assert_eq!((jobs_of(7).next(), jobs_of(7).last(), jobs_of(8).next()), (Some(56), Some(63), None));
    // The groups follow one another without a gap.
    assert!(GROUPS.windows(2).all(|pair| pair[0].0 + pair[0].1 == pair[1].0));
}

#[test]
fn a_place_is_a_region_and_a_map_of_it() {
    assert_eq!([place_code(0, 3), place_code(1, 0), place_code(2, 1)], [0, 0x0100, 0x0201]);
    assert_eq!(location(&regions(), 0x0102).as_deref(), Some("Prontera Culvert F2"));
    assert_eq!(location(&regions(), 0x0201).as_deref(), Some("Geffen Tower F1"));
    assert_eq!([location(&regions(), 0x0203), location(&regions(), 0x0900), location(&regions(), 0)], [None, None, None]);
    assert_eq!([levels(5, 200), levels(80, 99), levels(190, 200)], ["05~20", "80~95", "190~200"]);
    assert_eq!(clock(9, 5), " 09 : 05 ");
}

#[test]
fn registering_is_refused_as_the_client_refuses_it() {
    assert_eq!(registered(&[], "50", 200, 0, 0), Err(Refused::NoJob));
    assert_eq!(registered(&[1, 2, 3, 4, 5, 6, 7], "50", 200, 0, 0), Err(Refused::TooMany(7)));
    assert_eq!(registered(&[1], "5o", 200, 0, 0), Err(Refused::NotNumber));
    assert_eq!(registered(&[1], "", 200, 0, 0), Err(Refused::Level(200)));
    assert_eq!(registered(&[1], "201", 200, 0, 0), Err(Refused::Level(200)));
    assert_eq!(registered(&[2, 13], "85", 200, 1, 1), Ok((85, 0x0201, [2, 13, -1, -1, -1, -1])));
    assert_eq!(Refused::TooMany(7).said(), (line::TOO_MANY, Some(7)));
    assert_eq!(Refused::Level(200).said(), (line::LEVEL_RANGE, Some(200)));
    assert_eq!((Refused::NoJob.said().0, Refused::NotNumber.said().0), (line::NO_JOB, line::NOT_NUMBER));
}

#[test]
fn a_search_is_made_of_what_is_typed_and_picked() {
    assert_eq!(sought("", 200, (0, 0), (0, 0)), Ok(Sought { level: 0, place: 0, job: -1 }));
    assert_eq!(sought("70", 200, (2, 1), (3, 2)), Ok(Sought { level: 70, place: 0x0201, job: 13 }));
    // The first group's first job is 1; the client lets a slash by and reads what stands before it.
    assert_eq!(sought("7/", 200, (0, 0), (1, 0)), Ok(Sought { level: 7, place: 0, job: 1 }));
    assert_eq!(sought("x", 200, (0, 0), (0, 0)), Err(Refused::NotNumber));
    assert_eq!(sought("300", 200, (0, 0), (0, 0)), Err(Refused::Level(200)));
    assert_eq!(title("Searching - ", 0, None, None), "Searching - ");
    assert_eq!(title("Searching - ", 70, Some(("Geffen", "Tower F1")), Some("Mage")), "Searching -  Level: 70 Map: Geffen Tower F1 Job: Mage");
}

#[test]
fn an_ad_is_one_of_those_sought_by_its_level_its_place_and_its_jobs() {
    let ad = ad(1, 60, 0x0201, [2, 13, -1, -1, -1, -1]);
    assert!(Sought::NOTHING_YET.takes(&ad) && Sought::ANYTHING.takes(&ad));
    assert!(Sought { level: 60, place: 0, job: -1 }.takes(&ad) && Sought { level: 45, place: 0, job: -1 }.takes(&ad));
    assert!(!Sought { level: 44, place: 0, job: -1 }.takes(&ad) && !Sought { level: 61, place: 0, job: -1 }.takes(&ad));
    assert!(Sought { level: 0, place: 0x0201, job: 13 }.takes(&ad));
    assert!(!Sought { level: 0, place: 0x0200, job: -1 }.takes(&ad) && !Sought { level: 0, place: 0, job: 5 }.takes(&ad));
}

#[test]
fn a_page_with_more_to_come_ends_in_a_row_that_asks_for_the_next() {
    let mut board = Board::default();
    assert_eq!(search_pressed(&mut board, Sought::NOTHING_YET, Sought { level: 0, place: 0, job: -1 }), Search::Asked(0));
    board.add((1..=10).map(|index| (index, entry(index))), true);
    assert_eq!((board.rows.len(), board.rows.get(&11), board.last, board.more), (11, Some(&Row::More), 10, true));
    // The next page is asked from the last ad; the server gives that ad again, which changes nothing.
    assert_eq!(next_page(&mut board), Search::Asked(10));
    board.rows.remove(&11);
    board.add([(10, entry(10)), (11, entry(11)), (12, entry(12))], false);
    assert_eq!((board.rows.len(), board.last, board.more), (12, 12, false));
    assert!(board.rows.values().all(|row| *row != Row::More));
    // The same search again asks nothing; another empties the list.
    let same = Sought { level: 0, place: 0, job: -1 };
    assert_eq!(search_pressed(&mut board, same, same), Search::Same);
    assert_eq!(search_pressed(&mut board, same, Sought { level: 50, ..same }), Search::Asked(0));
    assert!(board.rows.is_empty());
}

#[test]
fn an_ad_whose_place_the_list_does_not_have_is_not_shown() {
    let mut board = Board::default();
    let strange = Entry::of(&ad(3, 50, 0x7700, [1, -1, -1, -1, -1, -1]), &regions());
    board.add([(2, entry(2)), (3, strange)], false);
    assert_eq!((board.rows.len(), board.last), (1, 2));
}

#[test]
fn an_ad_told_of_waits_while_more_pages_are_to_come() {
    let mut board = Board::default();
    let any = Sought { level: 0, place: 0, job: -1 };
    let new = ad(40, 50, 0x0100, [1, -1, -1, -1, -1, -1]);
    // The list is open and nothing was searched yet (`more` is true at first): the ad waits.
    assert!(board.advertised(&new, entry(40), Some((any, true))));
    assert_eq!((board.rows.len(), board.waiting.len()), (0, 1));
    // The last page takes those that waited in.
    board.add([(1, entry(1))], false);
    assert_eq!((board.rows.len(), board.waiting.len(), board.more), (2, 0, false));
    // Then an ad goes straight into the list; one not sought is passed by; the box not ticked tells nothing.
    assert!(!board.advertised(&ad(41, 50, 0x0100, [1, -1, -1, -1, -1, -1]), entry(41), Some((any, false))));
    assert!(board.rows.contains_key(&41));
    assert!(!board.advertised(&ad(42, 50, 0x0100, [1, -1, -1, -1, -1, -1]), entry(42), Some((Sought { level: 90, ..any }, true))));
    assert!(!board.rows.contains_key(&42));
    board.set_jobs(41, [7, -1, -1, -1, -1, -1]);
    assert!(matches!(board.rows.get(&41), Some(Row::Ad(entry)) if entry.jobs[0] == 7));
    board.remove(41);
    assert!(!board.rows.contains_key(&41));
}

#[test]
fn with_the_list_closed_an_ad_is_told_of_by_the_box_and_the_last_search() {
    let mut board = Board::default();
    let new = ad(40, 50, 0x0100, [1, -1, -1, -1, -1, -1]);
    assert!(!board.advertised(&new, None, None));
    board.alarm = true;
    assert!(board.advertised(&new, None, None));
    (board.searched, board.sought) = (true, Sought { level: 90, place: 0, job: -1 });
    assert!(!board.advertised(&new, None, None));
    assert!(board.rows.is_empty() && board.waiting.is_empty());
}

#[test]
fn many_ads_that_waited_come_in_eleven_at_a_time() {
    let mut board = Board::default();
    board.waiting = (100..125).map(|index| (index, entry(index).unwrap())).collect();
    board.add([(1, entry(1))], false);
    // Eleven came in and the rest wait behind a row that asks for more, though the server has no more.
    assert_eq!((board.rows.len(), board.waiting.len(), board.more), (13, 14, false));
    assert_eq!(board.rows.get(&2), Some(&Row::More));
    assert_eq!(next_page(&mut board), Search::Flushed);
    assert_eq!(board.waiting.len(), 3);
    board.reset();
    assert_eq!((board.rows.len(), board.waiting.len(), board.more, board.last), (0, 0, true, -1));
}

#[test]
fn the_servers_answers_about_ones_own_ad() {
    assert_eq!([register_refused(0), register_refused(1), register_refused(2)], [None, Some(line::REGISTER_FAILED), Some(line::PENDING)]);
    assert_eq!([deleted(0), deleted(1)], [(None, true), (None, true)]);
    assert_eq!([deleted(2), deleted(3), deleted(9)], [(Some(line::DELETE_FAILED), false), (Some(line::NO_AD), true), (None, false)]);
}

#[test]
fn the_jobs_of_ones_ad_are_changed_in_their_places() {
    let listed = listed(&[13, 2, 30, -1, 7, -1]);
    assert_eq!(listed, [(2, true), (13, true), (30, true)]);
    assert_eq!(changed(&listed), Some([2, 13, 30, -1, -1, -1]));
    assert_eq!(changed(&[(2, false), (13, true), (30, false)]), Some([-1, 13, -1, -1, -1, -1]));
    assert_eq!(changed(&[(2, false), (13, false)]), None);
}

#[test]
fn the_window_of_ones_ad_counts_and_moves_its_picture() {
    assert_eq!(own_title("Recruit party members", 80, 200), "Recruit party members - 80~95");
    assert_eq!(own_title("Recruit party members", 195, 200), "Recruit party members - 195~200");
    assert_eq!([frame(0, 0), frame(0, 999), frame(0, 4000), frame(0, 5000), frame(0, 6000)], [1, 1, 5, 0, 1]);
    assert_eq!([frame(2, 16_000), frame(2, 17_000)], [17, 0]);
    assert_eq!(frame_file(0, 3), "basic_interface/seekparty/type_00/seekparty_bibibic_00_3.bmp");
    assert_eq!(frame_file(3, 11), "basic_interface/seekparty/type_03/seekparty_달기린_03_11.bmp");
    assert_eq!(frame_file(4, 0), "basic_interface/seekparty/type_04/seekparty_boya_04_0.bmp");
    assert_eq!([lasted(0), lasted(61_500), lasted(3_723_000)], ["00:00:00", "00:01:01", "01:02:03"]);
}
