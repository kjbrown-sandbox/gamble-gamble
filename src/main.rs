// main.rs - Entry point for our Bevy game
// 
// This is a betting/combat game where two soldiers fight each other.
// The code is organized into modules (lib.rs, components, systems, resources)
// to demonstrate how to structure a larger Bevy project.

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

        // Startup systems run once when the app starts.
        // We use a tuple to run multiple startup systems.
        // Order within the tuple is not guaranteed unless you use .chain()
        .add_systems(Startup, (
            systems::spawn_soldiers,
            systems::setup_audio,  // Load audio assets at startup
        ))

        // OBSERVERS (Bevy 0.18's event system):
        // Observers are functions that react to triggered events.
        // Unlike regular systems that run every frame, observers only run
        // when their event type is triggered via commands.trigger().
        //
        // This decouples systems: attack_system doesn't need to know about audio,
        // it just triggers a DamageEvent. Any number of observers can react.
        .add_observer(systems::on_damage)

        // Update systems run every frame
        // We add them in order of logical execution:
        // 1. Process attacks and cooldowns (triggers DamageEvents)
        // 2. Display health information
        // 3. Check for deaths and game over conditions
        // 4. Handle game over state
        .add_systems(
            Update,
            (
                systems::game_over_system,
                (
                    systems::attack_system,
                    systems::death_check_system,
                    systems::render_health_bars,
                ).chain(),
            ),
        )
        // Finally, .run() starts the game loop
        .run();
}
