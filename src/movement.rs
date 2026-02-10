use std::time;

use bevy::prelude::*;

pub struct MoveToTargetPlugin;

impl Plugin for MoveToTargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_to_target_system);
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
