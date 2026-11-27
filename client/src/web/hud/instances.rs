//! The window of the party's instance (the client's `UIMemorialDunWnd`, "Memorial Dungeon"). Every place and
//! size is the client's, from its `OnCreate`, `Paint` and `ArrangeButtons`.

use super::*;
use crate::instances::{Instance, SIZE, State};
use crate::press::instances::Press;

/// The colour of the window's rim (the client's 0x8E8E8E) and of the instance's name in its first line
/// (0x41, 0x69, 0xE1).
const RIM: &str = "#8e8e8e";
const NAME: &str = "4169E1";

impl App {
    /// The window (`UIMemorialDunWnd::Paint`): the title bar with its two buttons; on white, the instance's
    /// name in blue within the line of its state, its middle 30 down; under it, by the state: the place in
    /// the queue in the window's middle; or by when the instance goes if nobody enters, written across the
    /// 180 in the middle and about 79 down, with the leader's button that destroys it (125 wide, 41 across
    /// and 115 down); or "Dungeon Mission Time Limit:" 10 across and 55 down with the time it ends at in red
    /// under it, 75 down.
    ///
    /// The left button of the title bar is drawn and does nothing, as in this client's other windows. Decided
    /// here: the lines about entering are stood about their middle whatever their number; the client stands a
    /// single line 55 down instead, which its texts never are.
    pub(in crate::web) fn instance_window(&self) -> String {
        let Some(instance) = &self.instances.known else { return String::new() };
        let (width, height) = SIZE;
        let centred = |y: i32, line: &str| format!(
            r#"<div class="a t" style="left:0;top:{y}px;width:{width}px;text-align:center;white-space:nowrap;color:{BLACK}">{}</div>"#,
            coloured(line)
        );
        let headline = Self::words(crate::instances::headline(instance.state), "%s").replacen("%s", &format!("^{NAME}[{}]^000000", instance.name), 1);
        let mut window = [
            title_bar(width, &Self::words(crate::instances::TITLE, "Memorial Dungeon")),
            base_button(3, 3),
            cross(width, Press::Close),
            format!(
                r#"<div class="a" style="{};background:#fff;border:1px solid {RIM};border-top:0;box-sizing:border-box"></div>"#,
                at(0, 17, width, height - 17)
            ),
            centred(30, &headline),
        ]
        .concat();
        window.push_str(&match instance.state {
            State::Waiting => {
                let place = Self::words(crate::instances::PRIORITY, "Your Standby Priority: ^ff0000%d^000000");
                centred((height - 17) / 2, &place.replacen("%d", &instance.queue.to_string(), 1))
            }
            State::Ready => self.instance_ready(instance),
            State::Running => [
                text(10, 55, BLACK, &Self::words(crate::instances::TIME_LIMIT, "Dungeon Mission Time Limit:")),
                centred(75, &format!("^ff0000{}", Self::instance_time(instance.limit))),
            ]
            .concat(),
        });
        window
    }

    /// What the window says of an instance nobody is in: by when it must be entered, and the leader's button.
    fn instance_ready(&self, instance: &Instance) -> String {
        let (width, _) = SIZE;
        let otherwise = "The requested dungeon will be removed if you do not enter within ^ff0000%s^000000.";
        let words = Self::words(crate::instances::ENTER_WITHIN, otherwise).replacen("%s", &Self::instance_time(instance.idle), 1);
        let mut ready = [
            format!(r#"<div class="a" style="{};display:flex;align-items:center;pointer-events:none">"#, at(10, 40, width - 20, 78)),
            format!(r#"<div class="t" style="line-height:17px;white-space:normal;color:{BLACK}">{}</div></div>"#, coloured(&words)),
        ]
        .concat();
        if crate::instances::destroy_offered(instance.state, self.leads_party()) {
            ready.push_str(&basic_button(41, 115, 125, &Self::words(crate::instances::DESTROY_WORDS, "Destroy instance"), Some(Press::Destroy.into())));
        }
        ready
    }

    /// A time of the server's as the window writes it, on the player's own clock (`strftime` of `_localtime32`).
    fn instance_time(seconds: u32) -> String {
        let date = js_sys::Date::new(&(f64::from(seconds) * 1000.0).into());
        crate::instances::time_written(
            &Self::words(crate::instances::TIME, "%m/%d %H:%M"),
            (date.get_month() + 1, date.get_date()),
            (date.get_hours(), date.get_minutes())
        )
    }
}
