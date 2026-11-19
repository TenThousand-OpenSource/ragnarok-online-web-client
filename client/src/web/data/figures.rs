//! How characters, monsters and NPCs are drawn, as the client's Lua says it: the size of the shadow under each,
//! and the order of what is worn on the head (the gateway's `tables/figures.rs`).

use super::*;

thread_local! {
    /// How many times its size the shadow under a job is drawn; 1 for one not named.
    static SHADOWS: RefCell<HashMap<u16, f32>> = RefCell::new(HashMap::new());
    /// The priority of what is worn low, on top and in the middle of the head when the table does not name
    /// it, and by view the priorities of those it names: for each of the three places, by direction.
    static USUAL: RefCell<[i32; 3]> = const { RefCell::new([300, 200, 100]) };
    static LAYERS: RefCell<HashMap<u16, [[i32; 8]; 3]>> = RefCell::new(HashMap::new());
}

/// How many times its own size the shadow under a job is drawn (`CSession::GetNpcLocationRadius`, which the
/// actors write to their `shadowScale`): a player's job, or the number a monster or an NPC is shown under.
pub fn shadow(job: u16) -> f32 {
    SHADOWS.with_borrow(|shadows| shadows.get(&job).copied().unwrap_or(1.0))
}

/// The priorities of what a character wears on its head (the views, as the packets list them: low, top,
/// middle) seen from a direction (`GetLayerPriority`): the lower is drawn first, -1 not at all. Nothing worn
/// in a place has -1.
pub fn head_layers(direction: usize, headgear: [u16; 3]) -> [i32; 3] {
    let usual = USUAL.with_borrow(|usual| *usual);
    LAYERS.with_borrow(|layers| std::array::from_fn(|slot| match headgear[slot] {
        0 => -1,
        view => layers.get(&view).map_or(usual[slot], |rows| rows[slot][direction % 8]),
    }))
}

/// Takes its tables out of what the gateway gave.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let figures = field(tables, "figures");
    SHADOWS.set(entries::<u16>(&field(&figures, "shadows")).into_iter().filter_map(|(job, factor)| Some((job, factor.as_f64()? as f32))).collect());
    let layers = field(&figures, "layers");
    let row = |row: wasm_bindgen::JsValue| -> Vec<i32> { list(&row).iter().filter_map(|number| number.as_f64()).map(|number| number as i32).collect() };
    if let Ok(usual) = <[i32; 3]>::try_from(row(field(&layers, "usual"))) {
        USUAL.set(usual);
    }
    let views = entries::<u16>(&field(&layers, "views")).into_iter().filter_map(|(view, rows)| {
        let rows: Vec<[i32; 8]> = list(&rows).into_iter().filter_map(|numbers| row(numbers).try_into().ok()).collect();
        Some((view, <[[i32; 8]; 3]>::try_from(rows).ok()?))
    });
    LAYERS.set(views.collect());
}
