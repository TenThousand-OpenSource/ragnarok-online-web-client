//! Taming a monster, and the menu on the player's own pet.

use super::*;

impl App {
    /// A monster was picked to tame: the slot machine opens for it (not for players, as in the client).
    pub(super) fn taming_begins(&mut self, target: u32) {
        self.skills.aimed = None;
        self.close_taming();
        // The fight, if there was one, stops for the taming.
        world::stand_down();
        if let Some(socket) = &self.socket {
            // CZ_CANCEL_LOCKON (0x0118): the server stops the blows it was dealing on its own.
            socket.send(&0x0118u16.to_le_bytes());
        }
        let tick = Closure::<dyn FnMut()>::new(|| dispatch(Action::TamingTick));
        let timer = web_sys::window().and_then(|window| window.set_interval_with_callback_and_timeout_and_arguments_0(tick.as_ref().unchecked_ref(), 16).ok()).unwrap_or(0);
        tick.forget();
        self.pets.taming = Some(Taming { target, state: 0, tick: 0, delay: 1, longest: 10000, wanted: -1, started: false, rested: None, frames: [Vec::new(), Vec::new()], shown: (usize::MAX, 0), timer });
        wasm_bindgen_futures::spawn_local(async {
            let mut read = [Vec::new(), Vec::new()];
            for (action, frames) in read.iter_mut().enumerate() {
                for frame in 0..64 {
                    if frame > 0 && figure::timing("slotmachine", action).is_some_and(|(count, _)| frame >= count) {
                        break;
                    }
                    match figure::frame_picture("slotmachine", action, frame).await {
                        Ok(picture) => frames.push(picture),
                        Err(_) => break,
                    }
                }
            }
            let [waiting, turning] = read;
            dispatch(Action::TamingRead(waiting, turning));
        });
        self.render();
    }

    /// The slot machine's moment: the client's `Paint` and `OnProcess` of the window.
    pub(super) fn taming_tick(&mut self) {
        let Some(taming) = &mut self.pets.taming else { return };
        let count = taming.frames[taming.state].len() as i32;
        if count == 0 {
            return;
        }
        let frame = taming.tick / taming.delay.max(1);
        let shown = (taming.state, frame.clamp(0, count - 1) as usize);
        if taming.wanted != frame || taming.delay > taming.longest {
            taming.tick += 1;
        }
        if frame >= count {
            taming.tick = 0;
            if taming.wanted != -1 {
                taming.delay += 1;
            }
        }
        let resting = taming.wanted == frame && taming.delay <= taming.longest;
        let over = match taming.rested {
            None => {
                if resting {
                    taming.rested = Some(js_sys::Date::now());
                }
                false
            }
            Some(since) => resting && js_sys::Date::now() - since >= 2000.0,
        };
        // (The page may have been written anew since, and the picture with it.)
        if shown != taming.shown || element("taming-picture").is_some_and(|picture| picture.get_attribute("src").is_none()) {
            taming.shown = shown;
            if let Some(picture) = element("taming-picture") {
                let (url, _, origin) = &taming.frames[shown.0][shown.1];
                let _ = picture.set_attribute("src", url);
                let _ = picture.set_attribute("style", &format!("position:absolute;left:{}px;top:{}px", 98.0 - origin.0, 128.0 - origin.1));
            }
        }
        if over {
            self.close_taming();
            self.render();
        }
    }

    /// The right button on someone. The client has a menu for the player's own pet (its status, feeding, a trick,
    /// the accessory off, back into the egg), for a homunculus, a mercenary and other players; for a monster or an
    /// NPC it has none. Here: the pet's.
    pub(super) fn unit_menu(&mut self, id: u32, at: (f32, f32)) {
        if id == self.pets.unit && self.pets.pet.is_some() {
            // The page's place, in the stage's own measure: the stage may be drawn larger or smaller.
            let stage = web_sys::window().and_then(|window| window.document()).and_then(|document| document.query_selector(".stage").ok().flatten());
            let scale = stage.as_ref().and_then(|stage| Some(stage.get_bounding_client_rect().width() / f64::from(stage.clone().dyn_into::<web_sys::HtmlElement>().ok()?.offset_width().max(1)))).filter(|scale| *scale > 0.0).unwrap_or(1.0);
            self.pets.menu_at = Some(((f64::from(at.0) / scale) as i32 + 1, (f64::from(at.1) / scale) as i32 + 1));
            self.render();
        }
    }
}
