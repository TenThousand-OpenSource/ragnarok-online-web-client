//! `PP_3DTEXTURE` (15): one picture standing in the world, moved by `MoveEffectPrimitive` as the crossed
//! pictures are and drawn by `DrawEffectPrimitive15`.
//!
//! Ported of `MoveEffectPrimitive` beyond what the crossed pictures use (`master.rs`): the speeds along the
//! three directions (`pattern & 0x100`), the end on reaching the place its effect aims at (`pattern & 0x400`),
//! the turn to the camera (`pattern & 0x40`), the turn about the upright by a speed (`longSpeed`, `longAccel`)
//! and the colour that goes round (`pattern & 0x800`, `tint.rs`).
//!
//! Not ported, as no effect ported uses them: a tip that changes by a speed (`latiSpeed`), the turn by the
//! camera's pitch alone (`0x200`), the picture that slides (`2`), the height that swings (`0x20`), the colours
//! that change without going round, and the picture drawn from its place to one side only by `pattern & 8`.
//!
//! The client keeps two turns for it: one the effect makes, along whose forward it goes (`transform`), and one
//! made anew every tick from its roll, latitude and longitude, by which it is drawn (`rotTransform`). Here its
//! latitude and longitude serve for both: of the effects ported, those that move it turn it to the camera, and
//! those that turn it do not move it.

use super::solid::add;
use super::*;

impl Prim {
    /// `MoveEffectPrimitive` for a picture in the world. Gives back whether it lives on.
    pub(super) fn plate_flight(&mut self, master: &Master) -> bool {
        self.take_given();
        self.width_speed += self.width_accel;
        self.speed += self.accel;
        self.height_speed += self.height_accel;
        self.long_speed += self.long_accel;
        if self.by_axes {
            self.drift();
        } else {
            self.go_forward();
        }
        if self.reached(master) {
            return false;
        }
        self.width += self.width_speed;
        self.height += self.height_speed;
        self.longitude += self.long_speed;
        if self.height > 100.0 {
            (self.height_accel, self.height_speed) = (0.0, 0.0);
        }
        if let Some(cycle) = &mut self.cycle {
            cycle.turn(&mut self.colour);
        }
        self.next_picture()
    }

    /// `DrawEffectPrimitive15`: the picture on a flat that stands along its own y (down) and z, its height up
    /// and down from its middle and its width to each side; one that flickers reaches twice its height up from
    /// its middle instead (the client reads the same `pattern & 0x80` for both). It is turned by its roll about
    /// its own z, then tipped and turned as the other kinds are (`MakeZRotation`, `AppendXRotation`,
    /// `AppendYRotation`). The picture's top is at the upper edge, its left where the flat's z is.
    ///
    /// One turned to the camera (`pattern & 0x40`: its longitude the camera's yaw and a quarter, its roll the
    /// camera's pitch) has its z along the screen's `right`; its y is taken to lie down the screen, so that the
    /// picture stands upright there.
    pub(super) fn plate(&self, right: [f32; 3], up: [f32; 3]) -> Vec<Lit> {
        let middle = self.middle([0.0; 3]);
        let (top, bottom) = if self.flickers { (-2.0 * self.height, 0.0) } else { (-self.height, self.height) };
        let (sin, cos) = (sine(self.roll), cosine(self.roll));
        let corner = |y: f32, z: f32, u: f32, v: f32| {
            let at = if self.faces_camera { std::array::from_fn(|axis| middle[axis] + right[axis] * z - up[axis] * y) } else { add(middle, turn(self.latitude, self.longitude, [-y * sin, y * cos, z])) };
            (at, [u, v])
        };
        let (first, second, third, fourth) = (corner(top, self.width, 0.0, 0.0), corner(top, -self.width, 1.0, 0.0), corner(bottom, self.width, 0.0, 1.0), corner(bottom, -self.width, 1.0, 1.0));
        vec![Lit { picture: self.shown(), corners: [first, second, third, fourth, third, second], alpha: self.alpha }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 3], b: [f32; 3]) -> bool {
        (0..3).all(|axis| (a[axis] - b[axis]).abs() < 0.001)
    }

    const RIGHT: [f32; 3] = [1.0, 0.0, 0.0];
    const UP: [f32; 3] = [0.0, 0.6, -0.8];

