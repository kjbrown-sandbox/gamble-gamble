// systems/audio.rs - Audio system for playing sound effects
//
// This system demonstrates Bevy 0.18's observer pattern for events.
// Observers are functions that run in response to triggered events.

use bevy::{audio::Volume, prelude::*};
use crate::components::DamageEvent;

/// Resource that holds our pre-loaded audio assets.
///
/// Resources are global, shared state in Bevy. Unlike Components (which are per-entity),
/// Resources exist once and can be accessed by any system.
///
/// We use a Resource here because:
/// 1. The audio file only needs to be loaded once, not per-entity
/// 2. Multiple systems might need access to the same sounds
/// 3. It's more efficient than loading the file every time we play it
///
/// #[derive(Resource)] is required for any struct used as a resource.
#[derive(Resource)]
pub struct GameAudio {
    /// Handle to the damage sound effect.
    /// Handle<T> is Bevy's way of referencing assets. The actual audio data
    /// lives in Bevy's asset storage; we just hold a lightweight reference.
    pub damage_sound: Handle<AudioSource>,
}

/// Startup system that loads audio assets.
///
/// This runs once when the game starts. We load the audio file and store
/// the handle in a Resource so other systems can use it.
///
/// AssetServer is Bevy's asset loading system. It:
/// - Loads files from the "assets/" directory by default
/// - Returns a Handle<T> immediately (loading happens asynchronously)
/// - Caches loaded assets so the same file isn't loaded twice
pub fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the sound file. This path is relative to the "assets/" folder.
    // The actual loading happens in the background - asset_server.load()
    // returns immediately with a Handle that will be valid once loading completes.
    let damage_sound = asset_server.load("audio/paper.wav");

    // Insert the GameAudio resource so other systems can access it.
    // commands.insert_resource() adds a new resource to the world.
    commands.insert_resource(GameAudio { damage_sound });
}

/// Observer that plays damage sounds when DamageEvents are triggered.
///
/// OBSERVERS (new in Bevy 0.18):
/// Observers are a replacement for the old EventReader/EventWriter pattern.
/// Instead of polling for events each frame, observers are called immediately
/// when an event is triggered via `commands.trigger(event)`.
///
/// Key differences from the old event system:
/// - Observers run immediately when triggered (not at a scheduled time)
/// - The first parameter is `On<EventType>` which contains the event data
/// - No need to track "already read" events - each trigger = one observer call
/// - Register with `.add_observer(function)` on the App
///
/// The `On<T>` wrapper:
/// - Contains the triggered event
/// - Dereferences to &T via `*trigger` or `trigger.event()`
/// - Also provides metadata about the trigger
pub fn on_damage(
    // On<DamageEvent> is the trigger wrapper. It contains the DamageEvent data.
    trigger: On<DamageEvent>,
    // We need Commands to spawn the audio player entity
    mut commands: Commands,
    // We need the GameAudio resource to get the sound handle
    audio: Res<GameAudio>,
) {
    // Access the event data from the trigger.
    // On<T> implements Deref, so we can use * to get a reference to the inner event.
    let damage = trigger.amount;

    // Calculate volume based on damage amount.
    // Our damage range is 10-20 (from attack_system), so we normalize it:
    // - 10 damage = minimum volume (0.3)
    // - 20 damage = maximum volume (1.0)
    //
    // The formula: volume = min_vol + (damage - min_dmg) / (max_dmg - min_dmg) * (max_vol - min_vol)
    // Simplified for our range:
    const MIN_DAMAGE: f32 = 10.0;
    const MAX_DAMAGE: f32 = 20.0;
    const MIN_VOLUME: f32 = 0.3;  // Quiet but audible for small hits
    const MAX_VOLUME: f32 = 1.0;  // Full volume for big hits

    // Normalize damage to 0.0-1.0 range, then scale to volume range
    let normalized = (damage as f32 - MIN_DAMAGE) / (MAX_DAMAGE - MIN_DAMAGE);
    // Clamp to handle any damage values outside expected range
    let normalized = normalized.clamp(0.0, 1.0);
    let volume_level = MIN_VOLUME + normalized * (MAX_VOLUME - MIN_VOLUME);

    // Spawn an entity with AudioPlayer AND PlaybackSettings components.
    //
    // In Bevy 0.18, audio is played by spawning an entity with AudioPlayer.
    // Adding PlaybackSettings lets us customize volume, speed, looping, etc.
    //
    // PlaybackSettings::DESPAWN is a preset that auto-removes the entity
    // when playback completes, preventing memory leaks from accumulating
    // audio entities.
    //
    // Volume::Linear(x) where x is 0.0 (silent) to 1.0 (full volume).
    // There's also Volume::Decibels for dB-based control.
    commands.spawn((
        AudioPlayer::new(audio.damage_sound.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume_level)),
    ));
}
