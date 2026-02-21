use bevy::prelude::*;
use rand::Rng;

use crate::{
    animation::AnimationType,
    combat::{Attack, AttackEffect, KnownAttacks},
    health::{DeathAnimation, Health},
    movement::Speed,
    pick_target::{PickTargetStrategy, Team},
    save_load::SaveData,
    setup_round::Inert,
    sprite_modifications::{LerpType, SpriteModification},
};

pub struct SpawnSlimesPlugin;

impl Plugin for SpawnSlimesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_slime_spawn_system)
            .add_systems(
                Update,
                spawn_slimes_system.run_if(resource_exists::<SlimeSpawnTimer>),
            );
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
    mut slimes_to_spawn: ResMut<SlimesToSpawn>,
    mut timer: ResMut<SlimeSpawnTimer>,
    game_time: Res<Time>,
) {
    if timer.0.just_finished() {
        if slimes_to_spawn.player_slimes > 0 {
            spawn_normal_slime(&mut commands, Team::Player);
            slimes_to_spawn.player_slimes -= 1;
        }

        if slimes_to_spawn.enemy_slimes > 0 {
            spawn_normal_slime(&mut commands, Team::Enemy);
            slimes_to_spawn.enemy_slimes -= 1;
        }
    }

    if slimes_to_spawn.player_slimes == 0 && slimes_to_spawn.enemy_slimes == 0 {
        commands.remove_resource::<SlimeSpawnTimer>();
        commands.remove_resource::<SlimesToSpawn>();
    }

    timer.0.tick(game_time.delta());
}

fn spawn_normal_slime(commands: &mut Commands, team: Team) {
    let mut rng = rand::thread_rng();

    let player_x = rng.gen_range(-500.0..-100.0);
    let enemy_x = rng.gen_range(100.0..500.0);
    let y = rng.gen_range(-300.0..300.0);
    let x = match team {
        Team::Player => player_x,
        Team::Enemy => enemy_x,
    };
    let scale = match team {
        Team::Player => 1,
        Team::Enemy => 2,
    };

    let player_animation_idle = AnimationType::SlimeMoveSmallJump;
    commands.spawn((
        player_animation_idle,
        Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(scale as f32)),
        team,
        PickTargetStrategy::Close,
        Sprite {
            flip_y: team == Team::Enemy,
            ..default()
        },
        DeathAnimation(AnimationType::SlimeDeath),
        Health(10),
        Speed(125.0),
        KnownAttacks(vec![Attack {
            animation: AnimationType::SlimeAttack,
            hit_frame: 3, // damage lands on frame 3 of the attack animation
            on_hit_effect: AttackEffect {
                damage: 2,
                knockback: 0.0,
            },
            range: 65.0,
        }]),
        Inert,
        SpriteModification {
            lerp: LerpType::EaseInOut,
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        },
    ));
}
