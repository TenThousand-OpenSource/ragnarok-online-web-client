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
    // The client's own list of the maps that are inside buildings: a line `name.rsw#` each.
    let indoor = match figure::text_file("data/indoorrswtable.txt").await {
        Some(list) => {
            let wanted = format!("{}.rsw#", map.to_ascii_lowercase());
            list.lines().any(|line| line.trim().to_ascii_lowercase().starts_with(&wanted))
        }
        None => false,
    };
    let korean = web_sys::TextDecoder::new_with_label("euc-kr").map_err(js_error)?;
    let gnd = Gnd::parse(&fetch_asset(&format!("data/{map}.gnd")).await?, |name| korean.decode_with_u8_array(name).unwrap_or_else(|_| String::from_utf8_lossy(name).into_owned()))?;

    // Every texture of the ground, fetched side by side.
    let pictures = futures_util::future::join_all(gnd.textures.iter().map(|name| async move {
        let path = format!("data/texture/{}", name.replace('\\', "/"));
        let picture = fetch_asset(&path).await.and_then(|bytes| bmp::decode(&bytes));
        if let Err(error) = &picture {
            web_sys::console::warn_1(&format!("texture {path}: {error}").into());
        }
        picture.ok()
    }))
    .await;

    let mesh = gnd.mesh();
    let ground = gpu.ground(&mesh, &pictures);
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
    if gpu.mark.is_none() {
        // The client's own picture for the cell under the pointer, in the colour the client gives it there
        // (0x6600FFAA: green, two fifths there).
        if let Ok(mut image) = picture("grid.tga").await {
            for pixel in image.rgba.chunks_exact_mut(4) {
                pixel[0] = 0;
                pixel[2] = (u16::from(pixel[2]) * 0xaa / 255) as u8;
                pixel[3] = (u16::from(pixel[3]) * 0x66 / 255) as u8;
            }
            let view = gpu.texture(image.width, image.height, &image.rgba);
            gpu.mark = Some(gpu.textured(&gpu.sprite_pipeline.get_bind_group_layout(1), &view, &gpu.pixel_sampler));
        }
    }
    if gpu.ring.is_none() {
        if let Ok(image) = picture("effect\\ring_blue.tga").await {
            let view = gpu.texture(image.width, image.height, &image.rgba);
            gpu.ring = Some(gpu.textured(&gpu.sprite_pipeline.get_bind_group_layout(1), &view, &gpu.pixel_sampler));
        }
    }
    for name in CAST_PICTURES.into_iter().chain(prims::pictures()).chain(ailments::PICTURES).chain(skill_units::PICTURES) {
        if !gpu.effect_pictures.contains_key(name) {
            if let Ok(image) = picture(&effect_picture(name)).await {
                let view = gpu.texture(image.width, image.height, &image.rgba);
                let binds = [0, 1].map(|which| gpu.textured(&gpu.effect_pipelines[which].get_bind_group_layout(1), &view, &gpu.sampler));
                gpu.effect_pictures.insert(name, Rc::new(binds));
            }
        }
    }
    // The sprites of the effects are asked for ahead of their use: an effect is short, and a sprite read only
    // once it is wanted would miss its first showing.
    for sprite in prims::sprites() {
        let _ = gpu::sheet(sprite);
    }
    pointers();
    progress(100);
    let weapon = STATE.with_borrow(|state| state.weapon);
    // (The shield is the one the app gave with the figure, until the server tells of another: `actors::looked`.)
    let shield = STATE.with_borrow(|state| if state.shield != 0 { state.shield } else { figure.shield });
    let mut player = Mover::new(Look::Character(Figure { weapon, shield, ..figure }), cell);
    player.state = STATE.with_borrow(|state| state.own_state);
    player.dress();
    player.ail = STATE.with_borrow(|state| state.own_ail.clone());
    Ok((gpu, Scene { indoor, gat, ground, models, water, player, units: HashMap::new(), drops: HashMap::new(), falls: HashMap::new(), aim: None, portals: HashMap::new(), spots: places, // A new map is looked at from the south, as the original does: only how near the camera is stays.
        camera, wanted: (camera, now()), scenery: surroundings, told: (-1, -1) }))
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

