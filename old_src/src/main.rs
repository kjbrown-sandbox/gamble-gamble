// main.rs - Entry point for our Bevy game
//
// This is a betting/combat game where two soldiers fight each other.
// The code is organized into modules (lib.rs, components, systems, resources)
// to demonstrate how to structure a larger Bevy project.
//
// =============================================================================
// APP STATES OVERVIEW
// =============================================================================
// This app uses Bevy's States system to control game flow:
//
// Menu state:     Show "Fight!" button, wait for player input
// Battle state:   Soldiers fight automatically, damage popups appear
// GameOver state: Show result (WIN/LOSE), "Play Again" button
//
// STATE SYSTEM CONCEPTS:
// - init_state::<T>()    - Register a state type and start in its Default variant
// - OnEnter(State)       - System runs once when entering this state
// - OnExit(State)        - System runs once when leaving this state
// - run_if(in_state(S))  - System only runs while in state S
// - NextState<T>         - Resource to request state transitions
//
// WHY STATES VS A SIMPLE BOOLEAN?
// 1. Type safety: Compiler ensures you handle all states
// 2. OnEnter/OnExit: Perfect for spawn/cleanup, run exactly once
// 3. run_if: Systems only run when relevant (no wasted CPU cycles)
// 4. Clean separation: Each state's code is isolated
// 5. Extensibility: Easy to add new states (Pause, Shop, etc.)

use gamble_game_2::*;
use bevy::prelude::*;

fn main() {
    // App::new() creates an empty Bevy application.
    // We then chain methods to configure it before calling .run()
    App::new()
        // DefaultPlugins provides all the core Bevy functionality
        .add_plugins(DefaultPlugins)
        // Set the background color to black
        .insert_resource(ClearColor(Color::BLACK))
        // Initialize the game status resource
        .init_resource::<GameStatus>()

        // =====================================================================
        // STATE INITIALIZATION
        // =====================================================================
        // init_state::<T>() does several things:
        // 1. Registers the State type with Bevy
        // 2. Creates the State<T> and NextState<T> resources
        // 3. Sets initial state to the #[default] variant (Menu)
        // 4. Enables OnEnter/OnExit/run_if systems for this state type
        .init_state::<GameState>()

        // =====================================================================
        // GLOBAL STARTUP SYSTEMS (run once, before any state)
        // =====================================================================
        // These systems run at app launch and set up resources needed by all states.
        // Camera is global because all states need to render.
        // Attack database and sprites are loaded once and reused.
        //
        // ORDERING WITH .chain():
        // setup_attacks and load_sprite_sheets must complete before any state
        // tries to spawn soldiers (they reference attacks and sprites).
        // Since these run on Startup and states begin after Startup, ordering is safe.
        .add_systems(Startup, (
            systems::spawn_camera,       // Camera needed for all states
            systems::setup_attacks,      // Initialize AttackDatabase resource
            systems::load_sprite_sheets, // Load sprite assets
        ).chain())
        .add_systems(Startup, systems::setup_audio)  // Can run in parallel

        // =====================================================================
        // MENU STATE SYSTEMS
        // =====================================================================
        // Menu is the starting state (marked with #[default] in GameState enum).
        //
        // OnEnter: Spawn the menu UI (runs once when entering Menu)
        // Update:  Check for button clicks (runs every frame while in Menu)
        // OnExit:  Cleanup menu UI (runs once when leaving Menu)
        .add_systems(OnEnter(GameState::Menu), systems::spawn_menu_ui)
        .add_systems(
            Update,
            systems::handle_menu_button.run_if(in_state(GameState::Menu))
        )
        .add_systems(OnExit(GameState::Menu), systems::cleanup_menu_ui)

        // =====================================================================
        // BATTLE STATE SYSTEMS
        // =====================================================================
        // Battle state is where the action happens.
        //
        // OnEnter: Spawn soldiers and battle UI
        // Update:  All combat, animation, and death systems
        // OnExit:  (cleanup happens in GameOver's cleanup instead)
        //
        // run_if(in_state(GameState::Battle)) ensures these systems ONLY run
        // during battle. In Menu or GameOver, they're completely skipped.
        // This is more efficient than checking a boolean each frame.
        .add_systems(OnEnter(GameState::Battle), systems::spawn_soldiers)
        .add_systems(
            Update,
            (
                // Combat systems
                systems::update_attack_cooldowns,     // Tick cooldown timers
                systems::cleanup_finished_attacks,    // Despawn finished attacks
                systems::attack_system,               // Execute attacks

                // Animation systems
                systems::animation_system,            // Update animation frames
                systems::animation_switcher_system,   // Switch sprites on animation change
                systems::animation_finished_system,   // Return to idle after animations

                // Health and death systems
                systems::death_check_system,          // Detect deaths, start death anim
                systems::death_animation_system,      // Despawn after death anim
                systems::check_battle_end,            // Check if battle is over

                // UI systems
                systems::render_health_bars,          // Update health display
                systems::update_damage_popups,        // Animate floating damage numbers
            )
                .chain()
                .run_if(in_state(GameState::Battle))
        )

        // =====================================================================
        // GAME OVER STATE SYSTEMS
        // =====================================================================
        // GameOver shows the result and allows restarting.
        //
        // OnEnter: Show WIN/LOSE message and "Play Again" button
        // Update:  Check for restart button clicks
        // OnExit:  Cleanup all battle entities and UI
        .add_systems(OnEnter(GameState::GameOver), systems::spawn_gameover_ui)
        .add_systems(
            Update,
            systems::handle_restart_button.run_if(in_state(GameState::GameOver))
        )
        .add_systems(OnExit(GameState::GameOver), systems::cleanup_gameover_ui)

        // =====================================================================
        // OBSERVERS (event-driven, run regardless of state)
        // =====================================================================
        // Observers react to triggered events, not state.
        // They run whenever their event type is triggered, in any state.
        //
        // This is useful because:
        // - Damage can happen during Battle (normal combat)
        // - Damage popup observer only spawns if there's a DamageEvent
        // - No need to add run_if checks; no events = no work
        .add_observer(systems::on_damage)               // Audio feedback on damage
        .add_observer(systems::on_damage_animation)     // Hurt animation on damage
        .add_observer(systems::on_damage_spawn_popup)   // Floating damage numbers

        // Finally, .run() starts the game loop
        .run();
}
