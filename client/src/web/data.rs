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
    /// The sprite files of NPCs and monsters, by the number the server shows them under.
    static SPRITES: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// The music of each map, as the client's `mp3nametable.txt` names it: `prontera` to `bgm\\08.mp3`.
    static MUSIC: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    /// The ending of a weapon's sprite files, by the client's number for the kind of weapon.
    static WEAPONS: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// By job: the job it grew out of, and where its skills stand in the skill window (place to skill).
    static TREE: RefCell<HashMap<u16, (Option<u16>, Vec<(u16, u16)>)>> = RefCell::new(HashMap::new());
    /// What the client says of each skill, as far as it was asked for (`None` once the gateway said it has
    /// nothing).
    static SKILL_TEXTS: RefCell<HashMap<u16, Option<Vec<String>>>> = RefCell::new(HashMap::new());
    /// Skills by ID: the name of their pictures, what they are called, and their highest level.
    static SKILLS: RefCell<HashMap<u16, (String, String, u16)>> = RefCell::new(HashMap::new());
    /// Items by ID, as far as they were asked for: what they are called, and the name of their pictures.
    static ITEMS: RefCell<HashMap<u32, (String, String)>> = RefCell::new(HashMap::new());
    /// What more is told of an item looked at: the client's description, line by line, and what the server's
    /// database says of it.
    static ABOUT: RefCell<HashMap<u32, (Vec<String>, Vec<(String, String)>)>> = RefCell::new(HashMap::new());
    /// What the translated skills are called in Russian, by ID.
    static SKILL_NAMES_RU: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// Items asked for and not answered yet (or not known to the gateway at all).
    static ITEMS_ASKED: RefCell<std::collections::HashSet<u32>> = RefCell::new(std::collections::HashSet::new());
}

/// The ending of the sprite files of a weapon (`_단검` for a dagger), by what the server's database calls its
/// kind (the item's `SubType`). The numbers are the client's `Weapon_IDs`.
pub fn weapon_sprite(kind: &str) -> Option<String> {
    let number = match kind {
        "Dagger" => 1,
        "1hSword" => 2,
        "2hSword" => 3,
        "1hSpear" => 4,
        "2hSpear" => 5,
        "1hAxe" => 6,
        "2hAxe" => 7,
        "Mace" => 8,
        "2hMace" => 9,
        "Staff" => 10,
        "Bow" => 11,
        "Knuckle" => 12,
        "Musical" => 13,
        "Whip" => 14,
        "Book" => 15,
        "Katar" => 16,
        "Revolver" => 17,
        "Rifle" => 18,
        "Gatling" => 19,
        "Shotgun" => 20,
        "Grenade" => 21,
        "Huuma" => 22,
        "2hStaff" => 23,
        _ => return None,
    };
    WEAPONS.with_borrow(|weapons| weapons.get(&number).cloned())
}

/// The name of the sprite files of an NPC or a monster.
pub fn sprite(job: u16) -> Option<String> {
    SPRITES.with_borrow(|sprites| sprites.get(&job).cloned())
}

/// The music file of a map, if the client's table has one for it.
pub fn music(map: &str) -> Option<String> {
    MUSIC.with_borrow(|music| music.get(&map.to_ascii_lowercase()).cloned())
}

/// Reads the client's table of which music plays on which map: lines of `map.rsw#bgm\\08.mp3#`.
pub async fn load_music() {
    let Ok(bytes) = super::fetch_asset("data/mp3nametable.txt").await else { return };
    let table = String::from_utf8_lossy(&bytes);
    let music = table
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .filter_map(|line| {
            let mut parts = line.split('#');
            let (map, track) = (parts.next()?.trim(), parts.next()?.trim());
            Some((map.to_ascii_lowercase().strip_suffix(".rsw")?.to_owned(), track.replace("\\\\", "\\")))
        })
        .filter(|(_, track)| !track.is_empty())
        .collect();
    MUSIC.set(music);
}

/// The pages of a job's skill window, as the client lays them out (`skilltreeview.lub`): one for each job it
/// grew through, the first with the Novice's skills among its own. Each page: the place of each skill (counting
/// the cells row by row, seven to a row) and the skill.
pub fn skill_pages(job: u16) -> Vec<Vec<(u16, u16)>> {
    TREE.with_borrow(|tree| {
        // From the job back to the Novice.
        let mut line = Vec::new();
        let mut at = Some(job);
        while let Some(job) = at.filter(|job| !line.contains(job) && line.len() < 8) {
            line.push(job);
            at = tree.get(&job).and_then(|(from, _)| *from).filter(|from| *from != job);
        }
        line.reverse();
        let mut pages: Vec<Vec<(u16, u16)>> = Vec::new();
        for (index, job) in line.iter().enumerate() {
            let slots = tree.get(job).map(|(_, slots)| slots.clone()).unwrap_or_default();
            // The Novice's page is the first job's too.
            if index == 1 && line[0] == 0 {
                pages[0].extend(slots);
            } else {
                pages.push(slots);
            }
        }
        pages
    })
}

/// The client's description of a skill, line by line, once the gateway has told it (see `want_skill`).
pub fn skill_text(id: u16) -> Option<Vec<String>> {
    SKILL_TEXTS.with_borrow(|texts| texts.get(&id).cloned().flatten())
}

