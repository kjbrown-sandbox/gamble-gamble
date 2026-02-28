use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::{
    animation::AnimationState,
    audio::GameAudio,
    combat::{FloatingText, IceImpactVfx},
    render,
    utils::DespawnAfter,
    ArenaBounds, GameFont, GameState,
};

pub struct SetupRoundPlugin;

impl Plugin for SetupRoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, leave_initial_loading)
            .add_systems(
                OnEnter(GameState::Combat),
                (start_pre_game_timer, setup_combat_arena),
            )
            .add_systems(
            Update,
            (
                pre_game_timer_system.run_if(resource_exists::<PreGameTimer>),
                stun_timer_system,
                on_add_stun_system,
            )
                .run_if(in_state(GameState::Combat)),
        );
    }
}

/// Marker component: entities with Inert cannot pick targets or move.
/// Removed from all entities when the pre-game timer expires.
#[derive(Component)]
pub struct Inert;

/// Resource that counts down the pre-game pause before combat begins.
/// Once it expires, it removes itself and strips Inert from every entity.
#[derive(Resource)]
pub struct PreGameTimer(Timer);

fn start_pre_game_timer(mut commands: Commands, game_font: Res<GameFont>, audio: Res<GameAudio>) {
    commands.insert_resource(PreGameTimer(Timer::from_seconds(3.2, TimerMode::Once)));

    commands.spawn((
        Text2d::new("READY"),
        TextFont {
            font: game_font.0.clone(),
            font_size: 100.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 25.0, 1.0),
        DespawnAfter(Timer::from_seconds(3.2, TimerMode::Once)),
    ));

    commands.spawn((AudioPlayer::new(audio.ready.clone()),));
}

fn pre_game_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<PreGameTimer>,
    inert_entities: Query<Entity, With<Inert>>,
    game_font: Res<GameFont>,
    audio: Res<GameAudio>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        // Remove Inert from every entity that has it
        for entity in &inert_entities {
            commands.entity(entity).remove::<Inert>();
        }

        // Timer's job is done — remove the resource so this system stops running
        commands.remove_resource::<PreGameTimer>();

        commands.spawn((
            Text2d::new("GO!"),
            TextFont {
                font: game_font.0.clone(),
                font_size: 100.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, 25.0, 1.0),
            DespawnAfter(Timer::from_seconds(0.6, TimerMode::Once)),
        ));

        commands.spawn((AudioPlayer::new(audio.go.clone()),));
    }
}

/// Per-entity timer that counts down a stun duration.
/// When this timer finishes, the entity is "unstunned" — Inert and StunTimer
/// are removed, letting movement/targeting/attacking systems pick it up again.
///
/// This is separate from PreGameTimer (which is a Resource) because stun is
/// per-entity state. Each stunned entity has its own countdown.
#[derive(Component)]
pub struct StunTimer(pub Timer);

/// Ticks each entity's StunTimer. When the timer finishes:
/// - Removes `Inert` (re-enables movement, targeting, attacking)
/// - Removes `StunTimer` itself (cleanup)
/// - Sets `AnimationState.finished = false` so the idle animation resumes
/// - Despawns any ice VFX children (entities with `IceImpactVfx`)
fn stun_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut StunTimer, &mut AnimationState, &Children), With<Inert>>,
    vfx_query: Query<Entity, With<IceImpactVfx>>,
) {
    for (entity, mut stun_timer, mut anim_state, children) in query.iter_mut() {
        stun_timer.0.tick(time.delta());

        if stun_timer.0.just_finished() {
            // Unstun: remove the markers so other systems can interact with this entity again
            commands
                .entity(entity)
                .remove::<Inert>()
                .remove::<StunTimer>();

            // Resume animation — finished was set to true when we stunned them
            anim_state.finished = false;

            // Despawn any ice impact VFX children that are still alive.
            // We iterate the entity's children and check if they have IceImpactVfx.
            for child in children.iter() {
                if vfx_query.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }
        }
    }
}

/// Spawns floating "STUNNED!" text when any entity first receives a StunTimer.
/// Added<StunTimer> fires once on the frame the component is inserted, covering
/// all stun sources (ice blast, wall slam, etc.) without each source needing
/// to spawn the text itself.
fn on_add_stun_system(
    query: Query<&GlobalTransform, Added<StunTimer>>,
    game_font: Res<GameFont>,
    mut commands: Commands,
) {
    for global_transform in &query {
        let pos = global_transform.translation();
        commands.spawn((
            FloatingText(Timer::from_seconds(0.8, TimerMode::Once)),
            Text2d::new("STUNNED!"),
            TextFont {
                font: game_font.0.clone(),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(pos.x, pos.y + 20.0, 10.0),
        ));
    }
}

/// Transitions out of InitialLoading into Combat. By running at Startup (which
/// fires after PreStartup), all resources like GameFont, SaveData, and GameAudio
/// are guaranteed to exist before any OnEnter(Combat) systems run.
fn leave_initial_loading(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Combat);
}

/// Spawns the combat arena background and vignette.
/// Runs on OnEnter(GameState::Combat). DespawnOnExit auto-cleans them when
/// leaving Combat, so we don't need manual cleanup queries.
fn setup_combat_arena(
    mut commands: Commands,
    arena: Res<ArenaBounds>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn((
        DespawnOnExit(GameState::Combat),
        render::Background,
        Sprite {
            image: asset_server.load("backgrounds/personal-stones.png"),
            custom_size: Some(Vec2::new(arena.width, arena.height)),
            color: Color::srgba(1.0, 1.0, 1.0, 0.05),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0).with_scale(Vec3::splat(3.0)),
    ));

    let vignette_height: u32 = 64;
    let mut pixel_data = Vec::with_capacity((vignette_height * 4) as usize);
    for y in 0..vignette_height {
        let t = y as f32 / (vignette_height - 1) as f32;
        let edge_dist = (2.0 * (t - 0.5)).powi(2);
        let alpha = (edge_dist * 215.0) as u8;
        pixel_data.extend_from_slice(&[0, 0, 0, alpha]);
    }
    let mut vignette_image = Image::new(
        Extent3d {
            width: 1,
            height: vignette_height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixel_data,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );
    vignette_image.sampler = ImageSampler::linear();
    let vignette_handle = images.add(vignette_image);

    commands.spawn((
        DespawnOnExit(GameState::Combat),
        render::Background,
        Sprite {
            image: vignette_handle,
            custom_size: Some(Vec2::new(arena.width, arena.height)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 9.0),
    ));
}
