//! The Game Settings window.

use super::*;

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
    /// Redraws the window where it is on the page; a list keeps its place.
    pub(in crate::web) fn refresh_settings(&self) {
        let Some(window) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("settings")) else { return };
        let scrolled = window.query_selector(".scroll").ok().flatten().map_or(0, |list| list.scroll_top());
        window.set_inner_html(&self.settings_window());
        if let Ok(Some(list)) = window.query_selector(".scroll") {
            list.set_scroll_top(scrolled);
        }
    }

    /// The client's `CUIGameSettingsUI`, 400 by 350: under the title five tabs 72 wide; the page 288 high from
    /// 34 down; Reset at the left of the bottom, Apply and Close at the right. The effects, the controls and
    /// the rest are lists of the options of the client's `GameSettings.lub`, a row 26 high each: the switch (a
    /// picture 34 by 15, 8 in), the name from 47 and what it does from 138. The first page has the sound: the
    /// music and the effects, each a slider 174 wide from 152 and a box to turn it off.
    ///
    /// The client's names of the tabs and of "off" are in messages its English table does not have; they are
    /// this client's words here.
    pub(in crate::web) fn settings_window(&self) -> String {
        let mut window = title_bar(400, "Game Settings");
        window.push_str(&art("basic_interface/sys_base_off.bmp", 3, 3, 11, 11));
        window.push_str(&button(386, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "settings", ""));
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0;border-radius:0 0 3px 3px;box-sizing:border-box"></div>"#, at(0, 17, 400, 333)));
        for (tab, name) in ["Basic", "Effects", "Controls", "Graphics", "Etc"].iter().enumerate() {
            let open = tab == self.settings.tab;
            window.push_str(&format!(
                r#"<div class="a t" style="{};text-align:center;line-height:17px;cursor:default;border:1px solid #c0c0c0;box-sizing:border-box;background:{};color:{}" data-click="settings-tab" data-arg="{tab}">{}</div>"#,
                at(tab as i32 * 71, 17, 72, 18),
                if open { "#fff" } else { "#e8e8e8" },
                if open { "#000" } else { "#808080" },
                escape(tr(name))
            ));
        }
        match page_of(self.settings.tab) {
            Some(page) => {
                let rows: String = data::options(page)
                    .iter()
                    .map(|option| {
                        let switch = if option.run {
                            // A thing done at a press: its button, faint for those this client does not do yet.
                            match Self::command_of(&option.id).or((option.id == "TT_EX_SIT").then_some("/sit")) {
                                Some(_) => format!(
                                    r#"<div class="a" style="{};background:url('{}') no-repeat;cursor:pointer" data-click="settings-run" data-arg="{}"></div>"#,
                                    at(7, 5, 34, 15),
                                    picture("GameSettingsUI/btn_optionapply_out.bmp"),
                                    escape(&option.id)
                                ),
                                None => format!(r#"<div class="a" style="{};background:url('{}') no-repeat;opacity:.45"></div>"#, at(7, 5, 34, 15), picture("GameSettingsUI/btn_optionapply_out.bmp")),
                            }
                        } else {
                            format!(
                                r#"<div class="a" style="{};background:url('{}') no-repeat;cursor:pointer" data-click="settings-switch" data-arg="{}"></div>"#,
                                at(8, 5, 34, 15),
                                picture(if self.option(&option.id) { "GameSettingsUI/toggle_on.bmp" } else { "GameSettingsUI/toggle_off.bmp" }),
                                escape(&option.id)
                            )
                        };
                        format!(
                            r#"<div style="position:relative;height:26px">{switch}<div class="a t" style="{};overflow:hidden;white-space:nowrap">{}</div><div class="a t" style="{};overflow:hidden;white-space:nowrap">{}</div></div>"#,
                            at(47, 6, 88, 14),
                            escape(tr(&option.title)),
                            at(138, 6, 245, 14),
                            escape(tr(&option.description))
                        )
                    })
                    .collect();
                window.push_str(&format!(r#"<div class="a scroll" style="{};overflow-y:auto">{rows}</div>"#, at(1, 35, 398, 286)));
            }
            None if self.settings.tab == 0 => {
                // The language, at the top of the page, where the client has the choice of a skin.
                window.push_str(&text(19, 34 + 12, BLACK, tr("Language")));
                for (at_x, lang) in [(152, crate::i18n::Lang::En), (200, crate::i18n::Lang::Ru)] {
                    let chosen = lang == self.lang;
                    window.push_str(&format!(
                        r#"<div class="a t" style="{};text-align:center;line-height:16px;cursor:pointer;border:1px solid #9a9a9a;box-sizing:border-box;background:{}" data-click="lang" data-arg="{}">{}</div>"#,
                        at(at_x, 34 + 10, 42, 18),
                        if chosen { "#c6d6f7" } else { "#fff" },
                        lang.code(),
                        lang.code().to_uppercase()
                    ));
                }
                // The sound, where the client's page has it: 36 down the page, 60 high.
                window.push_str(&text(19, 34 + 36 + 12, BLACK, tr("Audio Settings")));
                for (row, (name, level, off, which)) in [("BGM", self.settings.music, self.settings.music_off, "music"), ("SFX", self.settings.effects, self.settings.effects_off, "effects")].iter().enumerate() {
                    let y = 34 + 36 + 12 + row as i32 * 24;
                    // The name ends just before the slider, so a longer word does not run under it.
                    window.push_str(&text_ending(148, 400, y, BLACK, tr(name)));
                    window.push_str(&format!(
                        r#"<input class="a" type="range" min="0" max="100" value="{level}" style="{};margin:0" data-slide="settings-{which}">"#,
                        at(152, y - 1, 174, 13)
                    ));
                    window.push_str(&format!(
                        r#"<div class="a" style="{};background:url('{}') no-repeat;cursor:pointer" data-click="settings-off" data-arg="{which}"></div>"#,
                        at(334, y + 1, 10, 10),
                        picture(if *off { "checkbox_1.bmp" } else { "checkbox_0.bmp" })
                    ));
                    window.push_str(&text(348, y, BLACK, tr("Off")));
                }
            }
            // The graphics page chooses a graphics card, a screen size and the like: a browser has none of it.
            None => window.push_str(&format!(r#"<div class="a t" style="{};text-align:center;color:#808080;white-space:normal">{}</div>"#, at(20, 150, 360, 40), tr("The browser sets the graphics; there is nothing to choose here."))),
        }
        window.push_str(&picture_button(5, 326, "btn_reset", "settings-reset"));
        window.push_str(&picture_button(348, 326, "btn_close", "settings"));
        window
    }
}
