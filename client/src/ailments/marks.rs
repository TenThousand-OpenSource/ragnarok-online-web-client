//! The marks that stay by one who ails for as long as the state holds (`CMsgEffect`s the actor makes for
//! itself: `AM_MAKE_ICEEFFECT`, `AM_MAKE_CURSE_EFFECT`…), and the ice that breaks as a frozen one thaws.

use super::{Ailing, CURSE, FREEZE, SILENCE, SLEEP, STUN};

/// A mark that stays by the one ailing for as long as its state holds: a sprite of the client's
/// (`CGameMode::SpawnMsgEffect`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Ice,
    Curse,
    Sleep,
    Stun,
    Silence,
}

impl Mark {
    /// The sprite it is, under the client's folder of sprites, and the first of its actions it shows
    /// (`SpawnMsgEffect`: silence is the eleventh action of the emotions' sprite).
    pub fn sprite(self) -> (&'static str, usize) {
        match self {
            Mark::Ice => ("이팩트/얼음땡", 0),
            Mark::Curse => ("이팩트/status-curse", 0),
            Mark::Sleep => ("이팩트/status-sleep", 0),
            Mark::Stun => ("이팩트/status-stun", 0),
            Mark::Silence => ("이팩트/emotion", 10),
        }
    }

    /// How far over the feet of its owner it is kept (`CMsgEffect::FollowOwner`).
    pub fn above(self) -> f32 {
        match self {
            Mark::Ice => 0.0,
            Mark::Curse => 15.0,
            Mark::Sleep | Mark::Stun | Mark::Silence => 20.0,
        }
    }

    /// How much of it shows: the curse is half there (`AM_SETMASTERGID`: its alpha 0x7F), the rest whole.
    pub fn alpha(self) -> f32 {
        if self == Mark::Curse { 127.0 / 255.0 } else { 1.0 }
    }

    /// Whether its action follows the way its owner faces, one action to each of the eight ways (the curse:
    /// `FollowOwner` adds `Get8Dir` of the owner's angle to its action).
    pub fn faces(self) -> bool {
        self == Mark::Curse
    }
}

/// The sprite of the ice that breaks as a frozen one thaws, and which of its actions it is
/// (`MAKETYPE_ICEBREAK_EFFECT`). It plays once, 1.2 pictures to each of the client's frames of 24 milliseconds
/// (`CMsgEffect::OnProcess`, `MET_ICEBREAKEFFECT`), where its owner stood as it broke.
pub const ICE_BREAKS: (&str, usize) = ("이팩트/얼음땡", 1);
pub const ICE_BREAK_PACE: f64 = 1.2;

/// The marks someone in those states has by them. (The client makes and removes each as a state comes and
/// goes; none is made twice, so what there is follows from the states alone.)
pub fn marks(ailing: Ailing) -> Vec<Mark> {
    let mut marks = Vec::new();
    match ailing.body {
        FREEZE => marks.push(Mark::Ice),
        STUN => marks.push(Mark::Stun),
        SLEEP => marks.push(Mark::Sleep),
        _ => {}
    }
    if ailing.health & CURSE != 0 {
        marks.push(Mark::Curse);
    }
    if ailing.health & SILENCE != 0 {
        marks.push(Mark::Silence);
    }
    marks
}
