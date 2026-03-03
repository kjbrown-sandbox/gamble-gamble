use bevy::prelude::*;

use crate::GameState;

pub struct ScreenFadePlugin;

impl Plugin for ScreenFadePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, screen_fade_system);
    }
}

#[derive(Clone, Copy)]
enum FadePhase {
    FadingOut,
    FadingIn,
}

#[derive(Component)]
pub struct ScreenFade {
    timer: Timer,
    phase: FadePhase,
    target_state: GameState,
    phase_duration: f32,
}

/// Spawns a full-screen black overlay that fades out, transitions state at
/// the midpoint, then fades back in before despawning. Each phase lasts
/// `phase_duration` seconds. The high ZIndex covers all other UI.
pub fn spawn_screen_fade(commands: &mut Commands, target_state: GameState, phase_duration: f32) {
    commands.spawn((
        ScreenFade {
            timer: Timer::from_seconds(phase_duration, TimerMode::Once),
            phase: FadePhase::FadingOut,
            target_state,
            phase_duration,
        },
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        ZIndex(100),
    ));
}

fn screen_fade_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScreenFade, &mut BackgroundColor)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, mut fade, mut bg) in &mut query {
        fade.timer.tick(time.delta());
        let t = fade.timer.fraction();

        match fade.phase {
            FadePhase::FadingOut => {
                bg.0 = Color::srgba(0.0, 0.0, 0.0, t);

                if fade.timer.is_finished() {
                    next_state.set(fade.target_state.clone());
                    fade.timer = Timer::from_seconds(fade.phase_duration, TimerMode::Once);
                    fade.phase = FadePhase::FadingIn;
                }
            }
            FadePhase::FadingIn => {
                bg.0 = Color::srgba(0.0, 0.0, 0.0, 1.0 - t);

                if fade.timer.is_finished() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
