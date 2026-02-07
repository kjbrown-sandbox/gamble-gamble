// systems/spawn.rs - Spawning soldiers, attacks, and UI
//
// This module handles initial game setup:
// 1. Initialize the AttackDatabase resource with attack definitions
// 2. Spawn soldiers with attack children
// 3. Create UI elements

use bevy::prelude::*;
use crate::components::{
    Health, Team, Soldier, HealthDisplay, GameOverText,
    AttackDatabase, AttackDefinition, AttackEffects, Effect, AttackInstance, AttackId,
};

/// Initialize the attack database with all available attacks.
///
/// This is a separate startup system that MUST run before spawn_soldiers.
/// We use .chain() in main.rs to ensure proper ordering.
///
/// WHY A SEPARATE SYSTEM?
/// - Separation of concerns: database setup vs entity spawning
/// - The database needs to exist before soldiers can reference attacks
/// - Makes it easy to add more attacks later without touching spawn code
pub fn setup_attacks(mut commands: Commands) {
    let mut db = AttackDatabase::default();

    // -------------------------------------------------------------------------
    // ATTACK DEFINITIONS
    // -------------------------------------------------------------------------
    // Each attack has:
    // - name: For display/debugging
    // - hit_chance: 0.0 to 1.0 probability of success
    // - cooldown: Seconds before the attack can be used again
    // - effects: What happens on success/fail/use

    // Basic Attack: Reliable but low damage
    // 80% hit chance, always deals 15 damage on hit
    db.add(AttackDefinition {
        name: "Basic Attack".to_string(),
        hit_chance: 0.8,
        cooldown: 1.0,
        effects: AttackEffects {
            on_success: vec![Effect::DamageTarget(15)],
            on_fail: vec![], // Nothing happens on miss
            on_use: vec![],  // Nothing always happens
        },
    });

    // Power Strike: High damage but risky
    // 60% hit chance, deals 30 damage on hit, but 10 self-damage on miss
    db.add(AttackDefinition {
        name: "Power Strike".to_string(),
        hit_chance: 0.6,
        cooldown: 2.0,
        effects: AttackEffects {
            on_success: vec![Effect::DamageTarget(30)],
            on_fail: vec![Effect::DamageSelf(10)], // Overswing hurts yourself!
            on_use: vec![],
        },
    });

    // Reckless Slam: Very high damage, always costs HP
    // 70% hit chance, 40 damage on hit, always costs 5 HP to use
    db.add(AttackDefinition {
        name: "Reckless Slam".to_string(),
        hit_chance: 0.7,
        cooldown: 3.0,
        effects: AttackEffects {
            on_success: vec![Effect::DamageTarget(40)],
            on_fail: vec![],
            on_use: vec![Effect::DamageSelf(5)], // Always costs HP
        },
    });

    // Healing Strike: Weak attack that heals on hit
    // 90% hit chance, 10 damage, heals 5 HP on success
    db.add(AttackDefinition {
        name: "Healing Strike".to_string(),
        hit_chance: 0.9,
        cooldown: 2.5,
        effects: AttackEffects {
            on_success: vec![
                Effect::DamageTarget(10),
                Effect::HealSelf(5),
            ],
            on_fail: vec![],
            on_use: vec![],
        },
    });

    // Insert the database as a resource
    commands.insert_resource(db);
}

/// Helper function to spawn attack children for a soldier.
///
/// CHILD ENTITIES IN BEVY:
/// Bevy supports parent-child relationships between entities.
/// - Children are spawned with .with_children() or commands.entity(parent).add_child(child)
/// - Children automatically get Parent component, parents get Children component
/// - When a parent is despawned, all children are despawned too
/// - Children's Transforms are relative to their parent
///
/// We use this to attach attacks to soldiers - each attack is a child entity
/// with its own AttackInstance component tracking cooldown state.
fn spawn_attacks_for_soldier(
    commands: &mut Commands,
    soldier_entity: Entity,
    attack_ids: Vec<AttackId>,
) {
    // Get a mutable reference to the soldier entity
    // and add children to it
    commands.entity(soldier_entity).with_children(|parent| {
        for attack_id in attack_ids {
            // Each attack is a child entity with just the AttackInstance component
            // No Transform/Mesh needed since attacks are invisible game logic
            parent.spawn(AttackInstance::new(attack_id));
        }
    });
}

/// Spawn system - runs once at startup to create the initial game state.
/// This is a Startup system, so it runs exactly once when the app starts.
///
/// IMPORTANT: This must run AFTER setup_attacks so the AttackDatabase exists.
/// We ensure this with .chain() in main.rs.
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

    // Define which attacks each soldier gets (by AttackId index)
    // Player gets: Basic Attack (0), Power Strike (1), Healing Strike (3)
    let player_attacks = vec![AttackId(0), AttackId(1), AttackId(3)];
    // Enemy gets: Basic Attack (0), Reckless Slam (2)
    let enemy_attacks = vec![AttackId(0), AttackId(2)];

    // Spawn player soldier (left side)
    // Note: We no longer add AttackCooldown here - each attack has its own cooldown
    let player_entity = commands.spawn((
        Soldier,
        Health::new(100),
        Team { is_player: true },
        Transform::default().with_translation(Vec3::new(-150.0, 0.0, 0.0)),
        Mesh2d(soldier_mesh.clone()),
        MeshMaterial2d(white_material.clone()),
    )).id();

    // Spawn enemy soldier (right side)
    let enemy_entity = commands.spawn((
        Soldier,
        Health::new(100),
        Team { is_player: false },
        Transform::default().with_translation(Vec3::new(150.0, 0.0, 0.0)),
        Mesh2d(soldier_mesh),
        MeshMaterial2d(white_material),
    )).id();

    // Add attack children to each soldier
    // Each attack becomes a child entity with its own cooldown state
    spawn_attacks_for_soldier(&mut commands, player_entity, player_attacks);
    spawn_attacks_for_soldier(&mut commands, enemy_entity, enemy_attacks);

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
