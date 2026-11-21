//! How the client shows a Granny model on the map (`C3dGrannyGameActor`): which of its five actions it is
//! in and from which file that comes, how fast it plays, how it is turned and where it stands, and the grey
//! each corner is lit to.

use crate::vector::{Vector, dot, normalize};

/// The actions the client has names for (`g_animationNames`), in its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Stand,
    Move,
    Attack,
    Dead,
    Damage,
}

impl Action {
    pub const OTHERS: [Action; 4] = [Action::Move, Action::Attack, Action::Dead, Action::Damage];

    fn word(self) -> &'static str {
        ["stand", "move", "attack", "dead", "damage"][self as usize]
    }

    /// Whether it goes round and round. (`SetAction` loops whatever is not asked for once; who asks for
    /// what once was not followed through the client: here a blow, a flinch and a fall play once.)
    pub fn looped(self) -> bool {
        matches!(self, Action::Stand | Action::Move)
    }
}

/// The guild flag's job: the one model the client draws by its other way (`actorType == 2`), with the
/// guild's emblem on its second mesh.
pub const FLAG: u16 = 722;

/// A model's file among the client's, by its name in the table of jobs (`C3dGrannyModelRes`'s `model\3dmob\`).
pub fn file(name: &str) -> String {
    format!("data/model/3dmob/{}", name.to_ascii_lowercase())
}

/// The number in a model's name, after its `_` (`C3dGrannyModelRes::Load`: up to four digits, as `atoi`
/// reads them): the client's own number for the model, by which the files of its other actions are named.
pub fn number(name: &str) -> Option<usize> {
    let after = &name[name.find('_')? + 1..];
    let digits: String = after.chars().take_while(|letter| *letter != '.').take(4).take_while(char::is_ascii_digit).collect();
    Some(digits.parse().unwrap_or(0))
}

/// The file of another action than standing (`C3dGrannyModelRes::PlayAnimation`: `model\3dmob_bone\%d_%s.gr2`
/// for the numbers 0 to 9); none for the flag, number 1, whose other actions the client never reads.
pub fn action_file(name: &str, action: Action) -> Option<String> {
    let number = number(name).filter(|number| *number <= 9 && *number != 1)?;
    (action != Action::Stand).then(|| format!("data/model/3dmob_bone/{number}_{}.gr2", action.word()))
}

/// The seconds of its animation a model has played so many milliseconds after its action began: the client
/// adds twice the time gone by (`Render`: `GrannyGetSecondsElapsed(...) * 2`).
pub fn played(milliseconds: f64) -> f32 {
    (milliseconds.max(0.0) / 1000.0 * 2.0) as f32
}

/// How long an animation of so many seconds takes on the screen, in milliseconds.
pub fn lasts(duration: f32) -> f64 {
    f64::from(duration) * 1000.0 / 2.0
}

/// A place of the model as it stands in the world (east, up, north) from where the model itself stands:
/// the client lays a model whose name has "90" in it back by a quarter turn (what is up in the model is up
/// in the world; another is turned half round: its y is the world's up), turns it by its facing — half a
/// turn and the angle of the direction the server gave, `(direction + 4) * 45` degrees — and moves it to
/// the actor (`RenderLit`, `RenderShadowed`, with `OnProcess` for the angle; the client's own space has y
/// down and is given here y up).
pub fn stood(name: &str, direction: u8, [x, y, z]: Vector) -> Vector {
    let (sine, cosine) = (f32::from(direction % 8 + 4) * 45.0).to_radians().sin_cos();
    let (up, north) = if name.contains("90") { (z, y) } else { (y, -z) };
    [x * cosine - north * sine, up, x * sine + north * cosine]
}

/// Where the model stands from the actor's feet: the flag on them, the others 2 west and 2 south of them
/// (`RenderLit` takes 2 from x and z).
pub fn offset(job: u16) -> Vector {
    if job == FLAG { [0.0; 3] } else { [-2.0, 0.0, -2.0] }
}

/// The view the light is worked out from: where the eye is and how the camera looks, in the world (east,
/// up, north), and where the middle of the map's ground is, which is the nought of the client's own space.
pub struct Seen {
    pub eye: Vector,
    pub right: Vector,
    pub up: Vector,
    pub forward: Vector,
    pub middle: Vector,
}

/// The direction the light of a model is taken to come from, which the client then measures against the
/// corners' normals as the model has them, unturned.
///
/// The flag (`RenderShadowed`): the camera's forward put through the view's matrix with its move — so the
/// place of the client's nought as the camera sees it (right, down, on), one further on. The others
/// (`RenderLit`): the wind's direction put through the view's inverse with its move, which is the eye's
/// place in the client's space (east, down, north) but for the wind; the wind's own part, a length of
/// one against some hundreds, is left out here.
pub fn light(job: u16, seen: &Seen) -> Vector {
    let from_eye = crate::vector::subtract(seen.middle, seen.eye);
    if job == FLAG {
        normalize([dot(from_eye, seen.right), -dot(from_eye, seen.up), dot(from_eye, seen.forward) + 1.0])
    } else {
        normalize([-from_eye[0], from_eye[1], -from_eye[2]])
    }
}

/// The grey of a corner, of 255: by how squarely its normal meets the light, from 155 to 255 on the flag,
/// from 127 to 255 on the others (whose three colours the client adds 200 and 0x38 to, which is nothing
/// in a byte).
pub fn grey(job: u16, light: Vector, normal: Vector) -> u8 {
    let facing = dot(light, normal).clamp(0.0, 1.0);
    if job == FLAG { (facing * 100.0 + 155.0) as u8 } else { (facing * 128.0 + 127.0) as u8 }
}

