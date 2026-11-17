//! Choosing a server and a character, and making a character.

use super::*;

impl App {
    pub(in super::super) fn character_in(&self, slot: u8) -> Option<&Character> {
        self.characters.iter().find(|character| character.slot == slot)
    }

    /// The servers to play on, each with how many are playing there now: the original's window (280 by 200, its
    /// list 12 in and 22 down, OK and Cancel in the corner).
    pub(super) fn server_select_window(&self, view: (f64, f64)) -> String {
        let servers = self.account.as_ref().map(|account| &account.char_servers[..]).unwrap_or_default();
        let lines: String = servers
            .iter()
            .enumerate()
            .map(|(index, server)| {
                let picked = if index == self.server_picked { "background:#c6d6f7;" } else { "" };
                format!(
                    r#"<div style="{picked}height:16px;line-height:16px;padding:0 4px;cursor:default;white-space:nowrap" {} data-open-server="{index}">{} ({})</div>"#,
                    click("server", index),
                    escape(&server.name),
                    server.users
                )
            })
            .collect();
        let inner = format!(
            r#"{}<div class="a" style="{};background:#fff;border:1px solid #c3c3c3;border-top:0;box-sizing:border-box"></div>
               <div class="a" style="{};background:#f0f0f0;overflow-y:auto;font:12px Tahoma,Arial,sans-serif;color:#000">{lines}</div>{}{}"#,
            super::super::hud::title_bar(280, self.text(Key::SelectServer)),
            at(0, 17, 280, 183),
            at(12, 22, 256, 145),
            button(&at(189, 176, 42, 20), &states_ab("btn_ok"), &format!("{} autofocus", click("server-ok", "")), ""),
            button(&at(234, 176, 42, 20), &states_ab("btn_cancel"), &click("server-cancel", ""), ""),
        );
        let place = (((view.0 - 640.0) / 2.0 + 185.0).floor(), (view.1 * 300.0 / 480.0 - 80.0).floor());
        window(view, (280, 200), place, "", "", &inner)
    }

