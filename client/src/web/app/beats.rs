//! The two beats the connection lives by, the timer that keeps it alive and the watch on it
//! (`actions/link.rs`), kept going while the page is not shown.
//!
//! A browser holds back the timers of a page that is not shown: to one beat a second, and after some minutes
//! to one a minute. The server ends a connection it has heard nothing from for a minute (rAthena's
//! `stall_time`), so a tab left in the background lost the game after some minutes (seen in the game,
//! 2026-11-27: hidden at 19:08:03, logged off by the server at 19:12:15, with every other hidden tab of the
//! same browser in the same second). The timers of a worker are not held back so: the beats come from one
//! that does nothing but count time, and the page does at each what its own timer did. This is the browser's
//! need and no part of the Windows client, whose timers nothing holds back. Where no worker can be made, or
//! it fails, the page's own timers beat as they did before.

use super::*;

/// What the worker tells at a beat: which of the two it is.
const KEEP_ALIVE: f64 = 0.0;
const WATCH: f64 = 1.0;

/// The beats begin, for as long as the page lives.
pub(super) fn wire(window: &web_sys::Window) -> Result<(), String> {
    match from_worker() {
        Some(()) => Ok(()),
        None => from_page(window),
    }
}

/// The page's own timers: the beats as they were before there was a worker.
fn from_page(window: &web_sys::Window) -> Result<(), String> {
    let on_timer = Closure::<dyn FnMut()>::new(|| dispatch(Action::KeepAlive));
    let on_watch = Closure::<dyn FnMut()>::new(|| dispatch(Action::Watch));
    window.set_interval_with_callback_and_timeout_and_arguments_0(on_timer.as_ref().unchecked_ref(), KEEP_ALIVE_MS).map_err(js_error)?;
    window.set_interval_with_callback_and_timeout_and_arguments_0(on_watch.as_ref().unchecked_ref(), crate::link::WATCH_EVERY).map_err(js_error)?;
    // These live as long as the page.
    on_timer.forget();
    on_watch.forget();
    Ok(())
}

/// A worker that counts the two beats, written here as the few words of script it is; `None` when the browser
/// makes none. One that fails after it was made (a page not let to run workers) hands the beats back to the
/// page's own timers.
fn from_worker() -> Option<()> {
    let script = [
        format!("setInterval(()=>postMessage({KEEP_ALIVE}),{KEEP_ALIVE_MS});"),
        format!("setInterval(()=>postMessage({WATCH}),{});", crate::link::WATCH_EVERY)
    ].concat();
    let kind = web_sys::BlobPropertyBag::new();
    kind.set_type("text/javascript");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&js_sys::Array::of1(&script.into()), &kind).ok()?;
    let worker = web_sys::Worker::new(&web_sys::Url::create_object_url_with_blob(&blob).ok()?).ok()?;
    let on_beat = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(|event: web_sys::MessageEvent| {
        dispatch(if event.data().as_f64() == Some(KEEP_ALIVE) { Action::KeepAlive } else { Action::Watch });
    });
    let failed = worker.clone();
    let on_error = Closure::<dyn FnMut()>::new(move || {
        failed.terminate();
        if let Some(window) = web_sys::window() {
            let _ = from_page(&window);
        }
    });
    worker.set_onmessage(Some(on_beat.as_ref().unchecked_ref()));
    worker.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    // These live as long as the page, and the worker with them.
    on_beat.forget();
    on_error.forget();
    std::mem::forget(worker);
    Some(())
}
