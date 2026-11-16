//! The game's own tables: what a job is called and how its sprite files are named, and which sprite a headgear is.

use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use super::js_error;
use crate::net::packets;

/// The Doram's jobs; every other player job is human. (rodecomp, `CSession::IsDoramJob`.)
const DORAM_JOBS: [u16; 7] = [4217, 4218, 4219, 4220, 4221, 4308, 4315];
/// The job a new Doram character starts as.
pub const SUMMONER: u16 = 4218;

thread_local! {
    /// Body sprite names by sprite job, from the list taken out of the original client.
    static BODIES: HashMap<u16, &'static str> = include_str!("job_sprites.txt")
        .lines()
        .filter_map(|line| {
            let (job, name) = line.split_once('\t')?;
            Some((job.parse().ok()?, name))
        })
        .collect();
    /// From the gateway's `/tables.json`, read out of the client's Lua files: job names, and headgear sprite names
    /// by view ID.
    static JOB_NAMES: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    static HEADGEAR: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// Skills by ID: the name of their pictures, what they are called, and their highest level.
    static SKILLS: RefCell<HashMap<u16, (String, String, u16)>> = RefCell::new(HashMap::new());
    /// Items by ID, as far as they were asked for: what they are called, and the name of their pictures.
    static ITEMS: RefCell<HashMap<u32, (String, String)>> = RefCell::new(HashMap::new());
    /// Items asked for and not answered yet (or not known to the gateway at all).
    static ITEMS_ASKED: RefCell<std::collections::HashSet<u32>> = RefCell::new(std::collections::HashSet::new());
}

/// A skill's picture name, what it is called, and its highest level.
pub fn skill(id: u16) -> Option<(String, String, u16)> {
    SKILLS.with_borrow(|skills| skills.get(&id).cloned())
}

/// What an item is called and the name of its pictures, once the gateway has told (see `want_items`).
pub fn item(id: u32) -> Option<(String, String)> {
    ITEMS.with_borrow(|items| items.get(&id).cloned())
}

/// Asks the gateway about the items not known yet, and calls `then` when it has answered.
pub fn want_items(ids: impl Iterator<Item = u32>, then: fn()) {
    let missing: Vec<u32> = ids.filter(|id| *id != 0 && !ITEMS.with_borrow(|items| items.contains_key(id)) && ITEMS_ASKED.with_borrow_mut(|asked| asked.insert(*id))).collect();
    if missing.is_empty() {
        return;
    }
    wasm_bindgen_futures::spawn_local(async move {
        let list: Vec<String> = missing.iter().map(u32::to_string).collect();
        match fetch_json(&format!("/tables/items/{}", list.join(","))).await {
            Ok(answer) => {
                let found: Vec<(u32, (String, String))> = entries(&answer)
                    .into_iter()
                    .filter_map(|(id, about)| Some((id, (text_of(&about, "name")?, text_of(&about, "resource")?))))
                    .collect();
                ITEMS.with_borrow_mut(|items| items.extend(found));
                then();
            }
            Err(error) => web_sys::console::warn_1(&error.into()),
        }
    });
}

async fn fetch_json(url: &str) -> Result<wasm_bindgen::JsValue, String> {
    let window = web_sys::window().ok_or("no window")?;
    let response: web_sys::Response = JsFuture::from(window.fetch_with_str(url)).await.map_err(js_error)?.dyn_into().map_err(js_error)?;
    if !response.ok() {
        return Err(format!("{url}: HTTP {}", response.status()));
    }
    JsFuture::from(response.json().map_err(js_error)?).await.map_err(js_error)
}

/// A JSON object's entries whose keys are numbers.
fn entries<Id: std::str::FromStr>(object: &wasm_bindgen::JsValue) -> Vec<(Id, wasm_bindgen::JsValue)> {
    let Ok(object) = object.clone().dyn_into::<js_sys::Object>() else { return Vec::new() };
    js_sys::Object::entries(&object)
        .iter()
        .filter_map(|entry| {
            let entry: js_sys::Array = entry.dyn_into().ok()?;
            Some((entry.get(0).as_string()?.parse().ok()?, entry.get(1)))
        })
        .collect()
}

fn text_of(object: &wasm_bindgen::JsValue, field: &str) -> Option<String> {
    js_sys::Reflect::get(object, &field.into()).ok()?.as_string()
}

pub fn is_doram(job: u16) -> bool {
    DORAM_JOBS.contains(&job)
}

/// The name of a job's body sprite files. A job the client has no sprite for is drawn as a Novice.
pub fn body_sprite(job: u16) -> &'static str {
    // The sprites are numbered as the jobs, the later jobs moved down to follow the first ones.
    let sprite_job = if job > 3950 { job - 3950 } else { job };
    BODIES.with(|bodies| bodies.get(&sprite_job).or_else(|| bodies.get(&0)).copied().unwrap_or("초보자"))
}

/// What a job is called.
pub fn job_name(job: u16) -> String {
    JOB_NAMES.with_borrow(|names| names.get(&job).cloned()).unwrap_or_else(|| packets::job_name(job))
}

/// The name a headgear's sprite files end in, by view ID.
pub fn headgear(view: u16) -> Option<String> {
    if view == 0 {
        return None;
    }
    HEADGEAR.with_borrow(|names| names.get(&view).cloned())
}

/// Fetches the tables from the gateway. Until they are there, jobs have their built-in names and nobody wears
/// headgear.
pub async fn load() -> Result<(), String> {
    // The number changes when the tables gain something, so a browser's older copy is not used in their place.
    let tables = fetch_json("/tables.json?v=2").await?;
    let table = |name: &str| js_sys::Reflect::get(&tables, &name.into()).unwrap_or(wasm_bindgen::JsValue::UNDEFINED);
    let jobs = entries::<u16>(&table("jobs")).into_iter().filter_map(|(job, about)| Some((job, text_of(&about, "name")?))).collect();
    let headgear = entries::<u16>(&table("headgear")).into_iter().filter_map(|(view, name)| Some((view, name.as_string()?))).collect();
    let skills = entries::<u16>(&table("skills"))
        .into_iter()
        .filter_map(|(id, about)| {
            let max = js_sys::Reflect::get(&about, &"max".into()).ok().and_then(|max| max.as_f64()).unwrap_or(1.0) as u16;
            Some((id, (text_of(&about, "resource")?, text_of(&about, "name")?, max)))
        })
        .collect();
    JOB_NAMES.set(jobs);
    HEADGEAR.set(headgear);
    SKILLS.set(skills);
    Ok(())
}
