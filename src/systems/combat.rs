// systems/combat.rs - Combat system
//
// This system handles the core combat loop:
// 1. Find soldiers who are ready to attack (no active AttackInstance child)
// 2. Pick a random attack from their available_attacks
// 3. Spawn an AttackInstance child (locks the soldier until cooldown finishes)
// 4. Roll for hit, apply effects, fire events
// 5. A separate cleanup system despawns finished AttackInstances

use bevy::prelude::*;
use crate::components::{
    Health, Team, Soldier, DamageEvent, Dying,
    AttackDatabase, AttackInstance, AttackId, Effect,
    AnimationState, AnimationType,
};
use rand::Rng;

/// Collected data about a pending attack action.
/// We collect these first, then apply them, to avoid borrow conflicts.
struct PendingAttack {
    attacker: Entity,
    attack_id: AttackId,
    cooldown: f32,
    effects_to_apply: Vec<(Entity, EffectApplication)>,
}

/// Describes an effect to apply to an entity
enum EffectApplication {
    Damage(i32),
    Heal(i32),
}

/// System to update all attack cooldowns.
///
/// This ticks down the cooldown on all active AttackInstance entities.
/// When an AttackInstance's cooldown reaches 0, the cleanup_finished_attacks
/// system will despawn it, allowing the soldier to attack again.
pub fn update_attack_cooldowns(
    time: Res<Time>,
    mut attacks: Query<&mut AttackInstance>,
) {
    let delta = time.delta_secs();

    for mut attack in attacks.iter_mut() {
        attack.tick(delta);
    }
}

/// System to despawn AttackInstance entities whose cooldown has finished.
///
/// WHY A SEPARATE SYSTEM?
/// - Keeps cooldown ticking separate from entity despawning
/// - Clear single responsibility: this system cleans up finished attacks
/// - When an AttackInstance is despawned, the soldier has no children,
///   which signals they're ready to attack again
pub fn cleanup_finished_attacks(
    mut commands: Commands,
    attacks: Query<(Entity, &AttackInstance)>,
) {
    for (entity, attack) in attacks.iter() {
        if attack.is_finished() {
            // Despawn this attack instance - soldier is now free to attack again
            commands.entity(entity).despawn();
        }
    }
}

