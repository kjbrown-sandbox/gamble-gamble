use bevy::{audio::Volume, prelude::*, state::commands};
use rand::{seq::IteratorRandom, Rng};

use crate::{
    animation::{AnimationState, AnimationType, IdleAnimation},
    audio::GameAudio,
    health::{DamagedEvent, Dying, Health},
    movement::{Knockback, TargetEntity},
    pick_target::Team,
    setup_round::{Inert, StunTimer},
    shaders_lite::Flash,
    status::{CanAttack, CanBeTargeted},
    CombatState, GameFont, GameState,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_hit_observer);
        app.add_observer(on_stunned_observer);
        app.add_observer(on_block_attack_observer);

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
            )
                .chain()
                .run_if(in_state(CombatState::DuringCombat)),
        );

        // Cleanup and visual-finish systems run across all combat phases so
        // in-progress effects (ice traps, shield punches, attack animations)
        // can complete even after transitioning to PostCombat.
        app.add_systems(
            Update,
            (
                attack_cleanup_system,
                attack_cooldown_system,
                ice_vfx_cleanup_system,
                shield_scale_punch_system,
                floating_text_system,
            )
                .run_if(in_state(GameState::Combat)),
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
///
/// stun_chance and stun_duration default to 0.0 so existing attacks that don't
/// specify them are unaffected — they simply never roll for stun.
#[derive(Clone, Default)]
pub struct AttackEffect {
    pub damage: i32,
    pub knockback: f32,
    /// Probability of stunning the target on hit (0.0 = never, 1.0 = always).
    pub stun_chance: f32,
    /// How long the stun lasts in seconds. Only matters if stun_chance > 0.
    pub stun_duration: f32,
    /// If Some(dist), the attack splashes — damaging all enemies within `dist`
    /// of the primary target. Secondary hits have aoe_distance = None to prevent recursion.
    pub aoe_distance: Option<f32>,
}

/// Permanent config: how many seconds to wait between attacks.
/// Entities without this component attack as soon as they're in range (existing behavior).
/// This is useful for child entities like the frozen spear that would otherwise attack
/// every single frame (since they're always in range of something). The cooldown makes
/// their DPS controllable without changing animation speed or damage numbers.
#[derive(Component)]
pub struct TimeBetweenAttacks(pub f32);

/// Temporary timer: inserted after an attack finishes, removed when it expires.
/// While present, pick_attack_system skips this entity (via Without<AttackCooldown>).
/// This follows the same pattern as ActiveAttack — presence/absence of a component
/// acts as state. The entity is "cooling down" while this component exists.
#[derive(Component)]
pub struct AttackCooldown(pub Timer);

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

/// Fired when a target gets stunned. The on_stunned_observer reacts to this
/// by spawning ice impact VFX and playing the stun sound.
///
/// This is an Observer event (like OnHitEvent), not a Message — it fires
/// immediately via commands.trigger() and is handled inline by the observer.
#[derive(Event)]
pub struct StunnedEvent {
    pub entity: Entity,
}

/// Marker component for the ice impact VFX child entity.
/// Used by stun_timer_system to find and despawn VFX when the stun ends,
/// and by ice_vfx_cleanup_system to self-despawn when the animation finishes.
#[derive(Component)]
pub struct IceImpactVfx;

/// Marker for AoE ice trap VFX. Standalone entity (not a child) — spawned at
/// world position because the target might die or move after the hit.
#[derive(Component)]
pub struct IceTrapVfx;

/// Chance (0.0–1.0) that an incoming attack is completely blocked.
/// When a block succeeds, the attack deals no damage, no knockback, no stun —
/// the entire OnHitEvent is cancelled via early return.
///
/// This is a component on the *defender*, not the attacker. It's the defender's
/// passive ability: "I have a shield that might block your hit."
#[derive(Component)]
pub struct BlockChance(pub f32);

/// Marker component for the shield child entity (e.g., the iceberg sprite).
/// The on_block_attack_observer uses this to find the shield and flash it white.
#[derive(Component)]
pub struct Shield;

/// Temporary component that drives a "punch" scale animation on the shield.
/// Same pattern as Knockback and Flash — insert to start, system ticks, remove when done.
/// Stores the original scale so we restore it exactly rather than assuming a value.
#[derive(Component)]
pub struct ShieldScalePunch {
    pub original_scale: Vec3,
    pub timer: Timer,
}

/// Standalone floating text entity (not a child). Spawned at world position
/// so it doesn't follow the shield/slime — it just floats up and fades away.
#[derive(Component)]
pub struct FloatingText(pub Timer);

/// Fired when an attack is blocked. The on_block_attack_observer reacts to this
/// by flashing the shield white and playing a block sound.
#[derive(Event)]
pub struct BlockedAttackEvent {
    pub defender: Entity,
}

// ── Systems ─────────────────────────────────────────────────────────────────
/// Picks an attack for entities that are in range of their target but not already attacking.
/// Uses CanAttack (computed in status.rs) which excludes stunned, dying, merging, and
/// already-attacking entities.
fn pick_attack_system(
    attackers: Query<
        (Entity, &KnownAttacks, &GlobalTransform, &TargetEntity),
        With<CanAttack>,
    >,
    // GlobalTransform gives world-space position. This is critical for child entities
    // (like the frozen spear) whose local Transform is relative to their parent.
    // For top-level entities, GlobalTransform == Transform, so nothing changes for them.
    targets: Query<&GlobalTransform>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    for (entity, known_attacks, attacker_transform, target_entity) in attackers.iter() {
        let Ok(target_transform) = targets.get(target_entity.0) else {
            continue;
        };

        // Use translation() method on GlobalTransform (not .translation field like Transform)
        let distance = attacker_transform
            .translation()
            .distance(target_transform.translation());

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

/// Observer that reacts to OnHitEvent and applies damage + knockback + stun.
///
/// Observers are functions registered with app.add_observer(). They run immediately
/// when their event type is triggered via commands.trigger(). Unlike regular systems,
/// they don't need to be scheduled — they fire inline the moment the event happens.
///
/// The first parameter is On<OnHitEvent>, which wraps the event data.
/// You access the event fields by dereferencing: trigger.attacker, trigger.target, etc.
/// Additional parameters work just like regular system parameters (queries, commands, etc.).
///
/// ParamSet has three queries because they overlap on components:
///   p0 — read attacker + target world positions (GlobalTransform)
///   p1 — mutate target's Health/Transform/AnimationState
///   p2 — scan all entities with Health for AoE splash (reads GlobalTransform + Team)
///
/// Option<&BlockChance> in p1 lets us read BlockChance when it exists without
/// excluding entities that don't have it.
fn on_hit_observer(
    trigger: On<OnHitEvent>,
    mut params: ParamSet<(
        Query<&GlobalTransform>,
        Query<(
            &mut Health,
            &mut Transform,
            &mut AnimationState,
            Option<&BlockChance>,
        )>,
        Query<(Entity, &GlobalTransform, &Team), With<CanBeTargeted>>,
    )>,
    audio: Res<GameAudio>,
    mut commands: Commands,
) {
    // ── Phase 1: read positions from p0 ──
    // Scope the p0 borrow so it's released before we touch p1/p2.
    let (attacker_pos, target_pos) = {
        let p0 = params.p0();
        let Some(attacker_pos) = p0.get(trigger.attacker).ok().map(|t| t.translation()) else {
            return;
        };
        let Some(target_pos) = p0.get(trigger.target).ok().map(|t| t.translation()) else {
            return;
        };
        (attacker_pos, target_pos)
    };

    // ── Phase 2: apply primary hit via p1 ──
    if let Ok((mut health, transform, mut anim_state, block_chance)) =
        params.p1().get_mut(trigger.target)
    {
        // Blocked attacks cancel everything — including AoE splash.
        if let Some(block_chance) = block_chance {
            let mut rng = rand::thread_rng();
            if rng.gen::<f32>() < block_chance.0 {
                commands.trigger(BlockedAttackEvent {
                    defender: trigger.target,
                });
                return;
            }
        }

        // Apply damage
        if health.0 > 0 {
            health.0 -= trigger.effect.damage;
            commands.trigger(DamagedEvent {
                entity: trigger.target,
            });
        }

        // Apply knockback: insert a Knockback component
        if trigger.effect.knockback > 0.0 {
            let diff = transform.translation - attacker_pos;
            if diff.length() > 0.01 {
                let direction = diff.normalize();
                let target_pos = transform.translation
                    + Vec3::new(
                        direction.x * trigger.effect.knockback,
                        direction.y * trigger.effect.knockback,
                        0.0,
                    );
                commands.entity(trigger.target).insert(Knockback {
                    start_position: transform.translation,
                    target_position: target_pos,
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                });
            }
        }

        if trigger.effect.stun_chance > 0.0 && health.0 > 0 {
            let mut rng = rand::thread_rng();
            if rng.gen::<f32>() < trigger.effect.stun_chance {
                if let Ok(mut target_commands) = commands.get_entity(trigger.target) {
                    target_commands.insert((
                        Inert,
                        StunTimer(Timer::from_seconds(
                            trigger.effect.stun_duration,
                            TimerMode::Once,
                        )),
                    ));
                    target_commands.remove::<ActiveAttack>();
                }
                anim_state.finished = true;
                commands.trigger(StunnedEvent {
                    entity: trigger.target,
                });
            }
        }
    }

    // ── Phase 3: AoE splash via p2 ──
    // Only fires when the primary hit's effect has aoe_distance set.
    // Secondary OnHitEvents have aoe_distance = None, so this block is
    // skipped for them — preventing infinite recursion.
    if let Some(aoe_dist) = trigger.effect.aoe_distance {
        // Scope the p2 borrow — collect results into a Vec, then release p2
        // before triggering events (which need &mut commands).
        let splash_targets: Vec<Entity> = {
            let p2 = params.p2();
            let target_team = p2.get(trigger.target).ok().map(|(_, _, t)| *t);

            if let Some(team) = target_team {
                p2.iter()
                    .filter(|(e, pos, t)| {
                        *e != trigger.target
                            && **t == team
                            && pos.translation().distance(target_pos) <= aoe_dist
                    })
                    .map(|(e, _, _)| e)
                    .collect()
            } else {
                Vec::new()
            }
        };

        if !splash_targets.is_empty() {
            let mut splash_effect = trigger.effect.clone();
            splash_effect.aoe_distance = None;

            for splash_target in splash_targets {
                commands.trigger(OnHitEvent {
                    attacker: trigger.attacker,
                    target: splash_target,
                    effect: splash_effect.clone(),
                });
            }
        }

        // Spawn IceTrap VFX at the impact point regardless of whether
        // there were secondary targets — the AoE visual should always appear.
        commands
            .spawn((
                IceTrapVfx,
                AnimationType::IceTrapSpawn,
                Transform::from_xyz(target_pos.x, target_pos.y, 2.0).with_scale(Vec3::splat(3.0)),
            ))
            .with_child((
                IceImpactVfx,
                AnimationType::IceImpact,
                Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(1.0)),
            ));

        commands.spawn((
            AudioPlayer::new(audio.ice_trap.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.5)),
        ));
    }
}

fn attack_cleanup_system(
    mut commands: Commands,
    // Without<Dying> is critical here: if an entity is dying, the death system will
    // despawn it. If we also try to modify it, we race with the despawn and get
    // "Entity despawned" errors when our deferred command runs after the despawn.
    //
    // Without<Inert> prevents this system from interfering with stunned entities.
    // When an entity gets stunned, we set anim_state.finished = true to freeze it.
    // Without this filter, attack_cleanup would see finished == true and try to
    // return the entity to idle, fighting with the stun freeze.
    //
    // IdleAnimation tells us which animation to return to after the attack finishes.
    // Each entity carries its own IdleAnimation (set at spawn time), so this system
    // doesn't need to know about teams, merged status, or any other entity type.
    // Adding a new creature type? Just give it an IdleAnimation when you spawn it —
    // this system handles it automatically.
    mut query: Query<
        (
            Entity,
            &AnimationState,
            &mut AnimationType,
            &IdleAnimation,
            Option<&TimeBetweenAttacks>, // None for entities without a cooldown config
        ),
        (With<ActiveAttack>, Without<Dying>, Without<Inert>),
    >,
) {
    for (entity, anim_state, mut animation_type, idle_animation, time_between_attacks) in
        query.iter_mut()
    {
        if anim_state.finished {
            // Remove ActiveAttack so pick_attack_system can assign a new attack.
            // If TimeBetweenAttacks is present, also insert an AttackCooldown timer
            // so the entity waits before attacking again. Without this, the spear
            // (which is always in range) would attack every frame.
            let mut entity_commands = commands.entity(entity);
            entity_commands.remove::<ActiveAttack>();

            if let Some(TimeBetweenAttacks(duration)) = time_between_attacks {
                if *duration > 0.0 {
                    entity_commands.insert(AttackCooldown(Timer::from_seconds(
                        *duration,
                        TimerMode::Once,
                    )));
                }
            }

            // Return to this entity's idle animation. No need to match on team
            // or merged status — the entity already knows its own idle animation.
            *animation_type = idle_animation.0;
        }
    }
}

/// Ticks AttackCooldown timers and removes them when they expire.
/// This runs independently of the combat chain — it just counts down and removes.
/// Once the cooldown component is gone, pick_attack_system can assign a new attack
/// (because its Without<AttackCooldown> filter will match again).
fn attack_cooldown_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AttackCooldown)>,
) {
    for (entity, mut cooldown) in query.iter_mut() {
        cooldown.0.tick(time.delta());
        if cooldown.0.is_finished() {
            commands.entity(entity).remove::<AttackCooldown>();
        }
    }
}

