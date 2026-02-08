// systems/spawn.rs - Spawning soldiers, attacks, and UI
//
// This module handles initial game setup:
// 1. Initialize the AttackDatabase resource with attack definitions
// 2. Spawn soldiers with attack children
// 3. Create UI elements

use bevy::prelude::*;
use crate::components::{
    Health, Team, Soldier, HealthDisplay, GameOverText,
    AttackDatabase, AttackDefinition, AttackEffects, Effect, AttackId,
    AnimationState, AnimationType,
};
use crate::resources::SpriteSheets;

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

/// Load sprite sheets and create texture atlas layouts
/// This must run before spawn_soldiers so assets are available
pub fn load_sprite_sheets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load all sprite sheet images
    let slime_jump_idle = asset_server.load("sprites/slimes/Jump-Idle/Slime_Jump_Spritesheet.png");
    let slime_attack = asset_server.load("sprites/slimes/Attack/Slime_Attack_Spritesheet.png");
    let slime_move_small_jump = asset_server.load("sprites/slimes/Move-Small Jump/Slime_Move-Small Jump_Spritesheet.png");
    let slime_hurt = asset_server.load("sprites/slimes/Hurt/Slime_Hurt_Spritesheet.png");
    let slime_death = asset_server.load("sprites/slimes/Death/Slime_Death_Spritesheet.png");

    // Create texture atlas layouts (define frame grid)
    // Each sprite frame is 60x99 pixels
    // Jump-Idle: 6 frames in a row
    let jump_idle_layout = texture_atlas_layouts.add(
        TextureAtlasLayout::from_grid(UVec2::new(60, 99), 6, 1, None, None)
    );
    
    // Attack: 5 frames
    let attack_layout = texture_atlas_layouts.add(
        TextureAtlasLayout::from_grid(UVec2::new(60, 99), 5, 1, None, None)
    );
    
    // Move-Small Jump: 6 frames
    let move_small_jump_layout = texture_atlas_layouts.add(
        TextureAtlasLayout::from_grid(UVec2::new(60, 99), 6, 1, None, None)
    );
    
    // Hurt: 3 frames
    let hurt_layout = texture_atlas_layouts.add(
        TextureAtlasLayout::from_grid(UVec2::new(60, 99), 3, 1, None, None)
    );
    
    // Death: 6 frames
    let death_layout = texture_atlas_layouts.add(
        TextureAtlasLayout::from_grid(UVec2::new(60, 99), 6, 1, None, None)
    );

    // Store in resource for easy access
    commands.insert_resource(SpriteSheets {
        slime_jump_idle,
        slime_attack,
        slime_move_small_jump,
        slime_hurt,
        slime_death,
        jump_idle_layout,
        attack_layout,
        move_small_jump_layout,
        hurt_layout,
        death_layout,
    });
}

/// Spawn system - runs once at startup to create the initial game state.
/// This is a Startup system, so it runs exactly once when the app starts.
///
/// IMPORTANT: This must run AFTER setup_attacks and load_sprite_sheets.
/// We ensure this with .chain() in main.rs.
pub fn spawn_soldiers(
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    // Spawn camera first - required for rendering
    commands.spawn(Camera2d);

    // Define which attacks each soldier gets (by AttackId index)
    // Player gets: Basic Attack (0), Power Strike (1), Healing Strike (3)
    let player_attacks = vec![AttackId(0), AttackId(1), AttackId(3)];
    // Enemy gets: Basic Attack (0), Reckless Slam (2)
    let enemy_attacks = vec![AttackId(0), AttackId(2)];

    // Spawn player soldier (left side) with Jump-Idle animation
    let player_entity = commands.spawn((
        Soldier { available_attacks: player_attacks },
        Health::new(100),
        Team { is_player: true },
        Sprite {
            image: sprite_sheets.slime_jump_idle.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_sheets.jump_idle_layout.clone(),
                index: 0,
            }),
            custom_size: Some(Vec2::new(120.0, 198.0)), // 2x scale from 60x99
            ..default()
        },
        Transform::from_translation(Vec3::new(-150.0, 0.0, 0.0)),
        AnimationState::new(AnimationType::JumpIdle, 6, 0.1, true),
    )).id();

    // Spawn enemy soldier (right side) with Jump-Idle animation
    // Note: We'll flip this horizontally in a moment
    let enemy_entity = commands.spawn((
        Soldier { available_attacks: enemy_attacks },
        Health::new(100),
        Team { is_player: false },
        Sprite {
            image: sprite_sheets.slime_jump_idle.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_sheets.jump_idle_layout.clone(),
                index: 0,
            }),
            custom_size: Some(Vec2::new(120.0, 198.0)), // 2x scale from 60x99
            flip_x: true, // Flip enemy horizontally
            ..default()
        },
        Transform::from_translation(Vec3::new(150.0, 0.0, 0.0)),
        AnimationState::new(AnimationType::JumpIdle, 6, 0.1, true),
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
