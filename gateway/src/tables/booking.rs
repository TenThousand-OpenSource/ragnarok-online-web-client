//! The old party booking's list of places and its help: the client's `Lua Files\seekparty\
//! party_booking_config.lub` (the regions with their maps, and the help's lines) asked through the functions
//! of `party_booking_function.lub`, as the client's windows ask them (`UISeekPartyListWnd::InitRegionFilter`:
//! `queryRegionInfo`, `queryMapInfo`; `UIPartyBookingHelpWnd::OnCreate`: `GetPartyBookingHelp`). The config
//! names its places by the client's names for them (`MsgStrID.…`), which `Lua Files\msgstring_kr.lub` gives
//! their words.

use super::*;

/// The files, in the order they need one another.
const FILES: [&str; 3] = ["\\msgstring_kr", "seekparty\\party_booking_config", "seekparty\\party_booking_function"];

/// What a function of the files answers when called with those numbers; nothing if it is not there or fails.
fn asked(lua: &Lua, function: &str, numbers: &[i64]) -> Vec<Value> {
    let Ok(function) = lua.globals().get::<mlua::Function>(function) else { return Vec::new() };
    let numbers: mlua::Variadic<i64> = numbers.iter().copied().collect();
    function.call::<mlua::MultiValue>(numbers).map(|values| values.into_iter().collect()).unwrap_or_default()
}

/// The maps of a region, each with the colour its name is written in: `queryMapInfo(region, index)` from 1
/// until it names none (it answers a number, a level, the name, and the colour's red, green and blue).
fn maps(lua: &Lua, region: i64) -> Vec<serde_json::Value> {
    (1..=255)
        .map_while(|index| {
            let answer = asked(lua, "queryMapInfo", &[region, index]);
            let name = answer.get(2).and_then(text_only)?;
            let colour: Vec<i64> = (3..6).map(|place| answer.get(place).and_then(whole).unwrap_or(0).clamp(0, 255)).collect();
            Some(serde_json::json!({ "name": name, "colour": colour }))
        })
        .collect()
}

/// The regions in the client's order, the first of them "all maps": `queryRegionInfo(region)` from 1 until it
/// names none (it answers a number and the name).
fn regions(lua: &Lua) -> Vec<serde_json::Value> {
    (1..=255)
        .map_while(|region| {
            let name = asked(lua, "queryRegionInfo", &[region]).get(1).and_then(text_only)?;
            Some(serde_json::json!({ "name": name, "maps": maps(lua, region) }))
        })
        .collect()
}

/// What a state that ran the files tells: `{"regions": [{"name": …, "maps": [{"name": …, "colour": [r, g,
/// b]}]}], "help": "…", "highest": 200}`. `highest` is the game's highest level as the windows of the booking
/// take it (`CSession::m_maxLevel`: `MaxLevelTable.BaseLevel3rd` of the service's `ExternalSettings`).
fn told(lua: &Lua, highest: i64) -> serde_json::Value {
    let help = asked(lua, "GetPartyBookingHelp", &[-1]).first().and_then(text_only).unwrap_or_default();
    serde_json::json!({ "regions": regions(lua), "help": help, "highest": highest })
}

/// The booking's tables out of the archives.
pub fn booking(read: &Sources) -> serde_json::Value {
    let lua = read.run(&FILES);
    let settings = run(read.grf, &["service_korea\\externalsettings_kr"]);
    let highest = settings.globals().get::<Table>("MaxLevelTable").and_then(|table| table.get::<Value>("BaseLevel3rd"));
    told(&lua, highest.ok().as_ref().and_then(whole).unwrap_or(0).max(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Files of the client's shape: the list of places, the help, and the functions that are asked.
    const FILES: &str = r#"
        MsgStrID = { ALL = "All Maps", PRT = "Prontera", FIELD = "Fields", SEWER = "Culvert F4" }
        map_list = {
            { name = MsgStrID.ALL, list = { { name = MsgStrID.ALL } } },
            { name = MsgStrID.PRT, list = { { name = MsgStrID.FIELD }, { name = MsgStrID.SEWER, colorR = 255, colorG = 0, colorB = 0 } } },
        }
        PartyBookingHelp = { "/recruit: opens one window.", "/booking: opens the other." }
        function queryRegionInfo(region)
            local found = map_list[region]
            if found == nil then return 0, nil end
            return region, found.name
        end
        function queryMapInfo(region, index)
            local found = map_list[region] and map_list[region].list[index]
            if found == nil then return 0, 0, nil, 0, 0, 0 end
            return index, 0, found.name, found.colorR or 0, found.colorG or 0, found.colorB or 0
        end
        function GetPartyBookingHelp(index)
            return table.concat(PartyBookingHelp, "\n")
        end
    "#;

    #[test]
    fn the_places_and_the_help_are_asked_of_the_files_own_functions() {
        let lua = state();
        exec(&lua, FILES.as_bytes(), None).unwrap();
        let told = told(&lua, 200);
        assert_eq!(told["regions"].as_array().map(Vec::len), Some(2));
        assert_eq!(told["regions"][0]["name"], "All Maps");
        assert_eq!(told["regions"][1]["maps"][1], serde_json::json!({ "name": "Culvert F4", "colour": [255, 0, 0] }));
        assert_eq!(told["regions"][1]["maps"][0]["colour"], serde_json::json!([0, 0, 0]));
        assert_eq!(told["help"], "/recruit: opens one window.\n/booking: opens the other.");
        assert_eq!(told["highest"], 200);
    }

    #[test]
    fn files_without_the_functions_tell_of_no_place() {
        let told = told(&state(), 0);
        assert_eq!((told["regions"].as_array().map(Vec::len), told["help"].as_str()), (Some(0), Some("")));
    }
}
