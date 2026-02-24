use bevy::{audio::Volume, prelude::*};

pub struct ShadersLitePlugin;

impl Plugin for ShadersLitePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_add_damage_tint,
                damage_tint_system,
                on_add_flash,
                flash_system,
            ),
        );
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
            if let Ok(mut cmds) = commands.get_entity(entity) {
                cmds.remove::<DamageTint>();
            }
            sprite.color = Color::WHITE;
        }
    }
}

/// Identical to DamageTint but flashes white instead of red.
/// Used on the Shield child entity when a tank blocks an incoming attack.
/// The pattern is the same: insert the component → on_add sets the color →
/// timer ticks → when finished, remove component and reset to WHITE.
#[derive(Component, Default)]
pub struct Flash(pub Timer);

pub fn on_add_flash(mut query: Query<&mut Sprite, Added<Flash>>) {
    for mut sprite in query.iter_mut() {
        // Values > 1.0 multiply the texture color above its normal brightness.
        // This blows out every pixel toward pure white — even dark parts of the
        // shield texture hit 1.0 after the multiply, so the whole sprite flashes.
        sprite.color = Color::linear_rgba(10.0, 10.0, 10.0, 1.0);
    }
}

pub fn flash_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut Flash)>,
    time: Res<Time>,
) {
    for (entity, mut sprite, mut flash) in query.iter_mut() {
        flash.0.tick(time.delta());
        if flash.0.is_finished() {
            if let Ok(mut cmds) = commands.get_entity(entity) {
                cmds.remove::<Flash>();
            }
            sprite.color = Color::WHITE;
        }
    }
}
