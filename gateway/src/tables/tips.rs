//! The tip box: the client's tips, each with its title and its pages.

use super::*;

/// A tip of the client's tip box: its number, its title and its pages. A page is as the file has it, with the
/// client's marks of colour (`^rrggbb`) and its links (`<TIPBOX>`, `<NAVI>`, `<ITEM>`) in it.
pub(super) struct Tip {
    pub id: i64,
    pub title: String,
    pub pages: Vec<String>,
}

/// The client's tip box (`System\tipbox.lub`, the English one beside it where there is one; the client's
/// `UITipboxWnd` reads it through `AddTip` and `AddPage`), in the order of the tips' numbers: the one reading
/// of the file, for the Tipbox window (`tips`) and for the Adventure Guide, whose quests are tips
/// (`guide::guide`).
pub(super) fn read(client_dir: &std::path::Path) -> Vec<Tip> {
    let Some(lua) = beside(client_dir, &["SystemEN/tipbox.lub", "System/tipbox.lub"]) else { return Vec::new() };
    let Ok(table) = lua.globals().get::<Table>("tbl") else { return Vec::new() };
    // The English file is plain text with a few signs beyond ASCII; the client's own is Korean.
    let words = |text: &mlua::String| latin(&text.as_bytes());
    let mut tips: Vec<Tip> = table
        .pairs::<i64, Table>()
        .flatten()
        .map(|(id, tip)| {
            let title = tip.get::<Option<mlua::String>>("Title").ok().flatten().map(|title| words(&title)).unwrap_or_default();
            let pages = tip.get::<Option<Table>>("Page").ok().flatten();
            let pages: Vec<String> = pages.map(|pages| pages.sequence_values::<mlua::String>().flatten().map(|page| words(&page)).collect()).unwrap_or_default();
            Tip { id, title, pages }
        })
        .collect();
    tips.sort_by_key(|tip| tip.id);
    tips
}

/// The tips of the client's Tipbox window: `[{"id": 1, "title": …, "pages": [text, …]}, …]` in the order of the
/// tips' numbers.
pub fn tips(client_dir: &std::path::Path) -> serde_json::Value {
    read(client_dir).iter().map(|tip| serde_json::json!({ "id": tip.id, "title": tip.title, "pages": tip.pages })).collect()
}
