//! Reading a map from the client's files: ground, models, water, and the pictures the effects need.

use super::*;

/// Tells the page how far the map is read: the bar of the loading screen fills, with the figure on it.
pub(super) fn progress(percent: u32) {
    if let Some(fill) = crate::web::by_id("loading-fill") {
        let _ = fill.set_attribute("style", &format!("width:{}px", 238 * percent / 100));
    }
    if let Some(figure) = crate::web::by_id("loading-figure") {
        figure.set_text_content(Some(&format!("{percent:3}%")));
    }
}

pub(super) async fn load(map: &str, cell: (u16, u16), figure: Figure) -> Result<(Gpu, Scene), String> {
    progress(0);
    let gpu = match STATE.with_borrow_mut(|state| state.gpu.take()) {
        Some(gpu) => gpu,
        None => Gpu::new().await?,
    };
    progress(10);
    let gat = Gat::parse(&fetch_asset(&format!("data/{map}.gat")).await?)?;
    let indoor = indoor(map).await;
    let korean = web_sys::TextDecoder::new_with_label("euc-kr").map_err(js_error)?;
    let gnd = Gnd::parse(&fetch_asset(&format!("data/{map}.gnd")).await?, |name| korean.decode_with_u8_array(name).unwrap_or_else(
        |_| String::from_utf8_lossy(name).into_owned()
    ))?;

    let pictures = ground_pictures(&gnd).await;
    let ground = gpu.ground(&gnd.mesh(), &pictures);
    progress(50);

    // What stands on the ground, and the water. A map still shows without them, should they not read.
    let decode = |name: &[u8]| korean.decode_with_u8_array(name).unwrap_or_else(|_| String::from_utf8_lossy(name).into_owned());
    let described = match fetch_asset(&format!("data/{map}.rsw")).await {
        Ok(bytes) => Rsw::parse(&bytes, decode),
        Err(error) => Err(error),
    };
    let size = (gnd.width as f32 * gnd.zoom, gnd.height as f32 * gnd.zoom);
    let (models, water) = match &described {
        Ok(rsw) => (land::models(&gpu, rsw, size, decode).await, land::water(&gpu, rsw, &gnd).await),
        Err(error) => {
            web_sys::console::warn_1(&format!("{map}.rsw: {error}").into());
            (Models::none(), None)
        }
    };
    // What the map places about itself, and what the client's tables say of it.
    let (surroundings, places) = scenery::Surroundings::read(map, described.as_ref().ok(), size).await;
    let camera = Camera::entering(indoor, true, surroundings.view.as_ref());
    let cell = (i32::from(cell.0), i32::from(cell.1));
    // The pictures of the last map's people are not needed here.
    let mut gpu = gpu;
    gpu.sprite_textures.clear();
    effect_pictures_read(&mut gpu).await;
    pointers();
    progress(100);
    let player = own_mover(figure, cell);
    // A new map is looked at from the south, as the original does: only how near the camera is stays.
    let scene = Scene {
        indoor,
        gat,
        ground,
        models,
        water,
        player,
        units: HashMap::new(),
        drops: HashMap::new(),
        falls: HashMap::new(),
        aim: None,
        portals: HashMap::new(),
        spots: places,
        camera,
        wanted: (camera, now()),
        scenery: surroundings,
        told: (-1, -1)
    };
    Ok((gpu, scene))
}

/// Whether a map is inside a building: whether the client's own list of those has it, a line `name.rsw#` each.
async fn indoor(map: &str) -> bool {
    match figure::text_file("data/indoorrswtable.txt").await {
        Some(list) => {
            let wanted = format!("{}.rsw#", map.to_ascii_lowercase());
            list.lines().any(|line| line.trim().to_ascii_lowercase().starts_with(&wanted))
        }
        None => false,
    }
}

/// Every texture of the ground, fetched side by side; none for one that could not be read.
async fn ground_pictures(gnd: &Gnd) -> Vec<Option<bmp::Image>> {
    futures_util::future::join_all(gnd.textures.iter().map(|name| async move {
        let path = format!("data/texture/{}", name.replace('\\', "/"));
        let picture = fetch_asset(&path).await.and_then(|bytes| bmp::decode(&bytes));
        if let Err(error) = &picture {
            web_sys::console::warn_1(&format!("texture {path}: {error}").into());
        }
        picture.ok()
    }))
    .await
}

