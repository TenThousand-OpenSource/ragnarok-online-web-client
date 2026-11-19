//! The tests of the quest journal's rules (`quests.rs`).

use super::*;

#[test]
fn a_quest_lies_under_the_tab_of_its_state() {
    assert_eq!(tab_of(false, true), tab::PROCESS);
    assert_eq!(tab_of(false, false), tab::HOLD);
    assert_eq!(tab_of(true, true), tab::WAIT);
    assert_eq!(tab_of(true, false), tab::WAIT);
}

#[test]
fn a_filter_takes_its_icons_and_the_other_filter_what_none_takes() {
    let filters = vec![(0, Vec::new()), (1, Vec::new()), (2, vec!["ico_ep.bmp".to_owned(), "ico_gq.bmp".to_owned()]), (4, vec!["ico_dq.bmp".to_owned()])];
    assert!(in_filter(0, "ico_dq.bmp", &filters));
    assert!(in_filter(2, "ico_ep.bmp", &filters));
    assert!(!in_filter(2, "ico_dq.bmp", &filters));
    assert!(in_filter(4, "ico_dq.bmp", &filters));
    assert!(in_filter(1, "", &filters));
    assert!(in_filter(1, "ico_jq.bmp", &filters));
    assert!(!in_filter(1, "ico_dq.bmp", &filters));
    // A filter the client's table does not have takes nothing.
    assert!(!in_filter(7, "ico_dq.bmp", &filters));
}

#[test]
fn a_search_looks_in_the_title_and_the_summary() {
    assert!(found("", "Ransom", ""));
    assert!(found("Ran", "Ransom", ""));
    assert!(found("Talk", "Job Change", "Talk to Father"));
    assert!(!found("Poring", "Ransom", "Carapace"));
}

#[test]
fn a_monsters_name_is_the_servers_and_other_things_are_the_clients() {
    assert_eq!(named(1002), Named::Sent);
    assert_eq!(named(20500), Named::Sent);
    assert_eq!(named(0x1099), Named::Message(3558));
    assert_eq!(named(0x109a), Named::Message(3559));
    assert_eq!(named(0x287b), Named::Map);
    assert_eq!(named(4001), Named::Job);
}

#[test]
fn a_hunt_is_written_with_its_levels_and_its_count() {
    let hunt = |levels, killed, goal| Hunt { levels, killed, goal, ..Hunt::default() };
    assert_eq!(hunt_text(&hunt((0, 0), 3, 10), "Poring", "Complete"), "Poring ( 3 / 10 )");
    assert_eq!(hunt_text(&hunt((0, 0), 10, 10), "Poring", "Complete"), "Poring ( Complete )");
    assert_eq!(hunt_text(&hunt((0, 30), 0, 5), "Poring", "Complete"), "Poring Lv.30▼ ( 0 / 5 )");
    assert_eq!(hunt_text(&hunt((10, 0), 0, 5), "Poring", "Complete"), "Poring Lv.10▲ ( 0 / 5 )");
    assert_eq!(hunt_text(&hunt((10, 30), 0, 5), "Poring", "Complete"), "Poring Lv.10~Lv.30 ( 0 / 5 )");
    // Until the server has said how many are wanted there is no count.
    assert_eq!(hunt_text(&hunt((0, 0), 0, 0), "Poring", "Complete"), "Poring");
}

#[test]
fn the_chat_is_told_of_a_hunts_count() {
    assert_eq!(progress_line("[%s] quest - defeating [%s] progress (%d/%d).", "Hunting Porings", "Poring", 3, 10), "[Hunting Porings] quest - defeating [Poring] progress (3/10).");
}

#[test]
fn a_link_shows_its_words_only() {
    assert_eq!(shown("<NAVI>[Luanda]<INFO>com_d02_i,241,255,0,101,0</INFO></NAVI> will reward you."), "[Luanda] will reward you.");
    assert_eq!(shown("Bring <ITEM>Jellopy<INFO>909</INFO></ITEM> x 10"), "Bring Jellopy x 10");
    assert_eq!(shown("^0000FFFather^000000 waits"), "^0000FFFather^000000 waits");
    assert_eq!(shown("1 < 2 <b>"), "1 < 2 <b>");
    assert_eq!(shown("cut <INFO>short"), "cut ");
}

#[test]
fn a_text_loses_its_colours() {
    assert_eq!(uncoloured("^0000FFFather^000000 waits"), "Father waits");
    assert_eq!(uncoloured("2^3 and ^ff"), "2^3 and ^ff");
}

#[test]
fn a_mark_is_an_action_of_the_emotions() {
    assert_eq!(mark_action(0), Some(80));
    assert_eq!(mark_action(5), Some(85));
    assert_eq!(mark_action(6), Some(93));
    assert_eq!(mark_action(10), Some(97));
    assert_eq!(mark_action(11), None);
    assert_eq!(mark_action(crate::net::quests::NO_MARK), None);
}

#[test]
fn a_mark_with_a_colour_is_on_the_minimap() {
    assert_eq!(minimap_picture(1, 2).as_deref(), Some("minimap/quest_1_2.bmp"));
    assert_eq!(minimap_picture(1, 0), None);
    assert_eq!(minimap_picture(9999, 1), None);
}
