use bevy::prelude::*;

use crate::pick_target::Team;
use crate::setup_round::PreGameTimer;

pub struct EndRoundPlugin;

impl Plugin for EndRoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            check_round_end_system.run_if(
                not(resource_exists::<RoundResult>).and(not(resource_exists::<PreGameTimer>)),
            ),
        );
    }
}

/// Resource inserted once a winner is determined, preventing further checks.
#[derive(Resource)]
struct RoundResult;

/// Marker so we can find/despawn the result text later if needed.
#[derive(Component)]
struct RoundResultText;

fn check_round_end_system(mut commands: Commands, teams: Query<&Team>) {
    let mut has_player = false;
    let mut has_enemy = false;

    for team in &teams {
        match team {
            Team::Player => has_player = true,
            Team::Enemy => has_enemy = true,
        }
        // If both sides still alive, no need to keep checking
        if has_player && has_enemy {
            return;
        }
    }

    info!(has_player, has_enemy);
    // Determine the message — if neither side exists, treat it as defeat
    let message = if has_player && !has_enemy {
        "VICTORY!"
    } else {
        "DEFEAT!"
    };

    // Mark the round as over so this system stops running
    commands.insert_resource(RoundResult);

    // Full-screen centered container with the result text
    commands
        .spawn((
            RoundResultText,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(message),
                TextFont {
                    font_size: 120.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}
