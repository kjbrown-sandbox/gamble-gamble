// systems/spawn.rs - Spawning soldiers and UI
// The startup system that creates our two soldiers and UI elements.

use bevy::prelude::*;
use crate::components::{Health, Team, Soldier, AttackCooldown, HealthDisplay, GameOverText};

/// Spawn system - runs once at startup to create the initial game state.
/// This is a Startup system, so it runs exactly once when the app starts.
pub fn spawn_soldiers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn camera first - required for rendering
    commands.spawn(Camera2d);

    // Create reusable mesh and material for both soldiers
    let soldier_mesh = meshes.add(Circle::new(30.0));
    let white_material = materials.add(ColorMaterial::from(Color::WHITE));

    // Spawn player soldier (left side)
    let player_entity = commands.spawn((
        Soldier,
        Health::new(100),
        Team { is_player: true },
        AttackCooldown::new(1.0), // Attack every 1 second
        Transform::default().with_translation(Vec3::new(-150.0, 0.0, 0.0)),
        Mesh2d(soldier_mesh.clone()),
        MeshMaterial2d(white_material.clone()),
    )).id();

    // Spawn enemy soldier (right side)
    let enemy_entity = commands.spawn((
        Soldier,
        Health::new(100),
        Team { is_player: false },
        AttackCooldown::new(1.0),
        Transform::default().with_translation(Vec3::new(150.0, 0.0, 0.0)),
        Mesh2d(soldier_mesh),
        MeshMaterial2d(white_material),
    )).id();

    // Create a UI root node (invisible container for all UI)
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        BackgroundColor(Color::NONE),
    )).with_children(|parent| {
        // Top section for health displays
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(20.0),
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        )).with_children(|parent| {
            // Player health display (left)
            parent.spawn((
                Text::new("Player HP: 100"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                HealthDisplay {
                    soldier_entity: player_entity,
                    is_player: true,
                },
            ));

            // Enemy health display (right)
            parent.spawn((
                Text::new("Enemy HP: 100"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                HealthDisplay {
                    soldier_entity: enemy_entity,
                    is_player: false,
                },
            ));
        });

        // Center section for game over message (hidden by default)
        parent.spawn((
            Text::new(""),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Percent(45.0),
                left: Val::Percent(35.0),
                ..default()
            },
            GameOverText,
        ));
    });
}
