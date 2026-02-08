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

/// SpriteSheets resource - stores handles to loaded sprite sheets and their layouts
/// This prevents loading the same assets multiple times
#[derive(Resource)]
pub struct SpriteSheets {
    pub slime_jump_idle: Handle<Image>,
    pub slime_attack: Handle<Image>,
    pub slime_move_small_jump: Handle<Image>,
    pub slime_hurt: Handle<Image>,
    pub slime_death: Handle<Image>,
    
    pub jump_idle_layout: Handle<TextureAtlasLayout>,
    pub attack_layout: Handle<TextureAtlasLayout>,
    pub move_small_jump_layout: Handle<TextureAtlasLayout>,
    pub hurt_layout: Handle<TextureAtlasLayout>,
    pub death_layout: Handle<TextureAtlasLayout>,
}
