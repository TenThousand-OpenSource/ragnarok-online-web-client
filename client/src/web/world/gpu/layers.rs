//! The sprites of the effects made of the client's primitives. The client draws a frame of such a sprite layer
//! by layer, each layer's own picture as a quad in the primitive's colour (`CEffectPrim::RenderSegmentSprites2`;
//! the quads are `crate::sprite::layer_quads`), so here a sprite is read as it is, its pictures and its frames,
//! and each of its pictures goes on the GPU by itself. (The sprites of those who stand in the world, and the
//! numbers of damage, are put together a frame to a picture: `figure::picture`.)

use super::*;
use crate::sprite::{Act, Sprite};

/// A sprite as it was read: its pictures and how its frames are laid out.
pub(in crate::web::world) struct Sheet {
    pub(in crate::web::world) sprite: Sprite,
    pub(in crate::web::world) act: Act,
}

thread_local! {
    /// The sprites read for the effects, by path under the client's folder of sprites, without the ending;
    /// nothing for one being read or that could not be.
    static SHEETS: RefCell<HashMap<String, Option<Rc<Sheet>>>> = RefCell::new(HashMap::new());
}

/// Reads a sprite's two files.
async fn read(path: &str) -> Result<Sheet, String> {
    let sprite = crate::sprite::parse_spr(&fetch_asset(&format!("data/sprite/{path}.spr")).await?)?;
    let act = crate::sprite::parse_act(&fetch_asset(&format!("data/sprite/{path}.act")).await?)?;
    Ok(Sheet { sprite, act })
}

/// The sprite of that path, if it has been read. If not, reading it is started, once, and a later call has it.
pub(in crate::web::world) fn sheet(path: &str) -> Option<Rc<Sheet>> {
    if let Some(known) = SHEETS.with_borrow(|sheets| sheets.get(path).cloned()) {
        return known;
    }
    SHEETS.with_borrow_mut(|sheets| sheets.insert(path.to_owned(), None));
    let path = path.to_owned();
    wasm_bindgen_futures::spawn_local(async move {
        match read(&path).await {
            Ok(sheet) => {
                SHEETS.with_borrow_mut(|sheets| sheets.insert(path, Some(Rc::new(sheet))));
            }
            Err(error) => web_sys::console::warn_1(&format!("effect sprite: {error}").into()),
        }
    });
    None
}

/// How many frames an action of a sprite has, once the sprite has been read.
pub(in crate::web::world) fn frames_of(path: &str, action: usize) -> Option<usize> {
    Some(sheet(path)?.act.actions.get(action)?.len())
}

impl Gpu {
    /// One picture of a sprite for the effects' pipelines, put on the GPU the first time it is asked for;
    /// nothing for a picture the sprite lacks.
    pub(in crate::web::world) fn effect_layer(&mut self, path: &str, sheet: &Sheet, truecolor: bool, picture: usize) -> Option<Rc<[wgpu::BindGroup; 2]>> {
        if let Some(known) = self.effect_layers.get(path).and_then(|pictures| pictures.get(&(truecolor, picture))) {
            return Some(known.clone());
        }
        let (width, height) = sheet.sprite.size_of(truecolor, picture).filter(|(width, height)| width * height > 0)?;
        let view = self.texture(width as u32, height as u32, &sheet.sprite.rgba_of(truecolor, picture)?);
        let binds = Rc::new([0, 1].map(|which| self.textured(&self.effect_pipelines[which].get_bind_group_layout(1), &view, &self.pixel_sampler)));
        self.effect_layers.entry(path.to_owned()).or_default().insert((truecolor, picture), binds.clone());
        Some(binds)
    }
}
