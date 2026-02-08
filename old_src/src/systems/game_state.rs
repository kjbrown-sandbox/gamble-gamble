// systems/game_state.rs - Game over state handling

use bevy::prelude::*;
use crate::resources::GameStatus;

/// Game over system - runs every frame to handle the game-over state.
/// This is where you'd trigger animations, disable inputs, or transition to a menu.
/// Think of this as a state machine, and we're in "game over" state.
pub fn game_over_system(
    game_status: Res<GameStatus>,
) {
    if game_status.is_over {
        match game_status.result {
            Some(true) => {
                // Player won - you could trigger victory animations here
            },
            Some(false) => {
                // Player lost - you could trigger defeat animations here
            },
            None => {
                // Game is marked over but no result? Shouldn't happen
            }
        }
    }
}
