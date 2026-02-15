use bevy::prelude::*;

use crate::animation::AnimationType;
use crate::armies::EnemyArmies;
use crate::health::{DeathAnimation, Health};
use crate::pick_target::{PickTargetStrategy, Team};
use crate::save_load::SaveData;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            animation::AnimationPlugin,
            armies::ArmiesPlugin,
            movement::MovementPlugin,
            pick_target::PickTargetPlugin,
            render::RenderPlugin,
            save_load::SaveLoadPlugin,
            health::HealthPlugin,
        ))
        // spawn_slimes needs three resources to exist first:
        //   - SpriteSheets (from animation::load_sprite_sheets)
        //   - SaveData (from save_load's startup system)
        //   - EnemyArmies (from armies plugin, via init_resource — available immediately)
        .add_systems(Startup, spawn_slimes.after(animation::load_sprite_sheets))
        .run();
}

/// Spawns both armies using data from our resources instead of hardcoded values.
///
/// - Player army size comes from SaveData (persisted to disk between sessions)
/// - Enemy army comes from EnemyArmies (static game data defined in code)
///
/// Note: we take Res<T> (immutable reference) since we only need to read these.
/// If we needed to modify them, we'd use ResMut<T>.
fn spawn_slimes(mut commands: Commands, save_data: Res<SaveData>, enemy_armies: Res<EnemyArmies>) {
    // Player army — count comes from the save file
    for i in 0..save_data.slime_count {
        let y = -200.0 + (i as f32 * 100.0);
        commands.spawn((
            AnimationType::SlimeJumpIdle,
            Transform::from_xyz(-300.0, y, 0.0),
            Team::Player,
            PickTargetStrategy::Close,
            DeathAnimation(AnimationType::SlimeDeath),
            Health(10), // Starting health for player slimes
        ));
    }

    // Enemy army — use the first army definition for now.
    // Later, which army you fight could depend on what stage/round you're on.
    let enemy_army = &enemy_armies.armies[0];
    for i in 0..enemy_army.slime_count {
        let y = -200.0 + (i as f32 * 100.0);
        commands.spawn((
            AnimationType::SlimeJumpIdle,
            Transform::from_xyz(300.0, y, 0.0),
            Team::Enemy,
            PickTargetStrategy::Close,
            DeathAnimation(AnimationType::SlimeDeath),
            Health(10), // Starting health for enemy slimes
        ));
    }

    commands.spawn(Camera2d);
}

mod animation;
mod armies;
mod combat;
mod health;
mod movement;
mod pick_target;
mod render;
mod save_load;