/// Over what can be pressed in the windows the pointer is the client's finger, and while the button is held on
/// it the finger goes down picture by picture (`crate::cursor::frame` for action 2) and stays down: the page's
/// `--pressed` is the picture of the moment, from the button going down.
fn finger_goes_down(finger: Vec<String>) {
    let Some(document) = crate::web::document() else { return };
    let pressed_at = Rc::new(std::cell::Cell::new(None::<f64>));
    let step = {
        let pressed_at = pressed_at.clone();
        Closure::<dyn FnMut()>::new(move || {
            let Some(since) = pressed_at.get() else { return };
            let Some(root) = crate::web::document().and_then(|document| document.document_element()).and_then(|root| root.dyn_into::<web_sys::HtmlElement>().ok()) else { return };
            if let Some(picture) = finger.get(crate::cursor::frame(2, finger.len(), 0.0, Some(now() - since))) {
                let _ = root.style().set_property("--pressed", picture);
            }
        })
    };
    let down = {
        let pressed_at = pressed_at.clone();
        Closure::<dyn FnMut()>::new(move || pressed_at.set(Some(now())))
    };
    let up = Closure::<dyn FnMut()>::new(move || pressed_at.set(None));
    let _ = document.add_event_listener_with_callback("pointerdown", down.as_ref().unchecked_ref());
    let _ = document.add_event_listener_with_callback("pointerup", up.as_ref().unchecked_ref());
    // A tick of the client's pointer is 24 milliseconds.
    if let Some(window) = web_sys::window() {
        let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(step.as_ref().unchecked_ref(), 24);
    }
    // These live as long as the page.
    step.forget();
    down.forget();
    up.forget();
}

/// Reads the game's own mouse pointers, once.
pub(super) fn pointers() {
    if STATE.with_borrow(|state| !state.pointers.is_empty()) {
        return;
    }
    wasm_bindgen_futures::spawn_local(async {
        let mut read = Vec::new();
        for action in POINTERS {
            // The first picture tells how many there are.
            let Ok(first) = figure::cursor(action, 0).await else { return };
            let (count, shows) = figure::timing("cursors", action).unwrap_or((1, 100.0));
            let mut pictures = vec![first];
            for frame in 1..count.min(32) {
                match figure::cursor(action, frame).await {
                    Ok(picture) => pictures.push(picture),
                    Err(_) => break,
                }
            }
            read.push((pictures, shows.max(40.0)));
        }
        // Everything pressable on the page has the game's pointers too: the plain one, and the pressing one.
        if let (Some(root), Some(plain), Some(press)) = (
            crate::web::document().and_then(|document| document.document_element()).and_then(|root| root.dyn_into::<web_sys::HtmlElement>().ok()),
            read.first().and_then(|(pictures, _)| pictures.first()),
            figure::cursor(2, 0).await.ok(),
        ) {
            let _ = root.style().set_property("--point", plain);
            let _ = root.style().set_property("--press", &press);
            // The pointer of a skill being aimed (the client's cursor action 10), for over the windows.
            if let Ok(aim) = figure::cursor(10, 0).await {
                let _ = root.style().set_property("--aim", &aim);
            }
            // The finger down: the pictures of that action, which the client runs through once while the
            // button is held (`finger_goes_down`).
            let mut finger = vec![press];
            for frame in 1..figure::timing("cursors", 2).map_or(3, |(pictures, _)| pictures).min(16) {
                match figure::cursor(2, frame).await {
                    Ok(picture) => finger.push(picture),
                    Err(_) => break,
                }
            }
            if let Some(down) = finger.last() {
                let _ = root.style().set_property("--pressed", down);
            }
            finger_goes_down(finger);
        }
        // Every picture of every pointer is read by the browser ahead of its use and kept on the page, out of
        // sight: a pointer whose picture the browser has yet to read shows as the plain arrow for a moment, which
        // made the pointer flicker the first time it ran over a monster.
        if let Some(document) = crate::web::document() {
            if let (Ok(keep), Some(body)) = (document.create_element("div"), document.body()) {
                let _ = keep.set_attribute("style", "position:fixed;left:-99px;top:-99px;width:1px;height:1px;overflow:hidden;pointer-events:none");
                for picture in read.iter().flat_map(|(pictures, _)| pictures) {
                    let Some(url) = picture.strip_prefix("url(").and_then(|rest| rest.split(')').next()) else { continue };
                    if let Ok(image) = web_sys::HtmlImageElement::new() {
                        image.set_src(url);
                        let _ = wasm_bindgen_futures::JsFuture::from(image.decode()).await;
                        let _ = keep.append_child(&image);
                    }
                }
                let _ = body.append_child(&keep);
            }
        }
        STATE.with_borrow_mut(|state| {
            state.pointers = read;
            // Shown with the next frame.
            state.pointing = usize::MAX;
        });
    });
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
