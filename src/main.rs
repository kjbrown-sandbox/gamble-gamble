use bevy::prelude::*;
use rand::seq::IteratorRandom;
use rand::Rng;

use crate::animation::AnimationType;
use crate::armies::EnemyArmies;
use crate::combat::{Attack, AttackEffect, KnownAttacks};
use crate::health::{DeathAnimation, Health};
use crate::movement::Speed;
use crate::pick_target::{PickTargetStrategy, Team};
use crate::save_load::SaveData;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            save_load::SaveLoadPlugin,
            audio::AudioPlugin,
            animation::AnimationPlugin,
            render::RenderPlugin,
            armies::ArmiesPlugin,
            movement::MovementPlugin,
            pick_target::PickTargetPlugin,
            health::HealthPlugin,
            combat::CombatPlugin,
            end_round::EndRoundPlugin,
            setup_round::SetupRoundPlugin,
            spawn_slimes::SpawnSlimesPlugin,
            shaders_lite::ShadersLitePlugin,
            sprite_modifications::SpriteModificationsPlugin,
        ))
        // spawn_slimes needs three resources to exist first:
        //   - SpriteSheets (from animation::load_sprite_sheets)
        //   - SaveData (from save_load's startup system)
        //   - EnemyArmies (from armies plugin, via init_resource — available immediately)
        .add_systems(Startup, spawn_slimes.after(animation::load_sprite_sheets))
        .add_systems(Update, kill_random_on_spacebar)
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
    //  let mut rng = rand::thread_rng();

    //  // Player army — count comes from the save file
    //  for _ in 0..save_data.slime_count {
    //      let x = rng.gen_range(-500.0..-100.0);
    //      let y = rng.gen_range(-300.0..300.0);
    //      commands.spawn((
    //          AnimationType::SlimeJumpIdle,
    //          Transform::from_xyz(x, y, 0.0),
    //          Team::Player,
    //          PickTargetStrategy::Close,
    //          DeathAnimation(AnimationType::SlimeDeath),
    //          Health(10),
    //          Speed(125.0),
    //          // KnownAttacks is the entity's "move list" — all attacks it can perform.
    //          // pick_attack_system will choose from these based on distance to target.
    //          KnownAttacks(vec![Attack {
    //              animation: AnimationType::SlimeAttack,
    //              hit_frame: 3, // damage lands on frame 3 of the attack animation
    //              on_hit_effect: AttackEffect {
    //                  damage: 2,
    //                  knockback: 0.0,
    //              },
    //              range: 60.0, // must be >= 50.0 (movement stops at 50 units)
    //          }]),
    //      ));
    //  }

    //  // Enemy army — use the first army definition for now.
    //  // Later, which army you fight could depend on what stage/round you're on.
    //  let enemy_army = &enemy_armies.armies[0];
    //  for _ in 0..enemy_army.slime_count {
    //      let x = rng.gen_range(100.0..500.0);
    //      let y = rng.gen_range(-300.0..300.0);
    //      commands.spawn((
    //          AnimationType::SlimeJumpIdle,
    //          Transform::from_xyz(x, y, 0.0),
    //          Team::Enemy,
    //          PickTargetStrategy::Close,
    //          DeathAnimation(AnimationType::SlimeDeath),
    //          Health(10),
    //          Speed(125.0),
    //          KnownAttacks(vec![Attack {
    //              animation: AnimationType::SlimeAttack,
    //              hit_frame: 3,
    //              on_hit_effect: AttackEffect {
    //                  damage: 2,
    //                  knockback: 0.0,
    //              },
    //              range: 60.0,
    //          }]),
    //      ));
    //  }

    commands.spawn(Camera2d);
}

/// Debug system: press spacebar to kill a random slime.
fn kill_random_on_spacebar(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Health>) {
    if keyboard.just_pressed(KeyCode::Space) {
        let mut rng = rand::thread_rng();
        // iter_mut() gives us mutable access to Health components.
        // choose() picks one at random, returning Option (None if query is empty).
        if let Some(mut health) = query.iter_mut().choose(&mut rng) {
            health.0 = 0;
        }
    }
}

mod animation;
mod armies;
mod audio;
mod combat;
mod end_round;
mod health;
mod movement;
mod pick_target;
mod render;
mod save_load;
mod setup_round;
mod shaders_lite;
mod spawn_slimes;
mod sprite_modifications;
