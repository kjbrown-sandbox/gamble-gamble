use std::time;

use bevy::{prelude::*, state::commands};
use rand::seq::IteratorRandom;

use crate::health::Health;
use crate::movement::TargetEntity;
use crate::setup_round::Inert;
use crate::special_abilities::Merging;

pub struct PickTargetPlugin;

impl Plugin for PickTargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pick_target_system);
    }
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PickTargetStrategy {
    Close,
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
        (Without<TargetEntity>, Without<Inert>, Without<Merging>),
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
        };

        if let Some(target_entity) = target_entity {
            commands.entity(entity).insert(TargetEntity(target_entity));
        }
    }
}
