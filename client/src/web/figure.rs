//! Characters as the windows show them: body, head and headgear sprites put together and painted on a small
//! canvas (how, and in which order: `figure/compose.rs`).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

use super::{data, fetch_asset};
use crate::sprite::{self, Act, Canvas, Frame, Sprite};

mod compose;
mod weapon;

use compose::compose;

/// The size of a figure's canvas, and where in it the feet stand: as in a character slot of the select window.
pub const WIDTH: usize = 157;
pub const HEIGHT: usize = 195;
const FEET: (i32, i32) = (80, 137);
/// What putting a figure together answers when something it needs is still on its way.
const LATER: &str = "later";

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Figure {
    pub job: u16,
    /// 1 for male, 0 for female, as the servers send it.
    pub sex: u8,
    pub hair_style: u16,
    pub hair_colour: u16,
    /// The headgear worn, as view IDs (0 for none): bottom, top, middle.
    pub headgear: [u16; 3],
    /// Which recolouring of the job's clothes; 0 for the sprite's own colours.
    pub body_palette: u16,
    /// The weapon in hand, as its item ID; 0 for none.
    pub weapon: u32,
    /// The shield on the arm, as its item ID; 0 for none.
    pub shield: u32,
    /// Which of the sprite's actions: standing is 0-7 by direction (south first, then clockwise), walking 8-15,
    /// sitting 16-23.
    pub action: usize,
    /// Which picture of the action; a number past its last starts over.
    pub frame: usize,
}

/// Which picture: a character put together from its parts, or one frame of a monster's or an NPC's sprite (its
/// path under `data/sprite/`, the action, the frame); or the weapon alone of a character's frame, in a picture
/// of the same size with the feet at the same place (`figure/weapon.rs`).
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SpriteKey {
    Player(Figure),
    Creature(String, usize, usize),
    Weapon(Figure),
}

/// A picture ready to draw: its pixels, and where in it the feet stand.
#[derive(Clone)]
pub struct Picture {
    pub canvas: Rc<Canvas>,
    pub feet: (f32, f32),
}

/// The picture for a key, if it is ready. If not, making it is started, and a later call has it. A sprite that
/// cannot be read never has one.
pub fn picture(key: &SpriteKey) -> Option<Picture> {
    match key {
        SpriteKey::Player(figure) => painted(figure).map(|canvas| Picture { canvas, feet: (FEET.0 as f32, FEET.1 as f32) }),
        SpriteKey::Weapon(figure) => weapon::alone(figure).map(|canvas| Picture { canvas, feet: (FEET.0 as f32, FEET.1 as f32) }),
        SpriteKey::Creature(..) => {
            if let Some(known) = CREATURES.with_borrow(|creatures| creatures.get(key).cloned()) {
                return known;
            }
            // Marked as not there until it is: asked for once.
            CREATURES.with_borrow_mut(|creatures| creatures.insert(key.clone(), None));
            let key = key.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let SpriteKey::Creature(path, action, frame) = &key else { return };
                if let Ok(made) = creature(path, *action, *frame).await {
                    CREATURES.with_borrow_mut(|creatures| creatures.insert(key.clone(), Some(made)));
                    // The files are here now, so the rest of this action's pictures are made at once: an
                    // animation seen for the first time has all of them by its next picture, and does not
                    // show some and miss others while each is waited for in its turn.
                    let pictures = timing(path.split('#').next().unwrap_or(path), *action).map_or(0, |(pictures, _)| pictures);
                    for other in (0..pictures).filter(|other| other != frame) {
                        let other_key = SpriteKey::Creature(path.clone(), *action, other);
                        if CREATURES.with_borrow(|creatures| creatures.contains_key(&other_key)) {
                            continue;
                        }
                        CREATURES.with_borrow_mut(|creatures| creatures.insert(other_key.clone(), None));
                        if let Ok(made) = creature(path, *action, other).await {
                            CREATURES.with_borrow_mut(|creatures| creatures.insert(other_key, Some(made)));
                        }
                    }
                }
            });
            None
        }
    }
}

