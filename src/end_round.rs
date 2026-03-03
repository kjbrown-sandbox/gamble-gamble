use bevy::prelude::*;
use rand::Rng;

use crate::animation::{AnimationState, AnimationType, IdleAnimation, VictoryAnimation};
use crate::armies::create_enemy_army;
use crate::combat::{ActiveAttack, AttackCooldown};
use crate::health::Dying;
use crate::movement::{Knockback, Speed, TargetEntity, TargetTransform};
use crate::pick_target::Team;
use crate::render::Background;
use crate::save_load::SaveData;
use crate::screen_fade::{spawn_screen_fade, ScreenFade};
use crate::setup_round::{Inert, PreGameTimer, StunTimer};
use crate::spawn_slimes::{setup_slime_spawn, GoopValue, SlimeSpawnTimer, SlimesToSpawn};
use crate::special_abilities::{Merging, PreMerging};
use crate::{CombatState, GameFont, GameState};

pub struct EndRoundPlugin;

impl Plugin for EndRoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Combat), cleanup_combat_resources)
            .add_systems(
                OnEnter(GameState::Combat),
                (init_goop_earned, spawn_combat_hud),
            )
            .add_systems(OnEnter(CombatState::PostCombat), enter_post_combat)
            .add_systems(
                Update,
                (check_round_end_system, accumulate_goop_system)
                    .run_if(in_state(CombatState::DuringCombat)),
            )
            .add_systems(Update, update_goop_text.run_if(in_state(GameState::Combat)))
            .add_systems(
                Update,
                (
                    go_home_button_system,
                    venture_further_button_system,
                    button_hover_system,
                )
                    .run_if(in_state(CombatState::PostCombat)),
            );
    }
}

#[derive(Resource, PartialEq)]
pub enum RoundResult {
    Victory,
    Defeat,
}

#[derive(Resource, Default)]
pub struct GoopEarned(pub u32);

#[derive(Resource)]
pub struct CombatLevel(pub u32);

#[derive(Component)]
struct GoHomeButton;

#[derive(Component)]
struct VentureFurtherButton;

#[derive(Component)]
struct GoopText;

#[derive(Component)]
struct GoopMultiplierText;

#[derive(Component)]
struct DepthText;

const BUTTON_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
const BUTTON_HOVER_COLOR: Color = Color::srgb(0.35, 0.35, 0.35);
const BUTTON_PRESSED_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);

fn init_goop_earned(mut commands: Commands) {
    commands.insert_resource(GoopEarned(0));
    commands.insert_resource(CombatLevel(1));
}

fn accumulate_goop_system(
    query: Query<(&GoopValue, &Team), Added<Dying>>,
    mut goop_earned: ResMut<GoopEarned>,
) {
    for (value, team) in &query {
        if *team == Team::Enemy {
            goop_earned.0 += value.0;
        }
    }
}

