//! The pets: their pictures, the actions of those that wear something, and what they say.

use super::*;

thread_local! {
    /// The picture of each pet in its window, by the monster's number.
    static PETS: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// The client's table of what pets say, once asked for and read; empty while it is on its way or missing.
    static PET_TALK: RefCell<Option<String>> = const { RefCell::new(None) };
    /// By the accessory a pet wears: the file of the actions it is then drawn with, without its ending.
    static PET_ACTS: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
}

/// The file of a pet's picture in its window (in the interface's `illust` folder), by the monster's number.
pub fn pet_picture(job: u16) -> Option<String> {
    PETS.with_borrow(|pets| pets.get(&job).cloned())
}

/// The actions a pet wearing that accessory is drawn with (the client's `GetPetAccActName`): a file among the
/// monsters' sprites, named without its ending.
pub fn pet_act(accessory: u16) -> Option<String> {
    PET_ACTS.with_borrow(|acts| acts.get(&accessory).cloned())
}

/// Fetches the client's table of what pets say, once: wanted only by one whose pet is close enough to talk.
pub fn want_pet_talk() {
    if PET_TALK.with_borrow(|talk| talk.is_some()) {
        return;
    }
    PET_TALK.set(Some(String::new()));
    wasm_bindgen_futures::spawn_local(async {
        if let Ok(bytes) = crate::web::fetch_asset("data/pettalktable.xml").await {
            // The file says of itself `encoding="euc-kr"`: the lines of the pets that have no English ones are
            // Korean.
            PET_TALK.set(Some(crate::web::korean(&bytes)));
        }
    });
}

/// One of the lines a pet of that kind may say at a moment, in a state of hunger; none while the table is not
/// read, or has nothing for it. The client finds a monster's lines by a name it has for each (compiled into
/// it); here it is the name of the monster's sprite in small letters, which is what those names are.
pub fn pet_line(job: u16, hunger: usize, moment: i32) -> Option<String> {
    let monster = sprite(job)?.to_lowercase();
    PET_TALK.with_borrow(|talk| {
        let lines = crate::pets::lines(talk.as_deref()?, &monster, crate::pets::hunger_tag(hunger), crate::pets::moment_tag(moment));
        let pick = (js_sys::Math::random() * lines.len() as f64) as usize;
        lines.get(pick).map(|line| (*line).to_owned())
    })
}

/// Takes its tables out of what the gateway gave: the pets' pictures, and the actions of pets that wear something.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    PETS.set(names::<u16>(&field(tables, "pets")).into_iter().collect());
    PET_ACTS.set(entries::<u16>(&field(tables, "pet_acts")).into_iter().filter_map(|(item, name)| Some((item, name.as_string()?.trim_end_matches(".act").to_owned()))).collect());
}
