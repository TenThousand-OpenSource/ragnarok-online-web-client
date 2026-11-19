//! The copies of a character that an effect leaves standing: the client's `CHalluPC`, an actor of its own
//! made as its source is at that moment (its action and frame, what it wears), which neither moves nor
//! changes its picture. It is drawn in the colour it is told, at 180 of 255, and after 105 passes loses 8 of
//! that at every fourth pass, until less than 8 is left.
//!
//! At its 110th pass the client launches `EF_HALLUTIME` on it, which is not drawn here.
//!
//! The other kind is the client's `CBlurPC`, the afterimage `CRagEffect::SpawnBlurAfterimage` leaves of one
//! who moves (`EF_MAKEBLUR`, for as long as a quickening, Overthrust or Energy Coat holds): made the same
//! way and told its colour (`BM_RGB`: at 180 of 255), it loses 8 of that at every pass from its first
//! (`CBlurPC::OnProcess`), and is gone with less than 8 left: 22 passes in all. It has no shadow.

/// The kinds of copies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// `CHalluPC`: it stands for a while and then fades.
    Stays,
    /// `CBlurPC`: it fades at once; left only of one who walks or is in a motion (the client leaves none of
    /// one whose state is standing, sitting, standing ready or waiting to counter).
    Blur,
}

impl Kind {
    /// How much of a copy shows after so many passes of its life (of 255); nothing once it is gone.
    pub fn alpha(self, passes: i32) -> Option<u8> {
        match self {
            Kind::Stays => {
                let lost = (passes / 4 - 26).max(0) * 8;
                (passes <= 192).then_some((180 - lost) as u8)
            }
            Kind::Blur => (passes <= 22).then_some((180 - passes.max(0) * 8) as u8),
        }
    }
}

/// What the client's `EF_MAKEBLUR` does at a tick, having been told `colour` (`AM_SETCOUNT` with 0xFFFF or
/// 0xFFFE and the state's own number, which its handler turns into a colour: `crate::state_effects::blur`):
/// the colour it writes on the one it is on, and whether it leaves an afterimage in that colour. Told -1
/// (Fury) it leaves none and flickers the body pale red, a paler one once in fourteen (`lot` is the client's
/// `rand() % under`); told -2, a pale yellow. Otherwise the body is in the colour told, and every fifth tick
/// leaves an afterimage. (One whose colour a tint of the client's has saved, `savedTintChannels`, is mixed
/// with that tint: not done here.)
pub fn blur(colour: [f32; 3], tick: i32, lot: &mut dyn FnMut(u32) -> u32) -> ([u8; 3], bool) {
    if colour[0] == -1.0 {
        let pale = if lot(14) > 0 { 200 } else { 250 };
        return ([250, pale, pale], false);
    }
    if colour[0] == -2.0 {
        return ([250, 250, 200], false);
    }
    (colour.map(|part| part as i32 as u8), tick % 5 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_copy_stands_for_a_while_and_then_fades_by_eight_every_fourth_pass() {
        let alpha = |passes| Kind::Stays.alpha(passes);
        assert_eq!((alpha(0), alpha(107)), (Some(180), Some(180)));
        assert_eq!((alpha(108), alpha(111), alpha(112)), (Some(172), Some(172), Some(164)));
        // The last it shows is 4; with less than 8 left it is gone at its next pass.
        assert_eq!((alpha(191), alpha(192), alpha(193)), (Some(12), Some(4), None));
    }

    #[test]
    fn an_afterimage_fades_by_eight_at_every_pass() {
        let alpha = |passes| Kind::Blur.alpha(passes);
        assert_eq!((alpha(0), alpha(1), alpha(22), alpha(23)), (Some(180), Some(172), Some(4), None));
    }

    #[test]
    fn the_blur_colours_the_body_and_leaves_an_afterimage_every_fifth_tick() {
        let mut never = |_: u32| 1;
        assert_eq!((blur([200.0, 200.0, 0.0], 5, &mut never), blur([200.0, 200.0, 0.0], 6, &mut never)), (([200, 200, 0], true), ([200, 200, 0], false)));
        // Fury leaves none: a pale red, paler once in fourteen.
        assert_eq!((blur([-1.0; 3], 5, &mut never), blur([-1.0; 3], 5, &mut |_| 0)), (([250, 200, 200], false), ([250, 250, 250], false)));
        assert_eq!(blur([-2.0; 3], 0, &mut never), ([250, 250, 200], false));
    }
}
