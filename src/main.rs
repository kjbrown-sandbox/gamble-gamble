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
        // - load_sprite_sheets must run before spawn_soldiers (soldiers use sprites)
        // - setup_audio can run in parallel with either
        //
        // .chain() makes systems run sequentially in order.
        // Here we chain the attack setup, sprite loading, then spawn soldiers.
        .add_systems(Startup, (
            systems::setup_attacks,      // Initialize AttackDatabase resource
            systems::load_sprite_sheets, // Load sprite assets
            systems::spawn_soldiers,     // Create soldiers with sprites
        ).chain())
        .add_systems(Startup, systems::setup_audio)  // Can run in parallel

        // OBSERVERS (Bevy's event system):
        // Observers are functions that react to triggered events.
        // Unlike regular systems that run every frame, observers only run
        // when their event type is triggered via commands.trigger().
        //
        // This decouples systems: attack_system doesn't need to know about audio,
        // it just triggers a DamageEvent. Any number of observers can react.
        .add_observer(systems::on_damage)            // Audio feedback
        .add_observer(systems::on_damage_animation)  // Hurt animation

        // Update systems run every frame
        // We add them in order of logical execution:
        //
        // SYSTEM ORDER EXPLAINED:
        // 1. game_over_system - Check if game ended, block combat if so
        // 2. update_attack_cooldowns - Tick cooldown timers
        // 3. cleanup_finished_attacks - Despawn finished attacks
        // 4. attack_system - Execute attacks (triggers DamageEvent)
        // 5. animation_system - Update animation frames
        // 6. animation_switcher_system - Change sprites when animation type changes
        // 7. animation_finished_system - Return to idle after attack/hurt animations
        // 8. death_check_system - Detect deaths, start death animation
        // 9. death_animation_system - Despawn after death animation, check win/lose
        // 10. render_health_bars - Update UI
        //
        // IMPORTANT: cleanup_finished_attacks runs BEFORE attack_system so that
        // soldiers become "ready" on the same frame their cooldown finishes.
        .add_systems(
            Update,
            (
                systems::game_over_system,
                (
                    systems::update_attack_cooldowns,     // Tick cooldown timers
                    systems::cleanup_finished_attacks,    // Despawn finished attacks
                    systems::attack_system,               // Execute attacks
                    systems::animation_system,            // Update animation frames
                    systems::animation_switcher_system,   // Switch sprites on animation change
                    systems::animation_finished_system,   // Return to idle after animations
                    systems::death_check_system,          // Detect deaths, start death anim
                    systems::death_animation_system,      // Despawn after death anim
                    systems::render_health_bars,
                ).chain(),
            ),
        )
        // Finally, .run() starts the game loop
        .run();
}
