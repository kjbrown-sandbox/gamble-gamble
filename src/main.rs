use bevy::prelude::*;

use crate::animation::{AnimationType, SpriteSheets};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, animation::AnimationPlugin))
        .add_systems(Startup, spawn_slime.after(animation::load_sprite_sheets))
        .run();
}

fn spawn_slime(mut commands: Commands) {
    commands.spawn((
        AnimationType::SlimeJumpIdle,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn(Camera2d);
}

mod animation;
mod move_to_target;
