//! What a map's script sets on its particle emitters (the gateway's `tables/emitters.rs`), asked for as the
//! map is read; `crate::emitters` has what an emitter is.

use super::*;
use crate::emitters::Emitter;

/// The emitters of a map as its script sets them, by their number among those the map places: each as the
/// client makes an emitter, with the fields the gateway gives set (`Emitter::set`): an empty list for a map
/// without a script. Nothing when the gateway does not answer (one older than this table): the map's emitters
/// are then not shown at all, rather than all as the client makes them before its script sets them.
pub async fn emitters(map: &str) -> Option<Vec<Emitter>> {
    let told = fetch_json(&format!("/tables/emitters/{}", map.to_ascii_lowercase())).await.ok()?;
    let set = list(&told)
        .iter()
        .map(|set| {
            let mut emitter = Emitter::default();
            for (field, value) in entries::<String>(set) {
                // A field is numbers, one number, yes or no, or (the picture) a text.
                let numbers: Vec<f64> = match value.as_bool() {
                    Some(flag) => vec![f64::from(u8::from(flag))],
                    None => value.as_f64().map_or_else(|| list(&value).iter().filter_map(wasm_bindgen::JsValue::as_f64).collect(), |number| vec![number]),
                };
                emitter.set(&field, &numbers, value.as_string().as_deref());
            }
            emitter
        })
        .collect();
    Some(set)
}
