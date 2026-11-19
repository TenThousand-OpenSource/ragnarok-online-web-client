//! The mercenary as rules: the title of its window, which mercenary it is to its script, how the time its
//! contract ends is written. No packet is read here: those are in `net/mercenary.rs`.

/// The title of its window, as the line of the client's message table and the English of it, by the job it is
/// (`UIMerInfoWnd::Paint`: ten archers from 6017, ten spearmen from 6027, ten swordsmen from 6037, and a
/// monster otherwise).
pub fn title(job: u16) -> (usize, &'static str) {
    match job {
        6017..=6026 => (1181, "Mercenary: Archer"),
        6037..=6046 => (1182, "Mercenary: Swordman"),
        6027..=6036 => (1183, "Mercenary: Spearman"),
        _ => (1367, "Mercenary Info - Monster Type"),
    }
}

/// Which mercenary a job is to its script (`AiMercenaryGetV`, `V_MERTYPE`): the hirelings count from 1 (the
/// client takes the job modulo 6017 and adds one, for the jobs its `IsHirelingJob` holds for: 6018 to 6052),
/// anything else is 1.
pub fn kind(job: u16) -> i32 {
    if crate::jobs::hireling(job) { i32::from(job % 6017) + 1 } else { 1 }
}

/// The time its contract ends, written as the client writes it (its line 1260, `%m/%d %H:%M`, through
/// `strftime`): `month`, `day`, `hour` and `minute` are the local ones of that moment, which the page knows.
pub fn expiry(format: &str, (month, day, hour, minute): (u32, u32, u32, u32)) -> String {
    format.replace("%m", &format!("{month:02}")).replace("%d", &format!("{day:02}")).replace("%H", &format!("{hour:02}")).replace("%M", &format!("{minute:02}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_it_by_its_job() {
        assert_eq!((title(6017).0, title(6026).0, title(6027).0, title(6037).0, title(6046).0, title(1002).0), (1181, 1181, 1183, 1182, 1182, 1367));
        assert_eq!((kind(6017), kind(6018), kind(6046), kind(1002)), (1, 2, 30, 1));
        assert_eq!(expiry("%m/%d %H:%M", (3, 9, 7, 5)), "03/09 07:05");
    }
}
