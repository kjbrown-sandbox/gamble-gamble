use std::time;

use bevy::{prelude::*, state::commands};
use rand::seq::IteratorRandom;

use crate::move_to_target::TargetEntity;

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
    //  mut params: ParamSet<(
    //      Query<(Entity, &mut Transform, &TargetEntity)>,
    //      Query<&Transform>,
    //  )>,
    //  mut commands: Commands,
    //  time: Res<Time>,
    entities_needing_targets: Query<
        (Entity, &PickTargetStrategy, &Team, &Transform),
        Without<TargetEntity>,
    >,
    potential_targets: Query<(Entity, &Team, &Transform)>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    for (entity, strategy, team, transform) in entities_needing_targets {
        let mut target_entity;
        match strategy {
            PickTargetStrategy::Close => {
                // We need a random number generator to sample from the candidates

                // Step 1: Collect all enemies (entities on the opposite team)
                // Step 2: Randomly pick up to 3 of them
                // Step 3: Of those 3, find the nearest one
                //
                // Why not just pick the nearest overall? This adds variety—
                // units won't all dogpile the same target. It's a simple way
                // to simulate imperfect "awareness" of the battlefield.
                let candidates: Vec<(Entity, &Transform)> = potential_targets
                    .iter()
                    .filter(|(_, target_team, _)| *target_team != team) // Step 1: filter to enemies
                    .map(|(e, _, t)| (e, t))
                    .choose_multiple(&mut rng, 3); // randomly sample up to 3

                // Of our shortlist, pick whichever is closest
                candidates
                    .into_iter()
                    .min_by(|(_, a_transform), (_, b_transform)| {
                        let dist_a = transform.translation.distance(a_transform.translation);
                        let dist_b = transform.translation.distance(b_transform.translation);
                        // partial_cmp because f32 doesn't implement Ord (due to NaN).
                        // unwrap is safe here—distances between real positions won't be NaN.
                        dist_a.partial_cmp(&dist_b).unwrap()
                    })
                    .map(|(target_entity, _)| target_entity)
            }
        }
        commands.entity(entity).insert(TargetEntity(target_entity));
    }
}
