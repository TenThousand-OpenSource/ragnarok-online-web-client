//! The probabilities' entry of the list of windows.

use super::*;

/// The question the menu's icon of the probabilities asks.
pub(in crate::web) static PROBABILITY_ASKS: Window = Window::question("probability-asks", |app| app.probability.asking, App::probability_question);
