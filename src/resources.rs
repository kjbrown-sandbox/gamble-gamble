// resources.rs - Global game resources (singleton data)
// Resources exist once for the entire game, unlike components which are per-entity.

use bevy::prelude::*;

/// GameStatus resource - tracks the overall game state.
/// There's only one of these for the entire game.
#[derive(Resource)]
pub struct GameStatus {
    pub is_over: bool,
    /// Some(true) = player wins, Some(false) = player loses, None = game ongoing
    pub result: Option<bool>,
}

impl Default for GameStatus {
    fn default() -> Self {
        GameStatus {
            is_over: false,
            result: None,
        }
    }
}
