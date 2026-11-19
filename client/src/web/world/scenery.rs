//! What surrounds the player on a map besides its ground and its models (`crate::scenery` has the rules): the
//! effects and sounds the map places, its fog, the sky behind it, the view its camera is given, the weather
//! over it and the darkness of a night. Read with the map (`read`), and moved on once a frame (`step`).

use super::*;
use crate::model::MapSound;
use crate::scenery::dark::Dark;
use crate::scenery::fog::Fog;
use crate::scenery::makers::EffectMaker;
use crate::scenery::views::MapView;

mod placed;

pub(super) use placed::step;

/// One frame of the client's, which the camera's easing, the turning of the models, the waves of the water
/// and the darkening of a night are counted in: taken as a sixtieth of a second (the client's own pace is not
/// in what was read of it).
pub(super) const CLIENT_FRAME_MS: f64 = 1000.0 / 60.0;

/// The first of the IDs the places of a map's effects are known by: effects play on them as on someone
/// (`Scene::spots`), each at its own height (`Surroundings::raised`).
const FIRST_PLACE: u32 = 0xffe0_0000;

/// The effects of the weather the client itself starts on a map (`CWeatherEffect::ApplyMapWeather`, and
/// `CGameMode`'s own snow on two maps): `EF_SNOW` and `EF_SAKURA`.
const EF_SNOW: u32 = 162;
const EF_SAKURA: u32 = 163;

/// An effect a map places: which, the ID of its place, where that is in the world, the figures it is told, and
/// when it is launched.
struct Placed {
    effect: u32,
    spot: u32,
    at: Vector,
    figures: [f32; 4],
    maker: EffectMaker,
}

/// What surrounds the player on the map shown.
pub(super) struct Surroundings {
    effects: Vec<Placed>,
    /// The sounds placed, each with where it is in the world and when it was last played.
    sounds: Vec<(MapSound, Vector, f64)>,
    /// How high the place of each placed effect is, by its ID.
    pub(super) raised: HashMap<u32, f32>,
    /// The map's fog, if the client's table gives it one, and whether it shows now (the player's switch).
    pub(super) fog: Option<Fog>,
    pub(super) fogged: bool,
    /// The colour behind the map.
    pub(super) sky: [f32; 3],
    /// The view the client's table gives the camera here, if any.
    pub(super) view: Option<MapView>,
    /// The effects of the weather that go on over the player's character for as long as the map shows.
    weather: Vec<u32>,
    /// How much of the map's light the night leaves, as the frame draws it.
    pub(super) light: [f32; 3],
    /// When the makers were last moved on.
    stepped: Option<f64>,
}

/// What of the surroundings is kept from map to map: whether fog is shown at all (the client's `/fog`), and
/// the night.
pub(super) struct Kept {
    pub(super) fog: bool,
    pub(super) dark: Dark,
}

impl Default for Kept {
    fn default() -> Self {
        Self { fog: true, dark: Dark::default() }
    }
}

/// A file of the client's tables as text; nothing when it is not there.
async fn table(path: &str) -> String {
    figure::text_file(path).await.unwrap_or_default()
}

