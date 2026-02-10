use bevy::prelude::*;

use crate::animation::AnimationType;
use crate::pick_target::{PickTargetStrategy, Team};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            animation::AnimationPlugin,
            move_to_target::MoveToTargetPlugin,
            pick_target::PickTargetPlugin,
            render::RenderPlugin,
            // combat::CombatPlugin,
        ))
        .add_systems(Startup, spawn_slimes.after(animation::load_sprite_sheets))
        .run();
}

fn spawn_slimes(mut commands: Commands) {
    // Spawn 5 player slimes on the left side, spread out vertically
    for i in 0..5 {
        let y = -200.0 + (i as f32 * 100.0); // Spread from -200 to 200
        commands.spawn((
            AnimationType::SlimeJumpIdle,
            Transform::from_xyz(-300.0, y, 0.0),
            Team::Player,
            PickTargetStrategy::Close,
        ));
    }

    // Spawn 5 enemy slimes on the right side, spread out vertically
    for i in 0..5 {
        let y = -200.0 + (i as f32 * 100.0);
        commands.spawn((
            AnimationType::SlimeJumpIdle,
            Transform::from_xyz(300.0, y, 0.0),
            Team::Enemy,
            PickTargetStrategy::Close,
        ));
    }

    commands.spawn(Camera2d);
}

mod animation;
mod combat;
mod move_to_target;
mod pick_target;
mod render;