/// Observer that reacts to StunnedEvent by spawning ice VFX and playing a sound.
///
/// This is separate from on_hit_observer for the same reason DamagedEvent is
/// separate from OnHitEvent: decoupling "what happened" from "how to show it."
/// The hit observer handles game logic (damage, stun state), while this observer
/// handles presentation (VFX, audio). This makes it easy to add/change visual
/// feedback without touching combat logic.
fn on_stunned_observer(trigger: On<StunnedEvent>, audio: Res<GameAudio>, mut commands: Commands) {
    // Play the stun sound. DESPAWN means the AudioPlayer entity will be
    // automatically cleaned up after the sound finishes playing.
    commands.spawn((
        AudioPlayer::new(audio.stun.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.5)),
    ));

    // Spawn ice impact VFX as a child of the stunned entity.
    // Being a child means it follows the parent's position automatically,
    // and if the parent is despawned (e.g., dies while stunned), the VFX
    // is despawned too — no orphaned effects.
    // z = 2.0 draws it on top of everything else on the entity.
    //
    // We use get_entity() instead of entity() as a safety net.
    // get_entity() returns Option<EntityCommands> — if the entity was
    // despawned between when StunnedEvent was triggered and when this
    // deferred command runs, we just skip it instead of panicking.
    // entity() would cause an "Entity despawned" error in that case.
    if let Ok(mut entity_commands) = commands.get_entity(trigger.entity) {
        entity_commands.with_child((
            IceImpactVfx,
            AnimationType::IceImpact,
            Transform::from_xyz(0.0, 0.0, 2.0).with_scale(Vec3::splat(3.0)),
        ));
    }
}