fn spawn_combat_hud(mut commands: Commands, game_font: Res<GameFont>) {
    commands
        .spawn((
            DespawnOnExit(GameState::Combat),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|hud| {
            // Left spacer to balance the flexbox
            hud.spawn(Node::default());

            // Bottom center: goop counter + multiplier
            hud.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    GoopMultiplierText,
                    Text::new(""),
                    TextFont {
                        font: game_font.0.clone(),
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.3, 0.9, 0.2)),
                ));
                col.spawn((
                    GoopText,
                    Text::new("Goop collected: 0"),
                    TextFont {
                        font: game_font.0.clone(),
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // Bottom right: depth level
            hud.spawn((
                DepthText,
                Text::new("Depth: 1"),
                TextFont {
                    font: game_font.0.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn update_goop_text(
    goop_earned: Res<GoopEarned>,
    mut goop_query: Query<&mut Text, With<GoopText>>,
) {
    if !goop_earned.is_changed() {
        return;
    }
    for mut text in &mut goop_query {
        **text = format!("Goop collected: {}", goop_earned.0);
    }
}

/// Checks if one team has been eliminated. If so, transitions to PostCombat.
fn check_round_end_system(teams: Query<&Team>, mut next_state: ResMut<NextState<CombatState>>) {
    let mut has_player = false;
    let mut has_enemy = false;

    for team in &teams {
        match team {
            Team::Player => has_player = true,
            Team::Enemy => has_enemy = true,
        }
        if has_player && has_enemy {
            return;
        }
    }

    next_state.set(CombatState::PostCombat);
}

/// Runs once when entering PostCombat. Determines the winner, plays victory
/// animations, marks all survivors as Inert, strips combat components, and
/// spawns the result UI.
fn enter_post_combat(
    mut commands: Commands,
    teams: Query<&Team>,
    mut survivors: Query<(Entity, &mut AnimationType, &VictoryAnimation, &Team)>,
    frozen_mergers: Query<Entity, Or<(With<PreMerging>, With<Merging>)>>,
    game_font: Res<GameFont>,
    goop_earned: Res<GoopEarned>,
    mut multiplier_query: Query<&mut Text, With<GoopMultiplierText>>,
) {
    let mut has_player = false;
    let mut has_enemy = false;

    for team in &teams {
        match team {
            Team::Player => has_player = true,
            Team::Enemy => has_enemy = true,
        }
    }

    let (result, message) = if has_player && !has_enemy {
        (RoundResult::Victory, "VICTORY!")
    } else {
        (RoundResult::Defeat, "DEFEAT!")
    };

    let is_victory = result == RoundResult::Victory;
    commands.insert_resource(result);

    if is_victory {
        let doubled = goop_earned.0 * 2;
        for mut text in &mut multiplier_query {
            **text = format!("x2 -> {}", doubled);
        }
    }

    // PreMerging removes AnimationState and spawns "!" child text.
    // Restore AnimationState (switch_animation_system will overwrite it with
    // the correct values once AnimationType changes below) and despawn the
    // children so the "!" doesn't linger.
    for entity in &frozen_mergers {
        commands
            .entity(entity)
            .insert(AnimationState::new(0.1, 1, true));
        commands.entity(entity).despawn_children();
    }

    for (entity, mut anim_type, victory_anim, _team) in survivors.iter_mut() {
        *anim_type = victory_anim.0;

        commands.entity(entity).insert(Inert).remove::<(
            StunTimer,
            TargetEntity,
            ActiveAttack,
            AttackCooldown,
            Knockback,
            PreMerging,
            Merging,
        )>();
    }

    // Spawn result UI — DespawnOnExit(CombatState::PostCombat) auto-cleans it
    commands
        .spawn((
            DespawnOnExit(CombatState::PostCombat),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(40.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(message),
                TextFont {
                    font: game_font.0.clone(),
                    font_size: 120.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(Justify::Center),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    if is_victory {
                        spawn_button(row, &game_font, "Venture Further", VentureFurtherButton);
                    }
                    spawn_button(row, &game_font, "Go Home", GoHomeButton);
                });
        });
}

fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    game_font: &Res<GameFont>,
    label: &str,
    marker: impl Component,
) {
    parent
        .spawn((
            marker,
            Button,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(65.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BUTTON_COLOR),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font: game_font.0.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn go_home_button_system(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<GoHomeButton>)>,
    existing_fade: Query<(), With<ScreenFade>>,
    goop_earned: Res<GoopEarned>,
    mut save_data: ResMut<SaveData>,
) {
    if !existing_fade.is_empty() {
        return;
    }
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            save_data.goop += goop_earned.0;
            spawn_screen_fade(&mut commands, GameState::Home);
        }
    }
}

/// When the player clicks "Venture Further", reposition survivors, spawn new
/// enemies, and transition back to PreCombat. The UI is auto-despawned by
/// DespawnOnExit(CombatState::PostCombat), and OnEnter(PreCombat) handles
/// the countdown timer.
fn venture_further_button_system(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<VentureFurtherButton>)>,
    mut player_slimes: Query<(
        Entity,
        &Team,
        &mut AnimationType,
        &IdleAnimation,
        Has<ChildOf>,
    )>,
    backgrounds: Query<(Entity, &Transform), With<Background>>,
    mut next_state: ResMut<NextState<CombatState>>,
    mut combat_level: ResMut<CombatLevel>,
    mut goop_earned: ResMut<GoopEarned>,
    mut depth_query: Query<&mut Text, (With<DepthText>, Without<GoopMultiplierText>)>,
    mut multiplier_query: Query<&mut Text, (With<GoopMultiplierText>, Without<DepthText>)>,
) {
    let mut clicked = false;
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            clicked = true;
        }
    }
    if !clicked {
        return;
    }

    // Reposition surviving player slimes to random spots on the left side
    let mut rng = rand::thread_rng();
    for (entity, team, mut anim_type, idle_anim, is_child) in player_slimes.iter_mut() {
        if *team != Team::Player {
            continue;
        }

        // Only reposition top-level entities — children use local-space transforms
        if !is_child {
            let x = rng.gen_range(-500.0..-100.0);
            let y = rng.gen_range(-200.0..200.0);
            commands
                .entity(entity)
                .insert(TargetTransform(Vec3::new(x, y, 0.0)));
        }

        *anim_type = idle_anim.0;
    }

    // Scroll background left for a travel illusion
    for (entity, transform) in backgrounds.iter() {
        let target = Vec3::new(
            transform.translation.x - 150.0,
            transform.translation.y,
            transform.translation.z,
        );
        commands
            .entity(entity)
            .insert((TargetTransform(target), Speed(60.0)));
    }

    goop_earned.0 *= 2;
    combat_level.0 += 1;
    setup_slime_spawn(&mut commands, None, create_enemy_army(combat_level.0));

    for mut text in &mut depth_query {
        **text = format!("Depth: {}", combat_level.0);
    }
    for mut text in &mut multiplier_query {
        **text = String::new();
    }

    next_state.set(CombatState::PreCombat);
}

fn button_hover_system(
    mut query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut bg) in &mut query {
        *bg = match interaction {
            Interaction::Pressed => BUTTON_PRESSED_COLOR.into(),
            Interaction::Hovered => BUTTON_HOVER_COLOR.into(),
            Interaction::None => BUTTON_COLOR.into(),
        };
    }
}

fn cleanup_combat_resources(mut commands: Commands) {
    commands.remove_resource::<RoundResult>();
    commands.remove_resource::<PreGameTimer>();
    commands.remove_resource::<SlimeSpawnTimer>();
    commands.remove_resource::<SlimesToSpawn>();
    commands.remove_resource::<GoopEarned>();
    commands.remove_resource::<CombatLevel>();
}
