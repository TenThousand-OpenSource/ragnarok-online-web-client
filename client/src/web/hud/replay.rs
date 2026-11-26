//! The replays on the page: the recorder (the client's `UIReplayRECControlWnd`, 194 by 34, and 200 tall with
//! its options), the list of recordings before the game (`UISelectReplayDataWnd`, 280 by 311) and the player
//! (`UIReplayControlWnd`, 197 by 138, on `replay_base.bmp`).

use super::*;
use crate::press::replay::Press;
use crate::replay::option;

/// Where the pictures of the replays are.
const ART: &str = "replay_interface/";

/// One of the recorder's and the player's buttons: `name_a.bmp`, and `name_b.bmp` pressed.
fn button(x: i32, y: i32, size: (i32, i32), name: &str, tip: &str, press: Option<Press>) -> String {
    let (off, down) = (format!("{ART}{name}_a.bmp"), format!("{ART}{name}_b.bmp"));
    match press {
        Some(press) => pictured(x, y, size, [&off, &off, &down], press, &format!(r#" data-tip="{}""#, escape(tip))),
        None => format!(r#"<div class="a" style="{};background:url('{}');opacity:.5"></div>"#, at(x, y, size.0, size.1), picture(&off)),
    }
}

impl App {
    /// The recorder (`UIReplayRECControlWnd::Paint` and `ApplyMode`): a rounded strip in the client's blue
    /// with four rims; on it the picture at (6, 5) that turns while a recording runs, the time at (32, 12),
    /// and at 110, 136 and 162 the buttons that begin or stop, open the options and put the recorder away
    /// (the last two dim while a recording runs). Asking for a name, a field 151 wide at (7, 8) and the button
    /// that begins at 162. With its options, the strip says so (line 1971) and a grey panel under it has two
    /// white boxes: what is recorded (line 1973) and how the file is named (1974).
    pub(in crate::web) fn replay_recorder(&self) -> String {
        let replay = &self.replay;
        let rim = |inset: i32, colour: &str| {
            format!(
                r#"<div class="a" style="{};border:1px solid {colour};border-radius:4px;box-sizing:border-box"></div>"#,
                at(inset, inset, 194 - 2 * inset, 34 - 2 * inset)
            )
        };
        let mut window = [
            format!(r#"<div class="a" style="{};background:#99b8f5;border-radius:4px"></div>"#, at(0, 0, 194, 34)),
            rim(0, "#635a94"),
            rim(1, "#adc6f7"),
            rim(2, "#fff"),
            rim(3, "#847bad"),
        ]
        .concat();
        let passed = replay.recording.as_ref().map_or(0, |recording| crate::web::app::now().wrapping_sub(recording.started));
        let strip = |turn: usize| art(&format!("{ART}replay_recrun_{}.bmp", turn + 1), 6, 5, 103, 24);
        match replay.mode {
            1 => {
                window.push_str(&strip(crate::replay::turning(passed)));
                window.push_str(&text(32, 12, WHITE, &crate::replay::clock(passed)));
                window.push_str(&button(110, 5, (24, 24), "btn_stop", &Self::words(1978, "Stop"), Some(Press::Stop)));
                window.push_str(&button(136, 5, (24, 24), "btn_save", "", None));
                window.push_str(&button(162, 5, (24, 24), "btn_close", "", None));
            }
            2 => window.push_str(&self.replay_options()),
            3 => {
                window.push_str(&[
                    format!(r#"<form {} class="a" style="{}">"#, sent(Form::ReplayName), at(7, 8, 151, 18)),
                    format!(r#"<input id="replay-name" class="a in" style="{};background:#e8e8e8" type="text" "#, at(0, 0, 151, 18)),
                    format!(
                        r#"maxlength="80" autocomplete="off" value="{}" data-tip="{}">"#,
                        escape(&replay.name),
                        escape(&Self::words(1972, "Enter File Name"))
                    ),
                    r#"<button type="submit" hidden></button></form>"#.to_owned(),
                ].concat());
                window.push_str(&button(162, 5, (24, 24), "btn_start", &Self::words(1979, "Input FileName -> Start"), Some(Press::NameOk)));
            }
            _ => {
                window.push_str(&strip(0));
                window.push_str(&text(32, 12, WHITE, "000:00:00"));
                window.push_str(&button(110, 5, (24, 24), "btn_start", &Self::words(1977, "Start"), Some(Press::Start)));
                window.push_str(&button(136, 5, (24, 24), "btn_save", &Self::words(1980, "Open Option"), Some(Press::Options)));
                window.push_str(&button(162, 5, (24, 24), "btn_close", &Self::words(1982, "End"), Some(Press::End)));
            }
        }
        window
    }

    /// The recorder's options (`ApplyMode` with the mode 2, `Paint`): the boxes of what is recorded in two
    /// columns at 17 and 104, 18 apart from 63 down (the guild's and the status' boxes are not shown), and
    /// the two boxes of the file's name at 160 and 178.
    fn replay_options(&self) -> String {
        let replay = &self.replay;
        let boxed = |(x, y): (i32, i32), words: &str, place: usize| check_words(x, y, replay.options[place] != 0, words, Press::Option(place));
        let titled = |y: i32, line: usize, otherwise: &str| {
            format!(
                r#"<div class="a t" style="left:10px;top:{y}px;color:#003c7f;font-weight:bold;{}">{}</div>"#,
                outlined("#fff"),
                escape(&Self::words(line, otherwise))
            )
        };
        [
            art(&format!("{ART}replay_recrun_5.bmp"), 6, 5, 153, 24),
            text(42, 12, WHITE, &Self::words(1971, "Replay Option Setting")),
            button(162, 5, (24, 24), "btn_setup", &Self::words(1981, "Close Option"), Some(Press::OptionsClose)),
            fill(0, 34, 194, 166, "#dedede"),
            fill(4, 38, 186, 95, WHITE),
            fill(4, 138, 186, 57, WHITE),
            titled(42, 1973, "Set Replay Save Data"),
            titled(142, 1974, "Set Rec Option"),
            boxed((17, 63), "Basicinfo", option::BASIC),
            boxed((17, 81), "Equip", option::EQUIP),
            boxed((17, 99), "Item", option::ITEM),
            boxed((17, 117), "Skill", option::SKILL),
            boxed((104, 63), "ALL", option::ALL),
            boxed((104, 81), "Party", option::PARTY),
            boxed((104, 99), "Chatting", option::CHATTING),
            boxed((104, 117), "Shortcut", option::SHORTCUT),
            check_words(17, 160, replay.name_asked, "User Defined File Name", Press::NameAsked),
            check_words(17, 178, replay.checked, "Repeated File Check", Press::Checked),
        ]
        .concat()
    }

    /// The list of recordings (`UISelectReplayDataWnd::OnCreate`, `Paint`, `UpdateInfo`): the title (line
    /// 2084); "File info" (2085) over a grey box 268 by 77 at (6, 32) with five lines about the recording
    /// picked (the service, the character, the map, when it was recorded and how long it is: lines 1964 to
    /// 1968), 15 apart from (12, 35), in the client's blue; "File List" (2086) at 112 over the names, 16 a
    /// row, from (6, 127); on the bar OK at 189, the way back at 100 and, where the client has Exit, nothing
    /// (a page is not quit). Of this project's own, over the bar: the three buttons that save a recording as
    /// a file, take a file in, and throw a recording away.
    pub(in crate::web) fn replay_list(&self) -> String {
        let replay = &self.replay;
        let picked = replay.picked.and_then(|picked| replay.files.get(picked));
        let info: Vec<String> = match picked.map(|(_, bytes)| crate::replay::header(bytes)) {
            Some(Some(header)) => match crate::replay::service_name(header.service_type) {
                Some(service) => {
                    let when: Vec<String> = header.when.iter().map(u16::to_string).collect();
                    let (hours, minutes, seconds) = crate::replay::parts(header.duration);
                    let long = [hours, minutes, seconds].map(|part| part.to_string());
                    vec![
                        Self::words(1964, "Service Info: %s").replacen("%s", service, 1),
                        Self::words(1965, "Character Name: %s").replacen("%s", &header.name, 1),
                        Self::words(1966, "Map Name: %s").replacen("%s", &header.map, 1),
                        crate::numbers::filled(&Self::words(1967, ""), &when.iter().map(String::as_str).collect::<Vec<_>>()),
                        crate::numbers::filled(&Self::words(1968, ""), &long.iter().map(String::as_str).collect::<Vec<_>>()),
                    ]
                }
                None => vec![String::new(), Self::words(1969, "No Replay File.")],
            },
            Some(None) => vec![String::new(), Self::words(1969, "No Replay File.")],
            None => Vec::new(),
        };
        let rows: String = replay
            .files
            .iter()
            .enumerate()
            .map(|(row, (name, _))| {
                let ground = if replay.picked == Some(row) { "#c6d6f7" } else { "transparent" };
                format!(
                    r#"<div class="t" style="height:16px;padding-left:3px;cursor:pointer;background:{ground}" {} {}>{}</div>"#,
                    pressed(Press::Row(row)),
                    used(Use::Replay(row)),
                    escape(name)
                )
            })
            .collect();
        let own = |x: i32, words: &str, press: Press| basic_button(x, 263, 86, tr(words), Some(press.into()));
        [
            title_bar(280, &Self::words(2084, "Replay File List")),
            panel(280, 311),
            fill(6, 32, 268, 77, "#e7e7e7"),
            text(6, 19, BLACK, &Self::words(2085, "File info")),
            text(6, 112, BLACK, &Self::words(2086, "File List")),
            info.iter().enumerate().map(|(row, line)| fitted(12, 35 + 15 * row as i32, 256, "rgb(100,100,200)", line)).collect(),
            format!(r#"<div class="a bar" style="{};background:#f0f0f0">{rows}</div>"#, at(6, 127, 268, 132)),
            own(6, "Save to a file", Press::Save),
            own(97, "Load a file", Press::Load),
            own(188, "Delete", Press::Delete),
            picture_press_sized(100, 286, (85, 20), &format!("{ART}btn_selectserver"), Press::Back),
            picture_press(189, 286, "btn_ok", Press::Play),
        ]
        .concat()
    }

    /// The player (`UIReplayControlWnd::OnCreate`, `RefreshInfo`): on `replay_base.bmp`; how far it is and how
    /// long (line 1955) at (8, 26), the speed (1956 to 1962) at (8, 40), the map at (8, 54), in the client's
    /// blue; the bar of how far it is, 188 by 11 at (5, 72); and the buttons: to go on or hold at (5, 89),
    /// faster at (53, 89) and slower at (53, 111), from the beginning at (101, 89), out at (149, 89).
    pub(in crate::web) fn replay_player(&self) -> String {
        let Some(playing) = &self.replay.playing else { return String::new() };
        let duration = playing.file.header.duration;
        let (at_now, whole) = (crate::replay::parts(playing.elapsed as u32), crate::replay::parts(duration));
        let numbers = [at_now.0, at_now.1, at_now.2, whole.0, whole.1, whole.2].map(|part| part.to_string());
        let passed = crate::numbers::filled(&Self::words(1955, "Elapsed time: %d:%d:%d / %d:%d:%d"), &numbers.iter().map(String::as_str).collect::<Vec<_>>());
        let filled = if duration == 0 { 188.0 } else { (playing.elapsed / f64::from(duration) * 188.0).clamp(0.0, 188.0) };
        let blue = "rgb(100,100,200)";
        let map = self.game.as_ref().map(|game| data::map_name(crate::maps::bare(&game.map))).unwrap_or_default();
        let hold = if playing.held || playing.over { "replay_start" } else { "replay_pause" };
        [
            art(&format!("{ART}replay_base.bmp"), 0, 0, 197, 138),
            fitted(8, 26, 188, blue, &passed),
            fitted(8, 40, 188, blue, &Self::words(crate::replay::speed_line(playing.speed), "")),
            fitted(8, 54, 188, blue, &map),
            format!(
                r#"<div class="a" style="{};overflow:hidden"><div style="width:{filled:.0}px;height:7px;margin-top:2px;background:url('{}')"></div></div>"#,
                at(5, 72, 188, 11),
                picture(&format!("{ART}replay_gage.bmp"))
            ),
            button(5, 89, (43, 43), hold, "", Some(Press::Hold)),
            button(53, 89, (43, 22), "replay_speedadd", "", Some(Press::Faster)),
            button(53, 111, (43, 22), "replay_speedmin", "", Some(Press::Slower)),
            button(101, 89, (43, 43), "replay_stop", "", Some(Press::Again)),
            button(149, 89, (43, 43), "replay_end", "", Some(Press::Out)),
        ]
        .concat()
    }
}
