// systems/mod.rs - Re-exports all systems for clean imports

mod spawn;
mod combat;
mod health;
mod game_state;
mod render;

pub use spawn::spawn_soldiers;
pub use combat::attack_system;
pub use health::health_display_system;
pub use health::death_check_system;
pub use game_state::game_over_system;
pub use render::render_health_bars;
