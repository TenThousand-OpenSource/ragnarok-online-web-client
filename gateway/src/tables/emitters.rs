//! The particle emitters of a map (smoke over a chimney, dust in the wind): what the map's own script of the
//! client's `effecttool` folder sets on each of the emitters its `.rsw` places (the effect 974, `EF_EMITTER`).

use super::*;

/// The scripts the client has run before any map's (`CSession`, Source/Client/Session/Session.cpp: the numbers
/// of the settings, the functions that read a map's tables, and what those ask a table with).
const BEFORE: [&str; 3] = ["effecttool\\effectmsgid", "effecttool\\effectfunc", "effecttool\\effecttoolutil_ver_2"];

/// The client's `SetEmitterInfo`, which the scripts' functions call for every setting of every emitter, put
/// down in Lua: here it only keeps what it was called with.
const KEPT: &str = "EMITTER_CALLS = {} SetEmitterInfo = function(...) EMITTER_CALLS[#EMITTER_CALLS + 1] = { ... } end";

/// The name of a map's script (`CurrentMapScriptName`, Source/Contents/Effects/Core/EffectLists.cpp): the map's,
/// and of a copy made for one party (`0011@gef`) without the three characters that stand four before the `@`.
/// None for what is no map's name: the name becomes a path in the archives.
fn script_name(map: &str) -> Option<String> {
    let mut name = map.to_ascii_lowercase();
    if let Some(at) = name.rfind('@').filter(|&at| at >= 4) {
        name.replace_range(at - 4..at - 1, "");
    }
    let name = name.split('.').next().unwrap_or_default().to_owned();
    (!name.is_empty() && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"_-@".contains(&byte))).then_some(name)
}

/// The global a script keeps something of its map in (`EmitterKey`, Source/Contents/Effects/Particles/
/// Particle.cpp): `_<map>_<what>`, without the map's first `@`.
fn key(name: &str, what: &str) -> String {
    format!("_{name}_{what}").replacen('@', "", 1)
}

/// The emitters of a map as its script sets them (`EffectLists::LoadMapEffectScript`), for `GET
/// /tables/emitters/<map>`: a list in which the place is the emitter's number among those the map's `.rsw`
/// places, and an entry what the script sets of it, `null` for one it leaves as it is made:
/// `[{"low": [x, y, z], "high": [x, y, z], "gravity": [x, y, z], "spread": [x, y, z], "colour": [red, green,
/// blue, alpha], "rate": [least, most], "size": [least, most], "life": [least, most], "picture": "smoke2.bmp",
/// "pace": 9, "source": 10, "destination": 2, "most": 30, "depth": true, "flat": false, "turn": [x, y, z],
/// "lasting": false, "stretch": [wide, high]}]` (`told` says what each is). A map without a script has none.
///
/// Of a script's tables only the emitters' is read (`emitterInfo`): a script of the first version by
/// `ReadEmitterInfoz_ver_1_0`, one of the second by `ReadEmitterInfoz`, the client's own functions, run as they
/// are. The others (`windEffectInfo`, `animatedEmitterInfo`, `magicfloorInfo`, `evilsPawInfo`, `ez2strInfo`, and
/// the first version's `_<map>_Effect`) set effects no map of the classic game places.
pub fn emitters(grf: &Grf, map: &str) -> serde_json::Value {
    let Some(name) = script_name(map) else { return serde_json::json!([]) };
    let (file, lua) = (format!("effecttool\\{name}"), state());
    let script = Script::read(grf, &file);
    // (Most maps have no script: that is no failure to note.)
    if script.bytes.is_err() {
        return serde_json::json!([]);
    }
    run_in(&lua, grf, &BEFORE);
    script.run_in(&lua);
    read(&lua, &name)
}

