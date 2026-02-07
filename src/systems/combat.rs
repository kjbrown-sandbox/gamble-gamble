// systems/combat.rs - Combat system
// Handles soldiers attacking each other.

use bevy::prelude::*;
use crate::components::{Health, Team, Soldier, AttackCooldown, DamageEvent};
use rand::Rng;

/// Attack system - runs every frame to handle combat.
/// 
/// This demonstrates an important Bevy pattern: when you need multiple queries
/// that would conflict (both accessing the same component), use ParamSet.
/// ParamSet ensures only one query runs at a time, preventing conflicts.
pub fn attack_system(
    mut param_set: ParamSet<(
        Query<(Entity, &mut AttackCooldown, &Health, &Team), With<Soldier>>,
        Query<&mut Health, With<Soldier>>,
    )>,
    time: Res<Time>,
    // Commands lets us queue operations to run after the system completes.
    // We use it here to "trigger" events via the observer pattern.
    mut commands: Commands,
) {
    let delta_time = time.delta_secs();
    let mut rng = rand::thread_rng();

    // First query: read soldier info and update cooldowns
    // We collect data here before mutating health
    let soldiers: Vec<(Entity, i32, bool)> = param_set
        .p0()
        .iter()
        .map(|(entity, _, health, team)| (entity, health.current, team.is_player))
        .collect();

    // Check if game is still ongoing
    let player_alive = soldiers.iter().any(|(_, hp, is_player)| *hp > 0 && *is_player);
    let enemy_alive = soldiers.iter().any(|(_, hp, is_player)| *hp > 0 && !*is_player);
    
    if !player_alive || !enemy_alive {
        return;
    }

    // Collect all attacks to be processed
    let mut attacks: Vec<Entity> = Vec::new();

    // Update cooldowns and determine targets
    for (_attacker_entity, mut cooldown, health, team) in param_set.p0().iter_mut() {
        let is_ready = cooldown.update(delta_time);
        
        if is_ready && health.current > 0 {
            // Find all living enemies
            let enemies: Vec<Entity> = soldiers
                .iter()
                .filter(|(_, hp, is_player)| *hp > 0 && *is_player != team.is_player)
                .map(|(entity, _, _)| *entity)
                .collect();

            if !enemies.is_empty() {
                let target = enemies[rng.gen_range(0..enemies.len())];
                attacks.push(target);
            }

            cooldown.reset();
        }
    }

    // Apply damage using the second query
    for target_entity in attacks {
        if let Ok(mut target_health) = param_set.p1().get_mut(target_entity) {
            let damage = rng.gen_range(10..=20);
            target_health.take_damage(damage);

            // Trigger a DamageEvent so observers can react.
            // In Bevy 0.18, events use the "observer" pattern:
            // - commands.trigger() fires a "global" event that any observer can see
            // - Observers registered with .add_observer() will run immediately
            // This replaces the old EventWriter/EventReader pattern.
            commands.trigger(DamageEvent {
                target: target_entity,
                amount: damage,
            });
        }
    }
}



