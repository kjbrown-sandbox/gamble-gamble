use bevy::prelude::*;
use rand::Rng;

use crate::animation::{AnimationType, IdleAnimation, VictoryAnimation};
use crate::audio::GameAudio;
use crate::movement::{Speed, TargetTransform};
use crate::pick_target::Team;
use crate::render::Background;
use crate::setup_round::{Inert, PreGameTimer};
use crate::spawn_slimes::{SlimeAmounts, SlimeSpawnTimer, SlimesToSpawn};
use crate::utils::DespawnAfter;
use crate::{GameFont, GameState};

pub struct EndRoundPlugin;

impl Plugin for EndRoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Combat), cleanup_combat_resources)
            .add_systems(
            Update,
            (
                check_round_end_system.run_if(
                    not(resource_exists::<RoundResult>)
                        .and(not(resource_exists::<PreGameTimer>)),
                ),
                go_home_button_system.run_if(resource_exists::<RoundResult>),
                venture_further_button_system.run_if(resource_exists::<RoundResult>),
                button_hover_system,
            )
                .run_if(in_state(GameState::Combat)),
        );
    }
}

/// Resource inserted once a winner is determined, preventing further checks.
#[derive(Resource)]
pub struct RoundResult;

#[derive(Component)]
struct RoundResultText;

#[derive(Component)]
struct GoHomeButton;

#[derive(Component)]
struct VentureFurtherButton;

const BUTTON_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
const BUTTON_HOVER_COLOR: Color = Color::srgb(0.35, 0.35, 0.35);
const BUTTON_PRESSED_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);

fn check_round_end_system(
    mut commands: Commands,
    teams: Query<&Team>,
    mut survivors: Query<(&mut AnimationType, &VictoryAnimation, &Team)>,
    game_font: Res<GameFont>,
) {
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

    let message = if has_player && !has_enemy {
        "VICTORY!"
    } else {
        "DEFEAT!"
    };

    commands.insert_resource(RoundResult);

    let winning_team = if has_player {
        Some(Team::Player)
    } else if has_enemy {
        Some(Team::Enemy)
    } else {
        None
    };

    if let Some(winner) = winning_team {
        for (mut anim_type, victory_anim, team) in survivors.iter_mut() {
            if *team == winner {
                *anim_type = victory_anim.0;
            }
        }
    }

    let is_victory = has_player && !has_enemy;

    commands
        .spawn((
            RoundResultText,
            DespawnOnExit(GameState::Combat),
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

fn spawn_button(parent: &mut ChildSpawnerCommands, game_font: &Res<GameFont>, label: &str, marker: impl Component) {
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
    query: Query<&Interaction, (Changed<Interaction>, With<GoHomeButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Home);
        }
    }
}

fn venture_further_button_system(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<VentureFurtherButton>)>,
    ui_query: Query<Entity, With<RoundResultText>>,
    mut player_slimes: Query<(Entity, &Team, &mut AnimationType, &IdleAnimation)>,
    backgrounds: Query<(Entity, &Transform), With<Background>>,
    game_font: Res<GameFont>,
    audio: Res<GameAudio>,
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

    // Despawn end-round UI
    for entity in &ui_query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<RoundResult>();

    // Reposition surviving player slimes to random spots on the left side
    let mut rng = rand::thread_rng();
    for (entity, team, mut anim_type, idle_anim) in player_slimes.iter_mut() {
        if *team != Team::Player {
            continue;
        }

        let x = rng.gen_range(-500.0..-100.0);
        let y = rng.gen_range(-200.0..200.0);
        commands
            .entity(entity)
            .insert(TargetTransform(Vec3::new(x, y, 0.0)))
            .insert(Inert);

        *anim_type = idle_anim.0;
    }

    // Scroll background left for a travel illusion
    for (entity, transform) in backgrounds.iter() {
        let target = Vec3::new(
            transform.translation.x - 200.0,
            transform.translation.y,
            transform.translation.z,
        );
        commands
            .entity(entity)
            .insert((TargetTransform(target), Speed(60.0)));
    }

    // Spawn new enemies
    commands.insert_resource(SlimesToSpawn {
        player_slimes: SlimeAmounts {
            normal_slimes: 0,
            tanks: 0,
            wizards: 0,
        },
        enemy_slimes: SlimeAmounts {
            normal_slimes: 1,
            tanks: 10,
            wizards: 1,
        },
    });
    commands.insert_resource(SlimeSpawnTimer(Timer::from_seconds(0.1, TimerMode::Repeating)));

    // Start the READY/GO sequence
    commands.insert_resource(PreGameTimer(Timer::from_seconds(3.2, TimerMode::Once)));
    commands.spawn((
        Text2d::new("READY"),
        TextFont {
            font: game_font.0.clone(),
            font_size: 100.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 25.0, 1.0),
        DespawnAfter(Timer::from_seconds(3.2, TimerMode::Once)),
    ));
    commands.spawn((AudioPlayer::new(audio.ready.clone()),));
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

/// Removes combat-only resources when leaving the Combat state.
fn cleanup_combat_resources(mut commands: Commands) {
    commands.remove_resource::<RoundResult>();
    commands.remove_resource::<PreGameTimer>();
    commands.remove_resource::<SlimeSpawnTimer>();
    commands.remove_resource::<SlimesToSpawn>();
}
