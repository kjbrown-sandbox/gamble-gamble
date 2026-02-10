use bevy::prelude::*;

use crate::movement::TargetEntity;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (y_sort_system, face_target_system));
    }
}

/// Flips sprites horizontally so they face their target.
fn face_target_system(
    mut sprites: Query<(&Transform, &TargetEntity, &mut Sprite)>,
    targets: Query<&Transform>,
) {
    for (transform, target, mut sprite) in &mut sprites {
        // Look up the target's transform. If the target no longer exists, skip.
        if let Ok(target_transform) = targets.get(target.0) {
            // Flip when the target is to the left of this entity
            sprite.flip_x = target_transform.translation.x < transform.translation.x;
        }
    }
}

/// Sorts sprites by y position so lower enemies appear in front
fn y_sort_system(mut query: Query<&mut Transform, With<Sprite>>) {
    for mut transform in &mut query {
        // Negate y: lower y (bottom of screen) -> higher z (drawn in front)
        // Scale down to keep z values small and leave room for other layers
        transform.translation.z = -transform.translation.y * 0.01;
    }
}
