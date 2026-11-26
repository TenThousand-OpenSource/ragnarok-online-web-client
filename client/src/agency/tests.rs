//! Tests of the Adventurer's Agency: the forms posted, the answers read, the levels typed, what joining says.

use super::*;

fn asker() -> Asker {
    Asker { account: 2_000_006, character: 150_006, world: "rAthena".to_owned(), token: "0123456789abcdef".to_owned() }
}

fn names(fields: &Fields) -> Vec<&'static str> {
    fields.iter().map(|(name, _)| *name).collect()
}

#[test]
fn the_forms_have_the_clients_fields_in_its_order() {
    let asker = asker();
    assert_eq!(asker.list(3), vec![
        ("GID", "150006".to_owned()),
        ("AID", "2000006".to_owned()),
        ("WorldName", "rAthena".to_owned()),
        ("AuthToken", "0123456789abcdef".to_owned()),
        ("page", "3".to_owned()),
    ]);
    assert_eq!(names(&asker.mine()), ["AID", "GID", "WorldName", "AuthToken"]);
    assert_eq!(names(&asker.info(2_000_007)), ["AID", "GID", "AuthToken", "WorldName", "QueryAID"]);
    assert_eq!(asker.info(2_000_007)[4].1, "2000007");
    assert_eq!(asker.stop(2_000_007), vec![
        ("AID", "2000006".to_owned()),
        ("WorldName", "rAthena".to_owned()),
        ("AuthToken", "0123456789abcdef".to_owned()),
        ("MasterAID", "2000007".to_owned()),
    ]);
}

#[test]
fn a_search_names_only_what_is_asked_for() {
    let asker = asker();
    let nothing = Notice { tanker: false, healer: false, dealer: false, assist: false, least: 94, most: 104, ..Notice::default() };
    assert_eq!(names(&asker.search(&nothing, 1)), ["GID", "AID", "WorldName", "AuthToken", "MinLV", "MaxLV", "page"]);
    let all = Notice { purpose: 3, memo: "odin".to_owned(), least: 1, most: 99, ..Notice::default() };
    let fields = asker.search(&all, 2);
    assert_eq!(names(&fields), ["GID", "AID", "WorldName", "AuthToken", "MinLV", "MaxLV", "Tanker", "Dealer", "Healer", "Assist", "Type", "Memo", "page"]);
    let values = [4, 5, 10, 11, 12].map(|place| fields[place].1.as_str());
    assert_eq!(values, ["1", "99", "3", "odin", "2"]);
}

#[test]
fn a_notice_is_put_up_with_every_field() {
    let wanted = Notice { tanker: true, healer: false, dealer: true, assist: false, ..Notice::default() };
    let notice = Notice { least: 90, most: 99, purpose: 4, memo: "ET".to_owned(), ..wanted };
    let fields = asker().add(&notice, "Six");
    assert_eq!(names(&fields), ["AID", "GID", "AuthToken", "WorldName", "CharName", "MinLV", "MaxLV", "Tanker", "Healer", "Dealer", "Assist", "Type", "Memo"]);
    let values: Vec<&str> = fields.iter().skip(4).map(|(_, value)| value.as_str()).collect();
    assert_eq!(values, ["Six", "90", "99", "1", "0", "1", "0", "4", "ET"]);
}

#[test]
fn a_form_is_written_as_the_service_reads_it() {
    let body = form(&vec![("AID", "2000006".to_owned()), ("Memo", "a b".to_owned())]);
    let expected = [
        "------roweb-agency-7f3a9c1e\r\nContent-Disposition: form-data; name=\"AID\"\r\n\r\n2000006\r\n",
        "------roweb-agency-7f3a9c1e\r\nContent-Disposition: form-data; name=\"Memo\"\r\n\r\na b\r\n",
        "------roweb-agency-7f3a9c1e--\r\n",
    ]
    .concat();
    assert_eq!(body, expected);
}

