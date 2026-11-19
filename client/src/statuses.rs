//! The states a character is in (its buffs and debuffs), as the client lists them at the right of the screen:
//! which are shown, in what order, where each stands, and how the time left is written.
//!
//! The pictures and descriptions are the client's Lua files' (`web/data/statuses.rs`); what the client has
//! compiled into it besides is `statuses.txt`, written out by `tools/client-tables/statuses.py`: the pictures of
//! the states its Lua files have none for, and the states its body and health states stand for.

/// The state the client keeps a second number for: 42 is listed, described and timed as 12 (Increase Agility),
/// while its picture is asked for by its own number (`CRagEffect::StatusIconColumn`,
/// `CGameMode::GetActiveSkillEffectTimeText`).
const ALIAS: (u32, u32) = (42, 12);
/// How many of the states the client lists at most (`CSession::m_statusIconIds` holds a hundred, the last unused).
const MOST: usize = 99;
/// The kinds the client goes through, one after another, when it lists the states (`StatusIconColumn`).
pub const KINDS: u8 = 5;

/// The lines of `statuses.txt` of one sort, each as its fields after the first.
fn lines(sort: &'static str) -> impl Iterator<Item = Vec<&'static str>> {
    crate::tables::sorted(include_str!("statuses.txt"), sort)
}

/// The picture the client itself has for a state listed under a kind (`CSession::GetEFSTImgFileName`, where
/// its Lua files have none): a file in `data\texture\effect`. The jobs of the Taekwon line have their own for
/// some (`IsTaekwonDisplayJob`).
pub fn compiled_icon(state: u32, kind: u8, taekwon: bool) -> Option<&'static str> {
    let of = |sort| lines(sort).find(|fields| fields.len() == 3 && fields[0].parse() == Ok(kind) && fields[1].parse() == Ok(state)).map(|fields| fields[2]);
    taekwon.then(|| of("taekwon")).flatten().or_else(|| of("icon"))
}

/// The state shown for a body state (stone, frozen, stunned, asleep…), as the server's state packet names it
/// (`BuildStatusLevelEffects`).
pub fn of_body(level: u16) -> Option<u32> {
    lines("body").find(|fields| fields.len() == 2 && fields[0].parse() == Ok(level)).and_then(|fields| fields[1].parse().ok())
}

/// The states shown for the bits of a health state (poison, curse, silence…), each with whether its bit is set
/// (`BuildStatusFlagEffects`, less those `ApplyStatusIcons` leaves out).
pub fn of_health(bits: u16) -> impl Iterator<Item = (u32, bool)> {
    lines("health").filter_map(move |fields| Some((fields.get(1)?.parse().ok()?, bits >> fields.first()?.parse::<u32>().ok()? & 1 == 1)))
}

/// One state as it is listed: the number it is known by in the list, and the kind and number its picture is
/// asked for with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Listed {
    pub state: u32,
    pub kind: u8,
    pub pictured: u32,
}

/// The states listed, in the client's order (`CRagEffect::StatusIconColumn`): kind by kind, within a kind in
/// the order they came, each once, those without a picture left out. `has` tells whether a state has a picture
/// under a kind.
pub fn listed(active: impl Iterator<Item = u32> + Clone, has: impl Fn(u32, u8) -> bool) -> Vec<Listed> {
    let mut found: Vec<Listed> = Vec::new();
    for kind in 0..KINDS {
        for pictured in active.clone() {
            let state = if pictured == ALIAS.0 { ALIAS.1 } else { pictured };
            if found.len() < MOST && has(pictured, kind) && !found.iter().any(|listed| listed.state == state) {
                found.push(Listed { state, kind, pictured });
            }
        }
    }
    found
}

/// Where the picture of the state at that place of the list stands on a screen so high: how far its right edge
/// is from the screen's, and its top. The client draws each 32 by 32 around a point 32 from the right edge and
/// 35 further down than the last, the first at 185; a column that would pass 50 above the bottom is followed by
/// another 45 to the left (`StatusIconColumn`).
pub fn place(index: usize, height: i32) -> (i32, i32) {
    let (mut across, mut down, mut row) = (0, 0, 0);
    for _ in 0..=index {
        if row * 35 + 150 < height - 50 {
            down += 35;
        } else {
            (row, down, across) = (0, 35, across + 45);
        }
        row += 1;
    }
    (16 + across, down + 150 - 16)
}

/// The time left as the client writes it into a state's description (`GetActiveSkillEffectTimeText`): hours,
/// minutes and seconds, each with its word from the message table, those that are nought left out.
pub fn time_left(seconds: u32, words: [&str; 3]) -> String {
    let (hours, minutes, seconds) = (seconds / 3600, seconds % 3600 / 60, seconds % 60);
    let mut text = if hours != 0 { format!("{hours} {}", words[0]) } else { String::new() };
    for (count, word) in [(minutes, words[1]), (seconds, words[2])] {
        if count != 0 {
            text.push_str(&format!(" {count} {word}"));
        }
    }
    text
}

