//! The pets: the picture of each in its window, and the actions one is drawn with by what it wears.

use super::*;

type Names = serde_json::Map<String, serde_json::Value>;

/// The pets' pictures by the monster's number and their actions by the accessory worn, out of `petinfo.lub`
/// run in the state the jobs' files left (`jobs::jobs`), as the client runs it.
pub(super) fn pets(lua: &Lua, grf: &Grf) -> (Names, Names) {
    // The picture of each pet in its window (a file of `유저인터페이스\illust`), by the monster's number: from
    // `petinfo.lub`, whose function `GetPetIllustName_V3` the client asks.
    run_in(lua, grf, &["petinfo"]);
    let pets: serde_json::Map<String, serde_json::Value> =
        names(lua, "PetIllustNameTable").into_iter().filter(|(id, _)| *id >= 0).map(|(id, name)| (id.to_string(), name.into())).collect();
    // Its `PetAccActNameTable`: by the accessory a pet wears (an item), the actions it is then drawn with.
    let pet_acts: serde_json::Map<String, serde_json::Value> = names(lua, "PetAccActNameTable").into_iter().filter(|(id, _)| *id >= 0).map(|(id, name)| (id.to_string(), name.into())).collect();
    (pets, pet_acts)
}
