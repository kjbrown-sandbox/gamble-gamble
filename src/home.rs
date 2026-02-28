use bevy::prelude::*;

use crate::{GameFont, GameState};

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Home), setup_home)
            .add_systems(
                Update,
                (battle_button_system, button_hover_system).run_if(in_state(GameState::Home)),
            );
    }
}

#[derive(Component)]
struct BattleButton;

const BG_COLOR: Color = Color::srgb(0.08, 0.18, 0.08);
const BUTTON_COLOR: Color = Color::srgb(0.15, 0.4, 0.15);
const BUTTON_HOVER_COLOR: Color = Color::srgb(0.2, 0.55, 0.2);
const BUTTON_PRESSED_COLOR: Color = Color::srgb(0.1, 0.3, 0.1);

fn setup_home(mut commands: Commands, game_font: Res<GameFont>) {
    commands
        .spawn((
            DespawnOnExit(GameState::Home),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BG_COLOR),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    BattleButton,
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_COLOR),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Battle"),
                        TextFont {
                            font: game_font.0.clone(),
                            font_size: 50.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn battle_button_system(
    query: Query<&Interaction, (Changed<Interaction>, With<BattleButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Combat);
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
