//! The world map: the client's worlds, the picture of each, and where each map lies on it.

use super::*;

thread_local! {
    /// The worlds, in the order of the client's `World_List` (`worldviewdata_list.lub`).
    static WORLDS: RefCell<std::rc::Rc<Vec<World>>> = RefCell::new(std::rc::Rc::new(Vec::new()));
}

/// One world of the World Map window (the client's `CWorldViewMgr`).
#[derive(Default)]
pub struct World {
    pub name: String,
    /// The picture, a file of the interface's, and its size: the rectangles are in its points.
    pub picture: String,
    pub size: (f32, f32),
    pub fields: Vec<Field>,
    /// The dungeons: the ID their floors share, the rectangle of the mark, the name and the levels.
    pub groups: Vec<(u32, [f32; 4], String, String)>,
}

/// One map on a world's picture: the ID (under 100 for a floor of a dungeon), the map, its rectangle (left,
/// top, right, bottom), what the client calls it and the level it is for.
pub struct Field {
    pub id: u32,
    pub map: String,
    pub at: [f32; 4],
    pub name: String,
    pub level: String,
}

/// The client's worlds.
pub fn worlds() -> std::rc::Rc<Vec<World>> {
    WORLDS.with_borrow(Clone::clone)
}

/// Takes the worlds out of what the gateway gave (`tables/worldmap.rs`).
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let list = |of: wasm_bindgen::JsValue| of.dyn_into::<js_sys::Array>().map(|list| list.to_vec()).unwrap_or_default();
    let number = |row: &js_sys::Array, at: u32| row.get(at).as_f64().unwrap_or(0.0) as f32;
    let text = |row: &js_sys::Array, at: u32| row.get(at).as_string().unwrap_or_default();
    let worlds = list(field(tables, "worldmap"))
        .into_iter()
        .map(|world| {
            let size = js_sys::Array::from(&field(&world, "size"));
            World {
                name: text_of(&world, "name").unwrap_or_default(),
                picture: text_of(&world, "picture").unwrap_or_default(),
                size: (number(&size, 0), number(&size, 1)),
                fields: list(field(&world, "fields"))
                    .into_iter()
                    .map(|row| js_sys::Array::from(&row))
                    .map(|row| Field { id: number(&row, 0) as u32, map: text(&row, 1), at: [number(&row, 2), number(&row, 3), number(&row, 4), number(&row, 5)], name: text(&row, 6), level: text(&row, 7) })
                    .collect(),
                groups: list(field(&world, "groups"))
                    .into_iter()
                    .map(|row| js_sys::Array::from(&row))
                    .map(|row| (number(&row, 0) as u32, [number(&row, 1), number(&row, 2), number(&row, 3), number(&row, 4)], text(&row, 5), text(&row, 6)))
                    .collect(),
            }
        })
        .collect();
    WORLDS.set(std::rc::Rc::new(worlds));
}
