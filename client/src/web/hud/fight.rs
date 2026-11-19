//! The windows a fight opens: what Sense tells of a monster (the client's `UIMonsterInfoWnd`), and the question
//! whether the map is to be a Star Gladiator's place (`UIStarPlaceAcceptWnd`).

use super::*;
use crate::press::fight::Press;

/// The size of the Monster Info window (`WID_MONSTERINFOWND`: 400 by 200).
pub(in crate::web) const SENSE: (i32, i32) = (400, 200);

/// One of the nine boxes that say how much of an attack of an element gets through
/// (`UIMonsterInfoWnd::DrawResistance`): 70 by 16 with a rim, "name:hundredths" in its middle, 2 down.
fn resistance(x: i32, y: i32, element: &str, taken: u8) -> String {
    format!(
        r#"<div class="a t" style="{};border:1px solid #c8c8c8;box-sizing:border-box;text-align:center;line-height:12px;padding-top:1px">{}:{taken}</div>"#,
        at(x, y, 70, 16),
        escape(element)
    )
}

impl App {
    /// The Monster Info window (`UIMonsterInfoWnd::Paint`): the usual title and panel with OK at the bottom,
    /// the name, level, life and defence in one column (their names 120 in, what they are 160 in, 20 apart
    /// from 30 down), the size, race, element and magic defence in another (260 and 300 in), and three rows
    /// of three boxes from 110 down for what each element does to it. The lines are the client's: 406 to 413
    /// the names, 3887 on the races, 3897 on the elements, 3907 on the sizes, 3910 "DEF". (The client also
    /// draws the monster itself in the left of the window, 116 by 152 from 2, 19: not drawn here yet.)
    pub(in crate::web) fn sense_window(&self) -> String {
        let Some((sensed, name)) = &self.fight.sensed else { return String::new() };
        let (width, height) = SENSE;
        let line = Self::words;
        let mut window = format!("{}{}", title_bar(width, &line(406, "Monster Info")), panel(width, height));
        let pair = |x: i32, y: i32, name: String, what: &str| format!("{}{}", text(x, y, "#000", &name), text(x + 40, y, "#000", what));
        window.push_str(&pair(120, 30, line(407, "Name"), name));
        window.push_str(&pair(120, 50, line(408, "Level"), &sensed.level.to_string()));
        window.push_str(&pair(120, 70, line(409, "HP"), &sensed.life.to_string()));
        window.push_str(&pair(120, 90, line(3910, "DEF"), &sensed.defence.to_string()));
        // A size, race or element the client's lines do not name is left empty, as its window leaves it.
        let named = |first: usize, which: u16, count: u16| if which < count { line(first + usize::from(which), "") } else { String::new() };
        window.push_str(&pair(260, 30, line(410, "Size"), &named(3907, sensed.size, 3)));
        window.push_str(&pair(260, 50, line(411, "Race"), &named(3887, sensed.race, 10)));
        window.push_str(&pair(260, 70, line(413, "Element"), &named(3897, sensed.element % 20, 10)));
        window.push_str(&pair(260, 90, line(412, "MDEF"), &sensed.magic_defence.to_string()));
        for (place, taken) in sensed.taken.iter().enumerate() {
            let (column, row) = ((place / 3) as i32, (place % 3) as i32);
            window.push_str(&resistance(120 + column * 80, 110 + row * 18, &line(3898 + place, ""), *taken));
        }
        window.push_str(&picture_press(width - 46, height - 24, "btn_ok", Press::SenseOk));
        window
    }

    /// The question whether this map is to be the place of the sun, the moon or the stars, in the client's
    /// message box (its line 1028).
    pub(in crate::web) fn star_question(&self) -> String {
        if self.fight.star.is_none() {
            return String::new();
        }
        message_box(Asked::Offer, &escape(&Self::words(1028, "Do you want to designate this map?")), Press::StarYes, Press::StarNo)
    }
}