/// One frame of a sprite that is a single file, in a picture just large enough for it.
async fn creature(path: &str, action: usize, frame_number: usize) -> Result<Picture, String> {
    // A path may end in `#rrggbb`: the picture is then tinted with that colour (the red of a blow taken).
    let (path, tint) = match path.split_once('#') {
        Some((path, colour)) => (path, u32::from_str_radix(colour, 16).ok()),
        None => (path, None),
    };
    // A path may also name, after `@`, another file to take the actions from: a pet wearing its accessory is
    // its own pictures put together another way (the client's `SetActionName`).
    let (pictures, actions) = match path.split_once('@') {
        Some((pictures, actions)) => {
            let missing = || format!("{actions} is missing");
            (sprite(&format!("data/sprite/{pictures}")).await?.0, sprite::parse_act(&file(&format!("data/sprite/{actions}.act")).await.ok_or_else(missing)?)?)
        }
        None => sprite(&format!("data/sprite/{path}")).await?,
    };
    TIMES.with_borrow_mut(|times| {
        times.entry(path.to_owned()).or_insert_with(|| actions.actions.iter().enumerate().map(|(index, frames)| (frames.len(), f64::from(actions.delays.get(index).copied().unwrap_or(4.0)) * 25.0)).collect());
    });
    // Sprites differ in how many actions they have: one too many starts over.
    let action = action % actions.actions.len().max(1);
    let shown = frame(&actions, action, frame_number)?;
    STRIKES.with_borrow_mut(|strikes| {
        strikes.entry(path.to_owned()).or_insert_with(|| lands_at(&actions));
    });
    VOICES.with_borrow_mut(|voices| {
        voices.entry(path.to_owned()).or_insert_with(|| actions.actions.iter().map(|frames| frames.iter().map(|frame| frame.sound.and_then(|sound| actions.sounds.get(sound)).filter(|name| name.as_str() != "atk" && !name.is_empty()).cloned()).collect()).collect());
    });
    // The frame's reach around its origin (the feet), from where each of its layers goes.
    let (mut low, mut high) = ((0i32, 0i32), (1i32, 1i32));
    for layer in &shown.layers {
        let source = if layer.truecolor { &pictures.truecolor } else { &pictures.indexed };
        let Some(picture) = usize::try_from(layer.picture).ok().and_then(|index| source.get(index)) else { continue };
        let Some((left, top, width, height)) = layer.reach((picture.width, picture.height)) else { continue };
        low = (low.0.min(left), low.1.min(top));
        high = (high.0.max(left + width), high.1.max(top + height));
    }
    // Nothing in the game is larger than this; a broken file does not get to ask for more.
    let (width, height) = (((high.0 - low.0) as usize).min(1024), ((high.1 - low.1) as usize).min(1024));
    let mut canvas = Canvas::new(width.max(1), height.max(1));
    canvas.draw((-low.0, -low.1), &pictures, None, &shown);
    if let Some(tint) = tint {
        let [_, red, green, blue] = tint.to_be_bytes();
        for pixel in canvas.rgba.chunks_exact_mut(4) {
            pixel[0] = (u16::from(pixel[0]) * u16::from(red) / 255) as u8;
            pixel[1] = (u16::from(pixel[1]) * u16::from(green) / 255) as u8;
            pixel[2] = (u16::from(pixel[2]) * u16::from(blue) / 255) as u8;
        }
    }
    Ok(Picture { canvas: Rc::new(canvas), feet: (-low.0 as f32, -low.1 as f32) })
}

/// A frame of a sprite as a picture for the page: its data URL, its size, and where in it the sprite's origin is.
pub async fn frame_picture(path: &str, action: usize, frame: usize) -> Result<(String, (usize, usize), (f32, f32)), String> {
    let picture = creature(path, action, frame).await?;
    let document = crate::web::document().ok_or("no document")?;
    let element: HtmlCanvasElement = document.create_element("canvas").map_err(|_| "no canvas")?.dyn_into().map_err(|_| "no canvas")?;
    let canvas = &picture.canvas;
    element.set_width(canvas.width as u32);
    element.set_height(canvas.height as u32);
    let context: CanvasRenderingContext2d = element.get_context("2d").ok().flatten().ok_or("no context")?.unchecked_into();
    let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&canvas.rgba), canvas.width as u32, canvas.height as u32).map_err(|_| "no picture")?;
    context.put_image_data(&data, 0.0, 0.0).map_err(|_| "no picture")?;
    Ok((element.to_data_url().map_err(|_| "no picture")?, (canvas.width, canvas.height), picture.feet))
}

