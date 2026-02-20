use bevy::prelude::*;

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
    mut query: Query<(Entity, &mut Transform, &mut SpriteModification)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut modification) in query.iter_mut() {
        modification.timer.tick(time.delta());

        if modification.timer.just_finished() {
            // Snap to exact final scale and remove the component
            transform.scale = Vec3::splat(1.0);
            commands.entity(entity).remove::<SpriteModification>();
        } else {
            let t = modification.timer.fraction();
            let scale = match modification.lerp {
                LerpType::EaseInOut => {
                    // BackOut overshoots past 1.0 then settles back.
                    // Lerp from 25% to 100% — the overshoot makes it
                    // temporarily exceed 100% before landing exactly there.
                    let eased_t = EaseFunction::BackOut.sample_clamped(t);
                    0.25 + (1.0 - 0.25) * eased_t
                }
            };

            transform.scale = Vec3::splat(scale);
        }
    }
}
