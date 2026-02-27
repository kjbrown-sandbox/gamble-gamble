use bevy::{ecs::spawn, prelude::*, render::render_resource::Texture};

pub struct UtilsPlugin;

impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_after_system);
    }
}

#[derive(Component, Clone, PartialEq, Eq)]
pub struct DespawnAfter(pub Timer);

fn despawn_after_system(
    mut commands: Commands,
    mut spawn_despawn_timer: Query<(Entity, &mut DespawnAfter)>,
    game_time: Res<Time>,
) {
    for (entity, mut timer) in spawn_despawn_timer.iter_mut() {
        timer.0.tick(game_time.delta());
        if timer.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// #[derive(Component, Clone, PartialEq, Eq)]
// pub struct Shake {

// };
