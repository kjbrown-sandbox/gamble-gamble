use bevy::prelude::*;
use rand::Rng;

use crate::{
    animation::{AnimationType, IdleAnimation, VictoryAnimation},
    armies::Army,
    combat::{Attack, AttackEffect, BlockChance, KnownAttacks, Shield, TimeBetweenAttacks},
    health::{DeathAnimation, Health},
    movement::{Speed, StaysNearParent},
    pick_target::{PickTargetStrategy, Team},
    save_load::SaveData,
    setup_round::Inert,
    sprite_modifications::{LerpType, SpriteModification},
    GameState,
};

#[derive(Component)]
pub struct GoopValue(pub u32);

pub struct SpawnSlimesPlugin;

impl Plugin for SpawnSlimesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Combat), start_combat_system)
            .add_systems(
                Update,
                spawn_slimes_system
                    .run_if(in_state(GameState::Combat))
                    .run_if(resource_exists::<SlimeSpawnTimer>),
            );
    }
}

#[derive(Resource)]
pub struct SlimesToSpawn {
    pub player_army: Option<Army>,
    pub enemy_army: Army,
}

#[derive(Resource)]
pub struct SlimeSpawnTimer(pub Timer);

/// Queues armies for staggered spawning. Pass `None` for `player_army` when
/// survivors are already on the field (e.g. venture further).
pub fn setup_slime_spawn(commands: &mut Commands, player_army: Option<Army>, enemy_army: Army) {
    commands.insert_resource(SlimesToSpawn {
        player_army,
        enemy_army,
    });
    commands.insert_resource(SlimeSpawnTimer(Timer::from_seconds(
        0.1,
        TimerMode::Repeating,
    )));
}

fn start_combat_system(mut commands: Commands, save_data: Res<SaveData>) {
    setup_slime_spawn(
        &mut commands,
        Some(save_data.army.clone()),
        crate::armies::create_enemy_army(),
    );
}

fn spawn_slimes_system(
    mut commands: Commands,
    mut slimes_to_spawn: ResMut<SlimesToSpawn>,
    mut timer: ResMut<SlimeSpawnTimer>,
    game_time: Res<Time>,
) {
    if timer.0.just_finished() {
        if let Some(ref mut player) = slimes_to_spawn.player_army {
            if player.normal.count > 0 {
                spawn_normal_slime(&mut commands, Team::Player, player.normal.hp);
                player.normal.count -= 1;
            } else if player.tanks.count > 0 {
                spawn_tank_slime(
                    &mut commands,
                    Team::Player,
                    player.tanks.hp,
                    player.tanks.block_chance,
                    player.tanks.stun_chance,
                );
                player.tanks.count -= 1;
            } else if player.wizards.count > 0 {
                spawn_wizard_slime(
                    &mut commands,
                    Team::Player,
                    player.wizards.hp,
                    player.wizards.spell_range,
                    player.wizards.aoe_damage,
                    player.wizards.spear_knockback,
                );
                player.wizards.count -= 1;
            }
        }

        let enemy = &mut slimes_to_spawn.enemy_army;
        if enemy.normal.count > 0 {
            spawn_normal_slime(&mut commands, Team::Enemy, enemy.normal.hp);
            enemy.normal.count -= 1;
        } else if enemy.tanks.count > 0 {
            spawn_tank_slime(
                &mut commands,
                Team::Enemy,
                enemy.tanks.hp,
                enemy.tanks.block_chance,
                enemy.tanks.stun_chance,
            );
            enemy.tanks.count -= 1;
        } else if enemy.wizards.count > 0 {
            spawn_wizard_slime(
                &mut commands,
                Team::Enemy,
                enemy.wizards.hp,
                enemy.wizards.spell_range,
                enemy.wizards.aoe_damage,
                enemy.wizards.spear_knockback,
            );
            enemy.wizards.count -= 1;
        }
    }

    let player_remaining = slimes_to_spawn
        .player_army
        .as_ref()
        .map(|p| p.normal.count + p.tanks.count + p.wizards.count)
        .unwrap_or(0);
    let enemy = &slimes_to_spawn.enemy_army;
    let enemy_remaining = enemy.normal.count + enemy.tanks.count + enemy.wizards.count;

    if player_remaining + enemy_remaining == 0 {
        commands.remove_resource::<SlimeSpawnTimer>();
        commands.remove_resource::<SlimesToSpawn>();
    }

    timer.0.tick(game_time.delta());
}

fn spawn_normal_slime(commands: &mut Commands, team: Team, hp: i32) -> Entity {
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

    let entity = commands
        .spawn((
            DespawnOnExit(GameState::Combat),
            idle_anim,
            IdleAnimation(idle_anim),
            VictoryAnimation(victory_anim),
            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(scale as f32)),
            team,
            PickTargetStrategy::Close,
            Sprite {
                flip_x: team == Team::Enemy,
                ..default()
            },
            DeathAnimation(death_anim),
            Health(hp),
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
        .id();

    if team == Team::Enemy {
        commands.entity(entity).insert(GoopValue(1));
    }

    entity
}

fn spawn_tank_slime(
    commands: &mut Commands,
    team: Team,
    hp: i32,
    block_chance: f32,
    stun_chance: f32,
) -> Entity {
    let entity = spawn_normal_slime(commands, team, hp);

    // Shield sits 30px to the right in the slime's facing direction.
    // Player slimes face right (+x), enemy slimes face left (−x).
    let shield_x = match team {
        Team::Player => 30.0,
        Team::Enemy => -30.0,
    };

    // Override the normal slime's attack with the tank's stun attack.
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
            stun_chance,
            stun_duration: 1.5,
            ..default()
        },
        range: 65.0,
    }]));
    if team == Team::Enemy {
        commands.entity(entity).insert(GoopValue(3));
    }

    commands
        .entity(entity)
        .insert(BlockChance(block_chance))
        .with_child((
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

fn spawn_wizard_slime(
    commands: &mut Commands,
    team: Team,
    hp: i32,
    spell_range: f32,
    aoe_damage: i32,
    spear_knockback: f32,
) -> Entity {
    let entity = spawn_normal_slime(commands, team, hp);

    let x_displacement = 30.0;
    let shield_x = match team {
        Team::Player => x_displacement,
        Team::Enemy => -x_displacement,
    };

    let mage_cast_anim = match team {
        Team::Player => AnimationType::MageCast,
        Team::Enemy => AnimationType::EnemyMageCast,
    };

    if team == Team::Enemy {
        commands.entity(entity).insert(GoopValue(2));
    }

    commands
        .entity(entity)
        .insert(KnownAttacks(vec![Attack {
            animation: mage_cast_anim,
            hit_frame: 0,
            on_hit_effect: AttackEffect {
                damage: aoe_damage,
                aoe_distance: Some(100.0),
                ..Default::default()
            },
            range: spell_range,
        }]))
        .with_child((
            AnimationType::FrozenSpearIdle,
            IdleAnimation(AnimationType::FrozenSpearIdle),
            Transform::from_xyz(shield_x, -10.0, 1.0).with_scale(Vec3::splat(4.0)),
            team,
            PickTargetStrategy::Closest,
            Speed(25.0),
            StaysNearParent(50.0),
            KnownAttacks(vec![Attack {
                animation: AnimationType::FrozenSpearAttack,
                hit_frame: 4,
                on_hit_effect: AttackEffect {
                    damage: 1,
                    knockback: spear_knockback,
                    ..Default::default()
                },
                range: 65.0,
            }]),
            TimeBetweenAttacks(2.0),
        ));

    entity
}
