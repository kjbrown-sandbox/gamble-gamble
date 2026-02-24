use std::time;

use bevy::{prelude::*, state::commands};
use rand::seq::IteratorRandom;

use crate::health::{Dying, Health};
use crate::movement::TargetEntity;
use crate::setup_round::Inert;
use crate::special_abilities::Merging;

pub struct PickTargetPlugin;

impl Plugin for PickTargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (pick_target_system, closest_target_system));
    }
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PickTargetStrategy {
    /// Picks from a random shortlist of 3 enemies, then chooses the closest.
    Close,
    /// Constantly re-evaluates every frame to always target the single closest enemy.
    Closest,
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Team {
    Player,
    Enemy,
}

pub fn pick_target_system(
    // GlobalTransform gives world-space position, which is necessary for child entities
    // (like the frozen spear) whose local Transform is relative to their parent.
    // For top-level entities, GlobalTransform equals Transform — no change in behavior.
    entities_needing_targets: Query<
        (Entity, &PickTargetStrategy, &Team, &GlobalTransform),
        (Without<TargetEntity>, Without<Inert>, Without<Merging>, Without<Dying>),
    >,
    potential_targets: Query<(Entity, &Team, &GlobalTransform), With<Health>>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    for (entity, strategy, team, transform) in entities_needing_targets {
        let target_entity: Option<Entity> = match strategy {
            PickTargetStrategy::Close => {
                let candidates: Vec<(Entity, &GlobalTransform)> = potential_targets
                    .iter()
                    .filter(|(_, target_team, _)| *target_team != team) // Step 1: filter to enemies
                    .map(|(e, _, t)| (e, t))
                    .choose_multiple(&mut rng, 3); // randomly sample up to 3

                // Of our shortlist, pick whichever is closest.
                // GlobalTransform uses .translation() method instead of .translation field.
                candidates
                    .into_iter()
                    .min_by(|(_, a_transform), (_, b_transform)| {
                        let dist_a = transform.translation().distance(a_transform.translation());
                        let dist_b = transform.translation().distance(b_transform.translation());
                        dist_a.partial_cmp(&dist_b).unwrap()
                    })
                    .map(|(target_entity, _)| target_entity)
            }
            // Closest is handled by closest_target_system, which re-evaluates
            // every frame. Skip here to avoid redundant work.
            PickTargetStrategy::Closest => continue,
        };

        if let Some(target_entity) = target_entity {
            commands.entity(entity).insert(TargetEntity(target_entity));
        }
    }
}

/// Re-evaluates the closest enemy every frame for entities with PickTargetStrategy::Closest.
///
/// Unlike pick_target_system (which only runs when no target is assigned),
/// this system runs every frame and overwrites TargetEntity. This means
/// the entity always attacks whoever is nearest — if a closer enemy walks by,
/// it switches targets immediately.
///
/// Note: there's no Without<TargetEntity> filter here. That's the whole point —
/// we want to run even when a target already exists so we can replace it
/// with a closer one.
fn closest_target_system(
    seekers: Query<
        (Entity, &PickTargetStrategy, &Team, &GlobalTransform),
        (Without<Inert>, Without<Merging>, Without<Dying>),
    >,
    potential_targets: Query<(Entity, &Team, &GlobalTransform), With<Health>>,
    mut commands: Commands,
) {
    for (entity, strategy, team, transform) in seekers.iter() {
        // Can't filter on enum variant at the query level — Bevy queries
        // filter on component presence, not component values. So we check
        // the variant in code and skip non-Closest entities.
        if *strategy != PickTargetStrategy::Closest {
            continue;
        }

        // Find the single closest enemy. No random shortlist like Close —
        // we check every potential target to guarantee we pick the actual closest.
        let closest = potential_targets
            .iter()
            .filter(|(_, target_team, _)| *target_team != team)
            .min_by(|(_, _, a), (_, _, b)| {
                let dist_a = transform.translation().distance(a.translation());
                let dist_b = transform.translation().distance(b.translation());
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(e, _, _)| e);

        if let Some(target) = closest {
            commands.entity(entity).insert(TargetEntity(target));
        }
    }
}
