use bevy::prelude::*;

pub struct SetupRoundPlugin;

impl Plugin for SetupRoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_pre_game_timer)
            .add_systems(
                Update,
                pre_game_timer_system.run_if(resource_exists::<PreGameTimer>),
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
struct PreGameTimer(Timer);

fn start_pre_game_timer(mut commands: Commands) {
    commands.insert_resource(PreGameTimer(Timer::from_seconds(3.0, TimerMode::Once)));
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
