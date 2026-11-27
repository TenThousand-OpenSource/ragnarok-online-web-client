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

/// The periods of the server's attendance database (rAthena's `db/import/attendance.yml`, written as
/// `AttendanceDatabase::parseBodyNode` of `src/map/pc.cpp` reads it: `- Start:`, `End:`, and under `Rewards:`
/// each `- Day:` with its `ItemId:` and `Amount:`, one when not given), each as the table above is; a comment
/// after a `#` is left out.
fn periods(text: &str) -> Vec<serde_json::Value> {
    let mut periods: Vec<(i64, i64, Vec<(i64, i64, i64)>)> = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        let (starts, field) = line.strip_prefix("- ").map_or((false, line), |field| (true, field));
        let Some((name, value)) = field.split_once(':') else { continue };
        let Ok(value) = value.trim().parse::<i64>() else { continue };
        match (name.trim(), periods.last_mut()) {
            ("Start", _) if starts => periods.push((value, 0, Vec::new())),
            ("End", Some(period)) => period.1 = value,
            ("Day", Some(period)) if starts => period.2.push((value, 0, 1)),
            ("ItemId", Some(period)) => period.2.last_mut().into_iter().for_each(|reward| reward.1 = value),
            ("Amount", Some(period)) => period.2.last_mut().into_iter().for_each(|reward| reward.2 = value),
            _ => {}
        }
    }
    periods
        .into_iter()
        .map(|(first, last, mut rewards)| {
            rewards.sort_unstable();
            let rewards: Vec<_> = rewards.iter().map(|(day, item, count)| serde_json::json!({ "day": day, "item": item, "count": count })).collect();
            serde_json::json!({ "first": first, "last": last, "rewards": rewards })
        })
        .collect()
}

/// A day written as the tables write one (`20261127`), by the days since 1970 (the civil calendar counted
/// from the days, as Howard Hinnant's `civil_from_days` does).
fn date(days: i64) -> i64 {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let of_era = shifted.rem_euclid(146_097);
    let year_of_era = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * of_year + 2) / 153;
    let (day, month) = (of_year - (153 * shifted_month + 2) / 5 + 1, if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 });
    (year_of_era + era * 400 + i64::from(month <= 2)) * 10_000 + month * 100 + day
}

/// The period of the server's database that a day falls in, as the server picks its own
/// (`pc_attendance_period`).
fn period_of(text: &str, today: i64) -> Option<serde_json::Value> {
    periods(text).into_iter().find(|period| period["first"].as_i64() <= Some(today) && Some(today) <= period["last"].as_i64())
}

/// The table of the attendance check. **This project's own, against the Windows client**: when the server's
/// attendance database (`server_file`: `server/db/attendance.yml`, which the Docker setup mounts as rAthena's
/// `db/import/attendance.yml`) has a period that runs today (by the world's clock, as the gateway starts), that
/// period is given, since the server gives its rewards and the window must show the same; a Windows client is
/// made to agree by writing its `System\CheckAttendance.lub` anew, and the client's files are never changed
/// here. Otherwise the client's own table (`System\CheckAttendance.lub`, which `CCheckAttendanceMgr::Load`
/// runs and whose `main` hands the dates and the rewards over). The files beside it named for a Korean server
/// (`CheckAttendance_<server>.lub`, read as the game is entered on a server of that name) are not read.
pub fn attendance(client_dir: &std::path::Path, server_file: &std::path::Path) -> serde_json::Value {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |since| since.as_secs() as i64);
    let server = std::fs::read_to_string(server_file).ok().and_then(|text| period_of(&text, date(now.div_euclid(86_400))));
    server.unwrap_or_else(|| std::fs::read(client_dir.join("System/CheckAttendance.lub")).map(|source| read(&source)).unwrap_or_default())
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

    #[test]
    fn the_server_s_period_of_today_is_given_as_the_client_s_table_is() {
        let text = "Header:\n  Type: ATTENDANCE_DB\nBody:\n  - Start: 20260101\n    End: 20260131\n    Rewards:\n      - Day: 1\n        ItemId: 501\n  \
                    - Start: 20261101\n    End: 20270131\n    Rewards:\n      - Day: 2\n        ItemId: 601   # Fly Wing\n        Amount: 10\n      \
                    - Day: 1\n        ItemId: 501\n        Amount: 20\n#      - Day: 3\n";
        assert_eq!(
            period_of(text, 20261127),
            Some(serde_json::json!({ "first": 20261101, "last": 20270131, "rewards": [{ "day": 1, "item": 501, "count": 20 }, { "day": 2, "item": 601, "count": 10 }] }))
        );
        // An amount not given is one; a day outside every period has none.
        assert_eq!(period_of(text, 20260131).unwrap()["rewards"], serde_json::json!([{ "day": 1, "item": 501, "count": 1 }]));
        assert_eq!(period_of(text, 20260201), None);
    }

    #[test]
    fn a_day_is_written_as_the_tables_write_it() {
        assert_eq!(date(0), 19700101);
        assert_eq!(date(20_784), 20261127);
        assert_eq!(date(19_782), 20240229);
    }
}
