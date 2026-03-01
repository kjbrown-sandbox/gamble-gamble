use std::time;

use bevy::prelude::*;

use crate::combat::ActiveAttack;
use crate::health::{Dying, Health};
use crate::setup_round::{Inert, StunTimer};
use crate::special_abilities::{Merging, PreMerging};
use crate::status::CanBeMoved;
use crate::{ArenaBounds, GameState};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                knockback_system,
                move_to_target_system,
                move_to_target_transform_system,
                unsmush_system,
                out_of_bounds_system,
            )
                .chain()
                .run_if(in_state(GameState::Combat)),
        );
    }
}

#[derive(Component)]
pub struct TargetEntity(pub Entity);

#[derive(Component)]
pub struct TargetTransform(pub Vec3);

#[derive(Component, Copy, Clone, PartialEq)]
pub struct Speed(pub f32);

/// Clamps how far a child entity can drift from its parent's position.
/// The f32 is the maximum distance in local-space units.
#[derive(Component, Copy, Clone)]
pub struct StaysNearParent(pub f32);

/// Temporary component that smoothly moves an entity from its current position
/// to a target position over 1 second with an ease-out curve.
#[derive(Component)]
pub struct Knockback {
    pub start_position: Vec3,
    pub target_position: Vec3,
    pub timer: Timer,
}

pub fn move_to_target_system(
    mut movers: Query<
        (
            Entity,
            &mut Transform,
            &GlobalTransform,
            &TargetEntity,
            &Speed,
            Option<&StaysNearParent>,
        ),
        (
            Without<Inert>,
            Without<PreMerging>,
            Without<Merging>,
            Without<Dying>,
            Without<Knockback>,    // Don't fight with knockback lerp
            Without<ActiveAttack>, // Freeze in place while attacking
        ),
    >,
    targets: Query<&GlobalTransform>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, global_tf, target, speed, stays_near) in movers.iter_mut() {
        let Ok(target_global) = targets.get(target.0) else {
            // Target no longer exists (despawned). Remove TargetEntity so
            // pick_target_system can assign a new one next frame.
            commands.entity(entity).remove::<TargetEntity>();
            continue;
        };

        // Use world-space positions for direction computation.
        // For top-level entities, GlobalTransform == Transform (no change).
        // For child entities (like the spear), this gives the correct
        // world position instead of the local offset from the parent.
        let mover_pos = global_tf.translation();
        let target_pos = target_global.translation();

        let x_diff = target_pos.x - mover_pos.x;
        if x_diff.abs() > 50.0 {
            transform.translation.x += speed.0 * delta * x_diff.signum();
        }

        let y_diff = target_pos.y - mover_pos.y;
        if y_diff.abs() > 35.0 {
            transform.translation.y += speed.0 * delta * y_diff.signum();
        }

        // If StaysNearParent is present, clamp the local position so the entity
        // can't drift too far from its parent. Since this is a child entity,
        // local (0, 0) is the parent's position — so we clamp the length
        // of the local translation vector. This creates a "leash" effect:
        // the spear strains toward enemies but snaps back when it reaches
        // the max distance.
        if let Some(max_dist) = stays_near {
            let local_xy = Vec2::new(transform.translation.x, transform.translation.y);
            if local_xy.length() > max_dist.0 {
                let clamped = local_xy.normalize() * max_dist.0;
                transform.translation.x = clamped.x;
                transform.translation.y = clamped.y;
            }
        }
    }
}

fn move_to_target_transform_system(
    mut commands: Commands,
    mut movers: Query<
        (Entity, &mut Transform, &TargetTransform, &Speed),
        (Without<Dying>, Without<Knockback>),
    >,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, target, speed) in movers.iter_mut() {
        let dest = target.0;
        let x_diff = dest.x - transform.translation.x;
        let y_diff = dest.y - transform.translation.y;

        if x_diff.abs() > 2.0 {
            transform.translation.x += speed.0 * delta * x_diff.signum();
        }
        if y_diff.abs() > 2.0 {
            transform.translation.y += speed.0 * delta * y_diff.signum();
        }

        if x_diff.abs() <= 2.0 && y_diff.abs() <= 2.0 {
            if let Ok(mut cmds) = commands.get_entity(entity) {
                cmds.remove::<TargetTransform>();
                cmds.remove::<TargetEntity>();
            }
        }
    }
}

/// Smoothly moves knocked-back entities from start to target position over 1 second.
fn knockback_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Knockback), Without<Dying>>,
) {
    for (entity, mut transform, mut knockback) in query.iter_mut() {
        knockback.timer.tick(time.delta());

        // Quadratic ease-out: fast start, slow finish.
        let t = knockback.timer.fraction();
        let t_eased = 1.0 - (1.0 - t).powi(2);

        // Lerp between start and target using the eased t value.
        transform.translation = knockback
            .start_position
            .lerp(knockback.target_position, t_eased);

        if knockback.timer.is_finished() {
            if let Ok(mut cmds) = commands.get_entity(entity) {
                cmds.remove::<Knockback>();
            }
        }
    }
}

