//! The jobs and what is drawn for them: names, body sprites, headgear, weapons, and the sprites of NPCs and monsters.

use super::*;

use crate::net::packets;

/// The Doram's jobs; every other player job is human. (rodecomp, `CSession::IsDoramJob`.)
const DORAM_JOBS: [u16; 7] = [4217, 4218, 4219, 4220, 4221, 4308, 4315];
/// The job a new Doram character starts as.
pub const SUMMONER: u16 = 4218;

thread_local! {
    /// Body sprite names by sprite job, from the list taken out of the original client.
    static BODIES: HashMap<u16, &'static str> = crate::tables::tabbed(include_str!("job_sprites.txt")).filter_map(|(job, name)| Some((job.parse().ok()?, name))).collect();
    /// From the gateway's `/tables.json`, read out of the client's Lua files: job names, and headgear sprite names
    /// by view ID.
    static JOB_NAMES: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    static HEADGEAR: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// The sprite files of NPCs and monsters, by the number the server shows them under.
    static SPRITES: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// The plain job of a mounted or re-dressed job, from the list taken out of the original client.
    static BASES: HashMap<u16, u16> = crate::tables::tabbed(include_str!("job_bases.txt")).filter_map(|(job, base)| Some((job.parse().ok()?, base.parse().ok()?))).collect();
    /// The job each job grew out of (the client's `jobinheritlist.lub`).
    static INHERITS: RefCell<HashMap<u16, u16>> = RefCell::new(HashMap::new());
    /// The ending of a weapon's sprite files, by the client's number for the kind of weapon.
    static WEAPONS: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// The sound of a weapon's blow, by the client's number for the kind of weapon.
    static WEAPON_HITS: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
}

/// The ending of the sprite files of a weapon (`_단검` for a dagger), by the client's number for its kind
/// (its `Weapon_IDs`; `weapon_kind` gives it for what the server's database calls the kind).
pub fn weapon_sprite_of(number: u16) -> Option<String> {
    WEAPONS.with_borrow(|weapons| weapons.get(&number).cloned())
}

/// The sound a weapon of that kind makes as its blow lands, by the client's number for the kind (its
/// `WeaponHitWaveNameTable`, asked through `ReqWeaponHitWaveName`).
pub fn weapon_hit(kind: u16) -> Option<String> {
    WEAPON_HITS.with_borrow(|hits| hits.get(&kind).cloned())
}

/// The client's number for a kind of weapon (its `Weapon_IDs`), by what the server's database calls the kind
/// (the item's `SubType`).
pub fn weapon_kind(kind: &str) -> Option<u16> {
    Some(match kind {
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
    })
}

/// The name of the sprite files of an NPC or a monster.
pub fn sprite(job: u16) -> Option<String> {
    SPRITES.with_borrow(|sprites| sprites.get(&job).cloned())
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

/// Takes its tables out of what the gateway gave: the jobs' names, the headgear, the sprites of NPCs and
/// monsters, the weapons.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    JOB_NAMES.set(entries::<u16>(&field(tables, "jobs")).into_iter().filter_map(|(job, about)| Some((job, text_of(&about, "name")?))).collect());
    HEADGEAR.set(entries::<u16>(&field(tables, "headgear")).into_iter().filter_map(|(view, name)| Some((view, name.as_string()?))).collect());
    SPRITES.set(entries::<u16>(&field(tables, "sprites")).into_iter().filter_map(|(job, name)| Some((job, name.as_string()?))).collect());
    INHERITS.set(entries::<u16>(&field(tables, "inherits")).into_iter().filter_map(|(job, from)| Some((job, from.as_f64()? as u16))).collect());
    WEAPONS.set(entries::<u16>(&field(tables, "weapons")).into_iter().filter_map(|(kind, name)| Some((kind, name.as_string()?))).collect());
    WEAPON_HITS.set(entries::<u16>(&field(tables, "weapon_hits")).into_iter().filter_map(|(kind, name)| Some((kind, name.as_string()?))).collect());
}

/// The first job a job grew out of, among the classic ones (0 novice, 1 swordman, 2 magician, 3 archer,
/// 4 acolyte, 5 merchant, 6 thief), by the client's own tables: a mounted job is its plain job
/// (`JobNames::BaseJob`), and each job grew out of the one `jobinheritlist.lub` names for it. The jobs that
/// table starts from a Novice (Taekwon, Ninja, Gunslinger, Super Novice) are of the Novice's line; a job it does
/// not name at all (a Doram) has none.
pub fn first_job(job: u16) -> Option<u16> {
    let mut job = BASES.with(|bases| bases.get(&job).copied()).unwrap_or(job);
    // No line of jobs is ten long: a table gone wrong does not hold this up for ever.
    for _ in 0..10 {
        if job <= 6 {
            return Some(job);
        }
        job = INHERITS.with_borrow(|inherits| inherits.get(&job).copied())?;
    }
    None
}
