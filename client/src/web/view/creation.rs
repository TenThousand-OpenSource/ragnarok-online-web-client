//! Making a character (the client's `UINewMakeCharWnd`).

use super::*;
use crate::press::account::{Press, Race, Sex, Turn};

/// Where the window's pictures are.
const FOLDER: &str = "make_character_ver2";

/// The pictures of a button that is one of several to choose from: its one `_on` picture while it is the one
/// chosen, and `_off`, `_over` and `_press` otherwise.
fn chosen_states(base: &str, chosen: bool) -> String {
    if chosen {
        let on = format!("{base}_on.bmp");
        states(&on, &on, &on)
    } else {
        states(&format!("{base}_off.bmp"), &format!("{base}_over.bmp"), &format!("{base}_press.bmp"))
    }
}

impl App {
    /// The window, 794 by 422: its title, the two races, the sex and the buttons that turn the new character,
    /// the character itself over its name, the hair styles and colours to choose from, and its buttons.
    ///
    /// The window's title is bold, the titles over the hair styles and colours are the smaller text in bold,
    /// and the words on its two buttons are (0, 14), bold (`UINewMakeCharWnd::Paint`, and its `OnCreate`:
    /// `SetFont(0, 0xB, 1, 0)`, `AddTextStyle(.., 0xE, .., 1, ..)`).
    pub(super) fn char_create_window(&self, view: (f64, f64)) -> String {
        let mut inner = format!(r#"<div class="a" style="{};{};color:#5f5f5f">{}</div>"#, at(44, 17, 300, 14), BOLD.style(), self.text(Key::CharacterCreation));
        inner.push_str(&self.creation_races());
        inner.push_str(&self.creation_figure());
        inner.push_str(&self.creation_hair_styles());
        inner.push_str(&self.creation_hair_colours());
        inner.push_str(&self.creation_buttons());

        let size = (794, 422);
        let place = centred(view, size);
        let inner = format!(r#"<form {}>{inner}</form>"#, sent(Form::Create));
        window(view, size, (place.0, place.1 - 70.0), "", &format!("{FOLDER}/bg_makebg.bmp"), &inner)
    }

    /// The two races: the chosen one is lit. A classic server makes no Doram (rAthena lets one be made only
    /// under Renewal's rules: its `allowed_job_flag`), so there the Doram's panel is shown and cannot be
    /// pressed. The client itself has no such switch, it is always shown a server of its own day: this is
    /// this project's.
    fn creation_races(&self) -> String {
        let mut inner = String::new();
        for (y, picture, race, name, about) in [(36, "human", Race::Human, Key::Human, Key::HumanAbout), (202, "doram", Race::Doram, Key::Doram, Key::DoramAbout)] {
            let lit = if (race == Race::Doram) == self.make.doram { "on" } else { "off" };
            let offered = race == Race::Human || data::renewal();
            inner.push_str(&format!(
                r#"<button type="button" class="a race" style="{};{}" {}><b>{}</b><p>{}</p></button>"#,
                at(21, y, 364, 166),
                background(&format!("{FOLDER}/img_{picture}_{lit}.bmp")),
                if offered { pressed(Press::Race(race)) } else { "disabled".to_owned() },
                self.text(name),
                self.text(about)
            ));
        }
        inner
    }

    /// The sex, the buttons that turn the new character, the character itself (its feet at 500, 228) and
    /// the field of its name.
    fn creation_figure(&self) -> String {
        let mut inner = String::new();
        for (x, sex, name) in [(437, Sex::Male, "male"), (500, Sex::Female, "female")] {
            let pictures = chosen_states(&format!("{FOLDER}/bt_{name}"), (self.make.sex == 1) == (sex == Sex::Male));
            inner.push_str(&button(&at(x, 59, 62, 25), &pictures, &pressed(Press::Sex(sex)), ""));
        }
        inner.push_str(&button(&at(426, 179, 22, 23), &states_of(&format!("{FOLDER}/bt_leftturn")), &pressed(Press::Turn(Turn::Left)), ""));
        inner.push_str(&button(&at(551, 179, 22, 23), &states_of(&format!("{FOLDER}/bt_rightturn")), &pressed(Press::Turn(Turn::Right)), ""));
        inner.push_str(&format!(r#"<canvas class="a pass" id="pc-make" width="{}" height="{}" style="left:420px;top:91px"></canvas>"#, figure::WIDTH, figure::HEIGHT));
        inner.push_str(&format!(r#"<input class="a in" id="char-name" maxlength="23" autocomplete="off" autofocus style="{}">"#, at(440, 268, 120, 20)));
        inner
    }

    /// The hair styles under their title: 24 places, four to a row, those the race has each a button with
    /// its picture.
    fn creation_hair_styles(&self) -> String {
        let make = &self.make;
        let mut inner = format!(r#"<div class="a" style="{};{};color:#404040">{}</div>"#, at(609, 44, 150, 14), Font(0, 11, true).style(), self.text(Key::HairStyle));
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
                inner.push_str(&format!(r#"<div class="a" style="{place};{}"></div>"#, background(&format!("{FOLDER}/img_hairstyle_none.bmp"))));
                continue;
            }
            let frame = if style == make.hair_style {
                let select = format!("{FOLDER}/bt_hairstyle_select.bmp");
                states(&select, &select, &select)
            } else {
                states_of(&format!("{FOLDER}/bt_hairstyle"))
            };
            let thumbnail = format!(r#"<img src="{}" alt="">"#, picture(&format!("{FOLDER}/img_hairstyle{thumbnails}{style:02}.bmp")));
            inner.push_str(&button(&place, &frame, &pressed(Press::Hair(style)), &thumbnail));
        }
        inner
    }

    /// The hair colours under their title, five to a row.
    fn creation_hair_colours(&self) -> String {
        let mut inner = format!(r#"<div class="a" style="{};{};color:#404040">{}</div>"#, at(609, 285, 150, 14), Font(0, 11, true).style(), self.text(Key::HairColor));
        for colour in 0..HAIR_COLOURS {
            let pictures = chosen_states(&format!("{FOLDER}/color{:02}", colour + 1), colour == self.make.hair_colour);
            inner.push_str(&button(&at(i32::from(colour % 5) * 27 + 619, i32::from(colour / 5) * 24 + 306, 22, 22), &pictures, &pressed(Press::Colour(colour)), ""));
        }
        inner
    }

    /// Return and Create along the bottom, and the cross in the corner.
    fn creation_buttons(&self) -> String {
        let large = Font(0, 14, true).style();
        let make_pictures = states_of(&format!("{FOLDER}/bt_make"));
        let mut inner = button(&format!("{};{large}", at(25, 379, 152, 27)), &make_pictures, &format!(r#"{} data-big"#, pressed(Press::Back)), self.text(Key::Return));
        inner.push_str(&format!(r#"<button class="a b" data-big style="{};{large};{make_pictures}">{}</button>"#, at(616, 379, 152, 27), self.text(Key::Create)));
        inner.push_str(&button(&at(768, 9, 17, 16), &states_of(&format!("{FOLDER}/bt_close")), &pressed(Press::Back), ""));
        inner
    }
}
