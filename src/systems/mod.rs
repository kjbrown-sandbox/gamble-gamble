// systems/mod.rs - Re-exports all systems for clean imports
//
// This module acts as a "facade" - it hides the internal organization
// of systems and presents a clean public API. External code just does
// `use crate::systems::*` without knowing about individual files.

mod spawn;
mod combat;
mod health;
mod game_state;
mod render;
mod audio;

// Startup systems (run once at app start)
pub use spawn::{setup_attacks, spawn_soldiers};
pub use audio::{setup_audio, on_damage, GameAudio};

// Update systems (run every frame)
pub use combat::{update_attack_cooldowns, attack_system, cleanup_finished_attacks};
pub use health::death_check_system;
pub use game_state::game_over_system;
pub use render::render_health_bars;
