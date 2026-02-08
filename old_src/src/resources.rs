// resources.rs - Global game resources (singleton data)
// Resources exist once for the entire game, unlike components which are per-entity.

use bevy::prelude::*;

// =============================================================================
// GAME STATE (State Machine)
// =============================================================================
// Bevy's States system provides a built-in state machine for controlling game flow.
// Unlike a simple Resource with an enum field, States integrate deeply with Bevy:
//
// WHY USE States INSTEAD OF A RESOURCE?
// 1. OnEnter/OnExit systems: Run code exactly once when entering/leaving a state
// 2. run_if(in_state(...)): Conditionally run systems only in certain states
// 3. NextState<T>: Thread-safe way to request state transitions
// 4. Automatic change detection: Bevy handles all the transition logic
//
// REQUIRED DERIVES:
// - States: The main derive that enables state functionality
// - Debug: For printing the state (useful for debugging)
// - Clone, Copy: States must be cheaply copyable (they're just enum variants)
// - Default: Tells Bevy which state to start in (via #[default] attribute)
// - PartialEq, Eq: For comparing states
// - Hash: Required by Bevy's internal state storage (uses HashMaps)
//
// The #[default] attribute marks which variant is the initial state.

/// GameState - the top-level state machine for our game.
///
/// The game flows: Menu → Battle → GameOver → Menu (repeat)
///
/// Each state has different systems running:
/// - Menu: Show "Fight!" button, no combat
/// - Battle: Soldiers fight, damage popups appear
/// - GameOver: Show result, "Play Again" button
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum GameState {
    /// Initial state: Show menu with "Fight!" button.
    /// #[default] means the game starts here automatically.
    #[default]
    Menu,

    /// Active gameplay: Soldiers are fighting.
    /// Entered when player clicks "Fight!" button.
    Battle,

    /// Battle has ended: One team was eliminated.
    /// Shows win/lose message and "Play Again" button.
    GameOver,
}

/// GameStatus resource - tracks the battle result.
///
/// NOTE: This works alongside GameState. GameState controls WHICH systems run,
/// while GameStatus stores DATA about the outcome (win/lose).
///
/// You might wonder: "Why not just use GameState::GameOver(result)?"
/// States in Bevy must be simple enums without data. If you need to store
/// data about a state, use a separate Resource like this.
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
