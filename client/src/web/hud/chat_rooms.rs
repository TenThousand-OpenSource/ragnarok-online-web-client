//! Chat rooms on the page: the sign over a room's owner (the client's `UIChatRoomTitle`), the window that
//! makes or changes a room (`UIChatRoomMakeWnd`), the room's own window (`UIChatRoomWnd`) with its menu, and
//! the box that asks a password (`UIPasswordWnd`).

use super::*;
use crate::net::chat_rooms::OWNER;
use crate::chat_rooms::{LIMITS, Room, kind};
use crate::press::chat_rooms::Press;
use crate::press::things::Menu;
use crate::web::parts::menued;

/// The colours of the room's window (the client's, written blue first there): the one the room belongs to in
/// the list of members (`0x808000`), the member picked (`0xC8C832`).
const OWNED: &str = "#008080";
const PICKED: &str = "#32c8c8";

impl App {
    /// The sign of a room, 140 by 34 (`UIChatRoomTitle::OnCreate`, `Paint`): the frame of the system boxes 28
    /// high with its arrow under the middle, at 3,3 the picture of a room with a password or of one without,
    /// and at 31,8 in 106 by 15 what the room is called and how many are in it, cut where it does not fit,
    /// with stars for a word that is not let be written (`UIChatRoomTitle::SendMsg`; so in the window's title
    /// and in what is said in a room: `UIChatRoomWnd::SendMsg`).
    /// A double click on it enters (`OnLBtnDblClk`); the hand shows over it.
    pub(in crate::web) fn room_sign(room: &Room) -> String {
        [
            format!(
                r#"<div class="a" style="{};cursor:pointer" {}>{}"#,
                at(0, 0, crate::web::world::SIGN.0, crate::web::world::SIGN.1),
                used(Use::Room(room.id)),
                system_box(0, 140, 28)
            ),
            format!(r#"<img class="a" src="{}" alt="" draggable="false" style="left:50%;bottom:0;"#, picture("sysbox_arr_l.bmp")),
            format!(r#"transform:translateX(-50%)" onerror="this.remove()">"#),
            format!(
                r#"<img class="a" src="{}" alt="" draggable="false" style="left:3px;top:3px" onerror="this.remove()">"#,
                picture(if room.kind == kind::PRIVATE { "chat_close.bmp" } else { "chat_open.bmp" })
            ),
            format!(
                r#"<div class="a t" style="{};overflow:hidden;white-space:nowrap;color:{BLACK}">{}</div></div>"#,
                at(31, 8, 106, 15),
                escape(&data::starred(&room.sign()))
            )
        ].concat()
    }

    /// The window that makes or changes a room (`UIChatRoomMakeWnd::OnCreate`, `OnDraw`): `titlebar_fix.bmp`
    /// over `chatwin0_bg.bmp`, which has the names of the lines painted on it; the title at 45,24 (228 by 16,
    /// 36 letters); the number of people at 48,49 with the button that drops its list at 119,46, and the
    /// kind of room at 187,49 (the client has the one kind, its message 130, and a button at 256,46 that drops
    /// a list of that one: drawn, not pressed); the two round buttons at 44 and 96, 70 down; the password at
    /// 182,70 (90 by 16, 8 letters, hidden); OK at 189,96 and Cancel at 234,96. The list of numbers drops to
    /// 45,64, 74 wide and three rows of 16 high.
    pub(in crate::web) fn room_make_window(&self) -> String {
        let Some(making) = &self.chat_rooms.making else { return String::new() };
        let title = if making.changes { Self::words(126, "Chat Room Setup") } else { Self::words(125, "Create Chat Room") };
        let people = |count: u16| format!("{count} {}", Self::words(131, "ppl"));
        let field = "border:0;background:#f2f2f2;padding:0 2px;box-sizing:border-box";
        let limits = if making.limits_open {
            let rows: String = LIMITS
                .iter()
                .map(|count| format!(
                    r#"<div class="t opt" style="height:16px;line-height:16px;padding:0 3px;cursor:default" {}>{}</div>"#,
                    pressed(Press::Limit(*count)),
                    escape(&people(*count))
                ))
                .collect();
            [
                format!(r#"<div class="a scroll" style="{};background:#fff;border:1px solid #9a9a9a;box-sizing:border-box;"#, at(45, 64, 74, 48)),
                format!(r#"overflow-y:auto">{rows}</div>"#)
            ].concat()
        } else {
            String::new()
        };
        format!(
            r#"{}{}{}{}
               <form {} class="a" style="{}">
                 <input id="room-title" class="a in" maxlength="36" autocomplete="off" autofocus value="{}" style="{};{field}">
                 <div class="a t" style="{};background:#f2f2f2;overflow:hidden;white-space:nowrap">{}</div>{}
                 <div class="a t" style="{};background:#f2f2f2;overflow:hidden;white-space:nowrap">{}</div>{}
                 {}{}
                 <input id="room-word" class="a in" type="password" maxlength="8" autocomplete="off" value="{}" style="{};{field}">
                 {}{}{limits}<button type="submit" hidden></button>
               </form>"#,
            art("basic_interface/titlebar_fix.bmp", 0, 0, 280, 17),
            art("basic_interface/chatwin0_bg.bmp", 0, 17, 280, 103),
            titled(8, 3, &title),
            cross(280, Press::MakeClose),
            sent(Form::RoomMake),
            at(0, 0, 280, 120),
            escape(&making.title),
            at(45, 24, 228, 16),
            at(48, 49, 68, 14),
            escape(&people(making.limit)),
            pictured_by(Convention::ABC, 119, 46, (16, 18), "basic_interface/txtbox_btn", Press::Limits, ""),
            at(187, 49, 68, 14),
            escape(&Self::words(130, "Chat Room")),
            format!(
                r#"<img class="a" src="{}" alt="" draggable="false" style="left:256px;top:46px" onerror="this.remove()">"#,
                picture("basic_interface/txtbox_btn_a.bmp")
            ),
            radio_marked(44, 70, making.public, true, &Self::words(201, "Public"), &pressed(Press::Public(true))),
            radio_marked(96, 70, !making.public, true, &Self::words(200, "Private"), &pressed(Press::Public(false))),
            escape(&making.password),
            at(182, 70, 90, 16),
            picture_press(189, 96, "btn_ok", Press::MakeOk),
            picture_press(234, 96, "btn_cancel", Press::MakeClose),
        )
    }

    /// The room's window (`UIChatRoomWnd::OnCreate`, `OnDraw`), 280 by 120: the title bar with what the room
    /// is called at 9,2 and the cross at its right, which leaves the room; what was said at 1,17 in 221 by
    /// 82, on white; a line down, and right of it the list of who is in, 56 wide, the owner in its colour and
    /// the one picked on a band; the bar along the bottom with the line to type in on it, 3 in and 30 short
    /// of the window's width, of 70 letters. (The client's window can be made larger by its corner, up to 512
    /// by 384: this one is of the one size. How high a row of the list is was not read: 14, as a line of
    /// what was said.)
    pub(in crate::web) fn room_window(&self) -> String {
        let Some(sitting) = &self.chat_rooms.sitting else { return String::new() };
        let (width, height) = (280, 120);
        let list = width * 56 / 280;
        let heading = sitting.room.heading(&if sitting.room.kind == kind::PRIVATE { Self::words(89, "Private: ") } else { Self::words(90, "Public: ") });
        let lines: String = sitting.lines.iter().map(|(words, colour)| log_line(colour, &escape(words))).collect();
        let members: String = sitting
            .members
            .iter()
            .enumerate()
            .map(|(row, (role, name))| {
                [
                    format!(r#"<div class="t" style="height:14px;line-height:14px;padding:0 2px;overflow:hidden;white-space:nowrap;"#),
                    format!(
                        r#"cursor:default;color:{};{}" {} {}>{}</div>"#,
                        if *role == OWNER { OWNED } else { BLACK },
                        if sitting.picked == Some(row) { format!("background:{PICKED}") } else { String::new() },
                        pressed(Press::Member(row)),
                        menued(Menu::RoomMember(row)),
                        escape(name)
                    )
                ].concat()
            })
            .collect();
        [
            format!(
                r#"{}{}{}{}<div class="a t" style="{};overflow:hidden;white-space:nowrap;color:{BLACK}">{}</div>{}
               "#,
                art("basic_interface/titlebar_left.bmp", 0, 0, 12, 17),
                art("basic_interface/titlebar_mid.bmp", 12, 0, width - 24, 17),
                art("basic_interface/titlebar_right.bmp", width - 12, 0, 12, 17),
                panel(width, height),
                at(9, 2, width - 30, 15),
                escape(&data::starred(&heading)),
                pictured(
                    width - 14,
                    3,
                    (11, 11),
                    ["basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "basic_interface/sys_close_on.bmp"],
                    Press::Leave,
                    r#" data-tip="/q""#
                )
            ),
            format!(r#"<div class="a t" style="{};background:#fff;padding:0 2px;box-sizing:border-box;"#, at(1, 17, width - list - 3, height - 38)),
            format!(r#"display:flex;flex-direction:column-reverse;overflow-x:hidden;overflow-y:auto;scrollbar-width:thin;"#),
            format!(
                r#"white-space:normal;line-height:14px;user-select:text"><div>{lines}</div></div>
               "#
            ),
            format!(
                r#"<div class="a" style="{};background:#c8c8c8"></div>
               <div class="a scroll" style="{};background:#fff;padding-top:3px;box-sizing:border-box;"#,
                at(width - list - 2, 17, 1, height - 38),
                at(width - list - 1, 17, list, height - 38)
            ),
            format!(
                r#"overflow-x:hidden;overflow-y:auto;scrollbar-width:thin">{members}</div>
               {}"#,
                typed_line(
                    &sent(Form::Room),
                    &at(0, height - 21, width, 21),
                    "room-input",
                    &at(3, 3, width - 30, 16),
                    "background:#f2f2f2;border:0;padding:0 2px;box-sizing:border-box"
                )
            )
        ].concat()
    }

    /// The box that asks a room's password (`UIPasswordWnd::OnCreate`), 176 by 46 in the frame of the system
    /// boxes: the client's message 15 at 15,6, the line at 15,22 (90 by 16, 8 letters, hidden), OK 55 from
    /// the right and 32 from the bottom.
    pub(in crate::web) fn room_password_window(&self) -> String {
        if self.chat_rooms.password_for.is_none() {
            return String::new();
        }
        [
            format!(
                r#"{}<form {} class="a" style="{}">{}"#,
                system_box(0, 176, 46),
                sent(Form::RoomPassword),
                at(0, 0, 176, 46),
                text(15, 6, BLACK, &Self::words(15, "Please enter the password."))
            ),
            format!(r#"<input id="room-key" class="a in" type="password" maxlength="8" autocomplete="off" style="{};"#, at(15, 22, 90, 16)),
            format!(r#"background:#e8e8e8">{}<button type="submit" hidden></button></form>"#, picture_press(121, 14, "btn_ok", Press::PasswordOk))
        ].concat()
    }

    /// The menu of the room's window (`UIChatRoomWnd::SendMsg`, `UM_MAKEMENU`). On a member, for the one the
    /// room belongs to: throwing that one out and giving them the room, unless it is the player's own
    /// character; for anyone: asking them to be a friend, unless they are one or it is the player's own
    /// character. For the owner, on a member or not: changing the room's setting. (The client's rows that
    /// look at a character, which it answers with "not supported yet", and that write what was said to a
    /// file are not made.)
    pub(in crate::web) fn room_menu(&self) -> String {
        let (Some((_, member)), Some(sitting)) = (self.chat_rooms.menu, &self.chat_rooms.sitting) else { return String::new() };
        let own = self.game.as_ref().map(|game| game.name.as_str()).unwrap_or_default();
        let owner = sitting.owned_by(own);
        let other = member.and_then(|row| sitting.members.get(row)).map(|(_, name)| name.as_str()).filter(|name| *name != own);
        let mut rows = String::new();
        if owner && other.is_some() {
            rows.push_str(&menu_row(&Self::words(127, "Kick from Chat Room"), Press::Expel));
            rows.push_str(&menu_row(&Self::words(128, "Give Master Authority"), Press::Give));
        }
        if other.is_some_and(|name| !self.friends.has(name)) {
            rows.push_str(&menu_row(&Self::words(358, "Register as a Friend"), Press::Friend));
        }
        if owner {
            rows.push_str(&menu_row(&Self::words(126, "Chat Room Setup"), Press::Change));
        }
        rows
    }
}
