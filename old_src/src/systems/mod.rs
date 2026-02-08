// systems/mod.rs - Re-exports all systems for clean imports
//
// This module acts as a "facade" - it hides the internal organization
// of systems and presents a clean public API. External code just does
// `use crate::systems::*` without knowing about individual files.
//
// SYSTEM CATEGORIES:
// 1. Global Startup: Run once when app starts (camera, assets, databases)
// 2. State OnEnter/OnExit: Run once when entering/leaving a state
// 3. State Update: Run every frame while in a specific state
// 4. Observers: Run when specific events are triggered

mod spawn;
mod combat;
mod health;
mod game_state;
mod render;
mod audio;
mod animation;
mod menu;
mod damage_popup;

// =============================================================================
// GLOBAL STARTUP SYSTEMS (run once at app start, before any state)
// =============================================================================
pub use spawn::{spawn_camera, setup_attacks, load_sprite_sheets};
pub use audio::{setup_audio, GameAudio};

// =============================================================================
// MENU STATE SYSTEMS
// =============================================================================
pub use menu::{spawn_menu_ui, handle_menu_button, cleanup_menu_ui};

// =============================================================================
// BATTLE STATE SYSTEMS
// =============================================================================
// OnEnter(Battle)
pub use spawn::spawn_soldiers;

// Update (during Battle)
pub use combat::{update_attack_cooldowns, attack_system, cleanup_finished_attacks};
pub use health::{death_check_system, death_animation_system, check_battle_end};
pub use animation::{animation_system, animation_switcher_system, animation_finished_system};
pub use render::render_health_bars;
pub use damage_popup::update_damage_popups;

// =============================================================================
// GAME OVER STATE SYSTEMS
// =============================================================================
pub use menu::{spawn_gameover_ui, handle_restart_button, cleanup_gameover_ui};

// =============================================================================
// OBSERVERS (run when events are triggered, regardless of state)
// =============================================================================
pub use audio::on_damage;
pub use animation::on_damage_animation;
pub use damage_popup::on_damage_spawn_popup;

// =============================================================================
// LEGACY (kept for compatibility, may be removed)
// =============================================================================
pub use game_state::game_over_system;
