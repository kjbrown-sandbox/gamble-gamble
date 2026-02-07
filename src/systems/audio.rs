// systems/audio.rs - Audio system for playing sound effects
//
// This system demonstrates Bevy 0.18's observer pattern for events.
// Observers are functions that run in response to triggered events.

use bevy::prelude::*;
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
    // The underscore prefix means we're not using this variable (avoiding warnings).
    _trigger: On<DamageEvent>,
    // We need Commands to spawn the audio player entity
    mut commands: Commands,
    // We need the GameAudio resource to get the sound handle
    audio: Res<GameAudio>,
) {
    // Spawn an entity with AudioPlayer component to play the sound.
    //
    // In Bevy 0.18, audio is played by spawning an entity with AudioPlayer.
    // The entity is automatically cleaned up when playback completes.
    //
    // We clone the handle because Handle is cheap to clone (it's just an ID)
    // and we need to keep the original in our resource for future use.
    commands.spawn(AudioPlayer::new(audio.damage_sound.clone()));

    // Note: We could access the event data if needed:
    // let event = &*_trigger;  // or _trigger.event()
    // println!("Entity {:?} took {} damage", event.target, event.amount);
    //
    // You could use this to:
    // - Play spatial audio at the entity's position
    // - Vary volume based on damage amount
    // - Select different sounds for small vs big hits
}
