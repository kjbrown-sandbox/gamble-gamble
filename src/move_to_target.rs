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
    query: Query<(&mut Transform, &TargetEntity)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (mut transform, target) in query.iter_mut() {
        if let Ok(target_transform) = query.get_component::<Transform>(target.0) {
            // Move towards the target's position
            let direction = (target_transform.translation - transform.translation).normalize();
            let speed = 100.0; // Units per second
            transform.translation += direction * speed * time.delta_seconds();
        } else {
            // Remove the TargetEntity component if the target is gone
            commands.entity(entity).remove::<TargetEntity>();
        }
    }
}
