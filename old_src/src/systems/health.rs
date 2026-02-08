// systems/health.rs - Health display and death checking
//
// This module handles:
// 1. Detecting when soldiers die (health <= 0)
// 2. Starting death animations instead of immediate despawn
// 3. Despawning soldiers after death animation completes
// 4. Checking if battle has ended (one team eliminated)
// 5. Triggering state transition to GameOver
//
// KEY CONCEPT - STATE TRANSITIONS:
// When one team is eliminated, we need to transition to GameOver state.
// We use NextState<GameState> to request this transition. The actual
// transition happens at a safe point in the frame, after all systems run.

use bevy::prelude::*;
use crate::components::{Health, Team, Soldier, Dying, AnimationState, AnimationType};
use crate::resources::{GameStatus, GameState};

/// Death check system - detects dead soldiers and starts their death animation.
///
/// DEATH ANIMATION FLOW:
/// 1. Soldier health drops to 0 or below
/// 2. This system adds the Dying component and sets Death animation
/// 3. The death_animation_system waits for animation to finish
/// 4. After animation, the soldier is despawned and game over is checked
///
/// We use the Dying component as a marker to:
/// - Prevent combat systems from targeting dying soldiers
/// - Prevent hurt animations from playing (they're already dying)
/// - Track that we've already started the death process
pub fn death_check_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Health, &mut AnimationState), (With<Soldier>, Without<Dying>)>,
) {
    for (entity, health, mut anim_state) in query.iter_mut() {
        if health.current <= 0 {
            // Mark as dying and start death animation
            commands.entity(entity).insert(Dying);
            anim_state.change_to(AnimationType::Death);
        }
    }
}

/// System to despawn soldiers after their death animation finishes.
///
/// This runs after death_check_system and animation_system.
/// It waits for the death animation to complete before despawning,
/// giving a visual indication of death before the entity disappears.
pub fn death_animation_system(
    mut commands: Commands,
    query: Query<(Entity, &AnimationState, &Team), With<Dying>>,
) {
    for (entity, anim_state, _team) in query.iter() {
        // Wait for death animation to finish
        if anim_state.finished && anim_state.animation_type == AnimationType::Death {
            // Despawn the dead soldier and all its children (like AttackInstance)
            commands.entity(entity).despawn();
        }
    }
}

/// Checks if the battle has ended (one team eliminated) and transitions to GameOver.
///
/// This system was separated from death_animation_system to make it clearer
/// that state transitions are their own concern. It also makes the code easier
/// to understand and modify.
///
/// HOW STATE TRANSITIONS WORK:
/// 1. We detect that the battle should end (one team has no living soldiers)
/// 2. We update GameStatus with the result (win/lose)
/// 3. We call next_state.set(GameState::GameOver) to request the transition
/// 4. Bevy processes this request after all Update systems finish
/// 5. OnExit(Battle) systems run (cleanup)
/// 6. OnEnter(GameOver) systems run (show result UI)
///
/// The transition doesn't happen immediately - it's queued and processed safely.
/// This prevents issues where some systems see the old state and others see the new.
pub fn check_battle_end(
    alive_query: Query<&Team, (With<Soldier>, Without<Dying>)>,
    mut game_status: ResMut<GameStatus>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Don't check if game is already over (prevents repeated transitions)
    if game_status.is_over {
        return;
    }

    // Count surviving soldiers (not dying) on each team
    let mut player_has_alive = false;
    let mut enemy_has_alive = false;

    for team in alive_query.iter() {
        if team.is_player {
            player_has_alive = true;
        } else {
            enemy_has_alive = true;
        }
    }

    // Game is over when one team has no living (non-dying) soldiers
    if !player_has_alive || !enemy_has_alive {
        game_status.is_over = true;

        if player_has_alive {
            game_status.result = Some(true); // Player wins
            println!("YOU WIN!");
        } else {
            game_status.result = Some(false); // Player loses
            println!("YOU LOSE!");
        }

        // Request transition to GameOver state
        // This queues the transition to happen after all Update systems finish
        next_state.set(GameState::GameOver);
    }
}
