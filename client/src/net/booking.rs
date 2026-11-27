//! The old party booking: an ad that a party looks for members, the search through the ads and what the
//! server tells of them. The layouts are the client's (`PACKET_CZ_PARTY_BOOKING_REQ_REGISTER` of
//! `UISeekPartyMBWnd.cpp`, `PACKET_CZ_PARTY_BOOKING_REQ_SEARCH` of `UISeekPartyListWnd.cpp`,
//! `PartyBookingUpdatePacket` of `UIJobListWnd.cpp`, `PARTY_BOOKING_AD_INFO` of `PartyBookingInfo.h` and the
//! `PACKET_ZC_PARTY_BOOKING_*` of `GameModeHandlers.cpp`) and rAthena's for PACKETVER 20260219
//! (`clif_parse_PartyBooking*`, `clif_PartyBooking*` in `clif.cpp`; `clif_shuffle.hpp` gives 0x0802 back to
//! the booking for clients after 2018-03-07). The client of this version has the newer CZ_PARTY_RECRUIT_*
//! (0x08e5…) only as names: nothing of it sends or reads them.

use super::bytes::{body, int, long, records, short, text_at, Writer};

/// ZC_PARTY_BOOKING_ACK_REGISTER (0x0803, 4 bytes): the answer to registering an ad.
pub const ZC_PARTY_BOOKING_ACK_REGISTER: u16 = 0x0803;
/// ZC_PARTY_BOOKING_ACK_SEARCH (0x0805): the ads a search found.
pub const ZC_PARTY_BOOKING_ACK_SEARCH: u16 = 0x0805;
/// ZC_PARTY_BOOKING_ACK_DELETE (0x0807, 4 bytes): the answer to taking one's ad away.
pub const ZC_PARTY_BOOKING_ACK_DELETE: u16 = 0x0807;
/// ZC_PARTY_BOOKING_NOTIFY_INSERT (0x0809, 50 bytes): someone registered an ad.
pub const ZC_PARTY_BOOKING_NOTIFY_INSERT: u16 = 0x0809;
/// ZC_PARTY_BOOKING_NOTIFY_UPDATE (0x080a, 18 bytes): an ad looks for other jobs now.
pub const ZC_PARTY_BOOKING_NOTIFY_UPDATE: u16 = 0x080a;
/// ZC_PARTY_BOOKING_NOTIFY_DELETE (0x080b, 6 bytes): an ad is gone.
pub const ZC_PARTY_BOOKING_NOTIFY_DELETE: u16 = 0x080b;

/// How many jobs an ad names at most (the client's `PARTY_BOOKING_DETAIL::job`).
pub const JOBS: usize = 6;

/// An ad as the server tells it (`PARTY_BOOKING_AD_INFO`, 48 bytes): `i32 index, char name[24], i32 time,
/// i16 level, i16 place, i16 job[6]`. The time is that of the registering, in seconds (the client calls it
/// the time the ad expires; rAthena sends when it began); a job nobody is sought for is -1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ad {
    pub index: u32,
    pub name: String,
    pub time: u32,
    pub level: i16,
    pub place: i16,
    pub jobs: [i16; JOBS],
}

/// The six jobs at that place of a packet.
fn jobs_at(bytes: &[u8], at: usize) -> Option<[i16; JOBS]> {
    let mut jobs = [-1; JOBS];
    for (place, job) in jobs.iter_mut().enumerate() {
        *job = short(bytes, at + 2 * place)?;
    }
    Some(jobs)
}

/// One ad out of its 48 bytes.
fn ad(record: &[u8]) -> Option<Ad> {
    Some(Ad {
        index: long(record, 0)?,
        name: text_at(record, 4, 24)?,
        time: long(record, 28)?,
        level: short(record, 32)?,
        place: short(record, 34)?,
        jobs: jobs_at(record, 36)?,
    })
}

/// The result alone of ZC_PARTY_BOOKING_ACK_REGISTER and ZC_PARTY_BOOKING_ACK_DELETE: `i16 result`.
pub fn result(packet: &[u8]) -> Option<i16> {
    short(packet, 2)
}

/// ZC_PARTY_BOOKING_ACK_SEARCH: `u16 length, u8 more, { ad }*`: the ads found, and whether more follow them.
pub fn found(packet: &[u8]) -> Option<(bool, Vec<Ad>)> {
    let ads = body(packet, 5)?;
    Some((*packet.get(4)? != 0, records(ads, 0, 48).filter_map(ad).collect()))
}

/// ZC_PARTY_BOOKING_NOTIFY_INSERT: one ad, after the packet's number.
pub fn inserted(packet: &[u8]) -> Option<Ad> {
    ad(packet.get(2..50)?)
}

/// ZC_PARTY_BOOKING_NOTIFY_UPDATE: `i32 index, i16 job[6]`: the jobs that ad looks for now.
pub fn updated(packet: &[u8]) -> Option<(u32, [i16; JOBS])> {
    Some((long(packet, 2)?, jobs_at(packet, 6)?))
}

/// ZC_PARTY_BOOKING_NOTIFY_DELETE: `i32 index`: the ad that is gone.
pub fn deleted(packet: &[u8]) -> Option<u32> {
    int(packet, 2).map(|index| index as u32)
}