    pub(super) fn char_select_window(&self, view: (f64, f64)) -> String {
        let mut inner = String::new();
        for slot in 0..SLOTS_PER_PAGE {
            let (x, y) = (i32::from(slot % 5) * 157 + 10, i32::from(slot / 5) * 195 + 21);
            let selected = slot == self.selected;
            let character = self.character_in(slot);
            if slot >= self.slots && character.is_none() {
                inner.push_str(&format!(r#"<div class="a" style="{};{}"></div>"#, at(x, y, 157, 195), background("select_character_ver3/img_slot_lock.bmp")));
                continue;
            }
            let base = if character.is_some() { "select_character_ver3/img_slot" } else { "select_character_ver3/img_slot2" };
            inner.push_str(&button(&at(x, y, 157, 195), &states_of(base), &format!(r#"{} data-open="{slot}""#, click("slot", slot)), ""));
            match (character, selected) {
                // The frame around a chosen character runs through eight pictures.
                (Some(_), true) => {
                    // All eight lie on one another and show in turn: a picture swapped in its place would blink
                    // while the browser makes it ready.
                    let frames: String = (0..8).map(|frame| format!(r#"<i style="background-image:url('{}')"></i>"#, chosen_frame(frame))).collect();
                    inner.push_str(&format!(r#"<div class="a chosen" style="{}">{frames}</div>"#, at(x - 2, y - 1, 161, 197)));
                }
                (None, true) => inner.push_str(&format!(r#"<div class="a pass" style="{};{}"></div>"#, at(x, y, 157, 195), background("select_character_ver3/img_slot2_select.bmp"))),
                _ => {}
            }
            if let Some(character) = character {
                inner.push_str(&format!(
                    r#"<canvas class="a pass" id="pc{slot}" width="{}" height="{}" style="left:{x}px;top:{y}px"></canvas>
                       <img class="a pass" style="left:{}px;top:{}px" src="{}" alt="" onerror="this.remove()">
                       <div class="a pass name" style="{}">{}</div>"#,
                    figure::WIDTH,
                    figure::HEIGHT,
                    x + 118,
                    y + 14,
                    picture(&format!("renewalparty/icon_jobs_{}.bmp", character.job)),
                    at(x, y + 167, 157, 14),
                    escape(&character.name),
                ));
                // When a character waiting to be deleted can be: red until then, blue from then on.
                if let Some(&when) = self.deletions.get(&character.char_id) {
                    let (colour, lines) = if when <= epoch() {
                        ("#00f", [self.text(Key::DeletionReady).to_owned(), String::new()])
                    } else {
                        let date = js_sys::Date::new(&(when * 1000.0).into());
                        ("#f00", [format!("{}/{}", date.get_month() + 1, date.get_date()), format!("{:02}:{:02}", date.get_hours(), date.get_minutes())])
                    };
                    for (line, text) in lines.iter().enumerate() {
                        inner.push_str(&format!(r#"<div class="a pass small centre" style="{};color:{colour}">{text}</div>"#, at(x + 20, y + 19 + 12 * line as i32, 96, 12)));
                    }
                }
            }
        }

        // The panel on the right: the chosen character's details, and what can be done with it.
        let chosen = self.character_in(self.selected);
        inner.push_str(&format!(r#"<div class="a" style="{};{}"></div>"#, at(809, 26, 173, 575), background("select_character_ver3/img_info.bmp")));
        if let Some(character) = chosen {
            let mut lines = vec![(116, character.map.clone()), (133, data::job_name(character.job)), (150, character.base_level.to_string()), (167, character.base_exp.to_string())];
            lines.push((184, character.hp.to_string()));
            lines.push((201, character.sp.to_string()));
            for (index, stat) in character.stats.iter().enumerate() {
                lines.push((218 + 17 * index as i32, stat.to_string()));
            }
            for (y, value) in lines {
                inner.push_str(&format!(r#"<div class="a small value" style="{}">{}</div>"#, at(868, y, 107, 15), escape(&value)));
            }
        }
        let info = states_of("select_character_ver3/bt_info");
        // The two buttons under the details change with the chosen character: one not being deleted can be, one
        // waiting can be kept after all, and one whose wait is over can be deleted for good.
        let deletion = chosen.and_then(|character| self.deletions.get(&character.char_id)).map(|&when| when <= epoch());
        let (first, second) = match (chosen, deletion) {
            (None, _) => (("disabled".to_owned(), Key::DeleteCharacter), ("disabled".to_owned(), Key::Notice)),
            (Some(_), None) => ((click("delete", ""), Key::DeleteCharacter), ("disabled".to_owned(), Key::Notice)),
            (Some(_), Some(false)) => ((click("cancel-delete", ""), Key::CancelDeletion), ("disabled".to_owned(), Key::Notice)),
            (Some(_), Some(true)) => ((click("cancel-delete", ""), Key::CancelDeletion), (click("delete-now", ""), Key::Delete)),
        };
        inner.push_str(&button(&at(830, 322, 131, 24), &info, &first.0, self.text(first.1)));
        inner.push_str(&button(&at(830, 351, 131, 24), &info, &second.0, self.text(second.1)));
        let chosen = chosen.filter(|_| deletion.is_none());
        let start = if chosen.is_some() {
            states("select_character_ver3/bt_gamestart_on.bmp", "select_character_ver3/bt_gamestart_over.bmp", "select_character_ver3/bt_gamestart_press.bmp")
        } else {
            let off = "select_character_ver3/bt_gamestart_off.bmp";
            states(off, off, off)
        };
        inner.push_str(&format!(
            r#"<button type="button" class="a b start" style="{};{start}" {} {}>{}</button>"#,
            at(816, 407, 165, 110),
            click("start", ""),
            if chosen.is_some() { "" } else { "disabled" },
            self.text(Key::GameStart)
        ));
        // One page: both arrows are off.
        for (x, side) in [(830, "left"), (939, "right")] {
            inner.push_str(&format!(r#"<div class="a" style="{};{}"></div>"#, at(x, 558, 21, 28), background(&format!("select_character_ver3/bt_{side}_off.bmp"))));
        }
        inner.push_str(&format!(r#"<div class="a list-title" style="{}">{}</div>"#, at(830, 538, 131, 14), self.text(Key::CharacterList)));
        inner.push_str(&format!(r#"<div class="a page" style="{}">1 / 1</div>"#, at(851, 560, 88, 22)));
        inner.push_str(&button(&at(945, 5, 17, 18), &states_of("select_character_ver3/bt_close2"), &click("logout", ""), ""));

        let size = (997, 626);
        window(view, size, centred(view, size), "", "select_character_ver3/bg_back1.tga", &inner)
    }

    pub(super) fn char_create_window(&self, view: (f64, f64)) -> String {
        let make = &self.make;
        let folder = "make_character_ver2";
        let mut inner = format!(r#"<div class="a" style="{};color:#5f5f5f">{}</div>"#, at(44, 17, 300, 14), self.text(Key::CharacterCreation));

        // The two races: the chosen one is lit.
        for (y, race, doram, name, about) in [(36, "human", false, Key::Human, Key::HumanAbout), (202, "doram", true, Key::Doram, Key::DoramAbout)] {
            let lit = if doram == make.doram { "on" } else { "off" };
            inner.push_str(&format!(
                r#"<button type="button" class="a race" style="{};{}" {}><b>{}</b><p>{}</p></button>"#,
                at(21, y, 364, 166),
                background(&format!("{folder}/img_{race}_{lit}.bmp")),
                click("race", race),
                self.text(name),
                self.text(about)
            ));
        }

        for (x, sex, name) in [(437, 1u8, "male"), (500, 0u8, "female")] {
            let base = format!("{folder}/bt_{name}");
            let pictures = if make.sex == sex {
                let on = format!("{base}_on.bmp");
                states(&on, &on, &on)
            } else {
                states(&format!("{base}_off.bmp"), &format!("{base}_over.bmp"), &format!("{base}_press.bmp"))
            };
            inner.push_str(&button(&at(x, 59, 62, 25), &pictures, &click("sex", sex), ""));
        }
        inner.push_str(&button(&at(426, 179, 22, 23), &states_of(&format!("{folder}/bt_leftturn")), &click("turn", -1), ""));
        inner.push_str(&button(&at(551, 179, 22, 23), &states_of(&format!("{folder}/bt_rightturn")), &click("turn", 1), ""));
        // The new character, feet at (500, 228).
        inner.push_str(&format!(r#"<canvas class="a pass" id="pc-make" width="{}" height="{}" style="left:420px;top:91px"></canvas>"#, figure::WIDTH, figure::HEIGHT));
        inner.push_str(&format!(r#"<input class="a in" id="char-name" maxlength="23" autocomplete="off" autofocus style="{}">"#, at(440, 268, 120, 20)));

        inner.push_str(&format!(r#"<div class="a small" style="{};color:#404040">{}</div>"#, at(609, 44, 150, 14), self.text(Key::HairStyle)));
        let thumbnails = match (make.doram, make.sex == 1) {
            (false, true) => "",
            (false, false) => "_girl",
            (true, true) => "_doramboy",
            (true, false) => "_doramgirl",
        };
        for index in 0..24u16 {
            let place = at(i32::from(index % 4) * 38 + 606, i32::from(index / 4) * 37 + 59, 36, 37);
            let style = index + 1;
            if style > hair_styles(make.doram) {
                inner.push_str(&format!(r#"<div class="a" style="{place};{}"></div>"#, background(&format!("{folder}/img_hairstyle_none.bmp"))));
                continue;
            }
            let frame = if style == make.hair_style {
                let select = format!("{folder}/bt_hairstyle_select.bmp");
                states(&select, &select, &select)
            } else {
                states_of(&format!("{folder}/bt_hairstyle"))
            };
            let thumbnail = format!(r#"<img src="{}" alt="">"#, picture(&format!("{folder}/img_hairstyle{thumbnails}{style:02}.bmp")));
            inner.push_str(&button(&place, &frame, &click("hair", style), &thumbnail));
        }

        inner.push_str(&format!(r#"<div class="a small" style="{};color:#404040">{}</div>"#, at(609, 285, 150, 14), self.text(Key::HairColor)));
        for colour in 0..HAIR_COLOURS {
            let base = format!("{folder}/color{:02}", colour + 1);
            let pictures = if colour == make.hair_colour {
                let on = format!("{base}_on.bmp");
                states(&on, &on, &on)
            } else {
                states(&format!("{base}_off.bmp"), &format!("{base}_over.bmp"), &format!("{base}_press.bmp"))
            };
            inner.push_str(&button(&at(i32::from(colour % 5) * 27 + 619, i32::from(colour / 5) * 24 + 306, 22, 22), &pictures, &click("colour", colour), ""));
        }

        let make_pictures = states_of(&format!("{folder}/bt_make"));
        inner.push_str(&button(&at(25, 379, 152, 27), &make_pictures, &format!(r#"{} data-big"#, click("back", "")), self.text(Key::Return)));
        inner.push_str(&format!(r#"<button class="a b" data-big style="{};{make_pictures}">{}</button>"#, at(616, 379, 152, 27), self.text(Key::Create)));
        inner.push_str(&button(&at(768, 9, 17, 16), &states_of(&format!("{folder}/bt_close")), &click("back", ""), ""));

        let size = (794, 422);
        let place = centred(view, size);
        let inner = format!(r#"<form data-submit="create">{inner}</form>"#);
        window(view, size, (place.0, place.1 - 70.0), "", &format!("{folder}/bg_makebg.bmp"), &inner)
    }
}
