use bevy::prelude::*;

use crate::animation::{AnimationState, AnimationType, SpriteSheets};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, animation::AnimationPlugin))
        .add_systems(Startup, spawn_slime.after(animation::load_sprite_sheets))
        .run();
}

fn spawn_slime(
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
    layouts: Res<Assets<TextureAtlasLayout>>,
) {
    let layout = layouts.get(&sprite_sheets.jump_idle_layout).unwrap();

    commands.spawn((
        AnimationType::SlimeJumpIdle,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn(Camera2d);
}

mod animation;