/// The six jobs as a packet has them.
fn jobs_written(writer: Writer, jobs: &[i16; JOBS]) -> Writer {
    jobs.iter().fold(writer, |writer, job| writer.u16(*job as u16))
}

/// CZ_PARTY_BOOKING_REQ_REGISTER (0x0802, 18 bytes): `i16 level, i16 place, i16 job[6]`: an ad that members
/// of that level and up are sought for that place, of those jobs (`UISeekPartyMBWnd::SendMsg`, `ID_OK`).
pub fn register(level: i16, place: i16, jobs: &[i16; JOBS]) -> Vec<u8> {
    jobs_written(Writer::new(0x0802).u16(level as u16).u16(place as u16), jobs).done()
}

/// CZ_PARTY_BOOKING_REQ_SEARCH (0x0804, 14 bytes): `i16 level, i16 place, i16 job, i32 last, i16 count`: the
/// ads for that level (0 for any), that place (0 for any) and that job (-1 for any) that come after the ad
/// `last`, so many at most (`UISeekPartyListWnd::SendMsg`, `ID_PARTY_SEARCH`).
pub fn search(level: i16, place: i16, job: i16, last: i32, count: i16) -> Vec<u8> {
    Writer::new(0x0804).u16(level as u16).u16(place as u16).u16(job as u16).i32(last).u16(count as u16).done()
}

/// CZ_PARTY_BOOKING_REQ_DELETE (0x0806, 2 bytes): one's own ad is taken away (`UISeekPartyWnd::SendMsg`,
/// `ID_CLOSE`).
pub fn delete() -> Vec<u8> {
    Writer::new(0x0806).done()
}

/// CZ_PARTY_BOOKING_REQ_UPDATE (0x0808, 14 bytes): `i16 job[6]`: the jobs one's ad looks for from now
/// (`UIJobListWnd::SendMsg`, `ID_OK`).
pub fn update(jobs: &[i16; JOBS]) -> Vec<u8> {
    jobs_written(Writer::new(0x0808), jobs).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::{field, packet};

    /// An ad as rAthena writes one (`clif_PartyBookingSearchAck`).
    fn record(index: u32, name: &str, jobs: [i16; JOBS]) -> Vec<u8> {
        let jobs: Vec<u8> = jobs.iter().flat_map(|job| job.to_le_bytes()).collect();
        [&index.to_le_bytes()[..], &field(name, 24), &1_700_000_000u32.to_le_bytes(), &80i16.to_le_bytes(), &0x0203i16.to_le_bytes(), &jobs].concat()
    }

    #[test]
    fn reads_the_ads_of_a_search_and_of_a_notice() {
        let (first, second) = (record(7, "Leader", [1, 12, -1, -1, -1, -1]), record(9, "Other", [63, -1, -1, -1, -1, -1]));
        let mut answer = packet(0x0805, &[&[0, 0], &[1], &first, &second]);
        let length = answer.len() as u16;
        answer[2..4].copy_from_slice(&length.to_le_bytes());
        let (more, ads) = found(&answer).unwrap();
        assert!(more);
        assert_eq!(ads.len(), 2);
        assert_eq!(ads[0], Ad { index: 7, name: "Leader".to_owned(), time: 1_700_000_000, level: 80, place: 0x0203, jobs: [1, 12, -1, -1, -1, -1] });
        assert_eq!((ads[1].index, ads[1].jobs[0]), (9, 63));
        // Nothing found: the packet ends after its flag.
        assert_eq!(found(&[0x05, 0x08, 5, 0, 0]), Some((false, Vec::new())));
        assert_eq!(found(&[0x05, 0x08, 5, 0]), None);
        assert_eq!(inserted(&packet(0x0809, &[&first])).map(|ad| ad.name), Some("Leader".to_owned()));
        assert_eq!(inserted(&packet(0x0809, &[&first[..40]])), None);
    }

    #[test]
    fn reads_the_answers_and_the_other_notices() {
        assert_eq!(result(&packet(0x0803, &[&2i16.to_le_bytes()])), Some(2));
        assert_eq!(result(&[0x07, 0x08, 3]), None);
        let jobs: Vec<u8> = [5i16, -1, 7, -1, -1, -1].iter().flat_map(|job| job.to_le_bytes()).collect();
        assert_eq!(updated(&packet(0x080a, &[&7u32.to_le_bytes(), &jobs])), Some((7, [5, -1, 7, -1, -1, -1])));
        assert_eq!(deleted(&packet(0x080b, &[&7u32.to_le_bytes()])), Some(7));
    }

    #[test]
    fn what_is_sent_is_as_long_as_the_server_expects() {
        let sought = register(85, 0x0102, &[2, 13, -1, -1, -1, -1]);
        assert_eq!(sought.len(), 18);
        assert_eq!(&sought[..10], [0x02, 0x08, 85, 0, 0x02, 0x01, 2, 0, 13, 0]);
        assert_eq!(&sought[10..12], [0xff, 0xff]);
        let asked = search(0, 0, -1, 0, 10);
        assert_eq!(asked, [0x04, 0x08, 0, 0, 0, 0, 0xff, 0xff, 0, 0, 0, 0, 10, 0]);
        assert_eq!(delete(), [0x06, 0x08]);
        assert_eq!(update(&[-1, 13, -1, -1, -1, -1]).len(), 14);
    }
}
