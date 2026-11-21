//! The castles in the guild window's first tab: picking one in the list, the menu the right button opens on
//! it, and its two rows (the client's `UIListBoxAgit`, `UIListBoxAgitMenu::SendMsg`, `UIAgitMessageBox::SendMsg`
//! and the `MM_REQ_MOVE_GUILD_AGIT` and `MM_REQ_AGIT_INVESTMENT` arms of `CGameMode::SendMsg`).

use super::*;

impl App {
    /// A press of the castles. `on` is the line the menu was open on when the press came: any press in the tab
    /// has put the menu away by now (`UIGuildInfoManageWnd::OnLBtnDown`).
    pub(super) fn guild_castle_pressed(&mut self, press: crate::press::guild::Press, on: Option<usize>) {
        use crate::press::guild::Press;
        match press {
            // The left button picks the line (`UIListBox::OnLBtnDown`).
            Press::Castle(line) => self.guild.castle = Some(line),
            Press::CastleGo => self.guild_castle_go(on),
            Press::CastleAsk => {
                // The server answers with the castle's economy and defence (`MM_REQ_AGIT_INVESTMENT`).
                if let Some(castle) = self.guild_castle_on(on) {
                    self.send_on_map(&guild::ask_castle(castle));
                }
            }
            _ => {}
        }
        // The box that asks stands where its entry puts it as the screen is written (`wnd::GUILD_ASKS`).
        if matches!(self.guild.asks, Some(GuildAsk::Castle(_))) { self.render() } else { self.redraw(&[&wnd::GUILD]) }
    }

    /// The castle a line of the list stands for: the client takes the castle at that place among those the
    /// server told (`GuildInfo::GetAgitId(m_agitIndex)`), which is the line's own castle as long as the
    /// client's table names every castle told. None for castle 0, of which the client asks nothing.
    fn guild_castle_on(&self, line: Option<usize>) -> Option<u8> {
        self.guild.castles.get(line?).copied().filter(|castle| *castle != 0)
    }

    /// The first row of the menu: the box asks whether to go, with the castle's name and what it costs. Under
    /// a siege (the client's `GetSiegeMode`) the chat says line 3633 in red and nothing is asked; a castle the
    /// client's table does not name asks nothing either.
    fn guild_castle_go(&mut self, on: Option<usize>) {
        if self.pvp.zone.siege {
            return self.chat_line(Self::words(3633, "This feature is not available in WoE territory."), RED);
        }
        if let Some(castle) = self.guild_castle_on(on).filter(|castle| data::castle(*castle).is_some()) {
            self.guild.asks = Some(GuildAsk::Castle(castle));
        }
    }

    /// The right button on a line of the list picks it and opens the menu by it (`UIListBoxAgit::OnRBtnDown`,
    /// `UM_SHOW_AGIT_MENU`); on no line, the menu is put away.
    pub(in crate::web::app) fn guild_castle_menu_opens(&mut self, line: usize) {
        let known = line < self.guild_castles_listed().len();
        (self.guild.castle, self.guild.castle_menu) = if known { (Some(line), Some(line)) } else { (None, None) };
        (self.guild.menu, self.guild.combo) = (None, None);
        self.redraw(&[&wnd::GUILD, &wnd::GUILD_MENU]);
    }

    /// The character is on another map: the castles are forgotten until the server tells them again, which it
    /// does when the first tab of the guild window asks for the guild's numbers (the client's
    /// `CGameMode::InitializeState`: `ClearAgitIds`).
    pub(in crate::web::app) fn guild_castles_moved(&mut self, same: bool) {
        if !same && !self.guild.castles.is_empty() {
            let guild = &mut self.guild;
            (guild.castle, guild.castle_menu) = (None, None);
            guild.castles.clear();
            self.redraw(&[&wnd::GUILD]);
        }
    }
}
