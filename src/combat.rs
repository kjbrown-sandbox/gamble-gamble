use bevy::{prelude::*, state::commands};
use rand::seq::IteratorRandom;

use crate::{
    animation::{AnimationState, AnimationType},
    health::{DamagedEvent, Dying, Health},
    movement::TargetEntity,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_hit_observer);

        // Chain these systems so they run in order within a single frame.
        // The data flows like a pipeline:
        //   pick_attack → attack → hit_frame_check → cleanup
        //
        // on_hit_observer is NOT in this chain — it runs immediately when
        // hit_frame_check_system calls commands.trigger(OnHitEvent { ... }).
        //
        // "Chaining" means each system runs after the previous one finishes,
        // which guarantees that e.g. ActiveAttack exists before attack_system tries to read it.
        // Without .chain(), Bevy could run them in any order (or even in parallel).
        app.add_systems(
            Update,
            (
                pick_attack_system,
                attack_system,
                hit_frame_check_system,
                attack_cleanup_system,
            )
                .chain(),
        );
    }
}

// ── Data types ──────────────────────────────────────────────────────────────

/// A collection of attacks that an entity knows how to perform.
/// This lives on the entity permanently — it's the entity's "move list."
///
/// Note: KnownAttacks is a Component, but Attack itself is NOT a component.
/// Attack is just plain data stored inside this Vec. This is a common ECS pattern:
/// not everything needs to be a component. Components are for things you want to
/// query/filter on. Attack is just configuration data, so it lives inside a component.
#[derive(Component)]
pub struct KnownAttacks(pub Vec<Attack>);

/// Describes a single attack: which animation to play, when damage lands,
/// what happens on hit, and how close the target needs to be.
///
/// Clone is derived instead of Component because Attack is stored inside
/// KnownAttacks (a Vec<Attack>), not placed on entities directly.
/// We need Clone so we can copy an Attack out of the Vec into ActiveAttack.
#[derive(Clone)]
pub struct Attack {
    pub animation: AnimationType,
    pub hit_frame: usize, // 0-indexed frame when damage should be applied
    pub on_hit_effect: AttackEffect,
    pub range: f32,
}

/// The effect that happens when an attack connects.
/// Separated from Attack so we can pass it around independently (e.g. in OnHitEvent).
///
/// Clone is needed so we can copy it out of Attack into OnHitEvent.
/// Default is derived so we could create a "no effect" AttackEffect easily.
#[derive(Clone, Default)]
pub struct AttackEffect {
    pub damage: i32,
    pub knockback: f32,
}

/// Marks an entity as currently performing an attack.
/// This is a *temporary* component — it gets added when an attack starts
/// and removed when the attack animation finishes. This is a classic ECS pattern:
/// using the presence/absence of a component as state.
///
/// Think of it like a flag: "this entity is busy attacking right now."
/// Systems use Without<ActiveAttack> to skip entities that are mid-attack.
#[derive(Component)]
pub struct ActiveAttack {
    pub attack: Attack,
    pub target: Entity,
    /// Prevents the hit from firing multiple frames in a row.
    /// Once we send OnHitEvent, we set this to true so we don't
    /// send it again on subsequent frames while still on the hit_frame.
    pub hit_triggered: bool,
}

/// An event that fires when an attack's hit frame is reached.
/// This uses the Observer pattern: hit_frame_check_system calls
/// commands.trigger(OnHitEvent { ... }), and on_hit_observer reacts immediately.
///
/// This decouples "detecting the hit" from "applying the damage" — a key ECS principle.
/// The hit detection system doesn't need to know anything about health or knockback.
///
/// #[derive(Event)] in Bevy 0.18 is specifically for the Observer pattern
/// (commands.trigger() + On<T>). For buffered system-to-system communication,
/// you'd use #[derive(Message)] with MessageWriter/MessageReader instead.
#[derive(Event)]
pub struct OnHitEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub effect: AttackEffect,
}

// ── Systems ─────────────────────────────────────────────────────────────────
/// Picks an attack for entities that are in range of their target but not already attacking.
fn pick_attack_system(
    attackers: Query<
        (Entity, &KnownAttacks, &Transform, &TargetEntity),
        (Without<ActiveAttack>, Without<Dying>),
    >,
    targets: Query<&Transform>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    for (entity, known_attacks, attacker_transform, target_entity) in attackers.iter() {
        let Ok(target_transform) = targets.get(target_entity.0) else {
            continue;
        };

        let distance = attacker_transform
            .translation
            .distance(target_transform.translation);

        // Filter to only attacks whose range covers the current distance,
        // then randomly choose one. choose() returns None if no attacks are in range.
        let chosen_attack = known_attacks
            .0
            .iter()
            .filter(|attack| distance <= attack.range)
            .choose(&mut rng);

        if let Some(attack) = chosen_attack {
            commands.entity(entity).insert(ActiveAttack {
                attack: attack.clone(),
                target: target_entity.0,
                hit_triggered: false,
            });
        }
    }
}