/// A guild's emblem as the flag's second mesh shows it: the client puts its 24 by 24 pixels into a texture,
/// and a texture's sides are rounded up to powers of two (`TextureCache::Upload` with
/// `DeviceOwnerDX9::AdjustTextureDimensions`: 32 by 32), the picture in its top left corner; the mesh shows
/// the whole texture, so the emblem takes three quarters of it each way and the rest shows nothing. Gives
/// the texture's sides and its pixels (four bytes each). (What the client's texture holds beside the picture
/// is whatever a new one holds: taken here as nothing, see-through.)
pub fn emblem(width: usize, height: usize, pixels: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    if width == 0 || height == 0 || pixels.len() != width * height * 4 {
        return None;
    }
    let (wide, high) = (width.next_power_of_two(), height.next_power_of_two());
    let mut texture = vec![0; wide * high * 4];
    for row in 0..height {
        texture[row * wide * 4..(row * wide + width) * 4].copy_from_slice(&pixels[row * width * 4..(row + 1) * width * 4]);
    }
    Some((wide, high, texture))
}

/// How far to either side and up and down of its feet a model is pointed at, in the world's units: the
/// client's rectangle for it is as wide and high as the model's `baseHeight` and `topHeight` of 10 make
/// it, each the root of half its square (`C3dGrannyModelRes::ProjectBounds`). (The two models the client
/// gives 30 are not among its files.)
pub const REACH: f32 = 7.071_068;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_models_number_names_the_files_of_its_actions() {
        assert_eq!((number("Guildflag90_1.gr2"), number("Empelium90_0.gr2"), number("TREASUREBOX_2.gr2")), (Some(1), Some(0), Some(2)));
        assert_eq!((number("flag.gr2"), number("a_x.gr2")), (None, Some(0)));
        assert_eq!(file("Guildflag90_1.gr2"), "data/model/3dmob/guildflag90_1.gr2");
        assert_eq!(action_file("Kguardian90_7.gr2", Action::Attack).as_deref(), Some("data/model/3dmob_bone/7_attack.gr2"));
        assert_eq!(action_file("Kguardian90_7.gr2", Action::Stand), None);
        // The flag's other actions are never read.
        assert_eq!(action_file("Guildflag90_1.gr2", Action::Attack), None);
    }

    #[test]
    fn an_emblem_takes_the_top_left_of_a_texture_of_powers_of_two() {
        let pixels: Vec<u8> = (0..24 * 24 * 4).map(|at| (at % 251) as u8 + 1).collect();
        let (wide, high, texture) = emblem(24, 24, &pixels).unwrap();
        assert_eq!((wide, high, texture.len()), (32, 32, 32 * 32 * 4));
        // The second row of the picture begins the second row of the texture; beside and under it is nothing.
        assert_eq!(texture[32 * 4..32 * 4 + 24 * 4], pixels[24 * 4..48 * 4]);
        assert!(texture[24 * 4..32 * 4].iter().all(|byte| *byte == 0) && texture[24 * 32 * 4..].iter().all(|byte| *byte == 0));
        // One already so large is as it is; one that is none is none.
        assert_eq!(emblem(16, 16, &pixels[..1024]).map(|(wide, high, texture)| (wide, high, texture == pixels[..1024])), Some((16, 16, true)));
        assert_eq!(emblem(24, 24, &pixels[..100]), None);
    }

    #[test]
    fn a_model_plays_twice_as_fast() {
        assert_eq!((played(500.0), played(-3.0)), (1.0, 0.0));
        assert_eq!(lasts(2.0), 1000.0);
    }

    fn near(a: Vector, b: Vector) -> bool {
        (0..3).all(|axis| (a[axis] - b[axis]).abs() < 1e-4)
    }

    #[test]
    fn a_model_is_laid_back_and_turned_by_its_facing() {
        // Facing south (the server's 4) a "90" model stands as it was made: its z is up, its y north.
        assert!(near(stood("Guildflag90_1.gr2", 4, [1.0, 2.0, 3.0]), [1.0, 3.0, 2.0]));
        // Facing north it is turned half round.
        assert!(near(stood("Guildflag90_1.gr2", 0, [1.0, 2.0, 3.0]), [-1.0, 3.0, -2.0]));
        // A model without "90" has its y up.
        assert!(near(stood("TREASUREBOX_2.gr2", 4, [1.0, 2.0, 3.0]), [1.0, 2.0, -3.0]));
        assert_eq!((offset(FLAG), offset(1288)), ([0.0; 3], [-2.0, 0.0, -2.0]));
    }

    #[test]
    fn a_corner_is_lit_by_how_it_meets_the_light() {
        let seen = Seen { eye: [0.0, 100.0, -100.0], right: [1.0, 0.0, 0.0], up: [0.0, 0.7071, 0.7071], forward: [0.0, -0.7071, 0.7071], middle: [0.0; 3] };
        // The others are lit from where the eye is, in the client's space: here above (its y is down) and south.
        assert!(near(light(1288, &seen), [0.0, -0.70710677, -0.70710677]));
        assert_eq!((grey(1288, [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]), grey(1288, [0.0, 0.0, 1.0], [0.0, 0.0, -1.0])), (255, 127));
        assert_eq!((grey(FLAG, [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]), grey(FLAG, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0])), (255, 155));
        // The flag's light is the middle of the map as the camera sees it: here straight on.
        assert!(near(light(FLAG, &seen), [0.0, 0.0, 1.0]));
    }
}
