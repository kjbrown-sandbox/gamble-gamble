use bevy::prelude::*;

use crate::{animation::AnimationState, combat::IceImpactVfx};

pub struct SetupRoundPlugin;

impl Plugin for SetupRoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_pre_game_timer).add_systems(
            Update,
            (
                pre_game_timer_system.run_if(resource_exists::<PreGameTimer>),
                stun_timer_system,
            ),
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

fn start_pre_game_timer(mut commands: Commands) {
    commands.insert_resource(PreGameTimer(Timer::from_seconds(2.0, TimerMode::Once)));
}

fn pre_game_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<PreGameTimer>,
    inert_entities: Query<Entity, With<Inert>>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        // Remove Inert from every entity that has it
        for entity in &inert_entities {
            commands.entity(entity).remove::<Inert>();
        }

        // Timer's job is done — remove the resource so this system stops running
        commands.remove_resource::<PreGameTimer>();
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
