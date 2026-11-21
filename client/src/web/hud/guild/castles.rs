//! The castles in the guild window's first tab: the list of those the guild holds (the client's
//! `UIListBoxAgit`, made by `UIGuildInfoManageWnd::OnCreate`), the menu the right button opens on one
//! (`UIListBoxAgitMenu`), the box that asks before the character is moved to one (`UIAgitMessageBox`), and the
//! box that tells a castle's economy and defence (`CGameMode::ReceiveAgitNotice`).

use super::*;
use crate::guild::{CASTLE_LINE, CASTLE_LIST, CASTLE_MENU, castle_investment, castle_menu_place, castle_question};

/// The size of the box that asks before going to a castle (`UIListBoxAgitMenu::SendMsg`:
/// `InitializeMessageBox(text, 310, 130)`), and of the one that tells of a castle (`ReceiveAgitNotice`:
/// `Confirm(text, 0, …, "Message", 250, 130, 0)`).
pub(in crate::web) const CASTLE_ASK: (i32, i32) = (310, 130);
pub(in crate::web) const CASTLE_TOLD: (i32, i32) = (250, 130);

impl App {
    /// The castles as the list has them: the name of each that the client's table names, in the order the
    /// server told them (`UIGuildInfoManageWnd::RefreshAgitList`, which leaves out a castle with no name).
    pub(in crate::web) fn guild_castles_listed(&self) -> Vec<String> {
        self.guild.castles.iter().filter_map(|castle| data::castle(*castle)).map(|castle| castle.name).collect()
    }

    /// The line of the castles (`UIGuildInfoManageWnd::Paint`: "Castles Owned : " and what the server words
    /// of them, at 8, 129) with the list over it: 95 by 158, 19 after the words of line 333 and so 11 after
    /// their end, a line 13 high with its words 3 in, the line picked on a band (`UIListBox::Paint`). The list
    /// is white as the panel is and lies over what the server words of the castles ("None Taken", "One
    /// Castle"…), so those words are never seen in the client and are not written here.
    ///
    /// The left button picks a line, the right button picks it and opens the menu by it. The band is the
    /// colour `hud/choosing.rs` has for the pick of a `UIListBox`. Past twelve castles the list scrolls, with
    /// no bar drawn (the client's has its `UIScrollBar` 13 from the right then).
    pub(super) fn guild_castles(&self) -> String {
        let lines: String = self.guild_castles_listed()
            .iter()
            .enumerate()
            .map(|(line, name)| {
                let band = if self.guild.castle == Some(line) { ";background:#c6d6f7" } else { "" };
                [
                    format!(r#"<div class="t" style="height:{CASTLE_LINE}px;line-height:{CASTLE_LINE}px;padding-left:3px;cursor:default{band}" "#),
                    format!(r#"{} {}>{}</div>"#, pressed(Press::Castle(line)), menued(Menu::GuildCastle(line)), escape(name))
                ].concat()
            })
            .collect();
        [
            format!(r#"<div class="a t" style="left:8px;top:129px;display:flex;overflow:visible;color:{BLACK}">"#),
            format!(
                r#"<span>{}</span><span style="flex:none;width:11px;overflow:hidden;white-space:pre"> : </span>"#,
                escape(&Self::words(333, "Castles Owned"))
            ),
            format!(
                r#"<div style="position:relative;width:0;height:0"><div class="a scroll" style="{};background:{PANEL}">{lines}</div>{}</div></div>"#,
                at(0, 0, CASTLE_LIST.0, CASTLE_LIST.1),
                self.guild_castle_menu()
            ),
        ]
        .concat()
    }

    /// The menu on a castle, a part of the guild window as the client's is (`m_agitMenu`, a child of the tab):
    /// 120 by 35 by the right edge of the line pressed, white with the four corners of the small system box,
    /// its two rows 16 high with their words 3 in: going there (line 3629) and the castle's economy and
    /// defence (line 3630). (The client clears it in cell 6, 6 of the skin's colours and fills it in cell 2, 2,
    /// both white in `colorchip.bmp`; the row under the pointer it marks in cell 6, 2, where this page has the
    /// one colour of every menu's row.)
    fn guild_castle_menu(&self) -> String {
        let Some(line) = self.guild.castle_menu else { return String::new() };
        let (x, y) = castle_menu_place(line);
        let (width, height) = CASTLE_MENU;
        let corner = |name: &str, x: i32, y: i32| art(&format!("basic_interface/sysboxs_{name}.bmp"), x, y, 7, 7);
        [
            format!(r#"<div class="a" style="{};background:{PANEL};z-index:1">"#, at(x, y, width, height)),
            corner("lu", 0, 0),
            corner("ld", 0, height - 7),
            corner("ru", width - 7, 0),
            corner("rd", width - 7, height - 7),
            format!(
                r#"<div class="a" style="left:0;top:1px;width:{width}px">{}{}</div></div>"#,
                menu_row(&Self::words(3629, "Go to destination"), Press::CastleGo),
                menu_row(&Self::words(3630, "Commercial/Defense Check"), Press::CastleAsk)
            ),
        ]
        .concat()
    }

    /// The two boxes of the castles, drawn as the guild's other questions are (`guild_question`); nothing when
    /// neither is open.
    ///
    /// Before going to a castle (`UIAgitMessageBox`, 310 by 130): the title bar with line 3640 ("Message"),
    /// the panel and the bar of buttons, line 3631 with the castle's name and its two costs at 10, 22 in 290
    /// by 75, OK and Cancel 24 from the bottom, each 5 and its own width from the one after it
    /// (`InitializeMessageBox`).
    ///
    /// A castle's economy and defence (the client's `UIMessageBox` of kind 0, 250 by 130, titled "Message" by
    /// the program itself): the title bar and the panel, line 3632 filled at 10, 22 in 230 by 75, OK 46 from
    /// the right and 24 from the bottom.
    pub(super) fn guild_castle_box(&self) -> Option<String> {
        let words_at = |width: i32, words: &str| format!(
            r#"<div class="a t" style="{};white-space:pre-wrap;color:{BLACK}">{}</div>"#,
            at(10, 22, width - 20, CASTLE_ASK.1 - 55),
            escape(words)
        );
        match self.guild.asks.as_ref()? {
            GuildAsk::Castle(number) => {
                let castle = data::castle(*number).unwrap_or_default();
                let line = Self::words(3631, "Would you like to move to %s Management Territory?\\n%d Zeny will be consumed with every move.");
                let (width, height) = CASTLE_ASK;
                Some([
                    title_bar(width, &Self::words(3640, "Message")),
                    panel(width, height),
                    words_at(width, &castle_question(&line, &castle.name, castle.cost, castle.siege_cost)),
                    picture_press(width - 47 * 2, height - 24, "btn_ok", Press::Yes),
                    picture_press(width - 47, height - 24, "btn_cancel", Press::No),
                ].concat())
            }
            GuildAsk::Invested(told) => {
                let name = data::castle(told.castle).map(|castle| castle.name).unwrap_or_default();
                let line = Self::words(3632, "\\nManagement Territory: %s\\n\t\\nDefense: %d / %d\\nCommercial: %d / %d");
                let (width, height) = CASTLE_TOLD;
                Some([
                    title_bar(width, "Message"),
                    panel_ground_round(0, 17, width, height - 17),
                    words_at(width, &castle_investment(&line, &name, told.economy, told.defense)),
                    picture_press(width - 46, height - 24, "btn_ok", Press::Yes),
                ].concat())
            }
            _ => None,
        }
    }
}
