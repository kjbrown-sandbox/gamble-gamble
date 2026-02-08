// systems/damage_popup.rs - Floating damage number system
//
// This module creates and animates floating damage numbers that appear
// when entities take damage. It demonstrates:
//
// KEY CONCEPTS:
// - Text2d vs Text (UI): World-space text that exists in the game world
// - Observers: Event-driven systems that react to triggered events
// - Timer::fraction(): Returns 0.0 → 1.0 progress for smooth animations
// - Lerp (linear interpolation): Smooth transitions between values
// - Alpha/opacity animation: Fade effects using TextColor
//
// TEXT2D VS TEXT (UI TEXT):
// - Text (UI): Lives in screen-space, always on top, positioned in pixels
//              Good for: HUD, menus, health bars, scores
// - Text2d: Lives in world-space, has a Transform, affected by camera
//           Good for: Damage numbers, speech bubbles, in-world labels
//
// We use Text2d because we want damage numbers to appear AT the entity's
// world position, not at a fixed screen location.

use bevy::prelude::*;
use crate::components::{DamageEvent, DamagePopup};

// Configuration constants for damage popups
// These could be made into a Resource for runtime tweaking
const POPUP_DURATION: f32 = 0.8;      // How long popup lives (seconds)
const POPUP_FLOAT_DISTANCE: f32 = 50.0; // How far up it floats (world units)
const POPUP_FONT_SIZE: f32 = 32.0;     // Text size

/// Observer system that spawns a damage popup when a DamageEvent is triggered.
///
/// OBSERVER vs REGULAR SYSTEM:
/// Regular systems run every frame whether or not there's work to do.
/// Observers only run when their event type is triggered.
///
/// This is more efficient for rare events (like damage) and creates
/// loose coupling - the combat system doesn't need to know about popups,
/// it just triggers DamageEvent and any number of observers can react.
///
/// ON<T> WRAPPER:
/// In observers, we use On<T> instead of EventReader<T>.
/// On<T> gives us the event data directly for the current trigger.
/// This is part of Bevy's "Entity Observers" system (added in 0.14+).
///
/// WHY USE COMMANDS?
/// We spawn entities via Commands because we're in an observer, which runs
/// outside the normal system schedule. Commands queue the spawn to happen
/// at a safe time (the next command flush point).
pub fn on_damage_spawn_popup(
    trigger: On<DamageEvent>,
    // We need to look up the damaged entity's position
    query: Query<&Transform>,
    mut commands: Commands,
) {
    let event = trigger.event();

    // Get the position of the damaged entity
    // If the entity doesn't have a Transform (shouldn't happen), skip
    let Ok(target_transform) = query.get(event.target) else {
        return;
    };

    // Spawn the damage popup as a Text2d entity
    // Text2d means this text exists in world-space, not UI-space
    commands.spawn((
        // Text2d is the component for world-space text
        // It takes the string to display
        Text2d::new(format!("{}", event.amount)),
        // Font configuration
        TextFont {
            font_size: POPUP_FONT_SIZE,
            ..default()
        },
        // Red color for damage (we'll fade this out over time)
        // TextColor wraps a Color and will be modified for alpha fade
        TextColor(Color::srgb(1.0, 0.2, 0.2)),
        // Position the popup at the damaged entity, slightly above
        // We offset Y a bit so it starts above the entity, not inside it
        Transform::from_translation(Vec3::new(
            target_transform.translation.x,
            target_transform.translation.y + 50.0, // Start above the entity
            10.0, // High Z to render on top of other sprites
        )),
        // Our custom component to track popup state
        DamagePopup {
            // Timer::from_seconds creates a timer with the given duration
            // TimerMode::Once means it runs once and stops (vs Repeating)
            timer: Timer::from_seconds(POPUP_DURATION, TimerMode::Once),
            // Store starting Y so we can calculate offset during animation
            start_y: target_transform.translation.y + 50.0,
            float_distance: POPUP_FLOAT_DISTANCE,
        },
    ));
}

/// Update system that animates damage popups: float upward, fade out, despawn.
///
/// This runs every frame and processes all existing DamagePopup entities.
///
/// ANIMATION TECHNIQUE:
/// We use Timer::fraction() which returns a value from 0.0 (just started)
/// to 1.0 (finished). This is perfect for lerp-based animations:
///
///   progress = timer.fraction();  // 0.0 → 1.0 over the timer's duration
///   y = start_y + progress * float_distance;  // Linear movement
///   alpha = 1.0 - progress;  // Fade from visible to invisible
///
/// WHAT IS LERP?
/// "Lerp" stands for Linear Interpolation. It calculates a value between
/// two endpoints based on a percentage (usually called 't'):
///
///   lerp(a, b, t) = a + t * (b - a)
///   lerp(0, 100, 0.0) = 0
///   lerp(0, 100, 0.5) = 50
///   lerp(0, 100, 1.0) = 100
///
/// When t goes from 0 to 1, the result smoothly transitions from a to b.
/// Timer::fraction() gives us this t value automatically!
pub fn update_damage_popups(
    mut commands: Commands,
    time: Res<Time>,
    // Query for all damage popups, getting mutable access to modify them
    mut query: Query<(Entity, &mut DamagePopup, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut popup, mut transform, mut text_color) in query.iter_mut() {
        // Tick the timer forward by the frame's delta time
        popup.timer.tick(time.delta());

        // Get animation progress (0.0 = just started, 1.0 = finished)
        // This is the key function! It gives us a normalized progress value.
        let progress = popup.timer.fraction();

        // ANIMATE POSITION: Float upward
        // start_y + (progress * float_distance) creates smooth upward movement
        // When progress=0, y=start_y. When progress=1, y=start_y+float_distance.
        transform.translation.y = popup.start_y + progress * popup.float_distance;

        // ANIMATE OPACITY: Fade out
        // We want alpha to go from 1.0 (fully visible) to 0.0 (invisible)
        // So we use (1.0 - progress) which goes from 1 to 0 as progress goes 0 to 1
        //
        // TextColor contains a Color. We need to reconstruct it with new alpha.
        // Color::srgba takes red, green, blue, alpha values.
        let alpha = 1.0 - progress;
        text_color.0 = Color::srgba(1.0, 0.2, 0.2, alpha);

        // When timer finishes, despawn the popup entity
        // just_finished() returns true only on the frame the timer completes.
        // This is safer than finished() which returns true every frame after completion.
        if popup.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
