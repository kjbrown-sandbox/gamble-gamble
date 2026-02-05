// systems/health.rs - Health display and death checking

use bevy::prelude::*;
use crate::components::{Health, Team, Soldier};
use crate::resources::GameStatus;

/// Health display system - runs every frame to show current health.
/// This helps us debug and see the game is working.
pub fn health_display_system(
    query: Query<(&Health, &Team), With<Soldier>>,
) {
    // Collect health for debugging - print occasionally to avoid spam
    let mut healths = Vec::new();
    for (_health, _team) in query.iter() {
        healths.push(_health.current);
    }
    
    // Only print sometimes (every 60 frames = roughly once per second at 60fps)
    // This is a crude way to reduce console spam
    if healths.len() == 2 && healths.iter().sum::<i32>() % 120 == 0 {
        println!("Player HP: {}, Enemy HP: {}", healths[0], healths[1]);
    }
}

/// Death check system - runs every frame to detect dead soldiers and end the game.
/// When a soldier dies, they're despawned from the world.
/// When all soldiers on one team are dead, the game ends.
pub fn death_check_system(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Team), With<Soldier>>,
    mut game_status: ResMut<GameStatus>,
) {
    // First, despawn any dead soldiers and track which teams have survivors
    let mut player_has_alive = false;
    let mut enemy_has_alive = false;

    for (entity, health, team) in query.iter() {
        if health.current <= 0 {
            // Soldier is dead - remove from world
            commands.entity(entity).despawn();
        } else {
            // Soldier is alive - track that this team has survivors
            if team.is_player {
                player_has_alive = true;
            } else {
                enemy_has_alive = true;
            }
        }
    }

    // Check win condition: if one team has no alive soldiers, game is over
    if !player_has_alive || !enemy_has_alive {
        if !game_status.is_over {
            game_status.is_over = true;
            
            // Determine who won
            if player_has_alive {
                game_status.result = Some(true); // Player wins
                println!("YOU WIN!");
            } else {
                game_status.result = Some(false); // Player loses
                println!("YOU LOSE!");
            }
        }
    }
}
