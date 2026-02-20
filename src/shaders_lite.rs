use bevy::{audio::Volume, prelude::*};

pub struct ShadersLitePlugin;

impl Plugin for ShadersLitePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (on_add_damage_tint, damage_tint_system));
    }
}

#[derive(Component, Default)]
pub struct DamageTint(pub Timer);

pub fn on_add_damage_tint(mut query: Query<(&mut Sprite), Added<DamageTint>>) {
    for (mut sprite) in query.iter_mut() {
        sprite.color = Color::srgba(1.0, 0.55, 0.55, 1.0);
    }
}

pub fn damage_tint_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut DamageTint)>,
    time: Res<Time>,
) {
    for (entity, mut sprite, mut damage_tint) in query.iter_mut() {
        damage_tint.0.tick(time.delta());
        if damage_tint.0.is_finished() {
            commands.entity(entity).remove::<DamageTint>();
            sprite.color = Color::WHITE;
        }
    }
}
