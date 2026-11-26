//! The battlegrounds one can queue for: the client's `Lua Files\EntryQueue\EntryQueueList.lub`, read as the
//! client reads it (`CEntryQueueMgr`'s constructor calls the file's own `ReadEntryQueueList`, which hands
//! every battleground to the client's `AddEntryQueue`: rodecomp's `Source/Script/LuaScript.cpp`).

use super::*;

/// Stands in for the client's `AddEntryQueue`: keeps what the file hands it, in the file's order.
const RECORDER: &str = "Recorded = {} function AddEntryQueue(...) Recorded[#Recorded + 1] = { ... } end";

/// What the file's `ReadEntryQueueList` hands to `AddEntryQueue`, call by call, each as the seventeen things
/// the client takes: nothing if the file has no such function.
fn handed(lua: &Lua) -> Vec<Vec<Value>> {
    if exec(lua, b"ReadEntryQueueList()", Some("ReadEntryQueueList")).is_err() {
        return Vec::new();
    }
    let Ok(recorded) = lua.globals().get::<Table>("Recorded") else { return Vec::new() };
    recorded.sequence_values::<Table>().flatten().map(|call| call.sequence_values::<Value>().flatten().collect()).collect()
}

/// One call of `AddEntryQueue` as the browser is given it, the things named for what the client's window does
/// with them (`UIEntryQueueWnd::SelectQueue`): the number, the name the server knows the battleground by, the
/// name shown, how many fight on each side, which of the three ways to apply the file allows, the kind of the
/// level's bounds (1 up to, 2 from, 3 between, else none) with the bounds, what is won, drawn and lost, the
/// goal, and the picture.
fn entry(call: &[Value]) -> Option<serde_json::Value> {
    let whole = |place: usize| call.get(place).and_then(whole).unwrap_or(0);
    let words = |place: usize| call.get(place).and_then(text_only).unwrap_or_default();
    (call.len() >= 17).then(|| {
        serde_json::json!({
            "id": whole(0), "name": words(1), "shown": words(2), "allies": whole(3), "enemies": whole(4),
            "alone": whole(5) != 0, "party": whole(6) != 0, "guild": whole(7) != 0, "jobs": whole(8),
            "level_kind": whole(9), "level": whole(10), "level_end": whole(11),
            "win": words(12), "draw": words(13), "loss": words(14), "goal": words(15), "picture": words(16)
        })
    })
}

/// The battlegrounds of a state that ran the file after `RECORDER`, in the order of their numbers (the client
/// keeps them in a map by number).
fn listed(lua: &Lua) -> Vec<serde_json::Value> {
    let mut entries: Vec<serde_json::Value> = handed(lua).iter().filter_map(|call| entry(call)).collect();
    entries.sort_by_key(|entry| entry["id"].as_i64());
    entries
}

/// `[{"id": 1, "name": …, "shown": …, "allies": 6, "enemies": 6, …}, …]` out of the archives.
pub fn battleground(read: &Sources) -> serde_json::Value {
    let lua = state();
    if let Err(error) = exec(&lua, RECORDER.as_bytes(), Some("recorder")) {
        eprintln!("tables: battleground: {error}");
    }
    read.run_in(&lua, &["entryqueue\\entryqueuelist"]);
    listed(&lua).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file of the client's shape: a table, and the function that hands its rows on.
    const FILE: &str = r#"
        EntryQueueList = {
            [2] = { "KVM", "KVM (Level 60~79)", 5, 5, 1, 1, 0, 0, 3, 60, 79, "2 Points", "1 Point", "none", "Annihilate.", "bat_c01.bmp" },
            [1] = { "Tierra Gorge", "Tierra", 6, 6, 1, 1, 1, 4, 2, 80, 0, "3 Badges", "1 Badge", "1 Badge", "Storehouse.", "bat_a01.bmp" },
        }
        function ReadEntryQueueList()
            for id, row in pairs(EntryQueueList) do
                AddEntryQueue(id, row[1], row[2], row[3], row[4], row[5], row[6], row[7], row[8], row[9], row[10], row[11],
                    row[12], row[13], row[14], row[15], row[16])
            end
        end
    "#;

    #[test]
    fn the_battlegrounds_are_what_the_file_hands_to_the_client_in_the_order_of_their_numbers() {
        let lua = state();
        exec(&lua, RECORDER.as_bytes(), None).unwrap();
        exec(&lua, FILE.as_bytes(), None).unwrap();
        let entries = listed(&lua);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["name"], "Tierra Gorge");
        assert_eq!(entries[0]["shown"], "Tierra");
        assert_eq!((entries[0]["allies"].as_i64(), entries[0]["guild"].as_bool()), (Some(6), Some(true)));
        assert_eq!((entries[0]["level_kind"].as_i64(), entries[0]["level"].as_i64()), (Some(2), Some(80)));
        assert_eq!(entries[1]["guild"], false);
        assert_eq!((entries[1]["level"].as_i64(), entries[1]["level_end"].as_i64()), (Some(60), Some(79)));
        assert_eq!((entries[1]["win"].as_str(), entries[1]["goal"].as_str()), (Some("2 Points"), Some("Annihilate.")));
        assert_eq!(entries[1]["picture"], "bat_c01.bmp");
    }

    #[test]
    fn a_file_without_the_function_gives_no_battlegrounds() {
        let lua = state();
        exec(&lua, RECORDER.as_bytes(), None).unwrap();
        assert!(listed(&lua).is_empty());
    }
}
