//! A connection to a game server, through the gateway's WebSocket bridge.

use wasm_bindgen::prelude::*;
use web_sys::{BinaryType, CloseEvent, MessageEvent, WebSocket};

use super::js_error;

pub enum SocketEvent {
    Open,
    Bytes(Vec<u8>),
    /// With the close code and reason: the gateway uses 4029 to turn a login away.
    Closed(u16, String),
}

/// Open while it lives: dropping it closes the connection and silences its events.
pub struct Socket {
    ws: WebSocket,
    _on_open: Closure<dyn FnMut()>,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_close: Closure<dyn FnMut(CloseEvent)>,
}

impl Socket {
    /// Connects to the game server's `port`. `on_event` may be called at any later time, never from inside this
    /// call; it must not drop the socket while it runs (hand the event on to a later task instead).
    pub fn open(port: u16, on_event: impl Fn(SocketEvent) + Clone + 'static) -> Result<Self, String> {
        let location = web_sys::window().ok_or("no window")?.location();
        let scheme = if location.protocol().map_err(js_error)? == "https:" { "wss" } else { "ws" };
        let ws = WebSocket::new(&format!("{scheme}://{}/ws/{port}", location.host().map_err(js_error)?)).map_err(js_error)?;
        ws.set_binary_type(BinaryType::Arraybuffer);

        let on_open = Closure::<dyn FnMut()>::new({
            let on_event = on_event.clone();
            move || on_event(SocketEvent::Open)
        });
        let on_message = Closure::<dyn FnMut(MessageEvent)>::new({
            let on_event = on_event.clone();
            move |message: MessageEvent| {
                if let Ok(buffer) = message.data().dyn_into::<js_sys::ArrayBuffer>() {
                    on_event(SocketEvent::Bytes(js_sys::Uint8Array::new(&buffer).to_vec()));
                }
            }
        });
        let on_close = Closure::<dyn FnMut(CloseEvent)>::new(move |event: CloseEvent| on_event(SocketEvent::Closed(event.code(), event.reason())));
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        Ok(Self { ws, _on_open: on_open, _on_message: on_message, _on_close: on_close })
    }

    pub fn send(&self, bytes: &[u8]) {
        if let Err(error) = self.ws.send_with_u8_array(bytes) {
            web_sys::console::warn_1(&format!("send: {}", js_error(error)).into());
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        self.ws.set_onopen(None);
        self.ws.set_onmessage(None);
        self.ws.set_onclose(None);
        let _ = self.ws.close();
    }
}
