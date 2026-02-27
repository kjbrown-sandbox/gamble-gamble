use bevy::prelude::*;
// In Bevy 0.18, camera types moved to bevy::camera (the bevy_camera crate),
// NOT bevy::render::camera. This is a common gotcha when reading older tutorials.
use bevy::camera::ScalingMode;
use bevy::image::{ImageSampler, TextureFormatPixelInfo};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::{WindowResizeConstraints, WindowResolution};
use rand::seq::IteratorRandom;
use rand::Rng;

use crate::animation::AnimationType;
use crate::armies::EnemyArmies;
use crate::combat::{Attack, AttackEffect, KnownAttacks};
use crate::health::{DeathAnimation, Health};
use crate::movement::Speed;
use crate::pick_target::{PickTargetStrategy, Team};
use crate::save_load::SaveData;

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
    ))
    // spawn_slimes needs three resources to exist first:
    //   - SpriteSheets (from animation::load_sprite_sheets)
    //   - SaveData (from save_load's startup system)
    //   - EnemyArmies (from armies plugin, via init_resource — available immediately)
    .add_systems(PreStartup, load_game_font)
    .add_systems(Startup, spawn_slimes.after(animation::load_sprite_sheets))
    .add_systems(Update, kill_random_on_spacebar)
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

/// Spawns both armies using data from our resources instead of hardcoded values.
///
/// - Player army size comes from SaveData (persisted to disk between sessions)
/// - Enemy army comes from EnemyArmies (static game data defined in code)
///
/// Note: we take Res<T> (immutable reference) since we only need to read these.
/// If we needed to modify them, we'd use ResMut<T>.
fn spawn_slimes(
    mut commands: Commands,
    save_data: Res<SaveData>,
    enemy_armies: Res<EnemyArmies>,
    arena: Res<ArenaBounds>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    //  let mut rng = rand::thread_rng();

    //  // Player army — count comes from the save file
    //  for _ in 0..save_data.slime_count {
    //      let x = rng.gen_range(-500.0..-100.0);
    //      let y = rng.gen_range(-300.0..300.0);
    //      commands.spawn((
    //          AnimationType::SlimeJumpIdle,
    //          Transform::from_xyz(x, y, 0.0),
    //          Team::Player,
    //          PickTargetStrategy::Close,
    //          DeathAnimation(AnimationType::SlimeDeath),
    //          Health(10),
    //          Speed(125.0),
    //          // KnownAttacks is the entity's "move list" — all attacks it can perform.
    //          // pick_attack_system will choose from these based on distance to target.
    //          KnownAttacks(vec![Attack {
    //              animation: AnimationType::SlimeAttack,
    //              hit_frame: 3, // damage lands on frame 3 of the attack animation
    //              on_hit_effect: AttackEffect {
    //                  damage: 2,
    //                  knockback: 0.0,
    //              },
    //              range: 60.0, // must be >= 50.0 (movement stops at 50 units)
    //          }]),
    //      ));
    //  }

    //  // Enemy army — use the first army definition for now.
    //  // Later, which army you fight could depend on what stage/round you're on.
    //  let enemy_army = &enemy_armies.armies[0];
    //  for _ in 0..enemy_army.slime_count {
    //      let x = rng.gen_range(100.0..500.0);
    //      let y = rng.gen_range(-300.0..300.0);
    //      commands.spawn((
    //          AnimationType::SlimeJumpIdle,
    //          Transform::from_xyz(x, y, 0.0),
    //          Team::Enemy,
    //          PickTargetStrategy::Close,
    //          DeathAnimation(AnimationType::SlimeDeath),
    //          Health(10),
    //          Speed(125.0),
    //          KnownAttacks(vec![Attack {
    //              animation: AnimationType::SlimeAttack,
    //              hit_frame: 3,
    //              on_hit_effect: AttackEffect {
    //                  damage: 2,
    //                  knockback: 0.0,
    //              },
    //              range: 60.0,
    //          }]),
    //      ));
    //  }

    // Background image. z = -1 places it behind all game entities (which default to z = 0+).
    // custom_size overrides the sprite dimensions so it fills the arena regardless of
    // the image's native resolution. The image field holds the asset Handle<Image>.
    commands.spawn((
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
    // This is created in code rather than loaded from a file — Bevy's Assets<Image>
    // lets you add images you build yourself, not just ones from disk.
    let vignette_height: u32 = 64;
    let mut pixel_data = Vec::with_capacity((vignette_height * 4) as usize);
    for y in 0..vignette_height {
        let t = y as f32 / (vignette_height - 1) as f32;
        // Squaring the distance from center makes the fade nonlinear:
        // mostly transparent in the middle, ramping up sharply near edges.
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
    // Nearest-neighbor would show visible banding on a 64-pixel gradient
    // stretched to 800 pixels. Linear interpolation smooths between the
    // 64 samples so the fade looks continuous.
    vignette_image.sampler = ImageSampler::linear();
    let vignette_handle = images.add(vignette_image);

    commands.spawn((
        render::Background,
        Sprite {
            image: vignette_handle,
            custom_size: Some(Vec2::new(arena.width, arena.height)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 9.0),
    ));

    // Camera2d is a marker component that says "this is a 2D camera."
    // When spawned, Bevy's #[require] attribute automatically adds Camera,
    // Projection, and Frustum components with sensible 2D defaults.
    //
    // We override the Projection to use ScalingMode::Fixed so the camera
    // ALWAYS shows exactly 1200x800 world units, no matter the window size.
    // If the user makes the window larger, everything scales up — the visible
    // area stays the same. This is ideal for pixel art games where you want
    // a consistent viewport.
    //
    // Important Bevy 0.18 detail: OrthographicProjection is NOT a standalone
    // component. It's wrapped inside the Projection enum:
    //   Projection::Orthographic(OrthographicProjection { ... })
    // This is because a camera could also use Projection::Perspective for 3D.
    // The enum lets Bevy handle both projection types with one component slot.
    //
    // Alternative scaling modes worth knowing:
    // - WindowSize: 1 world unit = 1 pixel (visible area changes with window)
    // - AutoMin/AutoMax: adapts to aspect ratio while guaranteeing min/max area
    // - FixedVertical/FixedHorizontal: locks one axis, stretches the other
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