/// What the script a state ran sets on its map's emitters.
fn read(lua: &Lua, name: &str) -> serde_json::Value {
    let version = lua.globals().get::<Value>(key(name, "effect_version")).ok().and_then(|version| whole(&version)).unwrap_or(0);
    let reader = match version {
        0 | 1 => "ReadEmitterInfoz_ver_1_0",
        2 => "ReadEmitterInfoz",
        _ => return serde_json::json!([]),
    };
    let called = lua.load(KEPT).exec().and_then(|()| lua.globals().get::<mlua::Function>(reader)?.call::<()>(key(name, "emitterInfo")));
    if let Err(error) = called {
        eprintln!("tables: effecttool\\{name}.lub: {error}");
    }
    let Ok(calls) = lua.globals().get::<Table>("EMITTER_CALLS") else { return serde_json::json!([]) };
    let mut set: BTreeMap<usize, serde_json::Map<String, serde_json::Value>> = BTreeMap::new();
    for call in calls.sequence_values::<Table>().flatten() {
        let given: Vec<Value> = call.sequence_values::<Value>().flatten().collect();
        let (Some(index), Some(setting)) = (given.first().and_then(whole), given.get(1).and_then(whole)) else { continue };
        let Ok(index) = usize::try_from(index) else { continue };
        let emitter = set.entry(index).or_default();
        if let Some((field, value)) = told(setting, given.get(2..).unwrap_or_default()) {
            emitter.insert(field.to_owned(), value);
        }
    }
    let count = set.keys().next_back().map_or(0, |last| last + 1);
    (0..count).map(|index| set.remove(&index).map_or(serde_json::Value::Null, serde_json::Value::from)).collect::<Vec<_>>().into()
}

