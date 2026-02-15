use bevy::prelude::*;

pub struct ArmiesPlugin;

impl Plugin for ArmiesPlugin {
    fn build(&self, app: &mut App) {
        // Insert the resource directly at app build time.
        // Since this is static data defined in code, we don't need a startup system —
        // we can just call init_resource. Bevy will call EnemyArmies::default() for us.
        app.init_resource::<EnemyArmies>();
    }
}

/// Represents a predefined army configuration.
/// These are static game data — the same every time you play.
/// Think of them like level definitions or enemy encounter tables.
///
/// Unlike SaveData, these never change at runtime and never get written to disk.
/// They're defined right here in code, just like animation frame counts or attack stats.
#[derive(Debug, Clone)]
pub struct ArmyDefinition {
    pub name: String,
    pub slime_count: u32,
    // Eventually you might add:
    // pub units: Vec<UnitType>,
    // pub boss: Option<BossType>,
    // pub difficulty: f32,
}

/// Resource that holds all enemy army definitions.
///
/// This is a Resource (shared game-wide data), not a Component (per-entity data).
/// Any system can read it with Res<EnemyArmies>.
///
/// Why a Vec? Even though we only have one army now, the game design calls for
/// progressing through multiple fights. A list of armies = a list of encounters.
#[derive(Resource, Debug)]
pub struct EnemyArmies {
    pub armies: Vec<ArmyDefinition>,
}

/// Default gives us the starting set of enemy armies.
/// Bevy's init_resource uses Default to create the resource automatically.
impl Default for EnemyArmies {
    fn default() -> Self {
        Self {
            armies: vec![
                ArmyDefinition {
                    name: "Basic Slimes".to_string(),
                    slime_count: 5,
                },
                // Add more encounters here as the game grows:
                // ArmyDefinition {
                //     name: "Slime Horde".to_string(),
                //     slime_count: 10,
                // },
            ],
        }
    }
}
