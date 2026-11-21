//! The castles the guild holds, as the server tells of them (`crate::net::guild`, the file `castles.rs`): the
//! list, a castle's economy and defence, and why the character was not moved to one. What each does is the
//! client's `CGameMode::ReceiveGuildAgitList`, `ReceiveAgitNotice` and `ReceiveGuildAgitMoveResult`.

use super::*;

impl App {
    /// Reads a packet of the castles; `false` for any other.
    pub(super) fn guild_castles_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // The list is told anew whole; the list box is filled anew with nothing picked in it
            // (`UIGuildInfoManageWnd::RefreshAgitList`: `UIListBox::Clear`).
            guild::ZC_GUILD_AGIT_INFO => {
                if let Some(castles) = guild::castles(packet) {
                    let guild = &mut self.guild;
                    (guild.castles, guild.castle, guild.castle_menu) = (castles, None, None);
                }
            }
            // A box tells it, and only while the first tab of the guild window is there
            // (`g_windowMgr.guildInfoManageWnd`); a box that is open already is not put away for it
            // (`UIWindowMgr::Confirm` with nothing to replace).
            guild::ZC_REQ_ACK_AGIT_INVESTMENT => {
                if let (true, None, Some(told)) = (self.guild.open && self.guild.tab == 0, &self.guild.asks, guild::investment(packet)) {
                    self.guild.asks = Some(GuildAsk::Invested(told));
                    // The box stands where its entry puts it as the screen is written (`wnd::GUILD_ASKS`).
                    self.render();
                }
            }
            // The chat says why, in red (the client's 0xFF): line 1090 for too little zeny, line 3633 for a
            // place this is not done from. Anything else says nothing.
            guild::ZC_REQ_ACK_MOVE_GUILD_AGIT => match guild::castle_move(packet) {
                Some(guild::moved::NO_ZENY) => self.chat_line(Self::words(1090, "You do not have enough zeny."), RED),
                Some(guild::moved::NOT_HERE) => self.chat_line(Self::words(3633, "This feature is not available in WoE territory."), RED),
                _ => {}
            },
            _ => return false,
        }
        true
    }
}
