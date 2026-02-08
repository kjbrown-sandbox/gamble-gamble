// lib.rs - Public API for the game crate
// Re-exports components and systems so main.rs stays clean

pub mod components;
pub mod resources;
pub mod systems;

pub use components::*;
pub use resources::*;
