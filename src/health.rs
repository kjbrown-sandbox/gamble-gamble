use std::time;

use bevy::{prelude::*, state::commands};
use rand::seq::IteratorRandom;

use crate::{
    animation::{AnimationState, AnimationType},
    movement::TargetEntity,
};

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                set_dying_system,
                when_starts_dying_system,
                when_finishes_dying_system,
            ),
        );
    }
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Health(pub i32);

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Dying;

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DeathAnimation(pub AnimationType);

// I will want a death sound effect too when I do audio

// #[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
// pub enum PickTargetStrategy {
//     Close,
// }

// #[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
// pub enum Team {
//     Player,
//     Enemy,
// }

pub fn set_dying_system(
    mut commands: Commands,
    query: Query<(Entity, &Health), (Without<Dying>, Changed<Health>)>,
) {
    for (entity, health) in query.iter() {
        if health.0 <= 0 {
            commands.entity(entity).insert(Dying);
        }
    }
}

pub fn when_starts_dying_system(
    mut commands: Commands,
    mut query: Query<(&mut AnimationType, &DeathAnimation), Added<Dying>>,
) {
    for (mut current_animation, death_animation) in query.iter_mut() {
        *current_animation = death_animation.0;
        //  commands.entity(entity).insert(DeathAnimation(*death_animation));
    }
}

pub fn when_finishes_dying_system(
    mut commands: Commands,
    query: Query<(Entity, &AnimationState), (With<Dying>, With<AnimationState>)>,
) {
    for (entity, animation_state) in query.iter() {
        if animation_state.finished {
            commands.entity(entity).despawn();
        }
    }
}

// pub fn pick_target_system(
//     entities_needing_targets: Query<
//         (Entity, &PickTargetStrategy, &Team, &Transform),
//         Without<TargetEntity>,
//     >,
//     potential_targets: Query<(Entity, &Team, &Transform)>,
//     mut commands: Commands,
// ) {
//     let mut rng = rand::thread_rng();
//     for (entity, strategy, team, transform) in entities_needing_targets {
//         let target_entity: Option<Entity> = match strategy {
//             PickTargetStrategy::Close => {
//                 let candidates: Vec<(Entity, &Transform)> = potential_targets
//                     .iter()
//                     .filter(|(_, target_team, _)| *target_team != team) // Step 1: filter to enemies
//                     .map(|(e, _, t)| (e, t))
//                     .choose_multiple(&mut rng, 3); // randomly sample up to 3

//                 // Of our shortlist, pick whichever is closest
//                 candidates
//                     .into_iter()
//                     .min_by(|(_, a_transform), (_, b_transform)| {
//                         let dist_a = transform.translation.distance(a_transform.translation);
//                         let dist_b = transform.translation.distance(b_transform.translation);
//                         dist_a.partial_cmp(&dist_b).unwrap()
//                     })
//                     .map(|(target_entity, _)| target_entity)
//             }
//         };

//         if let Some(target_entity) = target_entity {
//             commands.entity(entity).insert(TargetEntity(target_entity));
//         }
//     }
// }