/// A notice as rAthena's `s_party_booking_entry::to_json` writes one.
const ENTRY: &str = concat!(
    r#"{ "AID": 2000007, "GID": 150007, "CharName": "Seven", "WorldName": "rAthena", "Tanker": 1, "Healer": 0, "Dealer": 1, "#,
    r#""Assist": 0, "MinLV": 90, "MaxLV": 99, "Memo": "ET run", "Type": 4}"#
);

fn seven() -> Notice {
    Notice {
        tanker: true,
        healer: false,
        dealer: true,
        assist: false,
        least: 90,
        most: 99,
        purpose: 4,
        memo: "ET run".to_owned(),
        account: 2_000_007,
        character: 150_007,
        name: "Seven".to_owned(),
    }
}

#[test]
fn the_list_is_read_with_its_pages() {
    let answer = format!(r#"{{ "Type": 1, "totalPage": 3, "data": [{ENTRY}, {{}}] }}"#);
    let (notices, pages) = listed(&answer).unwrap();
    assert_eq!(pages, Some(3));
    assert_eq!(notices[0], seven());
    // What an entry does not say is as the client fills it in.
    assert_eq!(notices[1], Notice { purpose: 1, ..Notice::default() });
    assert_eq!(notices[0].levels(), "90~99");
    assert_eq!(notices[0].roles(), [true, false, true, false]);
    assert_eq!(listed(r#"{ "Type": 1, "totalPage": 0, "data": [] }"#), Some((Vec::new(), None)));
    assert_eq!(listed(r#"{ "Type": 3 }"#), None);
    assert_eq!(listed("Error"), None);
}

#[test]
fn ones_own_notice_and_anothers_are_read() {
    assert_eq!(mine(&format!(r#"{{ "Type": 1, "data": {ENTRY} }}"#)), Some(Some(seven())));
    assert_eq!(mine(r#"{ "Type": 1 }"#), Some(None));
    assert_eq!(mine(r#"{ "Type": 2 }"#), None);
    assert_eq!(info(&format!(r#"{{ "Type": 1, "data": [{ENTRY}] }}"#)), Some(Some(seven())));
    assert_eq!(info(r#"{ "Type": 1 }"#), Some(None));
    assert_eq!(info("Error"), None);
    assert_eq!([done(r#"{ "Type": 1 }"#), done(r#"{ "Type": 2 }"#), done("Error")], [Some(true), Some(false), None]);
}

#[test]
fn the_levels_begin_around_ones_own_and_a_level_typed_is_digits() {
    assert_eq!(levels_around(99), (94, 104));
    assert_eq!(levels_around(3), (1, 8));
    assert_eq!(level_typed("80", 94), 80);
    assert_eq!(level_typed("", 94), 94);
    assert_eq!(level_typed("8o", 94), 94);
    assert_eq!(level_typed("-5", 94), 94);
}

#[test]
fn the_arrows_are_live_where_there_is_a_page_to_go_to() {
    assert_eq!(arrows(1, 1), (false, false));
    assert_eq!(arrows(1, 3), (false, true));
    assert_eq!(arrows(2, 3), (true, true));
    assert_eq!(arrows(3, 3), (true, false));
    assert_eq!(arrows(5, 3), (true, false));
}

#[test]
fn joining_says_what_the_client_says() {
    assert_eq!(join_result(0, true), Some((3501, Named::First, true, false)));
    assert_eq!(join_result(0, false), Some((3498, Named::Second, true, false)));
    assert_eq!(join_result(1, true), Some((3502, Named::First, false, false)));
    assert_eq!(join_result(1, false), Some((3499, Named::Second, false, false)));
    assert_eq!(join_result(9, false), Some((3494, Named::Nobody, false, true)));
    assert_eq!(join_result(5, false), join_result(6, false));
    assert_eq!(join_result(12, false), Some((608, Named::First, false, false)));
    assert_eq!(join_result(14, false), None);
    let notice = seven();
    assert_eq!(join_refused(&notice, 80, false), Some(3416));
    assert_eq!(join_refused(&notice, 100, false), Some(3416));
    assert_eq!(join_refused(&notice, 95, true), Some(79));
    assert_eq!(join_refused(&notice, 95, false), None);
}
