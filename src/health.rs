use std::time;

use bevy::{prelude::*, state::commands};
use rand::seq::IteratorRandom;

use crate::{
    animation::{AnimationState, AnimationType},
    movement::{Speed, TargetEntity},
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

#[derive(Component, Clone, PartialEq, Eq, Hash)]
pub struct HurtSfx(pub String); // I will want a hurt sound effect too when I do audio

// I will want a death sound effect too when I do audio

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
    mut dying_with_speed: Query<(Entity), (Added<Dying>, With<Speed>)>,
    //  mut remove_from_target: Query<(Entity, &TargetEntity),
    dying_entities: Query<Entity, Added<Dying>>,
    mut entities_with_targets: Query<(Entity, &TargetEntity)>,
) {
    for (mut current_animation, death_animation) in query.iter_mut() {
        *current_animation = death_animation.0;
        //  commands.entity(entity).insert(DeathAnimation(*death_animation));
    }

    for entity in dying_with_speed.iter() {
        // Remove speed so they stop moving immediately when they start dying
        commands.entity(entity).remove::<Speed>();
    }

    // Remove dying entities from being targeted by movers
    for dying_entity in dying_entities.iter() {
        for (entity, target) in entities_with_targets.iter_mut() {
            if target.0 == dying_entity {
                commands.entity(entity).remove::<TargetEntity>();
            }
        }
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

#[derive(Event)]
pub struct DamagedEvent {
    pub entity: Entity,
}

pub fn on_damaged_event(
    trigger: On<DamagedEvent>,
    mut commands: Commands,
    mut query: Query<&mut Health>,
) {
    // do nothing currently, but play hurt sfx here when I add audio
    // Also add shader eventually
}
