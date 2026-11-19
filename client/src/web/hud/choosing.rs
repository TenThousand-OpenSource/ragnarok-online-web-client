//! The windows of the lists a skill or a card opens to pick one thing of, in the client's three shapes: the
//! places of a warp (`UIChooseWarpWnd`), rows with a picture (`UIMakingArrowListWnd`, `UISpellListWnd`,
//! `UIItemCompositionWnd`: all `UIItemIdentifyWnd`, drawn by `pick_list`), and lines under a title bar
//! (`UIMakeTargetListWnd`, `UIItemRepairWnd`, `UIWeaponRefineWnd`, and `UIMakeTargetProcessWnd` for the thing
//! being forged).

use super::*;
use crate::item_names::Named;
use crate::net::choosing::{CANCEL, RANDOM, takes_more};
use crate::press::choosing::Press;
use crate::web::app::{Choice, Shape};

/// A list box of the client's (`UIListBox`): lines 16 high on a ground of 240 grey, the words 3 in, the
/// picked line on a band. A press picks a line, a double click picks it and confirms.
fn line_list(x: i32, y: i32, width: i32, height: i32, lines: &[String], picked: Option<usize>) -> String {
    let rows: String = lines
        .iter()
        .enumerate()
        .map(|(row, words)| {
            let band = if picked == Some(row) { ";background:#c6d6f7" } else { "" };
            format!(
                r#"<div class="t" style="height:16px;line-height:15px;padding-left:3px;white-space:nowrap;cursor:default{band}" {} {}>{}</div>"#,
                pressed(Press::Pick(row)),
                used(Use::Choice(row)),
                escape(words)
            )
        })
        .collect();
    format!(r#"<div class="a scroll" style="{};background:#f0f0f0">{rows}</div>"#, at(x, y, width, height))
}

/// What an item is called by its number alone, as the client names the rows of these lists
/// (`item.SetItemId(id)`, identified, `DisplayName(0)`).
fn called(id: u32) -> String {
    data::item_title(&Named { id, identified: true, ..Named::default() }, false)
}

impl App {
    /// The places of Teleport or Warp Portal (`UIChooseWarpWnd`, 280 by 120 in the frame of the system
    /// boxes): "Select an Area to Warp" (line 213) at 12, 9, the list at 12, 26 (256 by 65), OK and Cancel 92
    /// and 46 from the right, 24 from the bottom. "Random" is written as line 212, the client's own last row
    /// "cancel" as line 218, a map by the name the client's table has for it.
    pub(in crate::web) fn choose_place_window(&self) -> String {
        let Some(Choice::Warp(_, places)) = &self.choosing.open else { return String::new() };
        let lines: Vec<String> = places
            .iter()
            .map(|place| match place.as_str() {
                RANDOM => Self::words(212, "Random Area"),
                CANCEL => Self::words(218, "Cancel"),
                map => data::map_sign(map).map_or_else(|| map.to_owned(), |sign| sign.name),
            })
            .collect();
        format!(
            "{}{}{}{}{}",
            system_box(0, 280, 120),
            text(12, 9, BLACK, &Self::words(213, "Select an Area to Warp")),
            line_list(12, 26, 256, 65, &lines, self.choosing.picked),
            picture_press(188, 96, "btn_ok", Press::Ok),
            picture_press(234, 96, "btn_cancel", Press::Cancel)
        )
    }

    /// The lists whose rows have a picture, 200 by 200 (`pick_list`): what arrows are made of and what can
    /// be cooked (titled "LIST": `UIMakingArrowListWnd::Paint`), the spells of Auto Spell (line 697), the
    /// equipment a card goes into (line 522 and the card's name in brackets).
    pub(in crate::web) fn choose_pictured_window(&self) -> String {
        let does = Picking { pick: |row| Press::Pick(row).into(), used: Use::Choice, ok: Press::Ok.into(), cancel: Press::Cancel.into() };
        let (title, rows) = match &self.choosing.open {
            Some(Choice::Arrows(items) | Choice::Mix(_, items)) => {
                let rows = items.iter().map(|id| (Self::icon(data::item(*id).map(|(_, resource)| resource).as_deref()), called(*id))).collect();
                ("LIST".to_owned(), rows)
            }
            Some(Choice::Spells(skills)) => {
                let rows = skills
                    .iter()
                    .map(|id| {
                        let known = data::skill(*id as u16);
                        (Self::icon(known.as_ref().map(|(resource, ..)| resource.to_lowercase()).as_deref()), known.map_or_else(|| id.to_string(), |(_, name, _)| name))
                    })
                    .collect();
                (Self::words(697, "Spell List"), rows)
            }
            Some(Choice::Compound(card, places)) => {
                let card = self.bag.items.iter().find(|item| item.index == *card).map(Self::item_name).unwrap_or_default();
                (format!("{} ({card})", Self::words(522, "Item Compounding")), self.carried_rows(places))
            }
            _ => return String::new(),
        };
        pick_list(&title, &rows, self.choosing.picked, does)
    }

    /// The lists of lines, 280 by 150 under a title bar: what can be forged or brewed ("Manufacturing List",
    /// line 425), what can be repaired (812), the weapons to refine (910); the list at 12, 22 (256 by 95), OK
    /// and Cancel 91 and 46 from the right, 24 from the bottom. A thing picked to be forged is shown in the
    /// same place with what it needs (`forge_pane`).
    pub(in crate::web) fn choose_line_window(&self) -> String {
        let listed = |things: &[crate::net::choosing::Listed]| things.iter().map(|thing| called(thing.id)).collect::<Vec<String>>();
        let (title, inside) = match &self.choosing.open {
            Some(Choice::Make(items)) => (Self::words(425, "Manufacturing List"), line_list(12, 22, 256, 95, &items.iter().map(|id| called(*id)).collect::<Vec<_>>(), self.choosing.picked)),
            Some(Choice::Repair(things)) => (Self::words(812, "Repairable items"), line_list(12, 22, 256, 95, &listed(things), self.choosing.picked)),
            Some(Choice::Refine(things)) => (Self::words(910, "Upgradeable weapons"), line_list(12, 22, 256, 95, &listed(things), self.choosing.picked)),
            Some(Choice::Forge(make, put)) => (String::new(), Self::forge_pane(*make, put)),
            _ => return String::new(),
        };
        format!("{}{}{inside}{}{}", title_bar(280, &title), panel(280, 150), picture_press(189, 126, "btn_ok", Press::Ok), picture_press(234, 126, "btn_cancel", Press::Cancel))
    }

    /// A thing about to be forged (`UIMakeTargetProcessWnd::Paint`): its name and " Create" (line 426) in the
    /// title bar at 16, 2, its name and "'s required materials" (427) at 16, 22, the lines of
    /// `metalprocessitemlist.txt` for it from 42 down, 20 apart; and, for what takes more, three slots 32
    /// apart from 4 in, 53 from the bottom, each with what was put into it (pressed, it comes out: the client
    /// has it dragged out).
    fn forge_pane(make: u32, put: &[u32]) -> String {
        let name = called(make);
        let mut pane = format!(
            r#"<div class="a t titled" style="left:16px;top:1px;white-space:nowrap">{}{}</div>{}"#,
            escape(&name),
            escape(&Self::words(426, " Create")),
            text(16, 22, BLACK, &format!("{name}{}", Self::words(427, "'s required materials")))
        );
        for (row, line) in data::materials(make).iter().enumerate() {
            pane.push_str(&text(16, 42 + row as i32 * 20, BLACK, line));
        }
        if takes_more(make) {
            for slot in 0..3 {
                let inside = put.get(slot).map(|id| Self::icon(data::item(*id).map(|(_, resource)| resource).as_deref())).unwrap_or_default();
                pane.push_str(&format!(
                    r#"<div class="a cell" style="left:{}px;top:97px;background:url('{}') 0 0 no-repeat" {}>{inside}</div>"#,
                    4 + slot as i32 * 32,
                    picture("basic_interface/itemwin_mid.bmp"),
                    pressed(Press::Material(slot))
                ));
            }
        }
        pane
    }

    /// The rows of a list with pictures (`pick_list`) for things carried, by where each is: its picture and
    /// its name. A thing that is no longer carried keeps its row, empty, so that the rows stay those of the
    /// list.
    pub(super) fn carried_rows(&self, places: &[u16]) -> Vec<(String, String)> {
        places
            .iter()
            .map(|index| {
                let Some(item) = self.bag.items.iter().find(|item| item.index == *index) else { return Default::default() };
                let resource = data::item_as(item.id, item.identified).map(|(_, resource)| resource);
                (Self::icon(resource.as_deref()), Self::item_name(item))
            })
            .collect()
    }

    /// Which of the three windows the open list is in.
    pub(in crate::web) fn choosing_in(&self, shape: Shape) -> bool {
        self.choosing.open.as_ref().is_some_and(|choice| choice.shape() == shape)
    }
}
