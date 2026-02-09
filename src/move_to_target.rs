use std::time;

use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        //   app.add_systems()
    }
}

#[derive(Component)]
pub struct TargetEntity(Entity);

pub fn move_to_target_system(
    mut params: ParamSet<(
        Query<(Entity, &mut Transform, &TargetEntity)>,
        Query<&Transform>,
    )>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut transform, target) in params.p0().iter_mut() {
        if let Ok(target_transform) = params.p1().get(target.0) {
            if target_transform.translation == transform.translation {
                commands.entity(entity).remove::<TargetEntity>();
            } else {
                // Move towards the target's position
                let direction = (target_transform.translation - transform.translation).normalize();
                let speed = 100.0; // Units per second
                transform.translation += direction * speed * time.delta_secs();
            }
        } else {
            // Remove the TargetEntity component if the target is gone
            commands.entity(entity).remove::<TargetEntity>();
        }
    }
}
