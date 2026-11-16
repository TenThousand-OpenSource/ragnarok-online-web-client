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

#[derive(Default)]
struct Sounds {
    music: Option<HtmlAudioElement>,
    button: Option<HtmlAudioElement>,
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
                music.set_volume(MUSIC_VOLUME);
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