/// One of the game's own mouse pointers (an action of `cursors.spr`), as CSS: the picture and where in it the
/// pointer's tip is.
pub async fn cursor(action: usize, frame: usize) -> Result<String, String> {
    let picture = creature("cursors", action, frame).await?;
    let document = crate::web::document().ok_or("no document")?;
    let element: HtmlCanvasElement = document.create_element("canvas").map_err(|_| "no canvas")?.dyn_into().map_err(|_| "no canvas")?;
    let canvas = &picture.canvas;
    element.set_width(canvas.width as u32);
    element.set_height(canvas.height as u32);
    let context: CanvasRenderingContext2d = element.get_context("2d").ok().flatten().ok_or("no context")?.unchecked_into();
    let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&canvas.rgba), canvas.width as u32, canvas.height as u32).map_err(|_| "no picture")?;
    context.put_image_data(&data, 0.0, 0.0).map_err(|_| "no picture")?;
    let url = element.to_data_url().map_err(|_| "no picture")?;
    let tip = |at: f32, size: usize| (at.max(0.0) as usize).min(size.saturating_sub(1));
    Ok(format!("url({url}) {} {}, auto", tip(picture.feet.0, canvas.width), tip(picture.feet.1, canvas.height)))
}

/// The sound file a frame of a creature's sprite starts, once the sprite has been read. `first` tells whether
/// the frame is the first of its action's run, so that one-frame actions do not sound over and over.
pub fn sound_of(path: &str, action: usize, frame: usize) -> Option<String> {
    VOICES.with_borrow(|voices| {
        let actions = voices.get(path)?;
        let frames = actions.get(action % actions.len().max(1))?;
        frames.get(frame % frames.len().max(1))?.clone()
    })
}

/// How many frames an action of a sprite has and how long each shows (milliseconds), once the sprite was read.
pub fn timing(path: &str, action: usize) -> Option<(usize, f64)> {
    TIMES.with_borrow(|times| {
        let actions = times.get(path)?;
        actions.get(action % actions.len().max(1)).copied()
    })
}

/// The frame at which the blow of each action of a sprite lands (`crate::blow::frame`).
fn lands_at(actions: &crate::sprite::Act) -> Vec<usize> {
    let named = |frame: &crate::sprite::Frame| frame.sound.and_then(|sound| actions.sounds.get(sound)).map(String::as_str);
    actions.actions.iter().map(|frames| crate::blow::frame(frames.iter().map(named))).collect()
}

/// The frame of an action of a creature's sprite at which its blow lands, once the sprite has been read.
pub fn strike_frame(path: &str, action: usize) -> Option<usize> {
    STRIKES.with_borrow(|strikes| {
        let actions = strikes.get(path)?;
        actions.get(action % actions.len().max(1)).copied()
    })
}

/// The frame of an action of a character's body at which its blow lands, once a figure of that job and sex has
/// been put together.
pub fn body_strike_frame(job: u16, sex: u8, action: usize) -> Option<usize> {
    BODY_STRIKES.with_borrow(|strikes| strikes.get(&(job, sex))?.get(action).copied())
}

/// The sound file a frame of an action of a character's body starts (the swing of a weapon, the rustle of
/// clothes: the names of the body's `.act` that end in `.wav`, as `CRenderObject::PlayFrameSounds` asks),
/// once a figure of that job and sex has been put together.
pub fn body_sound_of(job: u16, sex: u8, action: usize, frame: usize) -> Option<String> {
    BODY_VOICES.with_borrow(|voices| voices.get(&(job, sex))?.get(action)?.get(frame)?.clone())
}

/// How many actions a sprite has, once it was read.
pub fn actions_of(path: &str) -> Option<usize> {
    TIMES.with_borrow(|times| times.get(path).map(Vec::len))
}

/// How many frames an action of a creature's sprite has, once the sprite has been read.
pub fn frames_of(path: &str, action: usize) -> Option<usize> {
    VOICES.with_borrow(|voices| {
        let actions = voices.get(path)?;
        Some(actions.get(action % actions.len().max(1))?.len())
    })
}

/// How many frames an action of a character's body has, once a figure of that job and sex has been put together.
pub fn body_frames(job: u16, sex: u8, action: usize) -> Option<usize> {
    body_timing(job, sex, action).map(|(frames, _)| frames)
}

/// How many frames an action of a character's body has and how long each shows (milliseconds), once a figure
/// of that job and sex has been put together.
pub fn body_timing(job: u16, sex: u8, action: usize) -> Option<(usize, f64)> {
    BODIES.with_borrow(|bodies| bodies.get(&(job, sex))?.get(action).copied())
}

