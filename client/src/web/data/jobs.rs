//! The jobs and what is drawn for them: names, body sprites, headgear, weapons, and the sprites of NPCs and monsters.

use super::*;


thread_local! {
    /// From the gateway's `/tables.json`, read out of the client's Lua files: job names, and headgear sprite names
    /// by view ID.
    static JOB_NAMES: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    static HEADGEAR: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// The sprite files of NPCs and monsters, by the number the server shows them under.
    static SPRITES: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// The job each job grew out of (the client's `jobinheritlist.lub`).
    static INHERITS: RefCell<HashMap<u16, u16>> = RefCell::new(HashMap::new());
    /// The ending of a weapon's sprite files, by the client's number for the kind of weapon.
    static WEAPONS: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// The sound of a weapon's blow, by the client's number for the kind of weapon.
    static WEAPON_HITS: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// Whether the client's files are set up for Korea (its `clientinfo.xml`); so they are until the gateway
    /// says otherwise, as the client's own `g_serviceType` starts.
    static KOREA: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Whether the client is set up for Korea: its table of the clothes' palettes differs by it
/// (`crate::dress::palette`).
pub fn korea() -> bool {
    KOREA.get()
}

/// The ending of the sprite files of a weapon (`_단검` for a dagger), by the client's number for its kind
/// (its `Weapon_IDs`; `crate::jobs::weapon_kind` gives it for what the server's database calls the kind).
pub fn weapon_sprite_of(number: u16) -> Option<String> {
    WEAPONS.with_borrow(|weapons| weapons.get(&number).cloned())
}

/// The sound a weapon of that kind makes as its blow lands, by the client's number for the kind (its
/// `WeaponHitWaveNameTable`, asked through `ReqWeaponHitWaveName`).
pub fn weapon_hit(kind: u16) -> Option<String> {
    WEAPON_HITS.with_borrow(|hits| hits.get(&kind).cloned())
}

/// The name of the sprite files of an NPC or a monster.
pub fn sprite(job: u16) -> Option<String> {
    SPRITES.with_borrow(|sprites| sprites.get(&job).cloned())
}

/// What a job is called.
pub fn job_name(job: u16) -> String {
    JOB_NAMES.with_borrow(|names| names.get(&job).cloned()).unwrap_or_else(|| crate::jobs::name(job))
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
    HEADGEAR.set(names::<u16>(&field(tables, "headgear")).into_iter().collect());
    SPRITES.set(names::<u16>(&field(tables, "sprites")).into_iter().collect());
    INHERITS.set(entries::<u16>(&field(tables, "inherits")).into_iter().filter_map(|(job, from)| Some((job, from.as_f64()? as u16))).collect());
    WEAPONS.set(names::<u16>(&field(tables, "weapons")).into_iter().collect());
    WEAPON_HITS.set(names::<u16>(&field(tables, "weapon_hits")).into_iter().collect());
    KOREA.set(field(tables, "korea").as_bool().unwrap_or(true));
}

/// The first job a job grew out of, among the classic ones (`crate::jobs::first_job`), each job grown out of
/// the one the client's `jobinheritlist.lub` names for it.
pub fn first_job(job: u16) -> Option<u16> {
    crate::jobs::first_job(job, |job| INHERITS.with_borrow(|inherits| inherits.get(&job).copied()))
}
