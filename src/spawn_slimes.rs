use bevy::{prelude::*, state::commands};
use rand::Rng;

use crate::{
    animation::{AnimationType, IdleAnimation, VictoryAnimation},
    combat::{Attack, AttackEffect, BlockChance, KnownAttacks, Shield, TimeBetweenAttacks},
    health::{DeathAnimation, Health},
    movement::{Speed, StaysNearParent},
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

struct SlimeAmounts {
    normal_slimes: u32,
    tanks: u32,
    wizards: u32,
}

#[derive(Resource)]
pub struct SlimesToSpawn {
    pub player_slimes: SlimeAmounts,
    pub enemy_slimes: SlimeAmounts,
}

#[derive(Resource)]
pub struct SlimeSpawnTimer(pub Timer);

fn setup_slime_spawn_system(mut commands: Commands, save_data: Res<SaveData>) {
    commands.insert_resource(SlimesToSpawn {
        player_slimes: SlimeAmounts {
            normal_slimes: save_data.normal_slimes,
            tanks: save_data.tanks,
            wizards: save_data.wizards,
        },
        enemy_slimes: SlimeAmounts {
            normal_slimes: 0,
            tanks: 1,
            wizards: 0,
        },
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
        if slimes_to_spawn.player_slimes.normal_slimes > 0 {
            spawn_normal_slime(&mut commands, Team::Player);
            slimes_to_spawn.player_slimes.normal_slimes -= 1;
        } else if slimes_to_spawn.player_slimes.tanks > 0 {
            spawn_tank_slime(&mut commands, Team::Player);
            slimes_to_spawn.player_slimes.tanks -= 1;
        } else if slimes_to_spawn.player_slimes.wizards > 0 {
            spawn_wizard_slime(&mut commands, Team::Player);
            slimes_to_spawn.player_slimes.wizards -= 1;
        }

        if slimes_to_spawn.enemy_slimes.normal_slimes > 0 {
            spawn_normal_slime(&mut commands, Team::Enemy);
            slimes_to_spawn.enemy_slimes.normal_slimes -= 1;
        } else if slimes_to_spawn.enemy_slimes.tanks > 0 {
            spawn_tank_slime(&mut commands, Team::Enemy);
            slimes_to_spawn.enemy_slimes.tanks -= 1;
        } else if slimes_to_spawn.enemy_slimes.wizards > 0 {
            spawn_wizard_slime(&mut commands, Team::Enemy);
            slimes_to_spawn.enemy_slimes.wizards -= 1;
        }
    }

    if slimes_to_spawn.player_slimes.normal_slimes
        + slimes_to_spawn.player_slimes.tanks
        + slimes_to_spawn.player_slimes.wizards
        + slimes_to_spawn.enemy_slimes.normal_slimes
        + slimes_to_spawn.enemy_slimes.tanks
        + slimes_to_spawn.enemy_slimes.wizards
        == 0
    {
        commands.remove_resource::<SlimeSpawnTimer>();
        commands.remove_resource::<SlimesToSpawn>();
    }

    timer.0.tick(game_time.delta());
}

fn spawn_normal_slime(commands: &mut Commands, team: Team) -> Entity {
    let mut rng = rand::thread_rng();

    let player_x = rng.gen_range(-500.0..-100.0);
    let enemy_x = rng.gen_range(100.0..500.0);
    let y = rng.gen_range(-200.0..200.0);
    let x = match team {
        Team::Player => player_x,
        Team::Enemy => enemy_x,
    };
    let scale = match team {
        Team::Player => 1,
        Team::Enemy => 2,
    };

    // Pick the correct animation variants based on team
    let (idle_anim, attack_anim, death_anim, victory_anim) = match team {
        Team::Player => (
            AnimationType::SlimeMoveSmallJump,
            AnimationType::SlimeAttack,
            AnimationType::SlimeDeath,
            AnimationType::SlimeJumpIdle,
        ),
        Team::Enemy => (
            AnimationType::EnemySlimeMoveSmallJump,
            AnimationType::EnemySlimeAttack,
            AnimationType::EnemySlimeDeath,
            AnimationType::EnemySlimeJumpIdle,
        ),
    };

    commands
        .spawn((
            idle_anim,
            IdleAnimation(idle_anim),
            VictoryAnimation(victory_anim),
            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(scale as f32)),
            team,
            PickTargetStrategy::Close,
            Sprite {
                flip_x: team == Team::Enemy, // Flip enemy slimes to face left
                ..default()
            },
            DeathAnimation(death_anim),
            Health(5),
            Speed(125.0),
            KnownAttacks(vec![Attack {
                animation: attack_anim,
                hit_frame: 3,
                on_hit_effect: AttackEffect {
                    damage: 1,
                    knockback: 0.0,
                    ..Default::default()
                },
                range: 65.0,
            }]),
            Inert,
            SpriteModification {
                lerp: LerpType::EaseInOut,
                timer: Timer::from_seconds(3.0, TimerMode::Once),
            },
        ))
        .id()
}

fn spawn_tank_slime(commands: &mut Commands, team: Team) -> Entity {
    let entity = spawn_normal_slime(commands, team);

    // Shield sits 30px to the right in the slime's facing direction.
    // Player slimes face right (+x), enemy slimes face left (−x).
    let shield_x = match team {
        Team::Player => 30.0,
        Team::Enemy => -30.0,
    };

    // Spawn the iceberg shield as a child entity so it moves, flips,
    // and despawns automatically with the parent slime.
    // z = 1.0 draws the shield in front of the slime sprite.
    // Override the normal slime's attack with the tank's stun attack.
    // We remove the old KnownAttacks and insert a new one. The tank hits harder
    // and stuns 100% of the time for 1.5 seconds, but has no knockback.
    let (attack_anim, _) = match team {
        Team::Player => (AnimationType::SlimeAttack, ()),
        Team::Enemy => (AnimationType::EnemySlimeAttack, ()),
    };

    commands.entity(entity).insert(KnownAttacks(vec![Attack {
        animation: attack_anim,
        hit_frame: 3,
        on_hit_effect: AttackEffect {
            damage: 2,
            knockback: 0.0,
            stun_chance: 0.1,
            stun_duration: 1.5,
            ..default()
        },
        range: 65.0,
    }]));
    // BlockChance is on the parent slime (the defender), not the shield child.
    commands
        .entity(entity)
        .insert((Health(10), BlockChance(0.2)))
        .with_child((
            // Shield marker lets on_block_attack_observer find this specific child
            Shield,
            AnimationType::IcebergIdle,
            Transform::from_xyz(shield_x, -20.0, 1.0).with_scale(Vec3::splat(3.0)),
            Sprite {
                flip_x: team == Team::Enemy,
                ..default()
            },
        ));

    entity
}

fn spawn_wizard_slime(commands: &mut Commands, team: Team) -> Entity {
    let entity = spawn_normal_slime(commands, team);

    let x_displacement = 30.0;
    let shield_x = match team {
        Team::Player => x_displacement,
        Team::Enemy => -x_displacement,
    };

    let mage_cast_anim = match team {
        Team::Player => AnimationType::MageCast,
        Team::Enemy => AnimationType::EnemyMageCast,
    };

    commands
        .entity(entity)
        .insert((
            Health(5),
            KnownAttacks(vec![Attack {
                animation: mage_cast_anim,
                hit_frame: 0, // single frame — damage lands immediately
                on_hit_effect: AttackEffect {
                    damage: 1,
                    aoe_distance: Some(100.0),
                    ..Default::default()
                },
                range: 1000.0,
            }]),
        ))
        .with_child((
            AnimationType::FrozenSpearIdle,
            IdleAnimation(AnimationType::FrozenSpearIdle),
            Transform::from_xyz(shield_x, -10.0, 1.0).with_scale(Vec3::splat(4.0)),
            // ── Combat components — spear fights independently ──
            team,                        // inherits parent's team so it targets enemies
            PickTargetStrategy::Closest, // constantly re-evaluates to always hit the nearest enemy
            Speed(25.0),                 // moves slowly toward its target
            StaysNearParent(50.0),       // but can't drift more than 50 units from the wizard
            KnownAttacks(vec![Attack {
                animation: AnimationType::FrozenSpearAttack,
                hit_frame: 5, // damage lands mid-animation
                on_hit_effect: AttackEffect {
                    damage: 1,
                    knockback: 200.0,
                    ..Default::default()
                },
                range: 65.0,
            }]),
            TimeBetweenAttacks(2.0), // 2-second cooldown prevents spamming every frame
        ));

    entity
}
