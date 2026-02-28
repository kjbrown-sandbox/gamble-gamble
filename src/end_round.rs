use bevy::prelude::*;

use crate::animation::{AnimationType, VictoryAnimation};
use crate::pick_target::Team;
use crate::setup_round::PreGameTimer;
use crate::{GameFont, GameState};

pub struct EndRoundPlugin;

impl Plugin for EndRoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                check_round_end_system.run_if(
                    not(resource_exists::<RoundResult>)
                        .and(not(resource_exists::<PreGameTimer>)),
                ),
                go_home_button_system.run_if(resource_exists::<RoundResult>),
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
                .spawn((
                    GoHomeButton,
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
                        Text::new("Go Home"),
                        TextFont {
                            font: game_font.0.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
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