    #[test]
    fn a_picture_in_the_world_stands_upright_across_the_north_until_it_is_turned() {
        let mut plate = Prim::new(Shape::Texture3d, "a");
        (plate.width, plate.height, plate.put.1) = (3.0, 2.0, -5.0);
        let lit = plate.plate(RIGHT, UP);
        assert_eq!((lit.len(), lit[0].picture, lit[0].alpha), (1, "a", 254.0));
        // Its top edge 2 over its middle, which is 5 up; its left end 3 to the north.
        assert!(close(lit[0].corners[0].0, [0.0, 7.0, 3.0]) && lit[0].corners[0].1 == [0.0, 0.0]);
        assert!(close(lit[0].corners[3].0, [0.0, 3.0, -3.0]) && lit[0].corners[3].1 == [1.0, 1.0]);
        // Rolled a quarter it lies flat: what was up is to one side, level with its middle.
        plate.roll = 90.0;
        let lit = plate.plate(RIGHT, UP);
        assert!(close(lit[0].corners[0].0, [2.0, 5.0, 3.0]) && close(lit[0].corners[3].0, [-2.0, 5.0, -3.0]));
    }

    #[test]
    fn a_picture_turned_to_the_camera_lies_along_the_screens_right_and_up() {
        let mut plate = Prim::new(Shape::Texture3d, "");
        (plate.faces_camera, plate.width, plate.height) = (true, 4.0, 1.0);
        let lit = plate.plate(RIGHT, UP);
        assert!(close(lit[0].corners[0].0, [4.0, 0.6, -0.8]) && close(lit[0].corners[3].0, [-4.0, -0.6, 0.8]));
        // One that flickers reaches twice its height up from its middle.
        plate.flickers = true;
        let lit = plate.plate(RIGHT, UP);
        assert!(close(lit[0].corners[0].0, [4.0, 1.2, -1.6]) && close(lit[0].corners[3].0, [-4.0, 0.0, 0.0]));
    }

    #[test]
    fn it_grows_takes_what_it_was_given_at_its_tick_and_runs_through_its_pictures() {
        // As the ring of Turn Undead: it widens faster and faster, then slowly and slowing.
        let mut ring = Prim::new(Shape::Texture3d, "");
        (ring.width_speed, ring.width_accel, ring.height_speed) = (1.5, 0.15, 1.0);
        ring.changes = Some(Change { width_speed: 0.5, width_accel: -0.1, ..Change::at(2) });
        ring.tick(100);
        ring.tick(100);
        assert!((ring.width - (1.65 + 1.8)).abs() < 0.001 && (ring.height - 2.0).abs() < 0.001);
        ring.tick(100);
        // Its height speed was not given anew and stays.
        assert!((ring.width - (3.45 + 0.4)).abs() < 0.001 && (ring.height - 3.0).abs() < 0.001);
        // Six pictures, one a tick, over and over.
        let mut ball = Prim::new(Shape::Texture3d, "");
        (ball.pictures, ball.anim_speed) = (&["a", "b", "c", "d", "e", "f"], 1);
        let shown: Vec<_> = (0..7).map(|_| (ball.tick(100), ball.shown()).1).collect();
        assert_eq!(shown, ["b", "c", "d", "e", "f", "a", "b"]);
    }

    #[test]
    fn it_turns_by_its_speed_and_its_colour_goes_round() {
        // As the mark of Sense: 4.5 degrees a tick, its green and blue falling by 15.
        let mut mark = Prim::new(Shape::Texture3d, "");
        (mark.longitude, mark.long_speed, mark.colour) = (45.0, 4.5, [250.0, 150.0, 150.0]);
        mark.cycle = Some(Cycle { speeds: [0.0, -15.0, -15.0], delay: 20, waited: 0 });
        mark.tick(100);
        assert_eq!((mark.longitude, mark.colour), (49.5, [250.0, 135.0, 135.0]));
        mark.tick(100);
        assert_eq!((mark.longitude, mark.colour), (54.0, [250.0, 120.0, 120.0]));
        // One given neither stays as it is.
        let mut still = Prim::new(Shape::Texture3d, "");
        still.tick(100);
        assert_eq!((still.longitude, still.colour), (0.0, [255.0; 3]));
    }

    #[test]
    fn it_flies_to_the_place_its_effect_aims_at_and_ends_there() {
        // As the ball of Jupitel Thunder: 10 to the east in 4 ticks.
        let mut ball = Prim::new(Shape::Texture3d, "");
        (ball.seeks, ball.faces_camera, ball.duration) = (true, true, 20);
        (ball.longitude, ball.speed) = (-faced(10.0, 0.0), -2.5);
        let mut master = Master::left(100);
        master.aim = [10.0, 0.0, 0.0];
        assert!(ball.tick_on(&mut master, 0, &|_, _| f32::NEG_INFINITY));
        assert!(close(ball.moved, [2.5, 0.0, 0.0]));
        assert!(ball.tick_on(&mut master, 0, &|_, _| f32::NEG_INFINITY));
        // Within 3 of the place at its third step.
        assert!(!ball.tick_on(&mut master, 0, &|_, _| f32::NEG_INFINITY));
    }
}