/// Has a character's pictures of an action put together ahead of their use, seen from every side: a motion
/// that plays once and briefly would otherwise be over before its first picture is ready.
pub fn prepare(figure: &Figure, action: usize) {
    let frames = body_frames(figure.job, figure.sex, action).unwrap_or(3);
    for side in 0..8 {
        for frame in 0..frames {
            let _ = painted(&Figure { action: action + side, frame, ..figure.clone() });
        }
    }
}

thread_local! {
    /// By sprite path: for each action, how many frames it has and how long each shows.
    static TIMES: RefCell<HashMap<String, Vec<(usize, f64)>>> = RefCell::new(HashMap::new());
    /// By job and sex: how many frames each action of the body has, and how long each shows.
    static BODIES: RefCell<HashMap<(u16, u8), Vec<(usize, f64)>>> = RefCell::new(HashMap::new());
    /// By sprite path, and by job and sex: for each action, the frame at which its blow lands.
    static STRIKES: RefCell<HashMap<String, Vec<usize>>> = RefCell::new(HashMap::new());
    static BODY_STRIKES: RefCell<HashMap<(u16, u8), Vec<usize>>> = RefCell::new(HashMap::new());
    /// By job and sex: for each action of the body, the sound file each of its frames starts.
    static BODY_VOICES: RefCell<HashMap<(u16, u8), Vec<Vec<Option<String>>>>> = RefCell::new(HashMap::new());
    /// By sprite path: for each action, the sound each of its frames starts.
    static VOICES: RefCell<HashMap<String, Vec<Vec<Option<String>>>>> = RefCell::new(HashMap::new());
    static CREATURES: RefCell<HashMap<SpriteKey, Option<Picture>>> = RefCell::new(HashMap::new());
    /// Game files by path; `None` for one that is not there.
    static FILES: RefCell<HashMap<String, Option<Rc<Vec<u8>>>>> = RefCell::new(HashMap::new());
    static PAINTED: RefCell<HashMap<Figure, Rc<Canvas>>> = RefCell::new(HashMap::new());
    /// By canvas on the page: the figure it is to show, and the picture it shows.
    static ON_PAGE: RefCell<HashMap<String, (Figure, Option<Rc<Canvas>>)>> = RefCell::new(HashMap::new());
    /// Figures being put together right now.
    static PENDING: RefCell<std::collections::HashSet<Figure>> = RefCell::new(std::collections::HashSet::new());
}

/// The figure's picture, if it has been put together. If not, that is started, and a later call has it.
pub fn painted(figure: &Figure) -> Option<Rc<Canvas>> {
    if let Some(painted) = PAINTED.with_borrow(|painted| painted.get(figure).cloned()) {
        return Some(painted);
    }
    if PENDING.with_borrow_mut(|pending| pending.insert(figure.clone())) {
        let figure = figure.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match compose(&figure).await {
                Ok(canvas) => {
                    PAINTED.with_borrow_mut(|painted| painted.insert(figure.clone(), Rc::new(canvas)));
                }
                // Not yet: asked for again with the next frame.
                Err(error) if error == LATER => {
                    PENDING.with_borrow_mut(|pending| pending.remove(&figure));
                }
                Err(error) => web_sys::console::warn_1(&format!("figure: {error}").into()),
            }
            // A figure that failed is not asked for again and again: it stays pending.
            if PAINTED.with_borrow(|painted| painted.contains_key(&figure)) {
                PENDING.with_borrow_mut(|pending| pending.remove(&figure));
            }
        });
    }
    None
}

/// Forgets the figures painted so far: the tables that name their sprites have changed.
pub fn forget() {
    PAINTED.with_borrow_mut(HashMap::clear);
    weapon::forget();
}

