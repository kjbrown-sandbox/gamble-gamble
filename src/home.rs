use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

use crate::save_load::SaveData;
use crate::screen_fade::{spawn_screen_fade, ScreenFade};
use crate::{GameFont, GameState};

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Home), setup_home)
            .add_systems(
                Update,
                (
                    battle_button_system,
                    army_button_system,
                    update_count_text_system,
                    update_goop_text_system,
                    update_cost_tooltip_system,
                    button_hover_system,
                )
                    .run_if(in_state(GameState::Home)),
            );
    }
}

#[derive(Component)]
struct BattleButton;

#[derive(Clone, Copy)]
enum SlimeType {
    Normal,
    Tank,
    Wizard,
}

impl SlimeType {
    fn base_cost(self) -> u32 {
        match self {
            SlimeType::Normal => 1,
            SlimeType::Tank | SlimeType::Wizard => 10,
        }
    }
}

fn get_count(slime_type: SlimeType, save_data: &SaveData) -> u32 {
    match slime_type {
        SlimeType::Normal => save_data.army.normal.count,
        SlimeType::Tank => save_data.army.tanks.count,
        SlimeType::Wizard => save_data.army.wizards.count,
    }
}

#[derive(Component)]
struct SlimeCountText(SlimeType);

#[derive(Component)]
struct GoopText;

#[derive(Component)]
struct CostTooltip;

#[derive(Component)]
struct ArmyButton {
    slime_type: SlimeType,
    delta: i32,
}

const BG_COLOR: Color = Color::srgb(0.08, 0.18, 0.08);
const BUTTON_COLOR: Color = Color::srgb(0.15, 0.4, 0.15);
const BUTTON_HOVER_COLOR: Color = Color::srgb(0.2, 0.55, 0.2);
const BUTTON_PRESSED_COLOR: Color = Color::srgb(0.1, 0.3, 0.1);

fn setup_home(mut commands: Commands, game_font: Res<GameFont>, save_data: Res<SaveData>) {
    let font = game_font.0.clone();

    commands
        .spawn((
            DespawnOnExit(GameState::Home),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(BG_COLOR),
        ))
        .with_children(|root| {
            root.spawn((
                GoopText,
                Text::new(format!("Slime Goop: {}", save_data.goop)),
                TextFont {
                    font: font.clone(),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.9, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ))
            .with_child((
                CostTooltip,
                TextSpan::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 36.0,
                    ..default()
                },
            ));

            root.spawn((
                Text::new("Your Army"),
                TextFont {
                    font: font.clone(),
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Slime rows
            let rows = [
                (SlimeType::Normal, "Normal", save_data.army.normal.count),
                (SlimeType::Tank, "Tank", save_data.army.tanks.count),
                (SlimeType::Wizard, "Wizard", save_data.army.wizards.count),
            ];

            for (slime_type, label, count) in rows {
                spawn_slime_row(root, &font, slime_type, label, count);
            }

            // Battle button
            root.spawn((
                BattleButton,
                Button,
                Node {
                    width: Val::Px(300.0),
                    height: Val::Px(80.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_COLOR),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Battle"),
                    TextFont {
                        font: font.clone(),
                        font_size: 50.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

fn spawn_slime_row(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font: &Handle<Font>,
    slime_type: SlimeType,
    label: &str,
    count: u32,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(15.0),
            ..default()
        })
        .with_children(|row| {
            // Count label
            row.spawn((
                SlimeCountText(slime_type),
                Text::new(format!("{label}: {count}")),
                TextFont {
                    font: font.clone(),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    width: Val::Px(200.0),
                    ..default()
                },
            ));

            // [-] button
            spawn_army_button(row, font, slime_type, -1, "-");

            // [+] button
            spawn_army_button(row, font, slime_type, 1, "+");
        });
}

fn spawn_army_button(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    font: &Handle<Font>,
    slime_type: SlimeType,
    delta: i32,
    symbol: &str,
) {
    parent
        .spawn((
            ArmyButton { slime_type, delta },
            Button,
            Node {
                width: Val::Px(50.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BUTTON_COLOR),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(symbol),
                TextFont {
                    font: font.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn army_button_system(
    query: Query<(&Interaction, &ArmyButton), Changed<Interaction>>,
    mut save_data: ResMut<SaveData>,
) {
    for (interaction, army_btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let base = army_btn.slime_type.base_cost();
        let current = get_count(army_btn.slime_type, &save_data);

        if army_btn.delta > 0 {
            let cost = base * (current + 1);
            if save_data.goop < cost {
                continue;
            }
            save_data.goop -= cost;
        } else if current == 0 {
            continue;
        } else {
            let refund = base * current;
            save_data.goop += refund;
        }

        let count = match army_btn.slime_type {
            SlimeType::Normal => &mut save_data.army.normal.count,
            SlimeType::Tank => &mut save_data.army.tanks.count,
            SlimeType::Wizard => &mut save_data.army.wizards.count,
        };
        let new_val = *count as i32 + army_btn.delta;
        *count = new_val.max(0) as u32;
    }
}

fn update_count_text_system(
    mut query: Query<(&SlimeCountText, &mut Text)>,
    save_data: Res<SaveData>,
) {
    if !save_data.is_changed() {
        return;
    }

    for (slime_text, mut text) in &mut query {
        let (label, count) = match slime_text.0 {
            SlimeType::Normal => ("Normal", save_data.army.normal.count),
            SlimeType::Tank => ("Tank", save_data.army.tanks.count),
            SlimeType::Wizard => ("Wizard", save_data.army.wizards.count),
        };
        **text = format!("{label}: {count}");
    }
}

fn update_goop_text_system(
    mut query: Query<&mut Text, With<GoopText>>,
    save_data: Res<SaveData>,
) {
    if !save_data.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!("Slime Goop: {}", save_data.goop);
    }
}

fn battle_button_system(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<BattleButton>)>,
    existing_fade: Query<(), With<ScreenFade>>,
) {
    if !existing_fade.is_empty() {
        return;
    }
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            spawn_screen_fade(&mut commands, GameState::Combat);
        }
    }
}

const COST_COLOR: Color = Color::srgb(0.9, 0.3, 0.3);
const REFUND_COLOR: Color = Color::srgb(0.3, 0.9, 0.3);

fn update_cost_tooltip_system(
    army_query: Query<(&Interaction, &ArmyButton)>,
    mut tooltip_query: Query<(&mut TextSpan, &mut TextColor), With<CostTooltip>>,
    save_data: Res<SaveData>,
) {
    let Ok((mut span, mut color)) = tooltip_query.single_mut() else {
        return;
    };

    let mut found_hover = false;
    for (interaction, army_btn) in &army_query {
        if *interaction != Interaction::Hovered {
            continue;
        }
        found_hover = true;

        let base = army_btn.slime_type.base_cost();
        let current = get_count(army_btn.slime_type, &save_data);

        if army_btn.delta > 0 {
            let cost = base * (current + 1);
            **span = format!(" -{cost}");
            *color = TextColor(COST_COLOR);
        } else if current > 0 {
            let refund = base * current;
            **span = format!(" +{refund}");
            *color = TextColor(REFUND_COLOR);
        } else {
            **span = String::new();
        }
        break;
    }

    if !found_hover {
        **span = String::new();
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
