use bevy::{prelude::*, render::render_resource::Texture};
use rand::seq::IteratorRandom;

use crate::{animation::AnimationType, health::Health, pick_target::Team};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, load_sprite_sheets)
        //     .add_systems(Update, (switch_animation_system, animation_system).chain());
    }
}

#[derive(Component)]
pub struct KnownAttacks(Vec<Attack>);

#[derive(Component)]
pub struct Attack {
    pub animation: AnimationType,
    pub hit_frame: usize, // 0-indexed frame when damage should be applied
    pub on_hit_effect: AttackEffect,
    pub on_miss_effect: Option<AttackEffect>, // Optional effect to apply if attack misses
    pub range: f32,
}

#[derive(Component, Default)]

pub struct AttackEffect {
    pub damage: i32,
    pub knockback: f32,
}

pub fn start_attack_system(
    // anyone who has KnownAttacks
    // anyone who has health
    attackers: Query<(&KnownAttacks, &Transform, &Team)>,
    targets: Query<(&Transform, &Team), With<Health>>,
) {
    for (known_attacks, attacker_transform, attacker_team) in attackers.iter() {
        for (target_transform, target_team) in targets.iter() {
            if attacker_team != target_team {
                let distance = attacker_transform
                    .translation
                    .distance(target_transform.translation);

                let attack: Vec<&Attack> = known_attacks
                    .0
                    .iter()
                    .filter(|attack| distance <= attack.range)
                    .choose(&mut rand::thread_rng());
            }
        }
    }
}

pub struct OnHitEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub attack_effect: AttackEffect,
}
