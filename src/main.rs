// main.rs - Entry point for our Bevy game
// 
// This file demonstrates the absolute basics of a Bevy application:
// 1. Creating an App (the core container for our game)
// 2. Adding plugins (bundles of functionality)
// 3. Adding systems (functions that run each frame or at specific times)
// 4. Spawning entities with components (the E and C in ECS)

use bevy::prelude::*;

fn main() {
    // App::new() creates an empty Bevy application.
    // We then chain methods to configure it before calling .run()
    App::new()
        // DefaultPlugins is a collection of essential plugins that most games need:
        // - WindowPlugin: creates and manages the game window
        // - RenderPlugin: handles rendering
        // - InputPlugin: handles keyboard/mouse/gamepad input
        // - AssetPlugin: handles loading assets (images, sounds, etc.)
        // - And many more!
        // 
        // We customize it here to set a dark background color (ClearColor).
        // insert_resource() adds a "resource" - global data that isn't tied to an entity.
        .add_plugins(DefaultPlugins)
        // ClearColor is the background color of the window.
        // Color::BLACK gives us a pure black background.
        // Resources are singleton data - there's only one ClearColor for the whole app.
        .insert_resource(ClearColor(Color::BLACK))
        // add_systems() registers functions to run at specific points in the game loop.
        // Startup means this system runs ONCE when the app first starts.
        // This is perfect for spawning initial entities like cameras and our circle.
        .add_systems(Startup, setup)
        // Finally, .run() starts the game loop. This call blocks and runs forever
        // (or until the window is closed).
        .run();
}

/// Setup system - runs once at startup to create our initial game state.
/// 
/// The `Commands` parameter is how we spawn/despawn entities and modify the world.
/// Bevy uses "dependency injection" - you just declare what you need as parameters,
/// and Bevy provides them automatically. This is a key pattern in Bevy!
/// 
/// Common things you can request:
/// - Commands: spawn/despawn entities, insert/remove components
/// - Query<...>: access entities that have specific components
/// - Res<T>: read-only access to a resource
/// - ResMut<T>: mutable access to a resource
/// - EventReader/EventWriter: for event handling
fn setup(
    mut commands: Commands,
    // We need access to meshes and materials to create 2D shapes.
    // These are "Assets" - resources that can be loaded or created dynamically.
    // ResMut gives us mutable access because we're adding new assets.
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // First, we MUST spawn a camera, otherwise nothing will be rendered!
    // Camera2d is a marker component that tells Bevy this entity is a 2D camera.
    // In Bevy, cameras determine what gets rendered and how.
    // Without a camera, the render pipeline has nothing to draw to.
    commands.spawn(Camera2d);

    // Now let's create our white circle!
    // In Bevy, 2D shapes are rendered using "meshes" (the geometry) and 
    // "materials" (the appearance/color).
    
    // Circle::new(radius) creates a circle mesh with the given radius in pixels.
    // meshes.add() stores the mesh in the asset system and returns a Handle<Mesh>.
    // Handles are like smart pointers to assets - they're lightweight references.
    let circle_mesh = meshes.add(Circle::new(50.0));
    
    // ColorMaterial is the simplest 2D material - just a solid color.
    // Color::WHITE gives us a pure white color.
    // Again, materials.add() stores it and returns a Handle<ColorMaterial>.
    let white_material = materials.add(ColorMaterial::from(Color::WHITE));

    // Now we spawn an entity with the components needed to render a 2D shape.
    // 
    // Mesh2d: tells Bevy this entity should be rendered as a 2D mesh
    // MeshMaterial2d: specifies what material (color/texture) to use
    // Transform: position, rotation, and scale in the world
    // 
    // Transform::default() puts the entity at (0, 0, 0) which is the center
    // of the screen in 2D (with the default camera setup).
    commands.spawn((
        // The Mesh2d component wraps our circle mesh handle
        Mesh2d(circle_mesh),
        // The MeshMaterial2d component wraps our white material handle
        MeshMaterial2d(white_material),
        // Transform controls where the entity is in the world.
        // default() gives us position (0,0,0), no rotation, scale (1,1,1).
        // In 2D, (0,0) is the center of the screen by default.
        Transform::default(),
    ));
}
