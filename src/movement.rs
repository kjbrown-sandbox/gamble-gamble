use std::time;

use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        // unsmush runs after move_to_target so it gets the "last word" each frame
        app.add_systems(Update, (move_to_target_system, unsmush_system).chain());
    }
}

#[derive(Component)]
pub struct TargetEntity(pub Entity);

pub fn move_to_target_system(
    mut params: ParamSet<(
        Query<(Entity, &mut Transform, &TargetEntity)>,
        Query<&Transform>,
    )>,
    mut commands: Commands,
    time: Res<Time>,
) {
    // Phase 1: Collect mover entity IDs and their target entity IDs.
    // We also need each target's position, so we read that from p1.
    // We CAN'T read the mover's transform here — we need p0 for that,
    // and we can't use p0 and p1 at the same time.
    let movers: Vec<(Entity, Entity)> = params
        .p0()
        .iter()
        .map(|(entity, _, target)| (entity, target.0))
        .collect();

    // Now read target positions from p1 (releases the p0 borrow)
    let mut move_orders: Vec<(Entity, Vec3)> = Vec::new();
    let mut lost_targets: Vec<Entity> = Vec::new();

    for (mover_entity, target_entity) in &movers {
        if let Ok(target_transform) = params.p1().get(*target_entity) {
            move_orders.push((*mover_entity, target_transform.translation));
        } else {
            lost_targets.push(*mover_entity);
        }
    }

    // Phase 2: Now use p0 mutably to move the real transforms
    let delta = time.delta_secs();
    for (mover_entity, target_pos) in &move_orders {
        if let Ok((_, mut transform, _)) = params.p0().get_mut(*mover_entity) {
            let diff = *target_pos - transform.translation;
            if diff.length() > 50.0 {
                let direction = diff.normalize();
                let speed = 125.0;
                transform.translation += direction * speed * delta;
            }
        }
    }

    for entity in lost_targets {
        commands.entity(entity).remove::<TargetEntity>();
    }
}

/// Pushes entities apart when they're within 50 units of each other.
/// The closer they are, the stronger the push. This prevents units
/// from stacking on top of each other.
pub fn unsmush_system(mut query: Query<(Entity, &mut Transform), With<Sprite>>, time: Res<Time>) {
    let min_distance = 50.0;
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

            // Only compare x and y; z is used for draw order, not spacing
            let diff = Vec3::new(pos_a.x - pos_b.x, pos_a.y - pos_b.y, 0.0);
            let distance = diff.length();

            if distance < min_distance && distance > 0.01 {
                // How much of the min_distance are we violating? (0.0 = barely touching, 1.0 = fully overlapping)
                let overlap_ratio = 1.0 - (distance / min_distance);

                // Direction from B to A (push A away from B and vice versa)
                let direction = diff.normalize();
                let force = direction * overlap_ratio * push_strength * time.delta_secs();

                pushes.push((entity_a, force));
                pushes.push((entity_b, -force)); // opposite direction
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
