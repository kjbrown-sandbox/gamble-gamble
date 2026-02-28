use bevy::camera::ScalingMode;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::{WindowResizeConstraints, WindowResolution};
use rand::seq::IteratorRandom;

use crate::health::Health;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    InitialLoading,
    Home,
    Combat,
}

#[derive(Resource)]
pub struct ArenaBounds {
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
pub struct GameFont(pub Handle<Font>);

impl ArenaBounds {
    /// Half-extents for bounds checking. Since the camera is centered at the
    /// origin, an 1200-wide arena spans from -600 to +600.
    pub fn half_width(&self) -> f32 {
        self.width / 2.0
    }
    pub fn half_height(&self) -> f32 {
        self.height / 2.0
    }
}

fn main() {
    let mut app = App::new();

    app.set_error_handler(bevy::ecs::error::warn);

    app.insert_resource(ArenaBounds {
        width: 1200.0,
        height: 800.0,
    })
    .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.15)))
    // Bevy's add_plugins() only supports tuples of up to 15 elements.
    // When you exceed that, you nest them into sub-tuples. Each sub-tuple
    // counts as one element in the outer tuple. This is a Bevy limitation,
    // not a Rust one — Bevy uses macros to implement the Plugins trait for
    // tuples up to a certain size.
    .add_plugins((
        // default_nearest() switches every image to nearest-neighbor
        // filtering instead of bilinear. Without this, pixel art looks
        // blurry when scaled up because bilinear interpolation blends
        // neighboring pixels together. Nearest-neighbor just picks the
        // closest texel, preserving hard pixel edges.
        // We chain multiple .set() calls on DefaultPlugins to override
        // individual sub-plugins. Each .set() replaces one plugin's config
        // while leaving the rest at their defaults.
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // Sets the initial window size to 1200x800 pixels.
                    // This matches our "arena" — the world space the camera
                    // will show. Having the window match the camera's fixed
                    // projection means 1 world unit = 1 pixel at default zoom.
                    // WindowResolution::new() takes u32 (physical pixels),
                    // not f32. This is the actual pixel count the OS allocates.
                    resolution: WindowResolution::new(1200, 800),
                    // Prevent the user from shrinking the window below the
                    // arena size. Without this, resizing smaller would either
                    // clip content or leave black bars depending on scaling.
                    resize_constraints: WindowResizeConstraints {
                        min_width: 1200.0,
                        min_height: 800.0,
                        ..default()
                    },
                    title: "Never Tell Me the Odds".into(),
                    ..default()
                }),
                ..default()
            }),
        save_load::SaveLoadPlugin,
        audio::AudioPlugin,
        animation::AnimationPlugin,
        render::RenderPlugin,
        armies::ArmiesPlugin,
        movement::MovementPlugin,
        pick_target::PickTargetPlugin,
        utils::UtilsPlugin,
    ))
    .add_plugins((
        health::HealthPlugin,
        combat::CombatPlugin,
        end_round::EndRoundPlugin,
        setup_round::SetupRoundPlugin,
        spawn_slimes::SpawnSlimesPlugin,
        special_abilities::SpecialAbilitiesPlugin,
        shaders_lite::ShadersLitePlugin,
        sprite_modifications::SpriteModificationsPlugin,
        home::HomePlugin,
    ))
    // init_state must come AFTER add_plugins(DefaultPlugins) because DefaultPlugins
    // includes StatesPlugin, which sets up the StateTransition schedule that
    // init_state depends on. Without StatesPlugin, there's no infrastructure
    // for tracking state changes, running OnEnter/OnExit, or evaluating in_state().
    .init_state::<GameState>()
    .add_systems(PreStartup, load_game_font)
    .add_systems(Startup, (spawn_camera, leave_initial_loading))
    .add_systems(OnEnter(GameState::Combat), setup_combat_arena)
    .add_systems(OnExit(GameState::Combat), cleanup_combat_resources)
    .add_systems(
        Update,
        kill_random_on_spacebar.run_if(in_state(GameState::Combat)),
    )
    .run();
}

/// Loads the game font and stores it as a resource for other systems to use.
///
/// asset_server.load() starts an async load and immediately returns a Handle.
/// The font isn't ready yet at this point — Bevy will finish loading it in the
/// background. This is fine because text entities that reference the handle will
/// automatically render once the asset is available.
pub fn load_game_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("typography/upheaval/upheaval-tt-brk.upheaval-tt-brk.ttf");
    commands.insert_resource(GameFont(font));
}

/// Transitions out of InitialLoading into Combat. By running at Startup (which
/// fires after PreStartup), all resources like GameFont, SaveData, and GameAudio
/// are guaranteed to exist before any OnEnter(Combat) systems run.
fn leave_initial_loading(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Combat);
}

/// Spawns the camera. This runs once at Startup and persists across all states.
fn spawn_camera(mut commands: Commands, arena: Res<ArenaBounds>) {
    // Camera2d is a marker component that says "this is a 2D camera."
    // We override the Projection to use ScalingMode::Fixed so the camera
    // ALWAYS shows exactly 1200x800 world units, no matter the window size.
    //
    // Important Bevy 0.18 detail: OrthographicProjection is NOT a standalone
    // component. It's wrapped inside the Projection enum:
    //   Projection::Orthographic(OrthographicProjection { ... })
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: arena.width,
                height: arena.height,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
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

    // Vignette overlay: a programmatically-generated gradient texture that's
    // dark/opaque at the top and bottom edges and transparent in the middle.
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

/// Removes combat-only resources when leaving the Combat state.
fn cleanup_combat_resources(mut commands: Commands) {
    commands.remove_resource::<end_round::RoundResult>();
    commands.remove_resource::<setup_round::PreGameTimer>();
    commands.remove_resource::<spawn_slimes::SlimeSpawnTimer>();
    commands.remove_resource::<spawn_slimes::SlimesToSpawn>();
}

/// Debug system: press spacebar to kill a random slime.
fn kill_random_on_spacebar(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Health>) {
    if keyboard.just_pressed(KeyCode::Space) {
        let mut rng = rand::thread_rng();
        // iter_mut() gives us mutable access to Health components.
        // choose() picks one at random, returning Option (None if query is empty).
        if let Some(mut health) = query.iter_mut().choose(&mut rng) {
            health.0 = 0;
        }
    }
}

mod animation;
mod armies;
mod audio;
mod combat;
mod end_round;
mod health;
mod home;
mod movement;
mod pick_target;
mod render;
mod save_load;
mod setup_round;
mod shaders_lite;
mod spawn_slimes;
mod special_abilities;
mod sprite_modifications;
mod utils;
