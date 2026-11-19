//! The colours `RenderTeiRect` has for the bands whose primitive's `size` picks one (its form that takes a
//! piece of 21 of the picture's width): the walls of light, the winds, the quarters.

/// The colour and the blend `RenderTeiRect` gives the pieces of a band by its primitive's `size`, as a whole
/// number: white and added for one it has no entry for. (Its entries 3 and 100 change more than the colour, a
/// fixed alpha and a lower edge of the picture; no kind here is given them, and they are not ported.)
pub(super) fn by_size(size: f32) -> ([u8; 3], bool) {
    let mixed = |colour| (colour, false);
    let added = |colour| (colour, true);
    match size as i32 {
        0 => mixed([255; 3]),
        1 => added([255, 175, 175]),
        2 => mixed([195, 195, 255]),
        4 => added([100, 100, 255]),
        6 => added([255, 100, 100]),
        7 => added([100, 255, 100]),
        8 => added([255, 255, 170]),
        9 => added([140, 210, 140]),
        10 => added([255, 89, 182]),
        11 => added([170, 170, 255]),
        12 => mixed([50; 3]),
        13 => mixed([255, 50, 255]),
        14 => mixed([120, 120, 255]),
        15 => added([10, 255, 10]),
        16 => added([20, 255, 20]),
        17 => added([0, 0, 255]),
        18 => added([255, 0, 0]),
        19 => added([0, 255, 0]),
        20 => added([255, 255, 0]),
        21 => added([255, 0, 255]),
        22 => added([0, 255, 255]),
        23 => mixed([41, 94, 188]),
        24 => added([10, 66, 114]),
        25 => added([35, 146, 185]),
        26 => added([164, 205, 254]),
        27 => added([84, 169, 205]),
        28 => added([108, 26, 255]),
        29 => added([177, 255, 58]),
        30 => added([85, 30, 185]),
        31 => added([244, 187, 124]),
        32 => mixed([0; 3]),
        40 => added([70, 70, 255]),
        41 => added([30, 30, 255]),
        42 => mixed([43, 40, 46]),
        43 => mixed([125, 182, 160]),
        44 => mixed([180, 55, 30]),
        45 => added([65, 141, 231]),
        46 => added([164, 47, 29]),
        47 => added([172, 154, 56]),
        48 => added([80, 20, 20]),
        49 => added([200, 130, 84]),
        50 => added([255, 198, 91]),
        _ => added([255; 3]),
    }
}
