//! The sounds of the game, as the original client plays them: its title music and the music of a map, which
//! fade out and in at a change of map or screen (`CSoundMgr::FadeIn`, `FadeOut`), a click for every button,
//! and the sounds of the world, each as loud as its distance leaves it (`sound::at`; the rules are
//! `crate::earshot`).
//!
//! A browser plays nothing until the player has pressed something on the page, so the music starts with the
//! first press, not with the page.

use std::cell::RefCell;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::HtmlAudioElement;

use super::{asset_url, now};

/// The music of the login and character windows (`CLoginMode` plays `bgm\01.mp3`).
const TITLE_MUSIC: &str = "/bgm/01.mp3";
/// The client's button sound.
const BUTTON: &str = "data/wav/버튼소리.wav";
const MUSIC_VOLUME: f64 = 0.5;

/// How many of one sound may sound at once: a crowd of the same monster is not that many times as loud.
const VOICES: usize = 3;

/// How often the music's loudness is stepped while it fades, in milliseconds.
const FADE_EVERY: i32 = 30;

/// A piece of music that plays, rises or falls away: its address, what plays it, and how much of its loudness
/// it has now (0 to 1).
struct Stream {
    address: String,
    audio: HtmlAudioElement,
    level: f64,
}

#[derive(Default)]
struct Sounds {
    /// The music: the piece wanted, and those still fading out.
    streams: Vec<Stream>,
    /// Whether the title music is wanted.
    title: bool,
    button: Option<HtmlAudioElement>,
    /// Whether the game's sounds play at all (the player's switch, and the page being looked at).
    on: bool,
    /// The music of the map the character is on: its track.
    track: Option<String>,
    /// A piece an NPC's script plays in the map's place (`piece`): its name, and whether it plays once only.
    piece: Option<(String, bool)>,
    /// The sounds an NPC's script has play over and over (`repeat`), by file name.
    repeats: std::collections::HashMap<String, HtmlAudioElement>,
    /// How loud the music and the effects are, each 0 to 1, as the player set them; full until set.
    levels: Option<(f64, f64)>,
    /// The sounds of the game by file name, a few players each.
    effects: std::collections::HashMap<String, Vec<HtmlAudioElement>>,
    /// When the music's loudness was last stepped, until when it falls for a change of map, and whether the
    /// timer that steps it runs.
    stepped: f64,
    dip_until: f64,
    timed: bool,
    /// When the browser was last asked to play the music.
    asked: f64,
}

thread_local! {
    static SOUNDS: RefCell<Sounds> = RefCell::new(Sounds::default());
}

impl Sounds {
    /// The address of the music wanted now: the title's, or the map's while the game's sounds are on.
    fn wanted(&self) -> Option<String> {
        if self.title {
            return Some(TITLE_MUSIC.to_owned());
        }
        if let Some((piece, _)) = self.piece.as_ref().filter(|_| self.on) {
            return Some(format!("/bgm/{piece}.mp3"));
        }
        self.track.as_ref().filter(|_| self.on).map(|track| format!("/{}", track.replace('\\', "/")))
    }

    /// Sees that the music wanted is among the streams (silent until it rises) and that the timer runs: every
    /// other stream fades out from now, and the wanted one rises once they are gone.
    fn tune(&mut self) {
        if let Some(wanted) = self.wanted().filter(|wanted| !self.streams.iter().any(|stream| &stream.address == wanted)) {
            if let Ok(audio) = HtmlAudioElement::new_with_src(&wanted) {
                // A script's piece that plays once is not begun again (`piece`).
                audio.set_loop(!self.piece.as_ref().is_some_and(|(piece, once)| *once && wanted == format!("/bgm/{piece}.mp3")));
                audio.set_volume(0.0);
                self.streams.push(Stream { address: wanted, audio, level: 0.0 });
            }
        }
        if !self.timed {
            self.timed = true;
            self.stepped = now();
            let step = Closure::<dyn FnMut()>::new(|| SOUNDS.with_borrow_mut(Sounds::step));
            if let Some(window) = web_sys::window() {
                let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(step.as_ref().unchecked_ref(), FADE_EVERY);
            }
            // It steps the music for as long as the page lives.
            step.forget();
        }
    }

    /// One step of the fades (`CSoundMgr::FadeThread`): what is not wanted falls and is put away once silent;
    /// then what is wanted rises, unless the music is falling for a change of map.
    fn step(&mut self) {
        let time = now();
        let change = crate::earshot::fade(time - std::mem::replace(&mut self.stepped, time));
        let (wanted, loud) = (self.wanted(), MUSIC_VOLUME * self.levels.map_or(1.0, |levels| levels.0));
        let dipping = time < self.dip_until;
        let leaving = self.streams.iter().any(|stream| Some(&stream.address) != wanted.as_ref());
        let mut asked = self.asked;
        self.streams.retain_mut(|stream| {
            let stays = Some(&stream.address) == wanted.as_ref();
            if !stays || dipping {
                stream.level = (stream.level - change).max(0.0);
            } else if !leaving {
                // The browser refuses until the first press on the page; it is asked again twice a second.
                if stream.audio.paused() && !stream.audio.ended() && time - asked > 500.0 {
                    asked = time;
                    let _ = stream.audio.play();
                }
                stream.level = (stream.level + change).min(1.0);
            }
            stream.audio.set_volume((loud * stream.level).clamp(0.0, 1.0));
            if !stays && stream.level <= 0.0 {
                let _ = stream.audio.pause();
            }
            stays || stream.level > 0.0
        });
        self.asked = asked;
    }
}

