//! The guild window's fifth tab: those expelled (`UIGuildBanishedMemberWnd`).

use super::*;

/// The widths of the columns of the expelled (`g_guildBanishedColumnWidths`).
const EXPELLED_COLUMNS: [i32; 2] = [82, 292];

impl App {
    /// The fifth tab (`UIGuildBanishedMemberWnd::Paint`): those expelled, and why: the name 8, 3 into its row,
    /// fitted into 78, the reason 91, 3, fitted into 288. The client has nothing here that takes one off the
    /// list.
    pub(super) fn guild_expelled(&self) -> String {
        let headers = [Self::words(407, "Name"), Self::words(462, "The Reason of Expulsion")];
        let rows: Vec<String> = self.guild.expelled.iter().map(|(name, reason)| [fitted(8, 3, EXPELLED_COLUMNS[0] - 4, BLACK, name), fitted(EXPELLED_COLUMNS[0] + 9, 3, EXPELLED_COLUMNS[1] - 4, BLACK, reason)].concat()).collect();
        table(&EXPELLED_COLUMNS, &headers, &rows)
    }
}
