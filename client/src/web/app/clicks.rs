//! What each thing on the page does when pressed (its `data-click`).

use super::*;

impl App {
    pub(super) fn click(&mut self, what: &str, arg: &str) {
        match what {
            "lang" => {
                if let Some(lang) = Lang::from_code(arg) {
                    self.lang = lang;
                    crate::i18n::speak(lang);
                    store("roweb.lang", Some(lang.code()));
                    // The items and skills are asked for again, in the new language.
                    data::forget_texts();
                    let known: Vec<u32> = self.inventory.iter().map(|item| item.id).chain(self.hotkeys.iter().filter(|hotkey| !hotkey.skill).map(|hotkey| hotkey.id)).chain(self.item_info).collect();
                    data::want_items(known.into_iter(), || dispatch(Action::ItemsKnown));
                    if let Some(skill) = self.skill_info {
                        data::want_skill(skill, || dispatch(Action::ItemsKnown));
                    }
                    self.render();
                }
            }
            "ok" => {
                self.message = None;
                self.render();
            }
            "inventory" => {
                self.inventory_open = !self.inventory_open;
                self.render();
            }
            "skills" => {
                self.skills_open = !self.skills_open;
                self.render();
            }
            "inventory-tab" => {
                self.inventory_tab = [Tab::Usable, Tab::Equipment, Tab::Other][arg.parse::<usize>().unwrap_or(0).min(2)];
                self.refresh_items();
            }
            "raise-skill" => {
                if let (Ok(id), Some(socket)) = (arg.parse(), &self.socket) {
                    socket.send(&items::raise_skill(id));
                }
            }
            "menu" => {
                self.menu_closed = !self.menu_closed;
                self.render();
            }
            "dialog-next" | "dialog-close" | "dialog-ok" | "dialog-cancel" | "dialog-pick" => {
                let (Some(dialog), Some(socket)) = (&mut self.dialog, &self.socket) else { return };
                let npc = dialog.npc;
                match what {
                    "dialog-next" => {
                        dialog.button = None;
                        dialog.stale = true;
                        socket.send(&npc::next(npc));
                    }
                    "dialog-close" => {
                        socket.send(&npc::close(npc));
                        self.dialog = None;
                    }
                    "dialog-pick" => dialog.picked = arg.parse().unwrap_or(0),
                    "dialog-ok" if !dialog.menu.is_empty() => {
                        // The choices are counted from 1.
                        socket.send(&npc::choose(npc, dialog.picked as u8 + 1));
                        dialog.menu.clear();
                        dialog.stale = true;
                    }
                    "dialog-ok" => {
                        let typed = field("dialog-input");
                        socket.send(&if dialog.asks == Some(true) { npc::words(npc, &typed) } else { npc::number(npc, typed.trim().parse().unwrap_or(0)) });
                        dialog.asks = None;
                        dialog.stale = true;
                    }
                    _ => {
                        // Turning a menu down ends the talk.
                        socket.send(&npc::choose(npc, 255));
                        self.dialog = None;
                    }
                }
                self.refresh_dialog();
            }
            "shop-pick" => {
                self.shop_picked = arg.parse().ok();
                self.refresh_trade();
            }
            "shop-buy" | "shop-sell" | "shop-cancel" | "shop-ok" => {
                let (Some(shop), Some(socket)) = (self.shop.take(), &self.socket) else { return };
                let basket = std::mem::take(&mut self.basket);
                self.shop_picked = None;
                match (what, shop) {
                    ("shop-buy", Shop::Choose(npc)) => socket.send(&trade::deal(npc, false)),
                    ("shop-sell", Shop::Choose(npc)) => socket.send(&trade::deal(npc, true)),
                    (_, Shop::Choose(_)) => {}
                    ("shop-ok", Shop::Buy(offers)) => {
                        let wanted: Vec<(u32, u16)> = basket.iter().filter_map(|(row, amount)| Some((offers.get(*row)?.id, *amount))).collect();
                        socket.send(&trade::buy(&wanted));
                    }
                    ("shop-ok", Shop::Sell(bids)) => {
                        let sold: Vec<(u16, u16)> = basket.iter().filter_map(|(row, amount)| Some((bids.get(*row)?.0, *amount))).collect();
                        socket.send(&trade::sell(&sold));
                    }
                    // Leaving a list tells the server so with an empty one.
                    (_, Shop::Buy(_)) => socket.send(&trade::buy(&[])),
                    (_, Shop::Sell(_)) => socket.send(&trade::sell(&[])),
                }
                self.refresh_trade();
            }
            "throw-cancel" => {
                self.throwing = None;
                self.refresh_throw();
            }
            "equipment" => {
                self.equipment_open = !self.equipment_open;
                self.render();
            }
            "chat-tab" => {
                self.chat_tab = arg.parse().unwrap_or(0);
                self.render();
            }
            "chat-lock" => {
                self.chat_locked = !self.chat_locked;
                self.render();
            }
            "chat-size" => {
                self.chat_resize();
                self.render();
            }
            // Hides the log, leaving the input line, or brings it back as tall as it was.
            "chat-mini" => {
                if self.chat_rows > 0 {
                    self.chat_rows_kept = self.chat_rows;
                    self.chat_rows = 0;
                } else {
                    self.chat_rows = self.chat_rows_kept.max(1);
                }
                self.render();
            }
            // A press on the slot machine starts its reels and asks the server for the catch.
            "taming-shoot" => {
                if let (Some(taming), Some(socket)) = (&mut self.taming, &self.socket) {
                    if !taming.started {
                        (taming.state, taming.wanted, taming.started, taming.tick) = (1, 7, true, 0);
                        socket.send(&[&0x019fu16.to_le_bytes()[..], &taming.target.to_le_bytes()].concat());
                    }
                }
            }
            "egg-pick" => {
                self.egg_picked = arg.parse().unwrap_or(0);
                self.render();
            }
            // CZ_SELECT_PETEGG (0x01a7): `u16 index`. The picked egg is hatched.
            "egg-ok" | "egg-cancel" => {
                if let (true, Some(socket), Some(index)) = (what == "egg-ok", &self.socket, self.eggs.as_ref().and_then(|eggs| eggs.get(self.egg_picked))) {
                    socket.send(&[0x01a7u16.to_le_bytes(), index.to_le_bytes()].concat());
                }
                self.eggs = None;
                self.render();
            }
            "skill-info-close" => {
                self.skill_info = None;
                self.refresh_items();
            }
            "item-info-close" => {
                self.item_info = None;
                self.refresh_items();
            }
            "esc-close" | "esc-save" | "esc-no" | "esc-yes" => {
                self.esc = match what {
                    "esc-save" => 2,
                    "esc-no" => 1,
                    _ => 0,
                };
                if what == "esc-yes" {
                    if let Some(socket) = &self.socket {
                        socket.send(&npc::restart(false));
                    }
                }
                self.refresh_dialog();
            }
            "hotkey-rows" => {
                self.hotkey_rows = if arg == "+" { (self.hotkey_rows + 1).min(3) } else { self.hotkey_rows.saturating_sub(1) };
                self.render();
            }
            "skill-tab" => {
                self.skill_tab = arg.parse().unwrap_or(0);
                self.render();
            }
            "storage-tab" => {
                self.storage_tab = arg.parse().unwrap_or(0);
                self.refresh_items();
            }
            "storage-close" => {
                if let Some(socket) = &self.socket {
                    socket.send(&trade::close_storage());
                }
            }
            "basic-size" => {
                self.basic_small = !self.basic_small;
                self.render();
            }
            "status" => {
                self.status_open = !self.status_open;
                self.render();
            }
            "raise" => {
                if let (Ok(stat), Some(socket)) = (arg.parse::<u16>(), &self.socket) {
                    socket.send(&packets::raise_stat(packets::parameter::STR + stat.min(5), 1));
                }
            }
            "sound" => {
                self.sound = !self.sound;
                store("roweb.sound", Some(if self.sound { "on" } else { "off" }));
                self.tune();
                self.music_held = false;
                self.render();
                check_music_soon();
            }
            "yes" => self.answer(true),
            "no" => self.answer(false),
            "delete" | "delete-now" => {
                if let Some(character) = self.character_in(self.selected) {
                    self.ask = Some(if what == "delete" { Ask::Reserve(character.char_id) } else { Ask::Delete(character.char_id) });
                    self.render();
                }
            }
            "cancel-delete" => {
                if let (Some(character), Some(socket)) = (self.character_in(self.selected), &self.socket) {
                    socket.send(&packets::delete_cancel(character.char_id));
                    self.wait = Some(Key::PleaseWait);
                    self.render();
                }
            }
            "race" => {
                self.make.doram = arg == "doram";
                self.make.hair_style = self.make.hair_style.min(hair_styles(self.make.doram));
                self.render();
            }
            "cancel-wait" => {
                // Whatever was being waited for is given up; a form stays as it was filled in.
                match self.screen {
                    Screen::Register | Screen::Recover | Screen::Reset(_) => {
                        self.wait = None;
                        self.render();
                    }
                    _ => self.leave(None),
                }
            }
            "save-id" => {
                self.save_id = !self.save_id;
                self.render();
            }
            "to-register" => self.show(Screen::Register),
            "to-recover" => self.show(Screen::Recover),
            "to-login" => self.show(Screen::Login),
            "slot" => {
                let Ok(slot) = arg.parse::<u8>() else { return };
                // A press on the one already chosen changes nothing: the page stays as it is, and a second press
                // makes a double click of it.
                if self.selected == slot && self.character_in(slot).is_some() {
                    return;
                }
                self.selected = slot;
                // An empty slot opens the making of a character at once, as in the original client.
                if self.character_in(slot).is_none() {
                    return self.open(slot);
                }
                self.render();
            }
            "start" => {
                self.chime();
                self.open(self.selected)
            }
            "sex" => {
                self.make.sex = u8::from(arg == "1");
                self.render();
            }
            "hair" => {
                if let Some(style) = arg.parse().ok().filter(|style| (1..=hair_styles(self.make.doram)).contains(style)) {
                    self.make.hair_style = style;
                    self.render();
                }
            }
            "colour" => {
                if let Some(colour) = arg.parse().ok().filter(|colour| *colour < HAIR_COLOURS) {
                    self.make.hair_colour = colour;
                    self.render();
                }
            }
            "turn" => {
                let step = if arg == "-1" { 7 } else { 1 };
                self.make.direction = (self.make.direction + step) % 8;
                self.render();
            }
            "back" => self.show(Screen::CharSelect),
            "server" => {
                let picked = arg.parse().unwrap_or(0);
                // A press on the one already picked leaves the page as it is, so a second makes a double click.
                if picked == self.server_picked {
                    return;
                }
                self.server_picked = picked;
                self.render();
            }
            "server-ok" => {
                self.chime();
                self.enter_server()
            }
            "server-cancel" | "exit" => self.leave(None),
            // In the game, leaving first goes back to the characters: another one may be wanted. The server
            // says when the character is out of the game; from the characters, leaving goes out altogether.
            "logout" if matches!(self.screen, Screen::Game) => {
                self.esc = 0;
                self.refresh_dialog();
                if let Some(socket) = &self.socket {
                    socket.send(&npc::restart(true));
                }
            }
            "logout" => self.leave(None),
            _ => {}
        }
    }
}
