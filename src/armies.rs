use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub struct ArmiesPlugin;

impl Plugin for ArmiesPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Army {
    pub normal: NormalSlime,
    pub tanks: TankSlime,
    pub wizards: WizardSlime,
}

impl Default for Army {
    fn default() -> Self {
        Self {
            normal: NormalSlime::default(),
            tanks: TankSlime::default(),
            wizards: WizardSlime::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NormalSlime {
    pub count: u32,
    pub hp: i32,
}

impl Default for NormalSlime {
    fn default() -> Self {
        Self { count: 1, hp: 5 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TankSlime {
    pub count: u32,
    pub hp: i32,
    pub block_chance: f32,
    pub stun_chance: f32,
}

impl Default for TankSlime {
    fn default() -> Self {
        Self {
            count: 0,
            hp: 10,
            block_chance: 0.2,
            stun_chance: 0.1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WizardSlime {
    pub count: u32,
    pub hp: i32,
    pub spell_range: f32,
    pub aoe_damage: i32,
    pub spear_knockback: f32,
}

impl Default for WizardSlime {
    fn default() -> Self {
        Self {
            count: 0,
            hp: 5,
            spell_range: 500.0,
            aoe_damage: 1,
            spear_knockback: 200.0,
        }
    }
}

/// Describes an enemy wave: normal army units plus any pre-made merged slimes.
/// Merged slimes are tracked separately because they bypass the normal Army
/// spawn logic — they use a different spawn function with BigSlime animations.
pub struct EnemyWave {
    pub army: Army,
    pub merged_count: u32,
}

pub fn create_enemy_army(level: u32) -> EnemyWave {
    let mut rng = rand::thread_rng();

    match level {
        1..=5 => EnemyWave {
            army: Army {
                normal: NormalSlime {
                    count: 1,
                    hp: rng.gen_range(4..=6),
                },
                ..Default::default()
            },
            merged_count: 0,
        },
        6..=9 => EnemyWave {
            army: Army {
                normal: NormalSlime {
                    count: rng.gen_range(1..=2),
                    hp: rng.gen_range(4..=6),
                },
                ..Default::default()
            },
            merged_count: 0,
        },
        10 => EnemyWave {
            army: Army {
                normal: NormalSlime { count: 0, hp: 5 },
                ..Default::default()
            },
            merged_count: 1,
        },
        _ => EnemyWave {
            army: Army {
                normal: NormalSlime { count: 1, hp: 5 },
                ..Default::default()
            },
            merged_count: 0,
        },
    }
}
