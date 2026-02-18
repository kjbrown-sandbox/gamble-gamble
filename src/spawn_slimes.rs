use bevy::prelude::*;
use rand::Rng;

use crate::{
    animation::AnimationType,
    combat::{Attack, AttackEffect, KnownAttacks},
    health::{DeathAnimation, Health},
    movement::Speed,
    pick_target::{PickTargetStrategy, Team},
    save_load::SaveData,
};

pub struct SpawnSlimesPlugin;

impl Plugin for SpawnSlimesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_slime_spawn_system)
            .add_systems(Update, spawn_slimes_system);
    }
}

#[derive(Resource)]
pub struct SlimesToSpawn {
    pub player_slimes: u32,
    pub enemy_slimes: u32,
}

#[derive(Resource)]
pub struct SlimeSpawnTimer(pub Timer);

fn setup_slime_spawn_system(mut commands: Commands, save_data: Res<SaveData>) {
    let player_slimes = save_data.slime_count;
    commands.insert_resource(SlimesToSpawn {
        player_slimes,
        enemy_slimes: 5,
    });

    commands.insert_resource(SlimeSpawnTimer(Timer::from_seconds(
        0.1,
        TimerMode::Repeating,
    )));
}

// When the timer is ready, spawn a new slime for each team and decrement each team's counter
// Each slime will need its own state for where it's at in the lerping process
// Probably need a separate system for the ease in/out
// Start with just spawning for now
fn spawn_slimes_system(
    mut commands: Commands,
    slimes_to_spawn: ResMut<SlimesToSpawn>,
    mut timer: ResMut<SlimeSpawnTimer>,
    game_time: Res<Time>,
) {
    let mut rng = rand::thread_rng();

    if timer.0.just_finished() {
        let x = rng.gen_range(-500.0..-100.0);
        let y = rng.gen_range(-300.0..300.0);
        commands.spawn((
            AnimationType::SlimeJumpIdle,
            Transform::from_xyz(x, y, 0.0),
            Team::Player,
            PickTargetStrategy::Close,
            DeathAnimation(AnimationType::SlimeDeath),
            Health(10),
            Speed(125.0),
            // KnownAttacks is the entity's "move list" — all attacks it can perform.
            // pick_attack_system will choose from these based on distance to target.
            KnownAttacks(vec![Attack {
                animation: AnimationType::SlimeAttack,
                hit_frame: 3, // damage lands on frame 3 of the attack animation
                on_hit_effect: AttackEffect {
                    damage: 2,
                    knockback: 0.0,
                },
                range: 60.0, // must be >= 50.0 (movement stops at 50 units)
            }]),
        ));

        let x = rng.gen_range(100.0..500.0);
        let y = rng.gen_range(-300.0..300.0);
        commands.spawn((
            AnimationType::SlimeJumpIdle,
            Transform::from_xyz(x, y, 0.0),
            Team::Enemy,
            PickTargetStrategy::Close,
            DeathAnimation(AnimationType::SlimeDeath),
            Health(10),
            Speed(125.0),
            KnownAttacks(vec![Attack {
                animation: AnimationType::SlimeAttack,
                hit_frame: 3,
                on_hit_effect: AttackEffect {
                    damage: 2,
                    knockback: 0.0,
                },
                range: 60.0,
            }]),
        ));
    }

    timer.0.tick(game_time.delta());
}
