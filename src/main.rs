use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, animation::AnimationPlugin))
        .run();
}

mod animation;
