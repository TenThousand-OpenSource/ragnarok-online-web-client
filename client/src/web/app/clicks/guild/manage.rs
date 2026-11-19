//! The guild: what its master does in its window and on other players — the positions and who has which, the
//! emblem, the alliances and oppositions, the skills, the notice — and what the window waits for to be read (the
//! client's `UIGuildPositionManageWnd::SendMsg`, `UIGuildMemberManageWnd::SendMsg`, `UIGuildInfoManageWnd::
//! SendMsg`, `UIGuildSkillWnd::SendMsg`, `UIGuildNoticeWnd::SendMsg`, and the `MM_*` arms they call).

use super::*;
use crate::i18n::tr;
use crate::web::figure;

/// The most a position takes of a member's experience, in hundredths (the client's check in `ID_PAYRATE_EDIT`).
const MOST_TAX: u32 = 50;
/// The largest file the client reads as an emblem (`CGuildMgr`'s 0xC800).
const MOST_EMBLEM_FILE: f64 = 51200.0;

impl App {
    /// What the master typed of the positions (their names, their taxes) joins what was changed and is not
    /// sent yet. A tax that is no number is 0, and one past 50 is 50; `true` when one was past it.
    fn guild_positions_read(&mut self) -> bool {
        let mut past = false;
        for position in self.guild.positions.clone() {
            let mut edited = self.guild.position_shown(&position);
            if let Some(name) = written(&format!("guild-position-{}", position.id)) {
                edited.name = name.trim().to_owned();
            }
            if let Some(tax) = written(&format!("guild-tax-{}", position.id)) {
                let tax = tax.trim().parse::<u32>().unwrap_or(0);
                past |= tax > MOST_TAX;
                edited.tax = tax.min(MOST_TAX);
            }
            self.guild_position_edited(position, edited);
        }
        past
    }

    /// A position as the master wants it is kept until it is sent; one that is as the server has it is not.
    fn guild_position_edited(&mut self, known: guild::Position, edited: guild::Position) {
        if edited == known {
            self.guild.position_edits.remove(&known.id);
        } else {
            self.guild.position_edits.insert(known.id, edited);
        }
    }

    /// A box of a position was pressed: the master switches whether it invites, expels or opens the storage
    /// (`ID_CHECK`).
    pub(super) fn guild_right_pressed(&mut self, id: u32, press: &crate::press::guild::Press) {
        use crate::press::guild::Press;
        let Some(known) = self.guild.positions.iter().find(|position| position.id == id).cloned().filter(|_| self.guild.master) else { return };
        self.guild_positions_read();
        let mut edited = self.guild.position_shown(&known);
        match press {
            Press::Invites(_) => edited.invites = !edited.invites,
            Press::Expels(_) => edited.expels = !edited.expels,
            _ => edited.storage = !edited.storage,
        }
        self.guild_position_edited(known, edited);
        self.redraw(&[&wnd::GUILD]);
    }

    /// The button of the positions: what was changed goes to the server (`MM_REQ_CHANGE_GUILD_POSITIONINFO`).
    /// A tax past 50 is told of in the chat, in red, with the client's line 3460.
    pub(super) fn guild_positions_sent(&mut self) {
        if !self.guild.master {
            return;
        }
        if self.guild_positions_read() {
            self.chat_line(Self::words(3460, "Guild experience can be accumulated up to %d%%.").replace("%d%%", &format!("{MOST_TAX}%")), RED);
        }
        let edits: Vec<guild::Position> = std::mem::take(&mut self.guild.position_edits).into_values().collect();
        if !edits.is_empty() {
            self.send_on_map(&guild::change_positions(&edits));
        }
        self.redraw(&[&wnd::GUILD]);
    }

    /// The button by a member's position: the master's list of positions to pick from opens under that row, or
    /// closes if it was open (`ID_OPEN_MEMBER_POS_COMBO`).
    pub(super) fn guild_combo_pressed(&mut self, row: usize) {
        if self.guild.master {
            (self.guild.picked, self.guild.menu) = (Some(row), None);
            self.guild.combo = (self.guild.combo != Some(row)).then_some(row);
            self.redraw(&[&wnd::GUILD, &wnd::GUILD_MENU]);
        }
    }

    /// A position was picked for the member whose list was open (`UM_COMBOBOX_SELECTED`): kept until the
    /// button sends it. The first position is not given this way, nor taken from the one who has it.
    pub(super) fn guild_position_picked(&mut self, position: u32) {
        let Some(member) = self.guild.combo.take().and_then(|row| self.guild_listed().get(row).map(|member| (*member).clone())) else { return };
        if position != 0 && member.position != 0 && self.guild.master {
            if position == member.position {
                self.guild.member_moves.remove(&member.character);
            } else {
                self.guild.member_moves.insert(member.character, (member.account, position));
            }
        }
        self.redraw(&[&wnd::GUILD]);
    }

    /// The button of the members: the positions given go to the server (`MM_REQ_CHANGE_MEMBERPOS`).
    pub(super) fn guild_members_sent(&mut self) {
        let moves: Vec<(u32, u32, u32)> = std::mem::take(&mut self.guild.member_moves)
            .into_iter()
            .map(|(character, (account, position))| (account, character, position))
            .collect();
        if self.guild.master && !moves.is_empty() {
            self.send_on_map(&guild::move_members(&moves));
        }
        self.redraw(&[&wnd::GUILD]);
    }

