//! Characters as the windows show them: body, head and headgear sprites put together and painted on a small
//! canvas.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

use super::{data, fetch_asset};
use crate::sprite::{self, Act, Canvas, Frame, Sprite};

/// The size of a figure's canvas, and where in it the feet stand: as in a character slot of the select window.
pub const WIDTH: usize = 157;
pub const HEIGHT: usize = 195;
const FEET: (i32, i32) = (80, 137);

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
    /// Which of the sprite's actions: standing is 0-7 by direction (south first, then clockwise), walking 8-15,
    /// sitting 16-23.
    pub action: usize,
    /// Which picture of the action; a number past its last starts over.
    pub frame: usize,
}

thread_local! {
    /// Game files by path; `None` for one that is not there.
    static FILES: RefCell<HashMap<String, Option<Rc<Vec<u8>>>>> = RefCell::new(HashMap::new());
    static PAINTED: RefCell<HashMap<Figure, Rc<Canvas>>> = RefCell::new(HashMap::new());
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
}

/// Paints the figure on the canvas element with that ID: at once if it was painted before, else once its files
/// have arrived (if the element is still there then).
pub fn paint(canvas_id: &str, figure: Figure) {
    if let Some(painted) = PAINTED.with_borrow(|painted| painted.get(&figure).cloned()) {
        return show(canvas_id, &painted);
    }
    let canvas_id = canvas_id.to_owned();
    wasm_bindgen_futures::spawn_local(async move {
        match compose(&figure).await {
            Ok(canvas) => {
                let canvas = Rc::new(canvas);
                PAINTED.with_borrow_mut(|painted| painted.insert(figure, canvas.clone()));
                show(&canvas_id, &canvas);
            }
            Err(error) => web_sys::console::warn_1(&format!("figure: {error}").into()),
        }
    });
}

fn show(canvas_id: &str, picture: &Canvas) {
    let element = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id(canvas_id));
    let Some(Ok(canvas)) = element.map(|element| element.dyn_into::<HtmlCanvasElement>()) else { return };
    let Ok(Some(context)) = canvas.get_context("2d") else { return };
    let context: CanvasRenderingContext2d = context.unchecked_into();
    if let Ok(data) = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&picture.rgba), picture.width as u32, picture.height as u32) {
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

async fn compose(figure: &Figure) -> Result<Canvas, String> {
    let sex = if figure.sex == 1 { "남" } else { "여" };
    let doram = data::is_doram(figure.job);
    let race = if doram { "도람족" } else { "인간족" };
    let job = data::body_sprite(figure.job);
    // The first human hair styles are numbered differently in the sprites than in the game's data.
    let hair = if doram { figure.hair_style } else { hair_sprite(figure.sex, figure.hair_style) };

    let (body_sprite, body_act) = sprite(&format!("data/sprite/{race}/몸통/{sex}/{job}_{sex}")).await?;
    let (head_sprite, head_act) = sprite(&format!("data/sprite/{race}/머리통/{sex}/{hair}_{sex}")).await?;
    // Colour 0 is the sprite's own; the others are palettes of their own.
    let palette_folder = if doram { "data/palette/도람족" } else { "data/palette" };
    let hair_palette = match figure.hair_colour {
        0 => None,
        colour => file(&format!("{palette_folder}/머리/머리{hair}_{sex}_{colour}.pal")).await.and_then(|bytes| sprite::parse_palette(&bytes)),
    };
    let body_palette = match figure.body_palette {
        0 => None,
        palette => file(&format!("{palette_folder}/몸/{job}_{sex}_{palette}.pal")).await.and_then(|bytes| sprite::parse_palette(&bytes)),
    };

    // Standing still, a head's pictures are it looking ahead, right and left: it looks ahead. In other actions
    // the head has a picture for each of the body's.
    let head_picture = if figure.action < 8 { 0 } else { figure.frame };
    let (body_frame, head_frame) = (frame(&body_act, figure.action, figure.frame)?, frame(&head_act, figure.action, head_picture)?);
    let attach = |part: &Frame| {
        let offset = sprite::head_offset(&body_frame, part);
        (FEET.0 + offset.0, FEET.1 + offset.1)
    };
    let mut canvas = Canvas::new(WIDTH, HEIGHT);
    canvas.draw(FEET, &body_sprite, body_palette.as_ref(), &body_frame);
    canvas.draw(attach(&head_frame), &head_sprite, hair_palette.as_ref(), &head_frame);

    // Headgear sits on the head by the same anchor: bottom first, then middle, then top. One whose sprite is not
    // there is left out, and the rest of the figure is still drawn.
    let [bottom, top, middle] = figure.headgear;
    for view in [bottom, middle, top] {
        let Some(name) = data::headgear(view) else { continue };
        match sprite(&format!("data/sprite/악세사리/{sex}/{sex}{name}")).await {
            Ok((pictures, actions)) => {
                if let Ok(part) = frame(&actions, figure.action, head_picture) {
                    canvas.draw(attach(&part), &pictures, None, &part);
                }
            }
            Err(error) => web_sys::console::warn_1(&format!("figure: {error}").into()),
        }
    }
    Ok(canvas)
}

/// Which head sprite a human hair style is: the first styles are numbered differently in the sprites than in the
/// game's data; later ones are the same. (The table is the client's, as korangar lists it.)
fn hair_sprite(sex: u8, style: u16) -> u16 {
    const MALE: [u16; 13] = [2, 2, 1, 7, 5, 4, 3, 6, 8, 9, 10, 12, 11];
    const FEMALE: [u16; 13] = [2, 2, 4, 7, 1, 5, 3, 6, 12, 10, 9, 11, 8];
    let table = if sex == 1 { &MALE } else { &FEMALE };
    table.get(usize::from(style)).copied().unwrap_or(style)
}