/// When an entity just received an ActiveAttack component, switch its animation
fn attack_system(mut query: Query<(&ActiveAttack, &mut AnimationType), Added<ActiveAttack>>) {
    for (active_attack, mut animation_type) in query.iter_mut() {
        *animation_type = active_attack.attack.animation;
    }
}

/// Checks if the current animation frame has reached the attack's "hit frame."
/// This is how we sync damage with the visual — the slime's attack animation
/// might show a slam on frame 3, so we fire the hit event on frame 3.
fn hit_frame_check_system(
    mut query: Query<(Entity, &mut ActiveAttack, &AnimationState), Without<Dying>>,
    mut commands: Commands,
) {
    for (entity, mut active_attack, anim_state) in query.iter_mut() {
        if anim_state.frame_index >= active_attack.attack.hit_frame && !active_attack.hit_triggered
        {
            // Trigger the event — on_hit_observer will run immediately and apply damage.
            // commands.trigger() fires a "global" event that any registered observer can react to.
            commands.trigger(OnHitEvent {
                attacker: entity,
                target: active_attack.target,
                effect: active_attack.attack.on_hit_effect.clone(),
            });
            // Mark as triggered so we don't fire duplicate events
            // on subsequent frames while frame_index is still >= hit_frame.
            active_attack.hit_triggered = true;
        }
    }
}

/// Observer that reacts to OnHitEvent and applies damage + knockback.
///
/// Observers are functions registered with app.add_observer(). They run immediately
/// when their event type is triggered via commands.trigger(). Unlike regular systems,
/// they don't need to be scheduled — they fire inline the moment the event happens.
///
/// The first parameter is On<OnHitEvent>, which wraps the event data.
/// You access the event fields by dereferencing: trigger.attacker, trigger.target, etc.
/// Additional parameters work just like regular system parameters (queries, commands, etc.).
///
/// ParamSet is still needed here because we read the attacker's Transform (p0)
/// and mutate the target's Health + Transform (p1). Both touch Transform,
/// so Bevy won't allow two overlapping queries without ParamSet.
fn on_hit_observer(
    trigger: On<OnHitEvent>,
    mut params: ParamSet<(
        Query<&Transform>,                    // p0: read attacker position
        Query<(&mut Health, &mut Transform)>, // p1: mutate target health + position
    )>,
    mut commands: Commands,
) {
    // Read the attacker's position first (using p0), then release it.
    let Some(attacker_pos) = params
        .p0()
        .get(trigger.attacker)
        .ok()
        .map(|t| t.translation)
    else {
        return;
    };

    // Now use p1 to mutate the target's health and position.
    if let Ok((mut health, mut transform)) = params.p1().get_mut(trigger.target) {
        // Apply damage
        if health.0 > 0 {
            health.0 -= trigger.effect.damage;
            commands.trigger(DamagedEvent {
                entity: trigger.target,
            });
        }

        // Apply knockback: push target away from attacker.
        // This is an instant displacement, not a velocity — the entity
        // teleports a short distance. For smoother knockback, you'd add
        // a Velocity component and let a physics system handle it over time.
        if trigger.effect.knockback > 0.0 {
            let diff = transform.translation - attacker_pos;
            if diff.length() > 0.01 {
                let direction = diff.normalize();
                // Only push on x/y, not z (z is for draw order)
                transform.translation += Vec3::new(
                    direction.x * trigger.effect.knockback,
                    direction.y * trigger.effect.knockback,
                    0.0,
                );
            }
        }
    }
}

fn attack_cleanup_system(
    mut commands: Commands,
    // Without<Dying> is critical here: if an entity is dying, the death system will
    // despawn it. If we also try to modify it, we race with the despawn and get
    // "Entity despawned" errors when our deferred command runs after the despawn.
    mut query: Query<(Entity, &AnimationState, &mut AnimationType), (With<ActiveAttack>, Without<Dying>)>,
) {
    for (entity, anim_state, mut animation_type) in query.iter_mut() {
        if anim_state.finished {
            // Remove ActiveAttack so pick_attack_system can assign a new attack.
            commands.entity(entity).remove::<ActiveAttack>();
            // Go back to idle animation
            *animation_type = AnimationType::SlimeJumpIdle;
        }
    }
}
