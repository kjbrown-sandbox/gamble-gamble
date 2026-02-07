// systems/combat.rs - Combat system
//
// This system handles the core combat loop:
// 1. Find soldiers and their attack children
// 2. Update attack cooldowns
// 3. Execute ready attacks (roll for hit, apply effects)
// 4. Fire events for audio/visual feedback

use bevy::prelude::*;
use crate::components::{
    Health, Team, Soldier, DamageEvent,
    AttackDatabase, AttackInstance, Effect,
};
use rand::Rng;

/// Collected data about a pending attack action.
/// We collect these first, then apply them, to avoid borrow conflicts.
struct PendingAttack {
    attacker: Entity,
    target: Entity,
    attack_index: usize,  // Index into the attacker's attack children
    hit: bool,
    effects_to_apply: Vec<(Entity, EffectApplication)>,
}

/// Describes an effect to apply to an entity
enum EffectApplication {
    Damage(i32),
    Heal(i32),
}

/// System to update all attack cooldowns.
///
/// WHY A SEPARATE SYSTEM?
/// Separating cooldown updates from attack execution:
/// - Makes each system simpler and easier to understand
/// - Allows different scheduling if needed (e.g., FixedUpdate for cooldowns)
/// - Follows single-responsibility principle
///
/// This system runs every frame and ticks down all attack cooldowns.
pub fn update_attack_cooldowns(
    time: Res<Time>,
    mut attacks: Query<&mut AttackInstance>,
) {
    let delta = time.delta_secs();

    for mut attack in attacks.iter_mut() {
        attack.tick(delta);
    }
}

/// Main attack system - processes attacks and applies effects.
///
/// This system:
/// 1. Finds soldiers who have ready attacks
/// 2. Picks a random ready attack for each soldier
/// 3. Picks a random enemy target
/// 4. Rolls for hit/miss based on attack's hit_chance
/// 5. Applies appropriate effects (on_success, on_fail, on_use)
/// 6. Triggers DamageEvent for audio feedback
///
/// PARAMSET FOR QUERY CONFLICTS:
/// Both our soldier query and health mutation query access the Health component.
/// Bevy's borrow checker prevents this at runtime. ParamSet solves this by
/// ensuring only one query is active at a time - you call .p0() or .p1()
/// to access each query, and can't use both simultaneously.
pub fn attack_system(
    // ParamSet wraps conflicting queries. We can only access one at a time.
    // p0: Read soldiers with their children and health
    // p1: Mutate health for applying damage/healing
    mut param_set: ParamSet<(
        Query<(Entity, &Health, &Team, &Children), With<Soldier>>,
        Query<&mut Health>,
    )>,
    // Query attack instances (children of soldiers) - no conflict
    mut attacks: Query<&mut AttackInstance>,
    // Attack definitions database
    attack_db: Res<AttackDatabase>,
    // Commands for triggering events
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    // -------------------------------------------------------------------------
    // PHASE 1: Collect soldier data using p0() (read-only soldier query)
    // -------------------------------------------------------------------------
    // ParamSet requires us to explicitly access queries one at a time.
    // We use p0() to access the soldier query, collect all data we need,
    // then the borrow ends when soldier_data is created.

    let soldier_data: Vec<(Entity, i32, bool, Vec<Entity>)> = param_set
        .p0()
        .iter()
        .map(|(entity, health, team, children)| {
            // Collect children as Vec<Entity> so we don't hold the borrow
            let child_entities: Vec<Entity> = children.iter().collect();
            (entity, health.current, team.is_player, child_entities)
        })
        .collect();

    // Check if game is still ongoing
    let player_alive = soldier_data.iter().any(|(_, hp, is_player, _)| *hp > 0 && *is_player);
    let enemy_alive = soldier_data.iter().any(|(_, hp, is_player, _)| *hp > 0 && !*is_player);

    if !player_alive || !enemy_alive {
        return; // Game over, stop combat
    }

    // -------------------------------------------------------------------------
    // PHASE 2: Determine attacks to execute
    // -------------------------------------------------------------------------
    // Now we work with our collected data (no longer borrowing the query)
    let mut pending_attacks: Vec<PendingAttack> = Vec::new();

    for (soldier_entity, soldier_hp, is_player, children) in &soldier_data {
        // Skip dead soldiers
        if *soldier_hp <= 0 {
            continue;
        }

        // Find all ready attacks for this soldier
        let ready_attacks: Vec<(usize, Entity)> = children
            .iter()
            .enumerate()
            .filter_map(|(idx, &child_entity)| {
                if let Ok(attack) = attacks.get(child_entity) {
                    if attack.is_ready() {
                        return Some((idx, child_entity));
                    }
                }
                None
            })
            .collect();

        if ready_attacks.is_empty() {
            continue;
        }

        // Pick a random ready attack
        let (attack_index, attack_entity) = ready_attacks[rng.gen_range(0..ready_attacks.len())];

        // Get the attack definition
        let attack_instance = attacks.get(attack_entity).unwrap();
        let attack_def = match attack_db.get(attack_instance.attack_id) {
            Some(def) => def,
            None => continue,
        };

        // Find a random living enemy
        let enemies: Vec<Entity> = soldier_data
            .iter()
            .filter(|(_, hp, enemy_is_player, _)| *hp > 0 && *enemy_is_player != *is_player)
            .map(|(entity, _, _, _)| *entity)
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
            target,
            attack_index,
            hit,
            effects_to_apply,
        });
    }

    // -------------------------------------------------------------------------
    // PHASE 3: Start cooldowns (uses attacks query, not ParamSet)
    // -------------------------------------------------------------------------
    for pending in &pending_attacks {
        // Find the attack child and start its cooldown
        if let Some((_, _, _, children)) = soldier_data.iter().find(|(e, _, _, _)| *e == pending.attacker) {
            if let Some(&attack_entity) = children.get(pending.attack_index) {
                if let Ok(mut attack) = attacks.get_mut(attack_entity) {
                    if let Some(def) = attack_db.get(attack.attack_id) {
                        attack.start_cooldown(def.cooldown);
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // PHASE 4: Apply effects using p1() (mutable health query)
    // -------------------------------------------------------------------------
    // Now we switch to p1() to mutate health. This is safe because we're
    // no longer using p0() (the soldier query).
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
