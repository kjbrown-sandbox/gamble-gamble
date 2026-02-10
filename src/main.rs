use bevy::prelude::*;

use crate::animation::AnimationType;
use crate::move_to_target::TargetEntity;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            animation::AnimationPlugin,
            move_to_target::MoveToTargetPlugin,
            // combat::CombatPlugin,
        ))
        .add_systems(Startup, spawn_slimes.after(animation::load_sprite_sheets))
        .run();
}

fn spawn_slimes(mut commands: Commands) {
    // Spawn unmoving target on the top left
    let top_left_slime = commands
        .spawn((
            AnimationType::SlimeJumpIdle,
            Transform::from_xyz(-300.0, 200.0, 0.0),
        ))
        .id();

    // Spawn the target slime on the right
    let target_slime = commands
        .spawn((
            AnimationType::SlimeJumpIdle,
            Transform::from_xyz(300.0, 200.0, 0.0),
            TargetEntity(top_left_slime),
        ))
        .id();

    // Spawn the chaser slime on the left, targeting the first
    commands.spawn((
        AnimationType::SlimeJumpIdle,
        Transform::from_xyz(-100.0, -200.0, 0.0),
        TargetEntity(target_slime),
    ));

    commands.spawn((
        AnimationType::SlimeJumpIdle,
        Transform::from_xyz(-300.0, -200.0, 0.0),
        TargetEntity(top_left_slime),
    ));

    commands.spawn(Camera2d);
}

mod animation;
mod combat;
mod move_to_target;
mod pick_target;
