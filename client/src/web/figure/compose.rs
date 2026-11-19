//! A character put together from its parts, as the client's `SpriteLayerRenderer::Render` draws one: the
//! body and the head in the order the job's `.imf` gives for the frame, the weapon, its trail and the shield
//! where the direction puts them (`crate::sprite::imf::order`) and only in the actions that show them
//! (`crate::pose::armed`), and what is worn on the head over all of that by its priorities
//! (`data::head_layers`).
//!
//! The client draws each part as a quad of its own; here the parts are painted into one picture in the
//! same order.

use super::*;
use crate::sprite::imf::{self, Part};

/// The names the client's shields end in, by the view of the item (`CSession::m_robeNames`, which
/// `GetShieldResourceName` asks): a guard, a buckler, a shield, a mirror shield.
const SHIELDS: [&str; 5] = ["", "_가드", "_버클러", "_쉴드", "_미러쉴드"];

/// The shield on a figure's arm as it is in the frame the figure is in (`GetShieldResourceName`): the sprite
/// made for that very shield if the client has it, else the one of its view. Nothing in an action that shows
/// no shield, or for a shield without a sprite.
async fn shield(figure: &Figure, sex: &str, job: &str) -> Option<(Sprite, Frame)> {
    if figure.shield == 0 || !crate::pose::armed(figure.action) {
        return None;
    }
    let base = format!("data/sprite/방패/{job}/{job}_{sex}");
    let (pictures, actions) = match sprite(&format!("{base}_{}_방패", figure.shield)).await {
        Ok(own) => own,
        Err(_) => {
            let view = data::about(figure.shield)?.1.into_iter().find(|(key, _)| key == "View").and_then(|(_, view)| view.parse::<usize>().ok())?;
            sprite(&format!("{base}{}", SHIELDS.get(view).filter(|name| !name.is_empty())?)).await.ok()?
        }
    };
    Some((pictures, frame(&actions, figure.action, figure.frame).ok()?))
}

/// Which of the body and the head lies under the other in the figure's frame, as the job's `.imf` says; the
/// body, where the file is missing or does not reach that far.
async fn head_under(figure: &Figure, sex: &str, job: &str) -> bool {
    let read = file(&format!("data/imf/{job}_{sex}.imf")).await.and_then(|bytes| imf::parse_imf(&bytes).ok());
    read.is_some_and(|imf| imf.head_under(figure.action, figure.frame))
}

pub(super) async fn compose(figure: &Figure) -> Result<Canvas, String> {
    let sex = if figure.sex == 1 { "남" } else { "여" };
    let doram = data::is_doram(figure.job);
    let race = if doram { "도람족" } else { "인간족" };
    let job = data::body_sprite(figure.job);
    // The first human hair styles are numbered differently in the sprites than in the game's data.
    let hair = if doram { figure.hair_style } else { hair_sprite(figure.sex, figure.hair_style) };

    // Which sprite a weapon or a shield has depends on what the gateway tells of the item: until it has, the
    // figure waits.
    for held in [figure.weapon, figure.shield] {
        if held != 0 && data::about(held).is_none() {
            data::want_items(std::iter::once(held), || {});
            return Err(LATER.to_owned());
        }
    }
    let (body_sprite, body_act) = sprite(&format!("data/sprite/{race}/몸통/{sex}/{job}_{sex}")).await?;
    BODIES.with_borrow_mut(|bodies| {
        bodies.entry((figure.job, figure.sex)).or_insert_with(|| body_act.actions.iter().enumerate().map(|(index, frames)| (frames.len(), f64::from(body_act.delays.get(index).copied().unwrap_or(4.0)) * 25.0)).collect());
    });
    BODY_STRIKES.with_borrow_mut(|strikes| {
        strikes.entry((figure.job, figure.sex)).or_insert_with(|| lands_at(&body_act));
    });
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

    // Standing or sitting still, the pictures of a body and a head are it looking ahead, right and left
    // (the client's `curMotion = headDir`): the frame says which. In other actions the head has a picture for
    // each of the body's.
    let (body_frame, head_frame) = (frame(&body_act, figure.action, figure.frame)?, frame(&head_act, figure.action, figure.frame)?);
    let attach = |part: &Frame| {
        let offset = sprite::head_offset(&body_frame, part);
        (FEET.0 + offset.0, FEET.1 + offset.1)
    };
    // The weapon in hand has a sprite of its own, with a picture for each of the body's; so have its trail
    // and the shield.
    let (weapon, trail, shield) = (weapon::part(figure, false).await, weapon::part(figure, true).await, shield(figure, sex, job).await);
    let mut canvas = Canvas::new(WIDTH, HEIGHT);
    for part in imf::order(figure.action % 8, head_under(figure, sex, job).await) {
        match part {
            Part::Body => canvas.draw(FEET, &body_sprite, body_palette.as_ref(), &body_frame),
            Part::Head => canvas.draw(attach(&head_frame), &head_sprite, hair_palette.as_ref(), &head_frame),
            Part::Weapon | Part::Trail | Part::Shield => {
                let held = match part {
                    Part::Weapon => &weapon,
                    Part::Trail => &trail,
                    _ => &shield,
                };
                if let Some((pictures, part)) = held {
                    canvas.draw(FEET, pictures, None, part);
                }
            }
        }
    }

    // Headgear sits on the head by the same anchor, the lowest priority first. One whose sprite is not there
    // is left out, and the rest of the figure is still drawn.
    for slot in imf::worn(data::head_layers(figure.action % 8, figure.headgear)) {
        let Some(name) = data::headgear(figure.headgear[slot]) else { continue };
        match sprite(&format!("data/sprite/악세사리/{sex}/{sex}{name}")).await {
            Ok((pictures, actions)) => {
                if let Ok(part) = frame(&actions, figure.action, figure.frame) {
                    canvas.draw(attach(&part), &pictures, None, &part);
                }
            }
            Err(error) => web_sys::console::warn_1(&format!("figure: {error}").into()),
        }
    }
    Ok(canvas)
}
