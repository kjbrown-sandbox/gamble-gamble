// systems/render.rs - Rendering health information on screen

use bevy::prelude::*;
use crate::components::{HealthDisplay, GameOverText, Soldier, Health};
use crate::resources::GameStatus;

/// Update health text displays and game over message.
/// Uses ParamSet to handle multiple Text queries that would otherwise conflict.
pub fn render_health_bars(
    mut params: ParamSet<(
        Query<(&HealthDisplay, &mut Text)>,
        Query<&mut Text, With<GameOverText>>,
    )>,
    healths: Query<(&Health, Entity), With<Soldier>>,
    game_status: Res<GameStatus>,
) {
    // Build a map of entity -> health for quick lookup
    let health_map: std::collections::HashMap<Entity, i32> = healths
        .iter()
        .map(|(health, entity)| (entity, health.current))
        .collect();

    // First: Update each health display
    for (display, mut text) in params.p0().iter_mut() {
        if let Some(&hp) = health_map.get(&display.soldier_entity) {
            let label = if display.is_player { "Player" } else { "Enemy" };
            text.0 = format!("{} HP: {}", label, hp);
        }
    }

    // Second: Update game over message
    if game_status.is_over {
        for mut text in params.p1().iter_mut() {
            text.0 = match game_status.result {
                Some(true) => "YOU WIN!".to_string(),
                Some(false) => "YOU LOSE!".to_string(),
                _ => String::new(),
            };
        }
    }
}