/// The pictures the world draws by code are put on the GPU, those it does not have yet: the mark of the cell
/// under the pointer, a portal's ring, the pictures of the effects; and the effects' sprites are asked for.
async fn effect_pictures_read(gpu: &mut Gpu) {
    if gpu.mark.is_none() {
        // The client's own picture for the cell under the pointer, in the colour the client gives it there
        // (0x6600FFAA: green, two fifths there).
        if let Ok(mut image) = picture("grid.tga").await {
            for pixel in image.rgba.chunks_exact_mut(4) {
                pixel[0] = 0;
                pixel[2] = (u16::from(pixel[2]) * 0xaa / 255) as u8;
                pixel[3] = (u16::from(pixel[3]) * 0x66 / 255) as u8;
            }
            gpu.mark = Some(gpu.sprite_bind(image.width, image.height, &image.rgba));
        }
    }
    if gpu.ring.is_none() {
        if let Ok(image) = picture("effect\\ring_blue.tga").await {
            gpu.ring = Some(gpu.sprite_bind(image.width, image.height, &image.rgba));
        }
    }
    for name in CAST_PICTURES.into_iter().chain(prims::pictures()).chain(ailments::PICTURES).chain(skill_units::PICTURES) {
        if !gpu.effect_pictures.contains_key(name) {
            if let Ok(image) = picture(&effect_picture(name)).await {
                let binds = gpu.effect_binds(image.width, image.height, &image.rgba, &gpu.sampler);
                gpu.effect_pictures.insert(name, binds);
            }
        }
    }
    // The sprites of the effects are asked for ahead of their use: an effect is short, and a sprite read only
    // once it is wanted would miss its first showing.
    for sprite in prims::sprites() {
        let _ = gpu::sheet(sprite);
    }
}

/// The player's own character as it comes onto a map: with the weapon, the shield, the state and the ailments
/// the world has kept of it.
fn own_mover(figure: Figure, cell: (i32, i32)) -> Mover {
    let weapon = STATE.with_borrow(|state| state.weapon);
    // (The shield is the one the app gave with the figure, until the server tells of another: `actors::looked`.)
    let shield = STATE.with_borrow(|state| if state.shield != 0 { state.shield } else { figure.shield });
    let mut player = Mover::new(Look::Character(Figure { weapon, shield, ..figure }), cell);
    player.state = STATE.with_borrow(|state| state.own_state);
    player.dress();
    player.ail = STATE.with_borrow(|state| state.own_ail.clone());
    player
}

/// Where a picture an effect names is, under `data/texture/`: among the client's pictures of effects, but for
/// one named from the folder of the interface's pictures (the pictures of items some effects show, as
/// `CRagEffect::TexturedSprayKind` does), which is where its name says.
fn effect_picture(name: &str) -> String {
    if name.starts_with("유저인터페이스\\") { name.to_owned() } else { format!("effect\\{name}") }
}

/// A picture of the game by its path under `data/texture/`: BMP files are read here, everything else (TGA with
/// its own transparency, JPEG) by the browser, from the gateway's PNG of it.
pub(super) async fn picture(name: &str) -> Result<bmp::Image, String> {
    let path = format!("data/texture/{}", name.replace('\\', "/"));
    if path.to_ascii_lowercase().ends_with(".bmp") {
        return bmp::decode(&fetch_asset(&path).await?);
    }
    let image = web_sys::HtmlImageElement::new().map_err(js_error)?;
    image.set_src(&image_url(&path));
    wasm_bindgen_futures::JsFuture::from(image.decode()).await.map_err(|_| format!("{path} could not be read"))?;
    let (width, height) = (image.natural_width(), image.natural_height());
    let document = crate::web::document().ok_or("no document")?;
    let canvas: HtmlCanvasElement = document.create_element("canvas").map_err(js_error)?.dyn_into().map_err(js_error)?;
    canvas.set_width(width);
    canvas.set_height(height);
    let context: web_sys::CanvasRenderingContext2d = canvas.get_context("2d").map_err(js_error)?.ok_or("no 2D canvas")?.unchecked_into();
    context.draw_image_with_html_image_element(&image, 0.0, 0.0).map_err(js_error)?;
    let rgba = context.get_image_data(0.0, 0.0, f64::from(width), f64::from(height)).map_err(js_error)?.data().0;
    Ok(bmp::Image { width, height, rgba })
}

/// The pictures of the effects drawn by code: the rings around a caster and the mark on the one aimed at.
const CAST_PICTURES: [&str; 7] = [
    // The circle on the ground of a skill cast on a place (`scopes.rs`).
    "magic_target.tga",
    "ring_blue.tga",
    "ring_yellow.tga",
    "ring_white.tga",
    "magic_green.tga",
    "magic_violet.tga",
    "lockon128.tga",
];
