//! The windows of the account: logging in, registering, recovering and resetting a password.

use super::*;
use crate::press::account::Press;

impl App {
    pub(super) fn login_window(&self, view: (f64, f64)) -> String {
        // The client writes these two in the middle of their fields; here, by the player's wish, the text starts
        // at the left. The box is where the client has it, 56 in: the window's picture has a box drawn there,
        // and one anywhere else stands beside that one.
        let field = |id: &str, y: i32, extra: &str| [
            format!(r#"<input class="a in" id="{id}" maxlength="23" style="{};text-align:left;padding-left:4px;"#, at(15, y, 153, 16)),
            format!(r#"box-sizing:border-box" {extra}>"#)
        ].concat();
        // The words by the box are the smaller text and those on the two buttons are bold (`UILoginWnd::OnCreate`:
        // `SetFont(0, 11, 0, 0)`, and `AddTextStyle` of height 12, bold).
        let (small, bold) = (SMALL.style(), BOLD.style());
        let checkbox = if self.save_id { "login_interface/checkbox_on.bmp" } else { "login_interface/checkbox_off.bmp" };
        let sends = sent(Form::Login);
        let inner = format!(
            r#"<form {sends}>
                <div class="a combo" style="{combo}">{service}</div>
                {id}{password}
                <button type="button" class="a check" style="{check};{check_picture}" {save}></button>
                <div class="a" style="{save_label};{small};color:#333">{save_text}</div>
                {restore}{register}
                <button class="a b" style="{login};{login_pictures}" aria-label="Login"></button>
            </form>"#,
            combo = at(13, 10, 160, 22),
            service = self.text(Key::Service),
            id = field(
                "user",
                40,
                &format!(
                    r#"name="username" autocomplete="username" autofocus placeholder="{}" value="{}""#,
                    self.text(Key::Id),
                    escape(&self.saved_id)
                )
            ),
            password = field(
                "password",
                64,
                &format!(r#"name="password" type="password" autocomplete="current-password" placeholder="{}""#, self.text(Key::Password))
            ),
            check = at(56, 85, 12, 12),
            check_picture = background(checkbox),
            save = pressed(Press::SaveId),
            save_label = at(73, 85, 90, 14),
            save_text = self.text(Key::SaveId),
            restore = button(
                &format!("{};{bold}", at(13, 104, 67, 21)),
                &states_of("login_interface/bt_close"),
                &pressed(Press::ToRecover),
                self.text(Key::Restore)
            ),
            register = button(
                &format!("{};{bold}", at(87, 104, 84, 21)),
                &states_of("login_interface/bt_join"),
                &pressed(Press::ToRegister),
                self.text(Key::Register)
            ),
            login = at(192, 32, 84, 84),
            login_pictures = states_of("login_interface/bt_start"),
        );
        // (The list of recordings stands in the login window's place, as in the client's replay mode.)
        if self.replay.list {
            return self.replays_view(view);
        }
        let login = window(view, (301, 132), login_place(view), "", "login_interface/bg_login.tga", &inner);
        format!("{login}{}", self.replays_view(view))
    }

    /// The way to the recordings from the login screen. The Windows client is started for them apart (its
    /// replay mode, which shows `UISelectReplayDataWnd` in the login's place); a page has one way in, so here
    /// its button `replay_interface\\btn_replay.bmp` stands under the login window and opens the list over it,
    /// 140 left of the middle of the screen and 160 over it, as the client puts it.
    fn replays_view(&self, view: (f64, f64)) -> String {
        use crate::press::replay::Press;
        if self.replay.list {
            let place = format!("left:{}px;top:{}px;width:280px;height:311px", view.0 as i32 / 2 - 140, (view.1 as i32 / 2 - 160).max(0));
            return format!(r#"<div id="replay-list" class="a hud" style="{place};z-index:5">{}</div>"#, self.replay_list());
        }
        let (x, y) = login_place(view);
        button(
            &at(x as i32 + 247, y as i32 + 136, 54, 20),
            &states_ab("replay_interface/btn_replay"),
            &format!(r#"{} title="{}""#, pressed(Press::List), escape(&Self::words(2350, "Replay"))),
            ""
        )
    }

    /// A window for a form the original client does not have, in the look of its newer windows. Its texts are
    /// none of the client's, and their sizes are this project's own (the page's `.panel`).
    pub(super) fn form(&self, view: (f64, f64), height: i32, submit: Form, title: Key, rows: &str) -> String {
        let buttons = format!(
            r#"<button class="a b" style="{};{pictures}">{}</button>{}"#,
            at(12, height - 36, 131, 24),
            self.text(Key::Ok),
            button(&at(158, height - 36, 131, 24), &states_of("select_character_ver3/bt_info"), &pressed(Press::ToLogin), self.text(Key::Cancel)),
            pictures = states_of("select_character_ver3/bt_info"),
        );
        let inner = format!(r#"<form {}><h1>{}</h1><div class="rows">{rows}</div>{buttons}</form>"#, sent(submit), self.text(title));
        let place = login_place(view);
        // Taller than the login window: moved up so it ends where that one ends.
        window(view, (301, height), (place.0, place.1 + 132.0 - f64::from(height)), "panel", "", &inner)
    }

    pub(super) fn row(&self, label: Key, input: &str) -> String {
        format!(r#"<label><span>{}</span>{input}</label>"#, self.text(label))
    }

    pub(super) fn register_window(&self, view: (f64, f64)) -> String {
        let rows = [
            self.row(Key::Id, r#"<input id="reg-id" maxlength="23" autocomplete="username" autofocus>"#),
            self.row(Key::PasswordLabel, r#"<input id="reg-password" type="password" maxlength="23" autocomplete="new-password">"#),
            self.row(Key::RepeatPassword, r#"<input id="reg-repeat" type="password" maxlength="23" autocomplete="new-password">"#),
            self.row(Key::Email, r#"<input id="reg-email" type="email" maxlength="39" autocomplete="email">"#),
        ];
        self.form(view, 188, Form::Register, Key::RegisterTitle, &rows.concat())
    }

    pub(super) fn recover_window(&self, view: (f64, f64)) -> String {
        let rows = [
            format!(r#"<p>{}</p>"#, self.text(Key::RestoreHint)),
            self.row(Key::Id, r#"<input id="rec-id" maxlength="23" autocomplete="username" autofocus>"#),
            self.row(Key::Email, r#"<input id="rec-email" type="email" maxlength="39" autocomplete="email">"#),
        ];
        self.form(view, 196, Form::Recover, Key::RestoreTitle, &rows.concat())
    }

    pub(super) fn reset_window(&self, view: (f64, f64)) -> String {
        let rows = [
            self.row(Key::PasswordLabel, r#"<input id="new-password" type="password" maxlength="23" autocomplete="new-password" autofocus>"#),
            self.row(Key::RepeatPassword, r#"<input id="new-repeat" type="password" maxlength="23" autocomplete="new-password">"#),
        ];
        self.form(view, 136, Form::Reset, Key::ResetTitle, &rows.concat())
    }
}
