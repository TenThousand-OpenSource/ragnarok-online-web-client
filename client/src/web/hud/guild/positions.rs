//! The guild window's third tab: the positions (`UIGuildPositionManageWnd`). Every place and size is the
//! client's, from its `OnCreate` and `Paint`.

use super::*;
use crate::net::guild::Position;

/// The widths of the columns of the positions (`g_guildPositionColumnWidths`).
const POSITION_COLUMNS: [i32; 6] = [50, 100, 60, 60, 60, 60];

impl App {
    /// The third tab (`UIGuildPositionManageWnd::Paint`): each position's number, what it is called, whether it
    /// invites, expels and opens the guild's storage, and what part of a member's experience it takes. For the
    /// master the name and the tax are fields (30 and 2 letters, on grey 242) and the boxes are pressed; once
    /// something was changed the button that sends it shows (`btn_ok`, 61 from the right, 23 from the bottom).
    pub(super) fn guild_positions(&self) -> String {
        let headers = [(510, "Rank"), (511, "Position Title"), (512, "Invite"), (513, "Expel"), (2499, "Storage"), (514, "Tax %")].map(|(line, otherwise)| Self::words(line, otherwise));
        let rows: Vec<String> = self.guild.positions.iter().map(|position| self.guild_position_row(&self.guild.position_shown(position))).collect();
        let mut tab = table(&POSITION_COLUMNS, &headers, &rows);
        if self.guild.master {
            let hidden = if self.guild.position_edits.is_empty() { " hidden" } else { "" };
            tab.push_str(&format!(r#"<div id="guild-positions-ok"{hidden}>{}</div>"#, picture_press(WIDTH - 61, HEIGHT - 23, "btn_ok", Press::PositionsOk)));
        }
        tab
    }

    /// A position's row, its places counted from the row's corner (the window's 0, 65): the number 8, 3; the
    /// name 59, 1 in 96; the boxes 160, 221 and 282, 3; the tax 343, 1 (for the master a field 26 wide, with
    /// the sign of hundredths 373, 3).
    fn guild_position_row(&self, position: &Position) -> String {
        let [number, name, invite, expel, storage] = [POSITION_COLUMNS[0], POSITION_COLUMNS[1], POSITION_COLUMNS[2], POSITION_COLUMNS[3], POSITION_COLUMNS[4]];
        let tax = number + name + invite + expel + storage + 13;
        let id = position.id;
        let master = self.guild.master;
        let box_of = |x: i32, on: bool, press: Press| check(x, 3, on, &if master { pressed(press) } else { String::new() });
        // What is typed shows the button that sends it.
        let field = |what: &str, x: i32, width: i32, most: usize, value: &str| {
            format!(
                r#"<input id="guild-{what}-{id}" class="a in" style="{};background:{FIELD}" type="text" maxlength="{most}" autocomplete="off" value="{}" oninput="document.getElementById('guild-positions-ok').hidden=false">"#,
                at(x, 1, width, 16),
                escape(&crate::web::app::written(&format!("guild-{what}-{id}")).unwrap_or_else(|| value.to_owned()))
            )
        };
        [
            text(8, 3, BLACK, &id.to_string()),
            if master { field("position", number + 9, name - 4, 30, &position.name) } else { fitted(number + 9, 1, name - 4, BLACK, &position.name) },
            box_of(number + name + 10, position.invites, Press::Invites(id)),
            box_of(number + name + invite + 11, position.expels, Press::Expels(id)),
            box_of(number + name + invite + expel + 12, position.storage, Press::Stores(id)),
            if master { [field("tax", tax, storage / 2 - 4, 2, &position.tax.to_string()), text(tax + POSITION_COLUMNS[5] / 2, 3, BLACK, "%")].concat() } else { text(tax, 1, BLACK, &format!("{} %", position.tax)) },
        ]
        .concat()
    }
}
