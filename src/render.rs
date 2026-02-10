use bevy::prelude::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, y_sort_system);
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