/// One call of `SetEmitterInfo` (Source/Script/LuaScript.cpp) as the field it sets and what it sets it to: the
/// setting by its number (`EffectMessageId`), then what was given with it.
///
/// - 1, 2 `low`, `high`: between which two a particle's speed is drawn, along each axis;
/// - 3 `gravity`: what is added to its speed every second;
/// - 5 `spread`: how far to each side of the emitter it is born;
/// - 6 `colour`: red, green, blue and alpha, none over 255;
/// - 7 `rate`, 8 `size`, 9 `life`: the least and the most of how many are born a second, of a particle's size
///   and of its seconds;
/// - 10 `picture`: its picture, under `data\texture\`;
/// - 11 `pace`: what its speed is taken times;
/// - 12, 13 `source`, `destination`: Direct3D's two blends, by their numbers;
/// - 14 `most`: how many particles the emitter has;
/// - 43 `depth`: whether what stands before a particle hides it;
/// - 45 `flat`: whether it is turned by `turn` (46, degrees about each axis) and not to the camera;
/// - 51 `lasting`: whether its particles neither fade nor end;
/// - 52 `stretch`: its size's width and height.
///
/// None for 0, after which the client only begins the emitter anew (`ResetParticles`), for 4, the emitter's
/// place, which no script's function sets (the place is the `.rsw`'s), and for what was given too little.
fn told(setting: i64, given: &[Value]) -> Option<(&'static str, serde_json::Value)> {
    let number = |at: usize| match given.get(at)? {
        Value::Integer(number) => Some(*number as f64),
        Value::Number(number) => Some(*number),
        _ => None,
    };
    let numbers = |count: usize| (0..count).map(number).collect::<Option<Vec<f64>>>().map(serde_json::Value::from);
    let flag = |is: fn(i64) -> bool| number(0).map(|number| is(number as i64).into());
    Some(match setting {
        1 => ("low", numbers(3)?),
        2 => ("high", numbers(3)?),
        3 => ("gravity", numbers(3)?),
        5 => ("spread", numbers(3)?),
        6 => ("colour", (0..4).map(|part| Some((number(part)? as i64).min(255))).collect::<Option<Vec<i64>>>()?.into()),
        7 => ("rate", numbers(2)?),
        8 => ("size", numbers(2)?),
        9 => ("life", numbers(2)?),
        10 => ("picture", text_only(given.first()?)?.into()),
        11 => ("pace", number(0)?.into()),
        12 => ("source", (number(0)? as i64).into()),
        13 => ("destination", (number(0)? as i64).into()),
        14 => ("most", (number(0)? as i64).into()),
        43 => ("depth", flag(|depth| depth != 0)?),
        45 => ("flat", flag(|flat| flat != 0)?),
        46 => ("turn", numbers(3)?),
        51 => ("lasting", flag(|lasting| lasting == 1)?),
        52 => ("stretch", numbers(2)?),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The client's folder, where this is run beside it.
    fn client() -> Option<Grf> {
        Grf::open_client_dir(std::path::Path::new("../../Feb-Client-CLEAN")).ok()
    }

    #[test]
    fn a_script_is_named_as_the_client_names_it() {
        assert_eq!(script_name("Prontera").as_deref(), Some("prontera"));
        assert_eq!(script_name("prontera.rsw").as_deref(), Some("prontera"));
        // A copy for one party is its map's; the key has no `@`.
        assert_eq!(script_name("0011@gef").as_deref(), Some("1@gef"));
        assert_eq!(key("1@gef", "emitterInfo"), "_1gef_emitterInfo");
        assert_eq!(key("mjolnir_07", "effect_version"), "_mjolnir_07_effect_version");
        assert_eq!((script_name(""), script_name("..\\secret"), script_name("a/b")), (None, None, None));
    }

    #[test]
    fn a_call_is_the_field_it_sets() {
        let given = |numbers: &[f64]| numbers.iter().map(|number| Value::Number(*number)).collect::<Vec<_>>();
        assert_eq!(told(3, &given(&[0.7, -2.0, 0.7])), Some(("gravity", serde_json::json!([0.7, -2.0, 0.7]))));
        assert_eq!(told(6, &given(&[300.0, 15.5, 20.0, 50.0])), Some(("colour", serde_json::json!([255, 15, 20, 50]))));
        // (The functions give a third value that is nothing with the pairs.)
        assert_eq!(told(7, &[Value::Integer(3), Value::Integer(4), Value::Nil]), Some(("rate", serde_json::json!([3.0, 4.0]))));
        assert_eq!(told(12, &given(&[9.0])), Some(("source", serde_json::json!(9))));
        assert_eq!((told(43, &given(&[1.0])), told(51, &given(&[2.0]))), (Some(("depth", true.into())), Some(("lasting", false.into()))));
        // Beginning anew, the place, a setting of another effect and one given too little set nothing.
        assert_eq!((told(0, &[]), told(4, &given(&[1.0, 2.0, 3.0])), told(53, &given(&[1.0])), told(1, &given(&[1.0]))), (None, None, None, None));
    }

    #[test]
    fn the_emitters_are_what_the_readers_calls_set() {
        // As the client's `ReadEmitterInfoz` calls for Prontera's table, cut short, with a number left out.
        let lua = state();
        exec(
            &lua,
            b"_town_effect_version = 2 function ReadEmitterInfoz(name) \
                SetEmitterInfo(2, 14, 20) SetEmitterInfo(2, 10, 'effect\\\\smoke1.bmp') SetEmitterInfo(2, 7, 3, 4, nil) SetEmitterInfo(2, 0) \
                SetEmitterInfo(0, 6, 5, 15, 20, 50) SetEmitterInfo(0, 0) asked = name end",
            Some("test")
        )
        .expect("the script");
        assert_eq!(
            read(&lua, "town"),
            serde_json::json!([{ "colour": [5, 15, 20, 50] }, null, { "most": 20, "picture": "effect\\smoke1.bmp", "rate": [3.0, 4.0] }])
        );
        assert_eq!(lua.globals().get::<String>("asked").unwrap(), "_town_emitterInfo");
        // A version the client reads nothing of.
        exec(&lua, b"_town_effect_version = 3", None).unwrap();
        assert_eq!(read(&lua, "town"), serde_json::json!([]));
    }

    #[test]
    fn prontera_has_its_three_smokes() {
        // Read out of the client's own files, where they are there.
        let Some(grf) = client() else { return };
        let emitters = emitters(&grf, "prontera");
        let smokes = emitters.as_array().expect("a list");
        assert_eq!(smokes.len(), 3);
        assert_eq!(
            smokes[0],
            serde_json::json!({
                "most": 20, "low": [-3.0, -5.0, -3.0], "high": [5.0, 0.0, 5.0], "gravity": [0.7, -2.0, 0.7], "spread": [0.0, 0.0, 0.0],
                "colour": [5, 15, 20, 50], "rate": [3.0, 4.0], "size": [15.0, 20.0], "life": [4.0, 4.0], "picture": "effect\\smoke1.bmp",
                "pace": 0.0, "source": 9, "destination": 2, "depth": true, "flat": false, "turn": [0.0, 0.0, 0.0]
            })
        );
        // Morocc's dust: eighty-one of them, each the same; a map without a script has none.
        let dust = super::emitters(&grf, "morocc");
        assert_eq!(dust.as_array().map(Vec::len), Some(81));
        assert_eq!((&dust[80]["pace"], &dust[80]["source"], &dust[80]["picture"]), (&9.0.into(), &10.into(), &"smoke2.bmp".into()));
        assert_eq!(super::emitters(&grf, "geffen"), serde_json::json!([]));
    }
}
