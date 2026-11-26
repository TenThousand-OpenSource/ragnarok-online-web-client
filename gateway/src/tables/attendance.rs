//! The attendance check: the days its event runs and what each day of attendance is rewarded with.

use super::*;

/// What the client gives the file to call (`CCheckAttendanceMgr::L_InsertCheckAttendanceConfig`,
/// `L_InsertCheckAttendanceReward`), written in Lua. Each reads what it was called with from the end, as the
/// client's do (the file's `main` hands the first one a switch before the dates, which the client's function
/// never looks at): the two dates are kept, and a reward is kept by its day
/// unless its item is none, its count is not above nothing or the day has one already; each answers as the
/// client's does, with whether it was taken and a word.
const GIVEN: &str = r##"
ATTENDANCE = { rewards = {} }
function InsertCheckAttendanceConfig(...)
    local given = { n = select("#", ...), ... }
    local first, last = given[given.n - 1], given[given.n]
    if type(first) ~= "number" or type(last) ~= "number" then return false, "[InsertCheckAttendanceConfig]" end
    ATTENDANCE.first, ATTENDANCE.last = math.floor(first), math.floor(last)
    return true, "good"
end
function InsertCheckAttendanceReward(...)
    local given = { n = select("#", ...), ... }
    local day, item, count = tonumber(given[given.n - 2]), tonumber(given[given.n - 1]), tonumber(given[given.n])
    if not day or not item or item == 0 or not count or count <= 0 or ATTENDANCE.rewards[math.floor(day)] then
        return false, "[InsertCheckAttendanceReward]"
    end
    ATTENDANCE.rewards[math.floor(day)] = { math.floor(item), math.floor(count) }
    return true, "good"
end
"##;

/// The table as a file's source leaves it: `{"first": 20260219, "last": 20260318, "rewards": [{"day": 1,
/// "item": 6909, "count": 10}, …]}`, the rewards in the order of their days; nothing where the file does not
/// run or its `main` says it failed (the client then has no attendance at all: `CCheckAttendanceMgr::Create`).
fn read(source: &[u8]) -> serde_json::Value {
    let lua = state();
    let ran = exec(&lua, GIVEN.as_bytes(), Some("attendance"))
        .and_then(|_| exec(&lua, source, Some("CheckAttendance")))
        .and_then(|_| exec(&lua, b"ATTENDANCE.done = main()", Some("CheckAttendance main")));
    if let Err(error) = &ran {
        eprintln!("tables: CheckAttendance.lub: {error}");
    }
    let Ok(table) = lua.globals().get::<Table>("ATTENDANCE") else { return serde_json::Value::Null };
    if ran.is_err() || !table.get::<bool>("done").unwrap_or(false) {
        return serde_json::Value::Null;
    }
    let mut rewards: Vec<(i64, i64, i64)> = table
        .get::<Table>("rewards")
        .map(|rewards| {
            rewards
                .pairs::<i64, Table>()
                .flatten()
                .map(|(day, reward)| (day, reward.get::<i64>(1).unwrap_or(0), reward.get::<i64>(2).unwrap_or(0)))
                .collect()
        })
        .unwrap_or_default();
    rewards.sort_unstable();
    serde_json::json!({
        "first": table.get::<i64>("first").unwrap_or(0),
        "last": table.get::<i64>("last").unwrap_or(0),
        "rewards": rewards.iter().map(|(day, item, count)| serde_json::json!({ "day": day, "item": item, "count": count })).collect::<Vec<_>>(),
    })
}

/// The client's table of the attendance check (`System\CheckAttendance.lub`, which `CCheckAttendanceMgr::Load`
/// runs and whose `main` hands the dates and the rewards over). The files beside it named for a Korean server
/// (`CheckAttendance_<server>.lub`, read as the game is entered on a server of that name) are not read.
pub fn attendance(client_dir: &std::path::Path) -> serde_json::Value {
    std::fs::read(client_dir.join("System/CheckAttendance.lub")).map(|source| read(&source)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_s_main_hands_over_the_dates_and_the_rewards_by_day() {
        let source = br#"
            Config = { StartDate = 20260219, EndDate = 20260318 }
            Reward = { [2] = { 607, 3 }, [1] = { "501", 10 } }
            function main()
                InsertCheckAttendanceConfig(Config.EvendOnOff, Config.StartDate, Config.EndDate)
                for day, reward in pairs(Reward) do
                    local result, msg = InsertCheckAttendanceReward(day, reward[1], reward[2])
                    if not result then return false, msg end
                end
                return true, "good"
            end
        "#;
        assert_eq!(
            read(source),
            serde_json::json!({ "first": 20260219, "last": 20260318, "rewards": [{ "day": 1, "item": 501, "count": 10 }, { "day": 2, "item": 607, "count": 3 }] })
        );
    }

    #[test]
    fn a_file_whose_main_fails_gives_nothing() {
        let source = br#"
            function main()
                return InsertCheckAttendanceReward(1, 0, 10)
            end
        "#;
        assert_eq!(read(source), serde_json::Value::Null);
        assert_eq!(read(b"broken ("), serde_json::Value::Null);
    }
}
