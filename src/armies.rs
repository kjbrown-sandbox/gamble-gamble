use bevy::prelude::*;
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

pub fn create_enemy_army() -> Army {
    Army {
        normal: NormalSlime { count: 1, hp: 5 },
        tanks: TankSlime {
            count: 0,
            ..Default::default()
        },
        wizards: WizardSlime {
            count: 0,
            ..Default::default()
        },
    }
}