/// Observer that reacts to BlockedAttackEvent by flashing the shield white
/// and playing a block sound. This is purely presentation — the game logic
/// (cancelling the attack) already happened in on_hit_observer's early return.
///
/// To find the shield, we iterate the defender's Children and check which one
/// has the Shield marker component. This is a standard Bevy pattern for
/// "find a specific child of an entity" — you can't query parent-child
/// relationships directly, so you walk the Children list and check each one.
fn on_block_attack_observer(
    trigger: On<BlockedAttackEvent>,
    children_query: Query<&Children>,
    shield_query: Query<(&GlobalTransform, &Transform), With<Shield>>,
    audio: Res<GameAudio>,
    game_font: Res<GameFont>,
    mut commands: Commands,
) {
    commands.spawn((
        AudioPlayer::new(audio.block.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.5)),
    ));

    let Ok(children) = children_query.get(trigger.defender) else {
        return;
    };

    for child in children.iter() {
        if let Ok((global_transform, transform)) = shield_query.get(child) {
            if let Ok(mut shield_commands) = commands.get_entity(child) {
                shield_commands.insert((
                    Flash(Timer::from_seconds(0.2, TimerMode::Once)),
                    ShieldScalePunch {
                        original_scale: transform.scale,
                        timer: Timer::from_seconds(0.3, TimerMode::Once),
                    },
                ));
            }

            // Spawn floating "BLOCKED!" text at the shield's world position
            let pos = global_transform.translation();
            commands.spawn((
                FloatingText(Timer::from_seconds(1.3, TimerMode::Once)),
                Text2d::new("blocked!"),
                TextFont {
                    font: game_font.0.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(pos.x, pos.y + 20.0, 10.0),
            ));
        }
    }
}

