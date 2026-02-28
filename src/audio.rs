use bevy::{audio::Volume, prelude::*};

use crate::save_load::SaveData;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_audio)
            .add_systems(Startup, start_background_music);
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
    /// Sound that plays when a shield blocks an incoming attack.
    pub block: Handle<AudioSource>,
    /// Sound that plays when an AoE ice trap spawns at the impact point.
    pub ice_trap: Handle<AudioSource>,
    pub ready: Handle<AudioSource>,
    pub go: Handle<AudioSource>,
}

fn start_background_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    // commands.spawn((
    //     AudioPlayer::new(asset_server.load("audio/pixabay/nickpanekaiassets-8-bit-chiptune-action-music-for-video-games-329940.mp3")),
    //     PlaybackSettings {
    //         mode: bevy::audio::PlaybackMode::Loop,
    //         volume: Volume::Linear(0.5),
    //         ..default()
    //     },
    // ));
}

pub fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    // The actual loading happens in the background - asset_server.load()
    // returns immediately with a Handle that will be valid once loading completes.
    let damage_sound = asset_server.load("audio/Stab.wav");
    let death_sound = asset_server.load("audio/Bomb.wav");
    let merge_alert_sound = asset_server.load("audio/beepbee.wav");
    let merge_complete_sound = asset_server.load("audio/Callsummon.wav");
    // let stun_sound = asset_server.load("audio/monsterdestroyed.wav");
    let stun_sound = asset_server.load("audio/Enemystunned.wav");
    // let block_sound = asset_server.load("audio/Frustratedenemy.wav");
    let block_sound = asset_server.load("audio/pixabay/existentialtaco-confirm-tap-394001.mp3");

    let ice_trap_sound = asset_server.load("audio/Select.wav");
    let ready_sound = asset_server.load("audio/pixabay/u_xmiiqyhi46-gamestart-272829.mp3");
    let go_sound =
        asset_server.load("audio/pixabay/freesound_community-pixel-sound-effect-4-82881.mp3");

    commands.insert_resource(GameAudio {
        slime_damage: damage_sound,
        slime_death: death_sound,
        merge_alert: merge_alert_sound,
        merge_complete: merge_complete_sound,
        stun: stun_sound,
        block: block_sound,
        ice_trap: ice_trap_sound,
        ready: ready_sound,
        go: go_sound,
    });
}
