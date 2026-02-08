// systems/menu.rs - Menu and Game Over UI systems
//
// This module handles the UI for:
// 1. Main menu with "Fight!" button (Menu state)
// 2. Game over screen with win/lose message and "Play Again" button (GameOver state)
//
// KEY CONCEPTS INTRODUCED:
// - Button component: Bevy's built-in clickable UI element
// - Interaction component: Tracks hover/press states automatically
// - NextState<T>: Resource for requesting state transitions
// - OnEnter/OnExit systems: Run once when entering/leaving a state
//
// HOW BUTTON + INTERACTION WORKS:
// When you add the Button component to a UI node, Bevy automatically:
// 1. Adds an Interaction component (if not present)
// 2. Tracks mouse position and clicks
// 3. Updates Interaction to Pressed/Hovered/None based on mouse state
//
// Your job is to query Interaction and react to its value.
// The interaction detection happens automatically each frame by Bevy's UI system.

use bevy::prelude::*;
use crate::components::{MenuUI, FightButton, GameOverUI, PlayAgainButton, Soldier, HealthDisplay};
use crate::resources::{GameState, GameStatus};

// =============================================================================
// MENU STATE SYSTEMS
// =============================================================================

/// Spawns the main menu UI with a "Fight!" button.
///
/// This system runs on OnEnter(GameState::Menu), meaning it executes exactly once
/// when the game transitions into the Menu state. This is different from Update
/// systems which run every frame.
///
/// OnEnter is perfect for spawning UI because we want to create it once,
/// not respawn it every frame!
pub fn spawn_menu_ui(mut commands: Commands) {
    // Create a full-screen container node
    // This acts as the root for all menu UI elements
    commands.spawn((
        // Node defines layout properties (like CSS flexbox)
        Node {
            width: Val::Percent(100.0),    // Full screen width
            height: Val::Percent(100.0),   // Full screen height
            // Center children both horizontally and vertically
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        // Transparent background (we just want this for layout)
        BackgroundColor(Color::NONE),
        // Marker component so we can find and despawn this later
        MenuUI,
    )).with_children(|parent| {
        // Spawn the "Fight!" button as a child of the container
        //
        // BUTTON STYLING NOTES:
        // We include the border styling in the Node itself using the `border` field.
        // The button has 5 components (the max tuple size for bundles in Bevy 0.18)
        // plus children for the text.
        parent.spawn((
            // Button is a special marker that tells Bevy this is clickable
            // Bevy will automatically track Interaction states for us
            Button,
            // Visual styling for the button - includes border setup
            Node {
                width: Val::Px(200.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                // Add border size (required for BorderColor to show)
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            // Dark red background color
            BackgroundColor(Color::srgb(0.5, 0.1, 0.1)),
            // Border color (requires border size in Node to be visible)
            BorderColor::all(Color::srgb(0.8, 0.3, 0.3)),
            // Marker component for our button detection query
            FightButton,
        )).with_children(|button| {
            // Spawn text as a child of the button
            button.spawn((
                Text::new("Fight!"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });
}

/// Handles clicks on the "Fight!" button.
///
/// This system runs every frame during the Menu state (via run_if(in_state(...))).
/// It queries for the FightButton's Interaction component and checks if it was pressed.
///
/// INTERACTION STATES:
/// - Interaction::Pressed: Mouse button is currently held down on the element
/// - Interaction::Hovered: Mouse is over the element but not clicking
/// - Interaction::None: Mouse is not interacting with this element
///
/// NEXTSTATE EXPLAINED:
/// NextState<T> is a resource that queues a state transition.
/// The actual transition happens at a specific point in the frame (after Update),
/// ensuring all systems see consistent state. This prevents race conditions.
///
/// You can't just set state directly because multiple systems might try to
/// change state in the same frame. NextState queues the request safely.
pub fn handle_menu_button(
    // Query for buttons with FightButton marker, get their Interaction state
    // The Changed<Interaction> filter means we only process buttons whose
    // Interaction changed this frame (optimization, not strictly necessary)
    query: Query<&Interaction, (Changed<Interaction>, With<FightButton>)>,
    // NextState lets us request a state transition
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in query.iter() {
        // Check if button was pressed
        if *interaction == Interaction::Pressed {
            // Queue transition to Battle state
            // The actual transition happens after all Update systems finish
            next_state.set(GameState::Battle);
        }
    }
}

/// Despawns all menu UI when leaving the Menu state.
///
/// This runs on OnExit(GameState::Menu), which fires once when we leave Menu.
/// We use Commands::entity().despawn() to remove the UI.
///
/// WHY DESPAWN?
/// UI elements take up memory and can interfere with other UI.
/// By cleaning up on state exit, we ensure a clean slate for the next state.
/// This is the "cleanup pattern" - every OnEnter that spawns should have
/// a matching OnExit that despawns.
pub fn cleanup_menu_ui(
    mut commands: Commands,
    // Query for any entity with MenuUI component
    query: Query<Entity, With<MenuUI>>,
) {
    for entity in query.iter() {
        // despawn removes the entity and all its children
        commands.entity(entity).despawn();
    }
}

// =============================================================================
// GAME OVER STATE SYSTEMS
// =============================================================================

/// Spawns the game over UI showing the result and "Play Again" button.
///
/// Runs on OnEnter(GameState::GameOver).
/// Reads GameStatus to determine if player won or lost.
pub fn spawn_gameover_ui(
    mut commands: Commands,
    game_status: Res<GameStatus>,
) {
    // Determine the result message and color
    let (message, color) = match game_status.result {
        Some(true) => ("YOU WIN!", Color::srgb(0.2, 0.8, 0.2)),   // Green for victory
        Some(false) => ("YOU LOSE!", Color::srgb(0.8, 0.2, 0.2)), // Red for defeat
        None => ("BATTLE OVER", Color::srgb(0.8, 0.8, 0.8)),      // Gray (shouldn't happen)
    };

    // Create full-screen container for game over UI
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column, // Stack children vertically
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(40.0), // Space between message and button
            ..default()
        },
        BackgroundColor(Color::NONE),
        GameOverUI,
    )).with_children(|parent| {
        // Result message (YOU WIN! or YOU LOSE!)
        parent.spawn((
            Text::new(message),
            TextFont {
                font_size: 72.0,
                ..default()
            },
            TextColor(color),
        ));

        // "Play Again" button
        // Similar to the Fight button, we limit the tuple to 5 components
        parent.spawn((
            Button,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.3, 0.5)),
            BorderColor::all(Color::srgb(0.4, 0.5, 0.7)),
            PlayAgainButton,
        )).with_children(|button| {
            button.spawn((
                Text::new("Play Again"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });
}

/// Handles clicks on the "Play Again" button.
///
/// When clicked, resets game status and transitions back to Menu state.
/// This creates the game loop: Menu → Battle → GameOver → Menu
pub fn handle_restart_button(
    query: Query<&Interaction, (Changed<Interaction>, With<PlayAgainButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_status: ResMut<GameStatus>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            // Reset game status for the next battle
            *game_status = GameStatus::default();
            // Go back to menu
            next_state.set(GameState::Menu);
        }
    }
}

/// Cleanup game over UI when leaving GameOver state.
/// Also cleans up any remaining soldiers and health displays from the battle.
pub fn cleanup_gameover_ui(
    mut commands: Commands,
    gameover_query: Query<Entity, With<GameOverUI>>,
    soldier_query: Query<Entity, With<Soldier>>,
    health_display_query: Query<Entity, With<HealthDisplay>>,
) {
    // Despawn game over UI
    for entity in gameover_query.iter() {
        commands.entity(entity).despawn();
    }

    // Despawn any remaining soldiers (in case some are still alive/dying)
    for entity in soldier_query.iter() {
        commands.entity(entity).despawn();
    }

    // Note: Health displays are children of the battle UI, so they'll be
    // despawned when their parent is despawned. But since they reference
    // soldier entities that will be gone next battle, we need to ensure
    // they're cleaned up. The spawn_soldiers system will create new ones.
    for entity in health_display_query.iter() {
        commands.entity(entity).despawn();
    }
}
