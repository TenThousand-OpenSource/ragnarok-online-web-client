//! The fields of the windows that are typed into, written once: a plain line of the page's (`class="a in"`)
//! at its place, with what stands in it at first and how many characters it takes.


use super::*;

/// A field of one line. `id` is its name on the page (`Window::field` for a box that asks), `value` what
/// stands in it at first, `most` how many characters it takes, and `ground` what its style says after its
/// place: its ground (`background:#e8e8e8`), and a rim where it has one.
pub(in crate::web) fn field(id: &str, (x, y, width, height): (i32, i32, i32, i32), value: &str, most: usize, ground: &str) -> String {
    format!(r#"<input id="{id}" class="a in" style="{};{ground}" type="text" maxlength="{most}" autocomplete="off" value="{}">"#, at(x, y, width, height), escape(value))
}

/// The line a talk is typed into, of 70 letters at most, in a form of its own that Enter sends: `form` is what
/// marks the form (`sent(..)`), `place` the form's place as a style and `inside` the field's within it, `look`
/// what the field's style says after that.
pub(in crate::web) fn typed_line(form: &str, place: &str, id: &str, inside: &str, look: &str) -> String {
    format!(r#"<form {form} class="a" style="{place}"><input id="{id}" class="a in" maxlength="70" autocomplete="off" style="{inside};{look}"><button type="submit" hidden></button></form>"#)
}

/// The same field for a number: a phone shows its figures to type it with.
pub(in crate::web) fn numeric_field(id: &str, (x, y, width, height): (i32, i32, i32, i32), value: &str, most: usize, ground: &str) -> String {
    format!(r#"<input id="{id}" class="a in" style="{};{ground}" type="text" inputmode="numeric" maxlength="{most}" autocomplete="off" value="{}">"#, at(x, y, width, height), escape(value))
}