/// Pushes entities apart when they're within 50 units of each other.
/// The closer they are, the stronger the push. This prevents units
/// from stacking on top of each other.
///
/// Without<Merging> excludes slimes that are currently walking toward their
/// merge partner. Without this filter, unsmush would push them apart as they
/// try to converge, creating a tug-of-war between the two systems.
pub fn unsmush_system(
    mut query: Query<
        (Entity, &mut Transform),
        (With<Sprite>, With<Health>, With<CanBeMoved>),
    >,
    time: Res<Time>,
) {
    let min_x_distance = 50.0;
    let min_y_distance = 35.0;
    let push_strength = 100.0;

    // Phase 1: Collect all positions so we can compare without borrow conflicts.
    // We need to read every entity's position while also mutating transforms,
    // so we snapshot positions first.
    let positions: Vec<(Entity, Vec3)> = query
        .iter()
        .map(|(entity, transform)| (entity, transform.translation))
        .collect();

    // Phase 2: For each pair, calculate push forces.
    // We accumulate forces first, then apply them — otherwise earlier pushes
    // would affect later distance calculations within the same frame.
    let mut pushes: Vec<(Entity, Vec3)> = Vec::new();

    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let (entity_a, pos_a) = positions[i];
            let (entity_b, pos_b) = positions[j];

            let x_diff = pos_a.x - pos_b.x;
            let y_diff = pos_a.y - pos_b.y;

            if x_diff.abs() < min_x_distance {
                // How much are we violating the min distance? (0.0 = barely touching, 1.0 = fully overlapping)
                let x_overlap_ratio = 1.0 - (x_diff.abs() / min_x_distance);

                // Push in x and y directions separately, scaled by how much we're overlapping in each direction
                let x_push = x_diff.signum() * x_overlap_ratio * push_strength * time.delta_secs();
                pushes.push((entity_a, Vec3::new(x_push, 0.0, 0.0)));
                pushes.push((entity_b, Vec3::new(-x_push, 0.0, 0.0))); // opposite direction
            }

            if y_diff.abs() < min_y_distance {
                // How much are we violating the min distance? (0.0 = barely touching, 1.0 = fully overlapping)
                let y_overlap_ratio = 1.0 - (y_diff.abs() / min_y_distance);

                // Push in x and y directions separately, scaled by how much we're overlapping in each direction
                let y_push = y_diff.signum() * y_overlap_ratio * push_strength * time.delta_secs();
                pushes.push((entity_a, Vec3::new(0.0, y_push, 0.0)));
                pushes.push((entity_b, Vec3::new(0.0, -y_push, 0.0))); // opposite direction
            }
        }
    }

    // Phase 3: Apply all pushes
    for (entity, force) in pushes {
        if let Ok((_, mut transform)) = query.get_mut(entity) {
            transform.translation += force;
        }
    }
}

/// Clamps top-level sprite entities to stay inside the arena.
///
/// Without<ChildOf> filters to root entities only — child sprites (like a
/// weapon attached to a character) are positioned relative to their parent,
/// so clamping them directly would fight with the parent's transform.
///
/// .clamp() is Rust's built-in method on f32: it returns the value pinned
/// between a min and max. Cleaner than chaining .min().max().
pub fn out_of_bounds_system(
    mut query: Query<
        (Entity, &mut Transform, Option<&Knockback>),
        (With<Sprite>, Without<ChildOf>),
    >,
    arena: Res<ArenaBounds>,
    mut commands: Commands,
) {
    let half_w = arena.half_width() - 50.0;
    let half_h = arena.half_height() - 50.0;

    for (entity, mut transform, knockback) in &mut query {
        let pos = transform.translation;
        let out_of_bounds = pos.x < -half_w || pos.x > half_w || pos.y < -half_h || pos.y > half_h;

        // Wall slam: if the entity is being knocked into the wall and the
        // knockback target is more than 50 units past the arena edge, stun it.
        if out_of_bounds {
            if let Some(kb) = knockback {
                let overflow_x = (kb.target_position.x.abs() - half_w).max(0.0);
                let overflow_y = (kb.target_position.y.abs() - half_h).max(0.0);
                let overflow = (overflow_x * overflow_x + overflow_y * overflow_y).sqrt();

                if overflow > 50.0 {
                    if let Ok(mut cmds) = commands.get_entity(entity) {
                        cmds.insert((Inert, StunTimer(Timer::from_seconds(1.5, TimerMode::Once))));
                        cmds.remove::<(ActiveAttack, Knockback)>();
                    }
                }
            }
        }

        transform.translation.x = pos.x.clamp(-half_w, half_w);
        transform.translation.y = pos.y.clamp(-half_h, half_h);
    }
}