    /// The button by the emblem. The client lists the pictures of its `emblem` folder to pick from
    /// (`ID_CHANGE_GUILE_EMBLEM`); a page has no such folder, so the player picks a file of their own
    /// computer, and what is in it comes back as `GuildEvent::EmblemPicked`. A file larger than the client
    /// reads comes back empty, which is no emblem.
    pub(super) fn guild_emblem_asked(&mut self) {
        let Some(input) =
            self.guild.master.then(crate::web::document).flatten().and_then(|document| document.create_element("input").ok()).and_then(
                |input| input.dyn_into::<web_sys::HtmlInputElement>().ok()
            ) else { return };
        input.set_type("file");
        input.set_accept(".bmp,image/bmp");
        let picked = input.clone();
        let read = Closure::once_into_js(move || {
            let Some(file) = picked.files().and_then(|files| files.get(0)) else { return };
            wasm_bindgen_futures::spawn_local(async move {
                let bytes = if file.size() > MOST_EMBLEM_FILE {
                    None
                } else {
                    wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await.ok().map(
                        |buffer| js_sys::Uint8Array::new(&buffer).to_vec()
                    )
                };
                dispatch(Action::Guild(GuildEvent::EmblemPicked(bytes.unwrap_or_default())));
            });
        });
        let _ = input.add_event_listener_with_callback("change", read.unchecked_ref());
        input.click();
    }

    /// Something the guild waited for was read.
    pub(in crate::web::app) fn guild_event(&mut self, event: GuildEvent) {
        match event {
            GuildEvent::Head(sprite, picture) => {
                self.guild.heads.insert(sprite, picture);
                self.redraw(&[&wnd::GUILD]);
            }
            // The file picked becomes the emblem (CZ_REGISTER_GUILD_EMBLEM_IMG: the client's
            // `MM_SEND_GUILD_EMBLEM`, the way rAthena takes one; the client of 2026 hands its emblems to a web
            // service instead, which this server has not). What is no emblem is refused in a message box with
            // the client's line 3587; one too see-through, which the server would refuse, with words of this
            // client's own. Anything else the server refuses it says in the chat itself (ZC_NPC_CHAT).
            GuildEvent::EmblemPicked(file) => match guild::emblem_packed(&file) {
                Ok(packed) if self.guild.master => self.send_on_map(&guild::set_emblem(&packed)),
                Ok(_) => {}
                Err(fault) => {
                    self.message = Some(match fault {
                        guild::EmblemFault::Unfit => Self::words(3587, "This file cannot be registered."),
                        guild::EmblemFault::Transparent => tr("Too much of the emblem is see-through.").to_owned(),
                    });
                    self.render();
                }
            },
        }
    }

    /// The heads of the members in the game are read for the list of members, each kind once (the client draws
    /// them from the head's sprite, `UIGuildMemberManageWnd::DrawMember`).
    pub(in crate::web::app) fn guild_heads_wanted(&mut self) {
        if !self.guild.open || self.guild.tab != 1 {
            return;
        }
        let wanted: Vec<String> = self.guild.members
            .iter()
            .filter(|member| member.online)
            .map(|member| figure::head_sprite(member.job, member.sex as u8, member.hair))
            .collect();
        for sprite in wanted {
            if self.guild.heads.contains_key(&sprite) {
                continue;
            }
            self.guild.heads.insert(sprite.clone(), None);
            wasm_bindgen_futures::spawn_local(async move {
                let picture = figure::frame_picture(&sprite, 0, 0).await.ok().map(|(picture, size, _)| (picture, size));
                dispatch(Action::Guild(GuildEvent::Head(sprite, picture)));
            });
        }
    }

    /// The master's row of the menu on another player that asks that player's guild for an alliance
    /// (`MM_REQ_ALLY_GUILD`); the server asks its master, if that is who was pressed.
    pub(in crate::web::app) fn guild_ally_asked(&mut self, who: u32) {
        if self.guild.master {
            self.send_on_map(&guild::ask_ally(who, self.own_account(), self.own_character()));
        }
    }

    /// The master's row that sets that player's guild as one opposed (`MM_REQ_HOSTILE_GUILD`).
    pub(in crate::web::app) fn guild_opposed(&mut self, who: u32) {
        if self.guild.master {
            self.send_on_map(&guild::oppose(who));
        }
    }

    /// The button by a skill: a point of the guild goes into it (`ID_LVLUP1`…: `MM_UPGRADE_SKILLLEVEL`, the
    /// same packet as for a skill of the character's own).
    pub(super) fn guild_skill_raised(&mut self, skill: u16) {
        if self.guild.master && self.guild.skill_points != 0 {
            self.send_on_map(&net::inventory::raise_skill(skill));
        }
    }

    /// The button that uses the skill picked, if it is learnt (`ID_OK`, `UM_USESKILL`): it is used as a skill
    /// of the character's own is.
    pub(super) fn guild_skill_used(&mut self) {
        let picked = self.guild.skill.and_then(|row| self.guild_skills_listed().get(row).map(|skill| (skill.id, skill.level)));
        if let Some((id, 1..)) = picked {
            self.used(Use::Skill(id));
        }
    }

    /// A skill of the guild's, as the skills of the character are kept: so it is used and aimed as they are.
    pub(in crate::web::app) fn guild_skill(&self, id: u16) -> Option<net::inventory::Skill> {
        self.guild.skills.iter().find(|skill| skill.id == id).map(guild::Skill::as_known)
    }

    /// The notice's button: what the master typed becomes the guild's notice (`ID_SEND_GUILD_NOTICE`,
    /// `MM_GUILD_NOTICE`); the server then tells every member of it.
    pub(super) fn guild_notice_sent(&mut self) {
        if self.guild.master {
            let (subject, notice) = (field("guild-subject"), written("guild-notice").unwrap_or_default());
            self.send_on_map(&guild::set_notice(self.guild.id, subject.trim(), notice.trim()));
        }
    }
}
