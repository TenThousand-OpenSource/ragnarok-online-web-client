//! The maps: their music, what the client calls them, and who stands where in the towns.

use super::*;

thread_local! {
    /// The music of each map, as the client's `mp3nametable.txt` names it: `prontera` to `bgm\\08.mp3`.
    static MUSIC: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    /// What the client calls each map, by the map's name: its full name, the large and the small line of its
    /// sign, the sign's picture, and whether the sign is shown on coming in.
    static MAPS: RefCell<HashMap<String, MapSign>> = RefCell::new(HashMap::new());
    /// Who of use stands where in each town, by the map's name (the client's `Towninfo.lub`): the name, the cell
    /// and the kind (0 tool dealer, 1 weapon dealer, 2 armour dealer, 3 smith, 4 guide, 5 inn, 6 Kafra, 7 styling).
    static TOWNS: RefCell<HashMap<String, Vec<(String, i32, i32, i32)>>> = RefCell::new(HashMap::new());
}

/// The music file of a map, if the client's table has one for it.
pub fn music(map: &str) -> Option<String> {
    MUSIC.with_borrow(|music| music.get(&map.to_ascii_lowercase()).cloned())
}

/// Reads the client's table of which music plays on which map: lines of `map.rsw#bgm\\08.mp3#`.
pub async fn load_music() {
    let Ok(bytes) = crate::web::fetch_asset("data/mp3nametable.txt").await else { return };
    let table = String::from_utf8_lossy(&bytes);
    let music = table
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .filter_map(|line| {
            let mut parts = line.split('#');
            let (map, track) = (parts.next()?.trim(), parts.next()?.trim());
            Some((map.to_ascii_lowercase().strip_suffix(".rsw")?.to_owned(), track.replace("\\\\", "\\")))
        })
        .filter(|(_, track)| !track.is_empty())
        .collect();
    MUSIC.set(music);
}

/// What the client calls a map (its `mapInfo_true.lub`).
#[derive(Clone, Default)]
pub struct MapSign {
    /// The full name, as `/where` gives it.
    pub name: String,
    /// The sign shown on coming in: its large line, the small one over it, its picture, and whether it shows.
    pub title: String,
    pub over: String,
    pub picture: String,
    pub shown: bool,
}

/// Those the minimap marks on that map (the client's `CGameMode::DrawMiniMap`: the table's entries in turn, up to
/// the first of kind 8 or 9, which are not for the minimap).
pub fn town_marks(map: &str) -> Vec<(String, i32, i32, i32)> {
    TOWNS.with_borrow(|towns| towns.get(map.trim_end_matches(".gat").trim_end_matches(".rsw")).map(|people| people.iter().take_while(|person| !matches!(person.3, 8 | 9)).cloned().collect()).unwrap_or_default())
}

/// What the client calls that map, if its table has it.
pub fn map_sign(map: &str) -> Option<MapSign> {
    MAPS.with_borrow(|maps| maps.get(map.trim_end_matches(".gat").trim_end_matches(".rsw")).cloned())
}

/// Takes its tables out of what the gateway gave: what the client calls each map, and who stands where in the
/// towns.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    TOWNS.set(
        entries::<String>(&field(tables, "towns"))
            .into_iter()
            .map(|(map, people)| {
                let people = js_sys::Array::from(&people)
                    .iter()
                    .filter_map(|person| {
                        let person = js_sys::Array::from(&person);
                        let number = |at: u32| person.get(at).as_f64().map(|number| number as i32);
                        Some((person.get(0).as_string()?, number(1)?, number(2)?, number(3)?))
                    })
                    .collect();
                (map, people)
            })
            .collect(),
    );
    MAPS.set(
        entries::<String>(&field(tables, "maps"))
            .into_iter()
            .map(|(map, about)| {
                let sign = MapSign {
                    name: text_of(&about, "name").unwrap_or_default(),
                    title: text_of(&about, "title").unwrap_or_default(),
                    over: text_of(&about, "over").unwrap_or_default(),
                    picture: text_of(&about, "picture").unwrap_or_default(),
                    shown: js_sys::Reflect::get(&about, &"shown".into()).ok().and_then(|shown| shown.as_bool()).unwrap_or(false),
                };
                (map, sign)
            })
            .collect(),
    );
}
