//! The presses of the lists a skill or a card opens to pick one thing of (`net::choosing`): a row, OK and
//! Cancel, whichever list is open; and a thing put into a forging, taken out again.

presses! {
    /// What can be pressed here.
    Press {
        /// A row of the list that is open.
        Pick(usize) = "choice-pick",
        /// The list's OK and Cancel.
        Ok = "choice-ok",
        Cancel = "choice-cancel",
        /// A slot of a forging: what was put into it comes out.
        Material(usize) = "choice-material",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [Press::Pick(3), Press::Ok, Press::Cancel, Press::Material(2)];
        for name in Press::NAMES {
            assert!(all.iter().any(|press| press.name().0 == *name), "{name} is not tried");
        }
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
