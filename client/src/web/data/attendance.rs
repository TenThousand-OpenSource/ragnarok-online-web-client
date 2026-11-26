//! The attendance check: the days its event runs and what each day of attendance is rewarded with.

use super::*;

thread_local! {
    /// The client's table of the attendance check (`System\CheckAttendance.lub`).
    static ATTENDANCE: RefCell<std::rc::Rc<Attendance>> = RefCell::new(std::rc::Rc::new(Attendance::default()));
}

/// The table: the first and the last day of the event, each written as a number (`20260219`), and the
/// rewards in the order of their days, each the day, the item and how many of it
/// (`CCheckAttendanceMgr::m_config0`, `m_config1`, `m_rewards`). The window shows them in that order, a cell each.
#[derive(Default)]
pub struct Attendance {
    pub first: u32,
    pub last: u32,
    pub rewards: Vec<(u32, u32, u32)>,
}

/// The client's table of the attendance check.
pub fn attendance() -> std::rc::Rc<Attendance> {
    ATTENDANCE.with_borrow(Clone::clone)
}

/// Takes the table out of what the gateway gave (`tables/attendance.rs`).
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let table = field(tables, "attendance");
    let whole = |object: &wasm_bindgen::JsValue, name: &str| number(object, name).unwrap_or(0.0) as u32;
    let rewards = list(&field(&table, "rewards")).iter().map(|reward| (whole(reward, "day"), whole(reward, "item"), whole(reward, "count"))).collect();
    ATTENDANCE.set(std::rc::Rc::new(Attendance { first: whole(&table, "first"), last: whole(&table, "last"), rewards }));
}