/// Watches for ice VFX entities whose animation has finished and despawns them.
/// Covers both IceImpactVfx (stun effect, child entity) and IceTrapVfx (AoE splash,
/// standalone entity). Both use one-shot animations that set finished = true.
fn ice_vfx_cleanup_system(
    mut commands: Commands,
    impact_query: Query<(Entity, &AnimationState), With<IceImpactVfx>>,
    trap_query: Query<(Entity, &AnimationState), With<IceTrapVfx>>,
) {
    for (entity, anim_state) in impact_query.iter() {
        if anim_state.finished {
            // IceImpactVfx is a child entity — parent could be cascade-despawned
            // by when_finishes_dying_system in the same command batch.
            if let Ok(mut cmds) = commands.get_entity(entity) {
                cmds.despawn();
            }
        }
    }
    for (entity, anim_state) in trap_query.iter() {
        if anim_state.finished {
            if let Ok(mut cmds) = commands.get_entity(entity) {
                cmds.despawn();
            }
        }
    }
}

/// Drives the shield's "punch" scale animation: grows to 1.5x then settles back.
/// Uses a sine curve for a snappy grow-then-shrink feel. Snaps to original_scale
/// on finish and removes the component so the shield returns to normal.
fn shield_scale_punch_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut ShieldScalePunch)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut punch) in query.iter_mut() {
        punch.timer.tick(time.delta());
        let t = punch.timer.fraction();

        if punch.timer.is_finished() {
            transform.scale = punch.original_scale;
            // Shield is a child entity — if the parent dies mid-punch, the shield
            // gets recursively despawned. get_entity() avoids panicking on a dead entity.
            if let Ok(mut cmds) = commands.get_entity(entity) {
                cmds.remove::<ShieldScalePunch>();
            }
        } else {
            // sin(π * t) gives 0→1→0 curve: peaks at t=0.5, returns to 0 at t=1.0
            let bulge = (std::f32::consts::PI * t).sin() * 0.5;
            transform.scale = punch.original_scale * (1.0 + bulge);
        }
    }
}

/// Floats text upward and fades it out, then despawns the entity.
fn floating_text_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut FloatingText)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut text_color, mut floating) in query.iter_mut() {
        floating.0.tick(time.delta());

        transform.translation.y += 50.0 * time.delta_secs();
        let alpha = 1.0 - floating.0.fraction();
        text_color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);

        if floating.0.is_finished() {
            if let Ok(mut cmds) = commands.get_entity(entity) {
                cmds.despawn();
            }
        }
    }
}
