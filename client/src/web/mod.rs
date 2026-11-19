//! Everything that needs the browser: the windows, the game world on the canvas, and the WebSocket to the gateway.

mod app;
mod data;
mod figure;
mod hud;
mod parts;
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

/// A client path as it is written in a URL: what `encodeURI` leaves alone because it means something in an
/// address is written out too, or a name with `#` in it (`blast_mine##clock.bmp`) would be cut there.
fn in_url(path: &str) -> String {
    String::from(js_sys::encode_uri(path)).replace('#', "%23").replace('?', "%3F")
}

/// The gateway's URL for a game file, by client path.
fn asset_url(path: &str) -> String {
    format!("/assets/{}", in_url(path))
}

/// The gateway's URL for a picture of the game as a PNG, by client path.
fn image_url(path: &str) -> String {
    format!("/image/{}", in_url(path))
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

/// A text of the client's files, which are written in Korean (EUC-KR, as the browser decodes it: its "euc-kr"
/// is Windows' code page 949): the names of textures and sprites in a map, a model or an effect, the lines of
/// a table. Where the browser cannot decode it, the bytes are read as UTF-8 and what is none is lost.
pub(crate) fn korean(bytes: &[u8]) -> String {
    thread_local! {
        static KOREAN: Option<web_sys::TextDecoder> = web_sys::TextDecoder::new_with_label("euc-kr").ok();
    }
    KOREAN.with(|korean| korean.as_ref().and_then(|korean| korean.decode_with_u8_array(bytes).ok())).unwrap_or_else(|| String::from_utf8_lossy(bytes).into_owned())
}

/// The time everything under `web/` that is drawn, heard or told to the servers goes by: the page's
/// milliseconds since it was opened.
pub(crate) fn now() -> f64 {
    web_sys::window().and_then(|window| window.performance()).map_or(0.0, |performance| performance.now())
}

/// The page.
pub(crate) fn document() -> Option<web_sys::Document> {
    web_sys::window().and_then(|window| window.document())
}

/// The thing on the page with that ID.
pub(crate) fn by_id(id: &str) -> Option<web_sys::Element> {
    document()?.get_element_by_id(id)
}

/// How wide and tall the page is, in its pixels (1024 by 768 where the browser will not say).
pub(crate) fn page_size() -> (f64, f64) {
    let size = |value: Result<JsValue, JsValue>, otherwise: f64| value.ok().and_then(|value| value.as_f64()).unwrap_or(otherwise);
    web_sys::window().map_or((1024.0, 768.0), |window| (size(window.inner_width(), 1024.0), size(window.inner_height(), 768.0)))
}

/// How much the stage is enlarged on a page larger than the client's 1024 by 768: by the lesser of the two
/// sides, never made smaller.
pub(crate) fn stage_scale() -> f64 {
    let (wide, tall) = page_size();
    (wide / 1024.0).min(tall / 768.0).max(1.0)
}

fn js_error(value: impl Into<JsValue>) -> String {
    format!("{:?}", value.into())
}
