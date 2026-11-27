//! One's own ad in the old party booking: what the server answers to registering it and to taking it away,
//! the window that stands while it is registered (the client's `UISeekPartyWnd`) and the window its jobs are
//! changed in (`UIJobListWnd`).

use super::line;
use crate::net::booking::JOBS;

/// What the server's answer to registering says in a box (`CGameMode::ReceivePartyBookingUpdateResult`, which
/// reads ZC_PARTY_BOOKING_ACK_REGISTER whatever its name): nothing for 0, the ad is registered; 2 is an ad
/// registered already; anything else a failure.
pub fn register_refused(result: i16) -> Option<usize> {
    match result {
        0 => None,
        2 => Some(line::PENDING),
        _ => Some(line::REGISTER_FAILED),
    }
}

/// What the server's answer to taking the ad away does (`ReceivePartyBookingRegisterResult`, which reads
/// ZC_PARTY_BOOKING_ACK_DELETE): the line it says in a box, if any, and whether the window of the ad closes.
pub fn deleted(result: i16) -> (Option<usize>, bool) {
    match result {
        0 | 1 => (None, true),
        2 => (Some(line::DELETE_FAILED), false),
        3 => (Some(line::NO_AD), true),
        _ => (None, false),
    }
}

/// The jobs an ad was registered with, as the window that changes them lists them (`UISeekPartyWnd::SetJobs`
/// into `g_partyBookingJobs`, a map by the job's number): those before the first that is none, in the order
/// of their numbers, each sought.
pub fn listed(jobs: &[i16; JOBS]) -> Vec<(i16, bool)> {
    let mut listed: Vec<(i16, bool)> = jobs.iter().take_while(|job| **job >= 0).map(|job| (*job, true)).collect();
    listed.sort_unstable();
    listed.dedup();
    listed
}

/// The jobs sent when the window that changes them is confirmed (`UIJobListWnd::SendMsg`, `ID_OK`): each job
/// in its place of the list, -1 for one no longer sought; none when no job is left ticked, which the client
/// refuses in a box.
pub fn changed(listed: &[(i16, bool)]) -> Option<[i16; JOBS]> {
    let mut jobs = [-1; JOBS];
    for (place, (job, sought)) in listed.iter().take(JOBS).enumerate() {
        if *sought {
            jobs[place] = *job;
        }
    }
    listed.iter().any(|(_, sought)| *sought).then_some(jobs)
}

/// The title of the window of one's own ad (`UISeekPartyWnd::OnCreate`: "%s - %d~%d"): the table's words and
/// the levels sought, fifteen from the one asked and the game's highest at most.
pub fn own_title(words: &str, level: i16, highest: i32) -> String {
    format!("{words} - {level}~{}", (i32::from(level) + 15).min(highest))
}

/// How many kinds of moving picture the window has to pick one of as it opens (`rand() % 5`), and the last
/// frame of each (`UISeekPartyWnd::OnProcess`: `frameCounts`).
pub const KINDS: u32 = 5;
const LAST_FRAME: [u32; 6] = [5, 5, 17, 11, 11, 11];
/// Who each kind shows, as its files are named (`UISeekPartyWnd::Paint`).
const SHOWN: [&str; 6] = ["bibibic", "bibibic", "달기린", "달기린", "boya", "boya"];

/// The frame of the picture so long after the window opened (`OnProcess`): it begins at 1, goes on by one a
/// second up to its kind's last, and begins anew at 0.
pub fn frame(kind: u32, milliseconds: u32) -> u32 {
    let last = LAST_FRAME[kind as usize % LAST_FRAME.len()];
    (1 + milliseconds / 1000) % (last + 1)
}

/// The file of a frame, under the folder of the windows' pictures (`Paint`:
/// `basic_interface\seekparty\type_0%d\seekparty_%s_0%d_%d.bmp`).
pub fn frame_file(kind: u32, frame: u32) -> String {
    format!("basic_interface/seekparty/type_0{kind}/seekparty_{}_0{kind}_{frame}.bmp", SHOWN[kind as usize % SHOWN.len()])
}

/// How long the ad has been registered, as the window writes it (`Paint`: "%02d:%02d:%02d", the hours counted
/// up to sixty as the client counts them).
pub fn lasted(milliseconds: u32) -> String {
    let seconds = milliseconds / 1000;
    format!("{:02}:{:02}:{:02}", seconds / 3600 % 60, seconds / 60 % 60, seconds % 60)
}