/// Asks the gateway for a skill's description, unless it is known; `then` is called when it has come.
pub fn want_skill(id: u16, then: fn()) {
    if SKILL_TEXTS.with_borrow(|texts| texts.contains_key(&id)) {
        return;
    }
    SKILL_TEXTS.with_borrow_mut(|texts| texts.insert(id, None));
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(answer) = fetch_json(&format!("/tables/{}skills/{id}", if russian() { "ru/" } else { "" })).await {
            let lines = js_sys::Reflect::get(&answer, &id.to_string().into()).ok().filter(|lines| !lines.is_undefined()).map(|lines| js_sys::Array::from(&lines).iter().filter_map(|line| line.as_string()).collect());
            SKILL_TEXTS.with_borrow_mut(|texts| texts.insert(id, lines));
            then();
        }
    });
}

/// A skill's picture name, what it is called, and its highest level.
pub fn skill(id: u16) -> Option<(String, String, u16)> {
    let (resource, name, most) = SKILLS.with_borrow(|skills| skills.get(&id).cloned())?;
    // In Russian, a skill that has a translation goes by it.
    let name = if russian() { SKILL_NAMES_RU.with_borrow(|names| names.get(&id).cloned()).unwrap_or(name) } else { name };
    Some((resource, name, most))
}

/// Whether the game speaks Russian now: the items' and skills' texts are then asked for in Russian.
fn russian() -> bool {
    crate::i18n::spoken() == crate::i18n::Lang::Ru
}

/// Forgets the items' and skills' texts: after the language changed, they are asked for anew in the new one.
pub fn forget_texts() {
    ITEMS.with_borrow_mut(HashMap::clear);
    ABOUT.with_borrow_mut(HashMap::clear);
    ITEMS_ASKED.with_borrow_mut(std::collections::HashSet::clear);
    SKILL_TEXTS.with_borrow_mut(HashMap::clear);
}

/// What an item is called and the name of its pictures, once the gateway has told (see `want_items`).
pub fn item(id: u32) -> Option<(String, String)> {
    ITEMS.with_borrow(|items| items.get(&id).cloned())
}

/// An item's description and facts, once the gateway has told of the item.
pub fn about(id: u32) -> Option<(Vec<String>, Vec<(String, String)>)> {
    ABOUT.with_borrow(|about| about.get(&id).cloned())
}

/// Asks the gateway about the items not known yet, and calls `then` when it has answered.
pub fn want_items(ids: impl Iterator<Item = u32>, then: fn()) {
    let missing: Vec<u32> = ids.filter(|id| *id != 0 && !ITEMS.with_borrow(|items| items.contains_key(id)) && ITEMS_ASKED.with_borrow_mut(|asked| asked.insert(*id))).collect();
    if missing.is_empty() {
        return;
    }
    wasm_bindgen_futures::spawn_local(async move {
        let list: Vec<String> = missing.iter().map(u32::to_string).collect();
        match fetch_json(&format!("/tables/{}items/{}", if russian() { "ru/" } else { "" }, list.join(","))).await {
            Ok(answer) => {
                let found: Vec<(u32, (String, String))> = entries(&answer)
                    .into_iter()
                    .filter_map(|(id, about)| Some((id, (text_of(&about, "name")?, text_of(&about, "resource")?))))
                    .collect();
                let strings = |value: wasm_bindgen::JsValue| js_sys::Array::from(&value).iter().filter_map(|line| line.as_string()).collect::<Vec<String>>();
                let told: Vec<(u32, (Vec<String>, Vec<(String, String)>))> = entries::<u32>(&answer)
                    .into_iter()
                    .map(|(id, about)| {
                        let field = |name: &str| js_sys::Reflect::get(&about, &name.into()).unwrap_or(wasm_bindgen::JsValue::UNDEFINED);
                        let facts = js_sys::Array::from(&field("facts")).iter().filter_map(|fact| <[String; 2]>::try_from(strings(fact)).ok()).map(|[key, value]| (key, value)).collect();
                        (id, (strings(field("description")), facts))
                    })
                    .collect();
                ABOUT.with_borrow_mut(|about| about.extend(told));
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
    let tables = fetch_json("/tables.json?v=5").await?;
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
    SPRITES.set(entries::<u16>(&table("sprites")).into_iter().filter_map(|(job, name)| Some((job, name.as_string()?))).collect());
    WEAPONS.set(entries::<u16>(&table("weapons")).into_iter().filter_map(|(kind, name)| Some((kind, name.as_string()?))).collect());
    TREE.set(
        entries::<u16>(&table("tree"))
            .into_iter()
            .map(|(job, about)| {
                let from = js_sys::Reflect::get(&about, &"from".into()).ok().and_then(|from| from.as_f64()).map(|from| from as u16);
                let slots = js_sys::Reflect::get(&about, &"slots".into()).map(|slots| entries::<u16>(&slots)).unwrap_or_default();
                (job, (from, slots.into_iter().filter_map(|(place, skill)| Some((place, skill.as_f64()? as u16))).collect()))
            })
            .collect(),
    );
    if let Ok(names) = fetch_json("/tables/ru/skill-names.json").await {
        SKILL_NAMES_RU.set(entries::<u16>(&names).into_iter().filter_map(|(id, name)| Some((id, name.as_string()?))).collect());
    }
    JOB_NAMES.set(jobs);
    HEADGEAR.set(headgear);
    SKILLS.set(skills);
    Ok(())
}
