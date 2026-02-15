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

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
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
