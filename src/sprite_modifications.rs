use bevy::prelude::*;

use crate::pick_target::Team;
use crate::setup_round::PreGameTimer;
use crate::sprite_modifications;

pub struct SpriteModificationsPlugin;

impl Plugin for SpriteModificationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, modify_sprite_system);
    }
}

#[derive(Component)]
pub struct SpriteModification {
    pub lerp: LerpType,
    pub timer: Timer,
}

pub enum LerpType {
    EaseInOut,
}

fn modify_sprite_system(
    mut commands: Commands,
    mut sprite_modifications: Query<(Entity, &Sprite, &mut SpriteModification)>,
    game_time: Res<Time>,
) {
    let total_animation_time = 1.0;
    for (entity, mut sprite, mut modification) in sprite_modifications.iter_mut() {
        modification.timer.tick(game_time.delta());

        if modification.timer.just_finished() {
            // Animation is done — remove the component to stop the system from running for this entity
            commands.entity(entity).remove::<SpriteModification>();
        } else {
            // Animation is still in progress — calculate the current lerp value based on the timer
            let t = modification.timer.fraction();
            match modification.lerp {
                LerpType::EaseInOut => {
                    let eased_t = EaseFunction::BackOut.sample_clamped(t);
                    sprite.custom_size = Some(Vec2::splat(1.0 + 0.5 * eased_t));
                    // Example: scale from 1.0 to 1.5
                }
            }
        }
    }
}