impl Surroundings {
    /// The surroundings of a map: what its `.rsw` places, on a ground of that size, and what the client's
    /// tables say of the map: `fogparametertable.txt`, `viewpointtable.txt`, the weather of `etcinfo.txt`, and
    /// the sky (`data::sky`). Gives with them the places of the effects, in cells, for `Scene::spots`.
    pub(super) async fn read(map: &str, rsw: Option<&Rsw>, map_size: (f32, f32)) -> (Self, HashMap<u32, (f32, f32)>) {
        let in_world = |[x, y, z]: Vector| [x + map_size.0 / 2.0, -y, z + map_size.1 / 2.0];
        let (mut effects, mut raised, mut spots) = (Vec::new(), HashMap::new(), HashMap::new());
        for (index, placed) in rsw.iter().flat_map(|rsw| &rsw.effects).enumerate() {
            let (spot, at) = (FIRST_PLACE + index as u32, in_world(placed.position));
            // (A place on cell 3 is drawn at 3.5 cells: `on_ground`.)
            spots.insert(spot, (at[0] / CELL - 0.5, at[2] / CELL - 0.5));
            raised.insert(spot, at[1]);
            effects.push(Placed { effect: placed.effect, spot, at, figures: placed.figures, maker: EffectMaker::new(placed.effect, placed.delay, &mut lot) });
        }
        // A sound is first played as soon as it is in earshot (`CSoundMaker`: its last playing is set a cycle back).
        let sounds = rsw.iter().flat_map(|rsw| &rsw.sounds).map(|sound| (sound.clone(), in_world(sound.position), f64::NEG_INFINITY)).collect();
        let (fogs, views, weathers) = futures_util::future::join3(table("data/fogparametertable.txt"), table("data/viewpointtable.txt"), table("data/etcinfo.txt")).await;
        let sky = data::sky(map);
        let surroundings = Self {
            effects,
            sounds,
            raised,
            fog: Fog::of(&fogs, map),
            fogged: false,
            sky: sky.as_ref().map_or([0.0; 3], |sky| sky.colour),
            view: MapView::of(&views, map),
            weather: weather(&weathers, map).into_iter().chain(sky.iter().flat_map(skies)).collect(),
            light: [1.0; 3],
            stepped: None,
        };
        (surroundings, spots)
    }
}

/// The weather the client starts by itself on a map: snow on the two maps it has it sewn in for
/// (`CGameMode`, `xmas.rsw` and `xmas_fild01.rsw`), and what its table gives the map (`etcinfo.txt`, lines of
/// `weather#map.rsw#kind#`: `LoadEtcInfo`). Of the table's kinds snow and sakura are made; its fireworks
/// (`pokjuk`) are not.
fn weather(table: &str, map: &str) -> Vec<u32> {
    let name = format!("{}.rsw", map.to_ascii_lowercase());
    let mut effects = Vec::new();
    if name == "xmas.rsw" || name == "xmas_fild01.rsw" {
        effects.push(EF_SNOW);
    }
    for line in table.lines().filter(|line| !line.trim_start().starts_with("//")) {
        let mut fields = line.split('#').map(|field| field.trim().to_ascii_lowercase());
        if let (Some("weather"), Some(of), Some(kind)) = (fields.next().as_deref(), fields.next(), fields.next()) {
            let effect = match kind.as_str() {
                "snow" => EF_SNOW,
                "sakura" => EF_SAKURA,
                _ => continue,
            };
            if of == name && !effects.contains(&effect) {
                effects.push(effect);
            }
        }
    }
    effects
}

/// The effects of a map's sky (`CMapSkyMgr::Apply`, `CWeatherEffect::Play`): its stars (`EF_CLOUD9`), and its
/// clouds of the kinds that are made here: 1, 2, 4 and 5 (`EF_CLOUD2`, `EF_CLOUD3`, `EF_CLOUD5`, `EF_CLOUD6`).
fn skies(sky: &data::Sky) -> Vec<u32> {
    let clouds = sky.clouds.iter().filter_map(|kind| match kind {
        1 => Some(230),
        2 => Some(233),
        4 => Some(516),
        5 => Some(592),
        _ => None,
    });
    sky.stars.then_some(945).into_iter().chain(clouds).collect()
}

impl Scene {
    /// Where someone, or a place effects play on, stands in the world now (none named: the player's own
    /// character): on the ground, but for a place a map put its effect at, which is as high as the map says.
    pub(super) fn stands(&self, who: Option<u32>) -> Option<Vector> {
        let mut place = on_ground(&self.gat, self.place_of(who)?);
        if let Some(height) = who.and_then(|id| self.scenery.raised.get(&id)) {
            place[1] = *height;
        }
        Some(place)
    }
}

/// Whether the fog of maps is shown (the client's `/fog`, `CGameMode::ApplyMapFog`).
pub fn fog(shown: bool) {
    STATE.with_borrow_mut(|state| state.scenery.fog = shown);
}

/// A state of someone's goes on or off (none named: the player's own character): the night begins or ends
/// with the player's own `EFST_SKE` (`crate::scenery::dark`).
pub(super) fn state_changed(on: Option<u32>, state: u32, holds: bool) {
    if on.is_none() && state == crate::scenery::dark::EFST_SKE {
        STATE.with_borrow_mut(|state| state.scenery.dark.on = holds);
    }
}
