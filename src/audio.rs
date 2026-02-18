use bevy::{audio::Volume, prelude::*};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_audio);
    }
}

#[derive(Resource)]
pub struct GameAudio {
    /// Handle<T> is Bevy's way of referencing assets.
    pub slime_damage_sound: Handle<AudioSource>,
}

pub fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    // The actual loading happens in the background - asset_server.load()
    // returns immediately with a Handle that will be valid once loading completes.
    let damage_sound = asset_server.load("audio/paper.wav");

    commands.insert_resource(GameAudio {
        slime_damage_sound: damage_sound,
    });
}
