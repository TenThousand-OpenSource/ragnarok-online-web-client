//! Reading a map from the client's files: ground, models, water, and the pictures the effects need.

use super::*;

/// Tells the page how far the map is read: the bar of the loading screen fills, with the figure on it.
pub(super) fn progress(percent: u32) {
    let Some(document) = crate::web::document() else { return };
    if let Some(fill) = document.get_element_by_id("loading-fill") {
        let _ = fill.set_attribute("style", &format!("width:{}px", 238 * percent / 100));
    }
    if let Some(figure) = document.get_element_by_id("loading-figure") {
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
    let (models, water) = match described {
        Ok(rsw) => {
            let size = (gnd.width as f32 * gnd.zoom, gnd.height as f32 * gnd.zoom);
            (models(&gpu, &rsw, size, decode).await, water(&gpu, &rsw, size).await)
        }
        Err(error) => {
            web_sys::console::warn_1(&format!("{map}.rsw: {error}").into());
            (Vec::new(), None)
        }
    };
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
    for name in CAST_PICTURES.into_iter().chain(prims::PICTURES) {
        if !gpu.effect_pictures.contains_key(name) {
            if let Ok(image) = picture(&format!("effect\\{name}")).await {
                let view = gpu.texture(image.width, image.height, &image.rgba);
                let binds = [0, 1].map(|which| gpu.textured(&gpu.effect_pipelines[which].get_bind_group_layout(1), &view, &gpu.sampler));
                gpu.effect_pictures.insert(name, Rc::new(binds));
            }
        }
    }
    pointers();
    progress(100);
    let weapon = STATE.with_borrow(|state| state.weapon);
    let player = Mover::new(Look::Character(Figure { weapon, ..figure }), cell);
    Ok((gpu, Scene { indoor, gat, ground, models, water, player, units: HashMap::new(), drops: HashMap::new(), falls: HashMap::new(), aim: None, portals: HashMap::new(), // A new map is looked at from the south, as the original does: only how near the camera is stays.
        camera: Camera::starting(indoor, true), told: (-1, -1) }))
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

/// Every model of the map, standing where the map puts it: fetched, put together, lit, and sorted by texture.
pub(super) async fn models(gpu: &Gpu, rsw: &Rsw, map_size: (f32, f32), decode: impl Fn(&[u8]) -> String + Copy) -> Models {
    let names: BTreeSet<&str> = rsw.placements.iter().map(|placement| placement.model.as_str()).collect();
    let fetched = futures_util::future::join_all(names.iter().map(|name| async move {
        let path = format!("data/model/{}", name.replace('\\', "/"));
        let model = match fetch_asset(&path).await {
            Ok(bytes) => Rsm::parse(&bytes, decode),
            Err(error) => Err(error),
        };
        (*name, model)
    }))
    .await;
    let mut read = BTreeMap::new();
    let mut failed: BTreeMap<String, usize> = BTreeMap::new();
    for (name, model) in fetched {
        match model {
            Ok(model) => {
                read.insert(name, model);
            }
            Err(error) => *failed.entry(error).or_default() += 1,
        }
    }
    for (error, count) in failed {
        web_sys::console::warn_1(&format!("{count} models left out: {error}").into());
    }

    // Per texture, the corners of all triangles drawn with it: three positions, two texture coordinates and a
    // colour each.
    let mut by_texture: BTreeMap<&str, Vec<f32>> = BTreeMap::new();
    for placement in &rsw.placements {
        let Some(model) = read.get(placement.model.as_str()) else { continue };
        for (texture, corners) in model::place(model, placement, map_size, &rsw.light) {
            let Some(name) = model.textures.get(texture) else { continue };
            let floats = by_texture.entry(name.as_str()).or_default();
            for corner in corners {
                floats.extend(corner.position);
                floats.extend(corner.uv);
                floats.extend(corner.colour);
            }
        }
    }
    let pictures = futures_util::future::join_all(by_texture.keys().map(|name| async move { (*name, picture(name).await) })).await;
    let layout = gpu.model_pipeline.get_bind_group_layout(1);
    let mut batches = Vec::new();
    for (name, loaded) in pictures {
        let view = match loaded {
            Ok(picture) => gpu.texture(picture.width, picture.height, &picture.rgba),
            Err(error) => {
                web_sys::console::warn_1(&format!("texture {name}: {error}").into());
                gpu.texture(1, 1, &[160, 160, 160, 255])
            }
        };
        let floats = &by_texture[name];
        let vertices = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(floats), usage: wgpu::BufferUsages::VERTEX });
        batches.push((vertices, (floats.len() / 8) as u32, gpu.textured(&layout, &view, &gpu.repeating)));
    }
    batches
}

/// The map's water: one flat surface over the whole map at the water's height. Where the ground is higher, the
/// ground hides it.
pub(super) async fn water(gpu: &Gpu, rsw: &Rsw, map_size: (f32, f32)) -> Option<WaterSurface> {
    let kind = rsw.water.kind;
    let loaded = futures_util::future::join_all((0..WATER_FRAMES).map(|frame| async move { picture(&format!("워터/water{kind}{frame:02}.jpg")).await })).await;
    let layout = gpu.water_pipeline.get_bind_group_layout(1);
    let frames: Vec<wgpu::BindGroup> = loaded
        .into_iter()
        .filter_map(|picture| picture.ok())
        .map(|picture| gpu.textured(&layout, &gpu.texture(picture.width, picture.height, &picture.rgba), &gpu.repeating))
        .collect();
    if frames.is_empty() {
        web_sys::console::warn_1(&format!("no pictures for water of kind {kind}").into());
        return None;
    }
    let level = rsw.water.level;
    let mut floats = Vec::with_capacity(30);
    for (x, z) in [(0.0, 0.0), (map_size.0, 0.0), (0.0, map_size.1), (0.0, map_size.1), (map_size.0, 0.0), (map_size.0, map_size.1)] {
        floats.extend([x, level, z, x / WATER_TILE, z / WATER_TILE]);
    }
    let vertices = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&floats), usage: wgpu::BufferUsages::VERTEX });
    Some(WaterSurface { vertices, frames, frame_ms: f64::from(rsw.water.frame_ticks) * 1000.0 / 60.0 })
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
            // The finger down: the last picture of that action, which the client runs to while the button is held.
            if let Ok(pressed) = figure::cursor(2, 2).await {
                let _ = root.style().set_property("--pressed", &pressed);
            }
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
const CAST_PICTURES: [&str; 14] = [
    "pikapika2.bmp",
    "ring_blue.tga",
    "ring_yellow.tga",
    "ring_white.tga",
    "magic_green.tga",
    "magic_violet.tga",
    "lockon128.tga",
    "icearrow.tga",
    "불화살1.tga",
    "불화살2.tga",
    "불화살3.tga",
    "불화살4.tga",
    "불화살5.tga",
    "불화살6.tga",
];
