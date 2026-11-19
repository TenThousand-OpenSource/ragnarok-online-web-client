//! The guild window's sixth tab: the notice (`UIGuildNoticeWnd`).

use super::*;

impl App {
    /// The sixth tab (`UIGuildNoticeWnd::OnCreate`, `Paint`): the notice's title (a field at 50, 66, the
    /// window's width less 60 by 16, 60 letters) and what it says (a field of many lines at 5, 104, the width
    /// less 10 by 156, 119 letters), both on grey 242; under the text how much of the 119 is used, ending 7
    /// from the right edge at 266; and for the master the button that sends them (`btn_send`, 46 from the
    /// right, 24 from the bottom). There is no other button: the fields are typed into as they stand.
    ///
    /// The count is of bytes, as the client's and the server's limits are; the title is cut to 59 and the text
    /// to 119 as it is sent (`MM_GUILD_NOTICE`, rAthena's `MAX_GUILDMES1` 60 and `MAX_GUILDMES2` 120 with
    /// their closing zero). Decided here: one who is not the master cannot type into them (the client lets
    /// anyone type and sends nothing).
    pub(super) fn guild_notice_tab(&self) -> String {
        let locked = if self.guild.master { "" } else { " readonly" };
        let notice = crate::web::app::written("guild-notice").filter(|_| self.guild.master).unwrap_or_else(|| self.guild.notice.clone());
        let subject = crate::web::app::written("guild-subject").filter(|_| self.guild.master).unwrap_or_else(|| self.guild.subject.clone());
        let mut tab = [
            text(5, 67, BLACK, &format!("{}:", Self::words(515, "Title"))),
            [
                format!(r#"<input id="guild-subject" class="a in" style="{};background:{FIELD}" type="text" maxlength="59" "#, at(50, 66, WIDTH - 60, 16)),
                format!(r#"autocomplete="off" value="{}"{locked}>"#, escape(&subject))
            ].concat(),
            text(5, 86, BLACK, &Self::words(516, "Contents")),
            [
                format!(r#"<textarea id="guild-notice" class="a in viewer" style="{};resize:none;background:{FIELD};"#, at(5, 104, WIDTH - 10, 156)),
                format!(r#"line-height:15px;white-space:pre-wrap;word-break:break-all" maxlength="119" "#),
                format!(r#"oninput="document.getElementById('guild-letters').textContent=new "#),
                format!(r#"TextEncoder().encode(this.value).length"{locked}>{}</textarea>"#, escape(&notice))
            ].concat(),
            format!(r#"<div class="a t" style="right:7px;top:266px;color:{BLACK}">[<span id="guild-letters">{}</span>/119]</div>"#, notice.len()),
        ]
        .concat();
        if self.guild.master {
            tab.push_str(&picture_press(WIDTH - 46, HEIGHT - 24, "btn_send", Press::Notice));
        }
        tab
    }
}
