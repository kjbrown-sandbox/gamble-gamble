use bevy::{audio::Volume, prelude::*};

use crate::save_load::SaveData;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_audio);
    }
}

#[derive(Resource)]
pub struct GameAudio {
    /// Handle<T> is Bevy's way of referencing assets.
    pub slime_damage: Handle<AudioSource>,
    pub slime_death: Handle<AudioSource>,
    pub merge_alert: Handle<AudioSource>,
    pub merge_complete: Handle<AudioSource>,
    /// Icy/frozen sound that plays when a target gets stunned.
    pub stun: Handle<AudioSource>,
}

pub fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    // The actual loading happens in the background - asset_server.load()
    // returns immediately with a Handle that will be valid once loading completes.
    let damage_sound = asset_server.load("audio/Stab.wav");
    let death_sound = asset_server.load("audio/Bomb.wav");
    let merge_alert_sound = asset_server.load("audio/beepbee.wav");
    let merge_complete_sound = asset_server.load("audio/Callsummon.wav");
    let stun_sound = asset_server.load("audio/Shower.wav");

    commands.insert_resource(GameAudio {
        slime_damage: damage_sound,
        slime_death: death_sound,
        merge_alert: merge_alert_sound,
        merge_complete: merge_complete_sound,
        stun: stun_sound,
    });
}
