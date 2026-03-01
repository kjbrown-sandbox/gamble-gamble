// Derived status components.
//
// Instead of every system independently filtering on the same set of
// status-effect components (Merging, Knockback, Inert, …), we compute
// a handful of "can this entity do X?" markers once per frame and let
// consumers query a single component.  Adding a new status effect means
// updating one place here instead of hunting through every system.

use bevy::prelude::*;

use crate::combat::ActiveAttack;
use crate::health::{Dying, Health};
use crate::movement::Knockback;
use crate::setup_round::Inert;
use crate::special_abilities::{Merging, PreMerging};
use crate::GameState;

pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (update_can_be_moved, update_can_be_targeted, update_can_move)
                .run_if(in_state(GameState::Combat)),
        );
    }
}

/// Present when an entity's position is allowed to be nudged by
/// external forces (unsmush, etc.).  Removed while merging, pre-merging,
/// or mid-knockback.
#[derive(Component)]
pub struct CanBeMoved;

/// Present when an entity is a valid combat target.
/// Removed when the entity has no Health or is Dying.
#[derive(Component)]
pub struct CanBeTargeted;

/// Present when an entity can voluntarily move toward a target.
/// Removed while inert, dying, merging, pre-merging, mid-knockback,
/// or mid-attack.
#[derive(Component)]
pub struct CanMove;

fn update_can_be_moved(
    mut commands: Commands,
    eligible: Query<
        Entity,
        (
            Without<CanBeMoved>,
            Without<Merging>,
            Without<PreMerging>,
            Without<Knockback>,
        ),
    >,
    ineligible: Query<
        Entity,
        (
            With<CanBeMoved>,
            Or<(With<Merging>, With<PreMerging>, With<Knockback>)>,
        ),
    >,
) {
    for entity in &eligible {
        commands.entity(entity).insert(CanBeMoved);
    }
    for entity in &ineligible {
        commands.entity(entity).remove::<CanBeMoved>();
    }
}

fn update_can_move(
    mut commands: Commands,
    eligible: Query<
        Entity,
        (
            Without<CanMove>,
            Without<Inert>,
            Without<PreMerging>,
            Without<Merging>,
            Without<Dying>,
            Without<Knockback>,
            Without<ActiveAttack>,
        ),
    >,
    ineligible: Query<
        Entity,
        (
            With<CanMove>,
            Or<(
                With<Inert>,
                With<PreMerging>,
                With<Merging>,
                With<Dying>,
                With<Knockback>,
                With<ActiveAttack>,
            )>,
        ),
    >,
) {
    for entity in &eligible {
        commands.entity(entity).insert(CanMove);
    }
    for entity in &ineligible {
        commands.entity(entity).remove::<CanMove>();
    }
}

fn update_can_be_targeted(
    mut commands: Commands,
    eligible: Query<
        Entity,
        (With<Health>, Without<Dying>, Without<CanBeTargeted>),
    >,
    ineligible: Query<
        Entity,
        (
            With<CanBeTargeted>,
            Or<(Without<Health>, With<Dying>)>,
        ),
    >,
) {
    for entity in &eligible {
        commands.entity(entity).insert(CanBeTargeted);
    }
    for entity in &ineligible {
        commands.entity(entity).remove::<CanBeTargeted>();
    }
}