/// Plays the title music, or lets it fade away. Asking again while it plays changes nothing.
pub fn music(play: bool) {
    SOUNDS.with_borrow_mut(|sounds| {
        sounds.title = play;
        sounds.tune();
    });
}

/// Whether the title music is sounding right now.
pub fn music_playing() -> bool {
    SOUNDS.with_borrow(|sounds| sounds.streams.iter().any(|stream| stream.address == TITLE_MUSIC && !stream.audio.paused()))
}

/// The click of a button.
pub fn button() {
    SOUNDS.with_borrow_mut(|sounds| {
        if sounds.button.is_none() {
            sounds.button = HtmlAudioElement::new_with_src(&asset_url(BUTTON)).ok();
        }
        if let Some(button) = &sounds.button {
            button.set_current_time(0.0);
            let _ = button.play();
        }
    });
}

/// How loud the music and the effects are from now on, each 0 to 1 (the Game Settings window's two sliders):
/// the music playing changes at its next step.
pub fn levels(music: f64, effects: f64) {
    SOUNDS.with_borrow_mut(|sounds| sounds.levels = Some((music.clamp(0.0, 1.0), effects.clamp(0.0, 1.0))));
}

/// Turns the sounds of the game itself on or off: the map's music and every effect.
pub fn game(on: bool) {
    SOUNDS.with_borrow_mut(|sounds| {
        sounds.on = on;
        sounds.tune();
    });
}

/// The music of the map the character is on (`bgm\\08.mp3` as the client's table names it), or none: told as
/// the character arrives on a map. Another map's music fades out and this one's in; the same music falls for
/// a moment and rises again, as the client's does while its screen goes black between two maps.
pub fn map_track(track: Option<String>) {
    SOUNDS.with_borrow_mut(|sounds| {
        if sounds.track == track && track.is_some() {
            sounds.dip_until = now() + crate::earshot::DIP_MS;
        }
        sounds.track = track;
        sounds.tune();
    });
}

/// A piece of music an NPC's script plays in the place of the map's (`playBGM`; the client's
/// `CGameMode::PlayBgm`, a file of `bgm\` named without its `.mp3`): over and over, or once, after which
/// nothing plays, as in the client, whose own music comes back with the next map. `None` goes back to the
/// map's music. The name is the server's: only what a file's name is made of is kept of it.
pub fn piece(piece: Option<(String, bool)>) {
    SOUNDS.with_borrow_mut(|sounds| {
        // A piece that played once and is over is put away, so that it can be played again.
        sounds.streams.retain(|stream| !stream.audio.ended());
        sounds.piece = piece
            .map(
                |(name, once)| (name.chars().filter(|letter| letter.is_ascii_alphanumeric() || matches!(letter, '_' | '-')).collect(), once)
            )
            .filter(|(name, _): &(String, bool)| !name.is_empty());
        sounds.tune();
    });
}

/// A sound an NPC's script has play over and over without a wait between (`soundeffect` of kind 1 with no
/// period) begins, or the one of that name ends. The client plays such a sound anew at every frame; here it
/// begins again as it ends, which is this client's own choice.
pub fn repeat(name: &str, play: bool) {
    SOUNDS.with_borrow_mut(|sounds| {
        if let Some(playing) = sounds.repeats.remove(name) {
            let _ = playing.pause();
        }
        if !play || !sounds.on {
            return;
        }
        if let Ok(playing) = HtmlAudioElement::new_with_src(&asset_url(&format!("data/wav/{}", name.replace('\\', "/")))) {
            playing.set_loop(true);
            playing.set_volume(sounds.levels.map_or(1.0, |levels| levels.1).clamp(0.0, 1.0));
            let _ = playing.play();
            sounds.repeats.insert(name.to_owned(), playing);
        }
    });
}

/// Every sound that plays over and over ends: the character has left the map.
pub fn repeats_end() {
    SOUNDS.with_borrow_mut(|sounds| {
        for (_, playing) in sounds.repeats.drain() {
            let _ = playing.pause();
        }
    });
}

/// Plays one of the world's sounds that starts so far east and north of the player's character, in world
/// units (a cell is 5): as loud as that distance leaves it, and not at all out of earshot (the client's
/// `PlayWave(name, dx, dy, dz, 250, 40, …)`). Every sound that has a place in the world goes through this.
pub fn at(name: &str, east: f32, north: f32) {
    if let Some(loudness) = crate::earshot::loudness(east, north) {
        effect(name, loudness);
    }
}

/// Plays one of the game's sounds (a file of `data/wav`), as loud as `volume` (0 to 1).
pub fn effect(name: &str, volume: f64) {
    if volume <= 0.02 || name.is_empty() {
        return;
    }
    SOUNDS.with_borrow_mut(|sounds| {
        if !sounds.on {
            return;
        }
        let players = sounds.effects.entry(name.to_owned()).or_default();
        // One that has finished plays again; else another is made, up to a few.
        let free = players.iter().position(|player| player.paused() || player.ended());
        let player = match free {
            Some(free) => &players[free],
            None if players.len() < VOICES => {
                let Ok(player) = HtmlAudioElement::new_with_src(&asset_url(&format!("data/wav/{}", name.replace('\\', "/")))) else { return };
                players.push(player);
                players.last().unwrap()
            }
            None => return,
        };
        player.set_volume((volume * sounds.levels.map_or(1.0, |levels| levels.1)).clamp(0.0, 1.0));
        player.set_current_time(0.0);
        let _ = player.play();
    });
}
