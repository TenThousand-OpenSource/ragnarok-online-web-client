//! What surrounds a map: the sky the client draws behind it (its `MapSkyData.lub`).

use super::*;

/// The sky of a map: the colour behind the map, whether stars are drawn, and the kinds of its clouds (what
/// the client's `CWeatherEffect::Play` is called with).
#[derive(Clone, Default)]
pub struct Sky {
    pub colour: [f32; 3],
    pub stars: bool,
    pub clouds: Vec<i32>,
}

thread_local! {
    /// The sky of each map that has one, by the map's name.
    static SKIES: RefCell<HashMap<String, Sky>> = RefCell::new(HashMap::new());
}

/// The sky of a map; none for a map the client's table does not have, which stands on black.
pub fn sky(map: &str) -> Option<Sky> {
    SKIES.with_borrow(|skies| skies.get(map.to_ascii_lowercase().trim_end_matches(".gat").trim_end_matches(".rsw")).cloned())
}

/// Reads the skies out of the gateway's tables: `{"<map>": {"colour": [r, g, b], "stars": true, "clouds": [1]}}`.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let numbers = |listed: wasm_bindgen::JsValue| list(&listed).iter().filter_map(|number| number.as_f64()).collect::<Vec<f64>>();
    let skies = entries::<String>(&field(tables, "sky"))
        .into_iter()
        .map(|(map, about)| {
            let colour = numbers(field(&about, "colour"));
            let colour = std::array::from_fn(|part| colour.get(part).map_or(0.0, |part| (*part / 255.0) as f32));
            (map, Sky { colour, stars: flag(&about, "stars"), clouds: numbers(field(&about, "clouds")).into_iter().map(|kind| kind as i32).collect() })
        })
        .collect();
    SKIES.set(skies);
}
