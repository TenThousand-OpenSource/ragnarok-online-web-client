//! The Game Settings window.

use super::*;
use crate::press::settings::{Press, Sound};

/// The pages of the option lists, by the window's tab: effects, controls and the rest (the client's numbers).
fn page_of(tab: usize) -> Option<u8> {
    match tab {
        1 => Some(1),
        2 => Some(2),
        4 => Some(4),
        _ => None,
    }
}

impl App {
    /// The client's `CUIGameSettingsUI`, 400 by 350: under the title five tabs 72 wide; the page 288 high from
    /// 34 down; Reset at the left of the bottom, Apply and Close at the right. The effects, the controls and
    /// the rest are lists of the options of the client's `GameSettings.lub`, a row 26 high each: the switch (a
    /// picture 34 by 15, 8 in), the name from 47 and what it does from 138. The first page has the sound: the
    /// music and the effects, each a slider 174 wide from 152 and a box to turn it off. All of it is the usual
    /// text; a group's title and an option's name are bold (`CUIGroupSound::OnCreate`, `CUIOptionItem`'s label,
    /// 85 wide: "Gulim" at 12 with `SetBold(1)`).
    ///
    /// The client's names of the tabs and of "off" are in messages its English table does not have; they are
    /// this client's words here.
    pub(in crate::web) fn settings_window(&self) -> String {
        let mut window = self.settings_frame();
        match page_of(self.settings.tab) {
            Some(page) => window.push_str(&self.options_page(page)),
            None if self.settings.tab == 0 => window.push_str(&self.basic_page()),
            // The graphics page chooses a graphics card, a screen size and the like: a browser has none of it.
            None => window.push_str(&format!(r#"<div class="a t" style="{};text-align:center;color:#808080;white-space:normal">{}</div>"#, at(20, 150, 360, 40), tr("The browser sets the graphics; there is nothing to choose here."))),
        }
        window.push_str(&picture_press(5, 326, "btn_reset", Press::Reset));
        window.push_str(&picture_press(348, 326, "btn_close", Press::Open));
        window
    }

    /// The window's title, its ground and its five tabs.
    fn settings_frame(&self) -> String {
        let mut window = title_bar(400, "Game Settings");
        window.push_str(&base_button(3, 3));
        window.push_str(&cross(400, Press::Open));
        window.push_str(&panel_ground_round(0, 17, 400, 333));
        for (tab, name) in ["Basic", "Effects", "Controls", "Graphics", "Etc"].iter().enumerate() {
            let open = tab == self.settings.tab;
            let press = pressed(Press::Tab(tab));
            window.push_str(&format!(
                r#"<div class="a t" style="{};text-align:center;line-height:17px;cursor:default;border:1px solid #c0c0c0;box-sizing:border-box;background:{};color:{}" {press}>{}</div>"#,
                at(tab as i32 * 71, 17, 72, 18),
                if open { "#fff" } else { "#e8e8e8" },
                if open { "#000" } else { "#808080" },
                escape(tr(name))
            ));
        }
        window
    }

    /// A page that lists options of the client's `GameSettings.lub`: a row to each.
    fn options_page(&self, page: u8) -> String {
        let rows: String = data::options(page)
            .iter()
            .map(|option| {
                format!(
                    r#"<div style="position:relative;height:26px">{}<div class="a t" style="{};{};overflow:hidden;white-space:nowrap">{}</div><div class="a t" style="{};overflow:hidden;white-space:nowrap">{}</div></div>"#,
                    self.option_switch(&option.id, option.run),
                    at(47, 6, 85, 14),
                    BOLD.style(),
                    escape(tr(&option.title)),
                    at(138, 6, 245, 14),
                    escape(tr(&option.description))
                )
            })
            .collect();
        format!(r#"<div class="a scroll" style="{};overflow-y:auto">{rows}</div>"#, at(1, 35, 398, 286))
    }

    /// What stands at the left of an option's row: its switch, or, for a thing done at a press (`run`), its
    /// button, faint for those this client does not do yet.
    fn option_switch(&self, id: &str, run: bool) -> String {
        if !run {
            return format!(
                r#"<div class="a" style="{};background:url('{}') no-repeat;cursor:pointer" {}></div>"#,
                at(8, 5, 34, 15),
                picture(if self.option(id) { "GameSettingsUI/toggle_on.bmp" } else { "GameSettingsUI/toggle_off.bmp" }),
                pressed(Press::Switch(id.to_owned()))
            );
        }
        match Self::command_of(id).or((id == "TT_EX_SIT").then_some("/sit")) {
            Some(_) => format!(
                r#"<div class="a" style="{};background:url('{}') no-repeat;cursor:pointer" {}></div>"#,
                at(7, 5, 34, 15),
                picture("GameSettingsUI/btn_optionapply_out.bmp"),
                pressed(Press::Run(id.to_owned()))
            ),
            None => format!(r#"<div class="a" style="{};background:url('{}') no-repeat;opacity:.45"></div>"#, at(7, 5, 34, 15), picture("GameSettingsUI/btn_optionapply_out.bmp")),
        }
    }

    /// The first page: the language and the sound.
    fn basic_page(&self) -> String {
        // The language, at the top of the page, where the client has the choice of a skin, its title bold
        // as that one's is.
        let mut page = text_in(BOLD, 19, 34 + 12, BLACK, tr("Language"));
        for (at_x, lang) in [(152, crate::i18n::Lang::En), (200, crate::i18n::Lang::Ru)] {
            let chosen = lang == self.lang;
            page.push_str(&format!(
                r#"<div class="a t" style="{};text-align:center;line-height:16px;cursor:pointer;border:1px solid #9a9a9a;box-sizing:border-box;background:{}" {}>{}</div>"#,
                at(at_x, 34 + 10, 42, 18),
                if chosen { "#c6d6f7" } else { "#fff" },
                pressed(crate::press::general::Press::Language(lang)),
                lang.code().to_uppercase()
            ));
        }
        // The sound, where the client's page has it: 36 down the page, 60 high.
        page.push_str(&text_in(BOLD, 19, 34 + 36 + 12, BLACK, tr("Audio Settings")));
        for (row, (name, level, off, which)) in [("BGM", self.settings.music, self.settings.music_off, Sound::Music), ("SFX", self.settings.effects, self.settings.effects_off, Sound::Effects)].iter().enumerate() {
            let y = 34 + 36 + 12 + row as i32 * 24;
            // The name ends just before the slider, so a longer word does not run under it.
            page.push_str(&text_ending(148, 400, y, BLACK, tr(name)));
            page.push_str(&format!(
                r#"<input class="a" type="range" min="0" max="100" value="{level}" style="{};margin:0" data-slide="settings-{which}">"#,
                at(152, y - 1, 174, 13)
            ));
            page.push_str(&format!(
                r#"<div class="a" style="{};background:url('{}') no-repeat;cursor:pointer" {}></div>"#,
                at(334, y + 1, 10, 10),
                picture(if *off { "checkbox_1.bmp" } else { "checkbox_0.bmp" }),
                pressed(Press::Off(*which))
            ));
            page.push_str(&text(348, y, BLACK, tr("Off")));
        }
        page
    }
}