/// The picture of a state listed under a kind, as the client finds it (`CSession::GetEFSTImgFileName`): what
/// its Lua files have for the state (`lua`: the kind it is listed under there, and the file) is asked first,
/// then what it has compiled into it (`compiled_icon`). A file under `data\texture`, written with forward
/// strokes.
pub fn icon(lua: Option<(u8, &str)>, state: u32, kind: u8, taekwon: bool) -> Option<String> {
    lua.filter(|(of, _)| *of == kind)
        .map(|(_, file)| file.replace('\\', "/"))
        .or_else(|| compiled_icon(state, kind, taekwon).map(|file| format!("effect/{file}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::taekwon;

    #[test]
    fn a_picture_is_the_lua_files_or_else_the_compiled_one() {
        assert_eq!(icon(Some((4, "effect\\bless.tga")), 10, 4, false).as_deref(), Some("effect/bless.tga"));
        assert_eq!(icon(Some((4, "")), 10, 4, false).as_deref(), Some(""));
        // Listed under another kind in the Lua files: the compiled one, if there is one under this kind.
        assert_eq!(icon(Some((1, "effect\\bless.tga")), 10, 4, false).as_deref(), Some("effect/블레싱.tga"));
        assert_eq!(icon(Some((4, "effect\\bless.tga")), 10, 1, false), None);
        assert_eq!(icon(None, 10, 4, false).as_deref(), Some("effect/블레싱.tga"));
        assert_eq!(icon(None, 90, 1, true).as_deref(), Some("effect/i_p_FIRE.tga"));
        assert_eq!(icon(None, 10, 1, false), None);
    }

    #[test]
    fn the_client_has_pictures_for_the_classic_states() {
        // Blessing and Increase Agility are among the last kind; Quagmire among the harmful ones.
        assert_eq!(compiled_icon(10, 4, false), Some("블레싱.tga"));
        assert_eq!(compiled_icon(12, 4, false), Some("민첩성증가.tga"));
        assert_eq!(compiled_icon(8, 1, false), Some("콰그마이어.tga"));
        assert_eq!(compiled_icon(10, 1, false), None);
        // A weapon of fire has another picture for the Taekwon line.
        assert_eq!(compiled_icon(90, 1, false), Some("프레임런쳐.tga"));
        assert_eq!(compiled_icon(90, 1, true), Some("i_p_FIRE.tga"));
        assert!(taekwon(4046) && !taekwon(7));
    }

    #[test]
    fn body_and_health_states_stand_for_states() {
        assert_eq!(of_body(2), Some(876));
        assert_eq!(of_body(0), None);
        assert_eq!(of_body(5), None);
        let poisoned: Vec<_> = of_health(0b1).collect();
        assert!(poisoned.contains(&(883, true)) && poisoned.contains(&(884, false)));
        // Blindness (bit 4) is one the client leaves out.
        assert_eq!(of_health(0xffff).count(), 6);
    }

    #[test]
    fn states_are_listed_kind_by_kind_and_once() {
        let has = |state: u32, kind: u8| matches!((state, kind), (10, 4) | (12, 4) | (42, 4) | (8, 1));
        let list = listed([10, 8, 99, 12, 42, 10].into_iter(), has);
        assert_eq!(list.iter().map(|listed| listed.state).collect::<Vec<_>>(), vec![8, 10, 12]);
        assert_eq!(list[0].kind, 1);
        // 42 is 12 in the list, and only once.
        let list = listed([42].into_iter(), has);
        assert_eq!(list, vec![Listed { state: 12, kind: 4, pictured: 42 }]);
    }

    #[test]
    fn pictures_go_down_and_then_into_another_column() {
        assert_eq!(place(0, 768), (16, 169));
        assert_eq!(place(1, 768), (16, 204));
        // On a screen 768 high a column has 17: rows 0 to 16 are above 718.
        assert_eq!(place(16, 768), (16, 169 + 16 * 35));
        assert_eq!(place(17, 768), (61, 169));
        assert_eq!(place(18, 768), (61, 204));
    }

    #[test]
    fn the_time_left_is_written_in_parts() {
        let words = ["hour", "min", "sec"];
        assert_eq!(time_left(3725, words), "1 hour 2 min 5 sec");
        assert_eq!(time_left(65, words), " 1 min 5 sec");
        assert_eq!(time_left(120, words), " 2 min");
        assert_eq!(time_left(0, words), "");
    }
}
