//! The sounds of the windows, as the original client plays them: its title music, and a click for every button.
//!
//! A browser plays nothing until the player has pressed something on the page, so the music starts with the
//! first press, not with the page.

use std::cell::RefCell;

use web_sys::HtmlAudioElement;

use super::asset_url;

/// The music of the login and character windows (`CLoginMode` plays `bgm\01.mp3`).
const TITLE_MUSIC: &str = "/bgm/01.mp3";
/// The client's button sound.
const BUTTON: &str = "data/wav/버튼소리.wav";
const MUSIC_VOLUME: f64 = 0.5;

/// How many of one sound may sound at once: a crowd of the same monster is not that many times as loud.
const VOICES: usize = 3;

#[derive(Default)]
struct Sounds {
    music: Option<HtmlAudioElement>,
    button: Option<HtmlAudioElement>,
    /// Whether the game's sounds play at all (the player's switch, and the page being looked at).
    on: bool,
    /// The music of the map the character is on: its track, and what plays it.
    track: Option<String>,
    map: Option<(String, HtmlAudioElement)>,
    /// How loud the music and the effects are, each 0 to 1, as the player set them; full until set.
    levels: Option<(f64, f64)>,
    /// The sounds of the game by file name, a few players each.
    effects: std::collections::HashMap<String, Vec<HtmlAudioElement>>,
}

thread_local! {
    static SOUNDS: RefCell<Sounds> = RefCell::new(Sounds::default());
}

/// Plays the title music, or stops it. Asking again while it plays changes nothing.
pub fn music(play: bool) {
    SOUNDS.with_borrow_mut(|sounds| {
        if !play {
            if let Some(music) = &sounds.music {
                let _ = music.pause();
            }
            return;
        }
        if sounds.music.is_none() {
            if let Ok(music) = HtmlAudioElement::new_with_src(TITLE_MUSIC) {
                music.set_loop(true);
                music.set_volume(MUSIC_VOLUME * sounds.levels.map_or(1.0, |levels| levels.0));
                sounds.music = Some(music);
            }
        }
        if let Some(music) = sounds.music.as_ref().filter(|music| music.paused()) {
            // The browser refuses until the first press on the page; the next press asks again.
            let _ = music.play();
        }
    });
}

/// Whether the title music is sounding right now.
pub fn music_playing() -> bool {
    SOUNDS.with_borrow(|sounds| sounds.music.as_ref().is_some_and(|music| !music.paused()))
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
/// the music playing changes at once.
pub fn levels(music: f64, effects: f64) {
    SOUNDS.with_borrow_mut(|sounds| {
        sounds.levels = Some((music.clamp(0.0, 1.0), effects.clamp(0.0, 1.0)));
        for playing in sounds.music.iter().chain(sounds.map.iter().map(|(_, music)| music)) {
            playing.set_volume(MUSIC_VOLUME * music.clamp(0.0, 1.0));
        }
    });
}

/// Turns the sounds of the game itself on or off: the map's music and every effect.
pub fn game(on: bool) {
    SOUNDS.with_borrow_mut(|sounds| {
        sounds.on = on;
        tune_map(sounds);
    });
}

/// The music of the map the character is on (`bgm\\08.mp3` as the client's table names it), or none.
pub fn map_track(track: Option<String>) {
    SOUNDS.with_borrow_mut(|sounds| {
        sounds.track = track;
        tune_map(sounds);
    });
}

fn tune_map(sounds: &mut Sounds) {
    let wanted = sounds.track.clone().filter(|_| sounds.on);
    // Another map's music stops; the same one plays on.
    if let Some((playing, music)) = &sounds.map {
        if Some(playing) != wanted.as_ref() {
            let _ = music.pause();
            sounds.map = None;
        }
    }
    let Some(wanted) = wanted else { return };
    if sounds.map.is_none() {
        let url = format!("/{}", wanted.replace('\\', "/"));
        if let Ok(music) = HtmlAudioElement::new_with_src(&url) {
            music.set_loop(true);
            music.set_volume(MUSIC_VOLUME * sounds.levels.map_or(1.0, |levels| levels.0));
            sounds.map = Some((wanted, music));
        }
    }
    if let Some((_, music)) = sounds.map.as_ref().filter(|(_, music)| music.paused()) {
        let _ = music.play();
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