/// Main attack system - processes attacks and applies effects.
///
/// NEW DESIGN:
/// A soldier can only attack if they have NO AttackInstance children.
/// When they attack:
/// 1. Pick a random attack from soldier.available_attacks
/// 2. Spawn an AttackInstance child with that attack's cooldown
/// 3. While the child exists, the soldier is "busy" and cannot attack
/// 4. When the cooldown finishes, cleanup_finished_attacks despawns it
/// 5. Soldier can now attack again
///
/// This ensures a soldier can only have ONE attack active at a time.
pub fn attack_system(
    // Query soldiers - now includes the Soldier component to access available_attacks
    // Option<&Children> because soldiers with no active attack have no children
    // Without<Dying> excludes soldiers who are playing their death animation
    mut param_set: ParamSet<(
        Query<(Entity, &Soldier, &Health, &Team, Option<&Children>), Without<Dying>>,
        Query<&mut Health>,
    )>,
    // Separate query for animations to avoid ParamSet complexity
    mut animation_query: Query<&mut AnimationState>,
    // Attack definitions database
    attack_db: Res<AttackDatabase>,
    // Commands for spawning AttackInstance and triggering events
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    // -------------------------------------------------------------------------
    // PHASE 1: Collect soldier data using p0() (read-only soldier query)
    // -------------------------------------------------------------------------
    // We collect:
    // - Entity, current HP, team, available attacks
    // - Whether they have any children (active attack = busy)

    let soldier_data: Vec<(Entity, i32, bool, Vec<crate::components::AttackId>, bool)> = param_set
        .p0()
        .iter()
        .map(|(entity, soldier, health, team, children)| {
            // has_active_attack: true if soldier has any children (AttackInstance)
            let has_active_attack = children.map(|c| !c.is_empty()).unwrap_or(false);
            (
                entity,
                health.current,
                team.is_player,
                soldier.available_attacks.clone(),
                has_active_attack,
            )
        })
        .collect();

    // Check if game is still ongoing
    let player_alive = soldier_data.iter().any(|(_, hp, is_player, _, _)| *hp > 0 && *is_player);
    let enemy_alive = soldier_data.iter().any(|(_, hp, is_player, _, _)| *hp > 0 && !*is_player);

    if !player_alive || !enemy_alive {
        return; // Game over, stop combat
    }

    // -------------------------------------------------------------------------
    // PHASE 2: Determine attacks to execute
    // -------------------------------------------------------------------------
    let mut pending_attacks: Vec<PendingAttack> = Vec::new();

    for (soldier_entity, soldier_hp, is_player, available_attacks, has_active_attack) in &soldier_data {
        // Skip dead soldiers
        if *soldier_hp <= 0 {
            continue;
        }

        // Skip soldiers who already have an active attack (busy)
        if *has_active_attack {
            continue;
        }

        // Skip soldiers with no attacks
        if available_attacks.is_empty() {
            continue;
        }

        // Pick a random attack from available attacks
        let attack_id = available_attacks[rng.gen_range(0..available_attacks.len())];

        // Get the attack definition
        let attack_def = match attack_db.get(attack_id) {
            Some(def) => def,
            None => continue,
        };

        // Find a random living enemy
        let enemies: Vec<Entity> = soldier_data
            .iter()
            .filter(|(_, hp, enemy_is_player, _, _)| *hp > 0 && *enemy_is_player != *is_player)
            .map(|(entity, _, _, _, _)| *entity)
            .collect();

        if enemies.is_empty() {
            continue;
        }

        let target = enemies[rng.gen_range(0..enemies.len())];

        // Roll for hit
        let hit = rng.gen::<f32>() < attack_def.hit_chance;

        // Determine which effects to apply
        let mut effects_to_apply = Vec::new();

        // Always apply on_use effects
        for effect in &attack_def.effects.on_use {
            if let Some(app) = resolve_effect(effect, *soldier_entity, target) {
                effects_to_apply.push(app);
            }
        }

        // Apply hit or miss effects
        let effects = if hit {
            &attack_def.effects.on_success
        } else {
            &attack_def.effects.on_fail
        };

        for effect in effects {
            if let Some(app) = resolve_effect(effect, *soldier_entity, target) {
                effects_to_apply.push(app);
            }
        }

        pending_attacks.push(PendingAttack {
            attacker: *soldier_entity,
            attack_id,
            cooldown: attack_def.cooldown,
            effects_to_apply,
        });
    }

    // -------------------------------------------------------------------------
    // PHASE 3: Spawn AttackInstance children and trigger attack animations
    // -------------------------------------------------------------------------
    // This "locks" the soldier until the cooldown finishes and the instance
    // is despawned by cleanup_finished_attacks.
    //
    // ATTACK ANIMATION MAPPING:
    // - Basic Attack (0), Healing Strike (3) → Attack animation
    // - Power Strike (1) → MoveSmallJump animation
    // - Reckless Slam (2) → JumpIdle animation (the "biggest" attack)
    for pending in &pending_attacks {
        // Spawn the AttackInstance child
        commands.entity(pending.attacker).with_children(|parent| {
            parent.spawn(AttackInstance::new(pending.attack_id, pending.cooldown));
        });

        // Trigger the appropriate attack animation
        if let Ok(mut anim_state) = animation_query.get_mut(pending.attacker) {
            let animation_type = match pending.attack_id.0 {
                0 => AnimationType::Attack,        // Basic Attack
                1 => AnimationType::MoveSmallJump, // Power Strike
                2 => AnimationType::JumpIdle,      // Reckless Slam (biggest)
                3 => AnimationType::Attack,        // Healing Strike
                _ => AnimationType::Attack,        // Default fallback
            };
            anim_state.change_to(animation_type);
        }
    }

    // -------------------------------------------------------------------------
    // PHASE 4: Apply effects using p1() (mutable health query)
    // -------------------------------------------------------------------------
    for pending in pending_attacks {
        for (target_entity, effect) in pending.effects_to_apply {
            match effect {
                EffectApplication::Damage(amount) => {
                    if let Ok(mut health) = param_set.p1().get_mut(target_entity) {
                        health.take_damage(amount);

                        // Trigger damage event for audio/visual feedback
                        commands.trigger(DamageEvent {
                            target: target_entity,
                            amount,
                        });
                    }
                }
                EffectApplication::Heal(amount) => {
                    if let Ok(mut health) = param_set.p1().get_mut(target_entity) {
                        health.current = (health.current + amount).min(health.max);
                    }
                }
            }
        }
    }
}

/// Convert an Effect enum into a concrete (entity, application) pair.
///
/// This function interprets what each effect means in context:
/// - DamageTarget → damage the target entity
/// - DamageSelf → damage the attacker entity
/// - HealSelf → heal the attacker entity
fn resolve_effect(
    effect: &Effect,
    attacker: Entity,
    target: Entity,
) -> Option<(Entity, EffectApplication)> {
    match effect {
        Effect::DamageTarget(amount) => {
            Some((target, EffectApplication::Damage(*amount)))
        }
        Effect::DamageSelf(amount) => {
            Some((attacker, EffectApplication::Damage(*amount)))
        }
        Effect::HealSelf(amount) => {
            Some((attacker, EffectApplication::Heal(*amount)))
        }
    }
}
