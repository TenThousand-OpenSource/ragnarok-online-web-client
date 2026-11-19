//! The tip box: the client's tips, each with its title and its pages.

use super::*;

/// The tips of the client's Tipbox window (`System\tipbox.lub`, the English one beside it where there is one;
/// the client's `UITipboxWnd` reads it through `AddTip` and `AddPage`): `[{"id": 1, "title": …, "pages": [text,
/// …]}, …]` in the order of the tips' numbers. A page is given as the file has it, with the client's marks of
/// colour (`^rrggbb`) and its links (`<TIPBOX>`, `<NAVI>`, `<ITEM>`) in it.
pub fn tips(client_dir: &std::path::Path) -> serde_json::Value {
    let Some(lua) = beside(client_dir, &["SystemEN/tipbox.lub", "System/tipbox.lub"]) else { return serde_json::json!([]) };
    let Ok(table) = lua.globals().get::<Table>("tbl") else { return serde_json::json!([]) };
    // The English file is plain text with a few signs beyond ASCII; the client's own is Korean.
    let words = |text: &mlua::String| match std::str::from_utf8(&text.as_bytes()) {
        Ok(text) => text.to_owned(),
        Err(_) => encoding_rs::WINDOWS_1252.decode_without_bom_handling(&text.as_bytes()).0.into_owned(),
    };
    let mut tips: Vec<(i64, serde_json::Value)> = table
        .pairs::<i64, Table>()
        .flatten()
        .map(|(id, tip)| {
            let title = tip.get::<Option<mlua::String>>("Title").ok().flatten().map(|title| words(&title)).unwrap_or_default();
            let pages: Vec<String> = tip.get::<Option<Table>>("Page").ok().flatten().map(|pages| pages.sequence_values::<mlua::String>().flatten().map(|page| words(&page)).collect()).unwrap_or_default();
            (id, serde_json::json!({ "id": id, "title": title, "pages": pages }))
        })
        .collect();
    tips.sort_by_key(|(id, _)| *id);
    tips.into_iter().map(|(_, tip)| tip).collect()
}