/// Paints the figure on the canvas element with that ID: at once if it was painted before, else once its files
/// have arrived (if the element is still there then).
pub fn paint(canvas_id: &str, figure: Figure) {
    // Which figure this canvas is to show now: one asked for earlier and put together later does not take its
    // place.
    let before = ON_PAGE.with_borrow_mut(|on_page| {
        let entry = on_page.entry(canvas_id.to_owned()).or_insert_with(|| (figure.clone(), None));
        entry.0 = figure.clone();
        entry.1.clone()
    });
    if let Some(painted) = PAINTED.with_borrow(|painted| painted.get(&figure).cloned()) {
        ON_PAGE.with_borrow_mut(|on_page| on_page.insert(canvas_id.to_owned(), (figure, Some(painted.clone()))));
        return show(canvas_id, &painted);
    }
    // Until the new one is put together (its files may be yet to come), the canvas, written anew with the page,
    // shows what it showed last: the character does not blink away while another hair is fetched.
    if let Some(before) = before {
        show(canvas_id, &before);
    }
    let canvas_id = canvas_id.to_owned();
    wasm_bindgen_futures::spawn_local(async move {
        match compose(&figure).await {
            Ok(canvas) => {
                let canvas = Rc::new(canvas);
                PAINTED.with_borrow_mut(|painted| painted.insert(figure.clone(), canvas.clone()));
                let wanted = ON_PAGE.with_borrow_mut(|on_page| match on_page.get_mut(&canvas_id) {
                    Some((wanted, shown)) if *wanted == figure => {
                        *shown = Some(canvas.clone());
                        true
                    }
                    _ => false,
                });
                if wanted {
                    show(&canvas_id, &canvas);
                }
            }
            Err(error) if error == LATER => {}
            Err(error) => web_sys::console::warn_1(&format!("figure: {error}").into()),
        }
    });
}

fn show(canvas_id: &str, picture: &Canvas) {
    let element = crate::web::document().and_then(|document| document.get_element_by_id(canvas_id));
    let Some(Ok(canvas)) = element.map(|element| element.dyn_into::<HtmlCanvasElement>()) else { return };
    let Ok(Some(context)) = canvas.get_context("2d") else { return };
    let context: CanvasRenderingContext2d = context.unchecked_into();
    if let Ok(data) = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&picture.rgba), picture.width as u32, picture.height as u32) {
        // Whatever it showed before goes first: the pictures are not all of one size.
        context.clear_rect(0.0, 0.0, f64::from(canvas.width()), f64::from(canvas.height()));
        let _ = context.put_image_data(&data, 0.0, 0.0);
    }
}

async fn file(path: &str) -> Option<Rc<Vec<u8>>> {
    if let Some(known) = FILES.with_borrow(|files| files.get(path).cloned()) {
        return known;
    }
    let fetched = fetch_asset(path).await.ok().map(Rc::new);
    FILES.with_borrow_mut(|files| files.insert(path.to_owned(), fetched.clone()));
    fetched
}

/// A text file of the game, read as Latin letters (what is not is lost; names of maps and files are).
pub async fn text_file(path: &str) -> Option<String> {
    file(path).await.map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// A sprite's pictures and actions, by its path without the extension.
async fn sprite(path: &str) -> Result<(Sprite, Act), String> {
    let missing = || format!("{path} is missing");
    let pictures = sprite::parse_spr(&file(&format!("{path}.spr")).await.ok_or_else(missing)?)?;
    let actions = sprite::parse_act(&file(&format!("{path}.act")).await.ok_or_else(missing)?)?;
    Ok((pictures, actions))
}

fn frame(actions: &Act, action: usize, frame: usize) -> Result<Frame, String> {
    let frames = actions.actions.get(action).filter(|frames| !frames.is_empty()).ok_or("the sprite lacks that action")?;
    Ok(frames[frame % frames.len()].clone())
}

/// The sprite of a head by itself, as `frame_picture` takes one: of that job's kind, sex and hair style.
pub fn head_sprite(job: u16, sex: u8, hair_style: u16) -> String {
    let (doram, sex_word) = (crate::jobs::doram(job), if sex == 1 { "남" } else { "여" });
    format!("{}/머리통/{sex_word}/{}_{sex_word}", if doram { "도람족" } else { "인간족" }, if doram { hair_style } else { hair_sprite(sex, hair_style) })
}

/// Which head sprite a human hair style is: the first styles are numbered differently in the sprites than in the
/// game's data; later ones are the same. (The table is the client's, as korangar lists it.)
fn hair_sprite(sex: u8, style: u16) -> u16 {
    const MALE: [u16; 13] = [2, 2, 1, 7, 5, 4, 3, 6, 8, 9, 10, 12, 11];
    const FEMALE: [u16; 13] = [2, 2, 4, 7, 1, 5, 3, 6, 12, 10, 9, 11, 8];
    let table = if sex == 1 { &MALE } else { &FEMALE };
    table.get(usize::from(style)).copied().unwrap_or(style)
}
