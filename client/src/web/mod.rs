//! Everything that needs the browser: the windows, the game world on the canvas, and the WebSocket to the gateway.

mod app;
mod data;
mod figure;
mod hud;
mod socket;
mod sound;
mod view;
mod world;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    app::start().map_err(|error| JsValue::from_str(&error))
}

/// The gateway's URL for a game file, by client path.
fn asset_url(path: &str) -> String {
    format!("/assets/{}", String::from(js_sys::encode_uri(path)))
}

/// The gateway's URL for a picture of the game as a PNG, by client path.
fn image_url(path: &str) -> String {
    format!("/image/{}", String::from(js_sys::encode_uri(path)))
}

/// Fetches a game file from the gateway by client path.
async fn fetch_asset(path: &str) -> Result<Vec<u8>, String> {
    let window = web_sys::window().ok_or("no window")?;
    let response: web_sys::Response = JsFuture::from(window.fetch_with_str(&asset_url(path))).await.map_err(js_error)?.dyn_into().map_err(js_error)?;
    if !response.ok() {
        return Err(format!("{path}: HTTP {}", response.status()));
    }
    let buffer = JsFuture::from(response.array_buffer().map_err(js_error)?).await.map_err(js_error)?;
    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}

/// The page.
pub(crate) fn document() -> Option<web_sys::Document> {
    web_sys::window().and_then(|window| window.document())
}

/// The thing on the page with that ID.
pub(crate) fn by_id(id: &str) -> Option<web_sys::Element> {
    document()?.get_element_by_id(id)
}

fn js_error(value: impl Into<JsValue>) -> String {
    format!("{:?}", value.into())
}
