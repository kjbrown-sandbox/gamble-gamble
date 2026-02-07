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
        //
        // ORDERING WITH .chain():
        // Systems in a tuple run in parallel by default (for performance).
        // But some systems depend on others completing first:
        // - setup_attacks must run before spawn_soldiers (soldiers reference attacks)
        // - setup_audio can run in parallel with either
        //
        // .chain() makes systems run sequentially in order.
        // Here we chain the attack setup, then spawn soldiers.
        .add_systems(Startup, (
            systems::setup_attacks,   // Initialize AttackDatabase resource
            systems::spawn_soldiers,  // Create soldiers with attack children
        ).chain())
        .add_systems(Startup, systems::setup_audio)  // Can run in parallel

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
        // 1. Update attack cooldowns (tick timers)
        // 2. Cleanup finished attacks (despawn AttackInstance when cooldown done)
        // 3. Process attacks (select attack, roll hit, apply effects)
        // 4. Check for deaths
        // 5. Update health bar UI
        // 6. Handle game over state
        //
        // IMPORTANT: cleanup_finished_attacks runs BEFORE attack_system so that
        // soldiers become "ready" on the same frame their cooldown finishes.
        // This order ensures:
        // - Cooldowns tick down
        // - Finished AttackInstances are despawned (soldier now has no children)
        // - attack_system sees soldier with no children → can attack again
        .add_systems(
            Update,
            (
                systems::game_over_system,
                (
                    systems::update_attack_cooldowns,     // Tick cooldown timers
                    systems::cleanup_finished_attacks,    // Despawn finished attacks
                    systems::attack_system,               // Execute attacks
                    systems::death_check_system,
                    systems::render_health_bars,
                ).chain(),
            ),
        )
        // Finally, .run() starts the game loop
        .run();
}
