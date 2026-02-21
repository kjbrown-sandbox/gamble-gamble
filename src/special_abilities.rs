use bevy::prelude::*;
use rand::Rng;

use crate::{
    animation::AnimationType,
    combat::{ActiveAttack, Attack, AttackEffect, KnownAttacks},
    health::{DeathAnimation, Dying, Health},
    movement::{Speed, TargetEntity},
    pick_target::{PickTargetStrategy, Team},
    setup_round::Inert,
};

pub struct SpecialAbilitiesPlugin;

impl Plugin for SpecialAbilitiesPlugin {
    fn build(&self, app: &mut App) {
        // Insert the timer resource that gates how often we check for merges.
        // 0.5s means we evaluate merge chances twice per second — frequent enough
        // to feel responsive, but not so frequent that merges happen instantly.
        app.insert_resource(MergeCheckTimer(Timer::from_seconds(
            0.5,
            TimerMode::Repeating,
        )));

        // Chain the 4 merge systems so they run in a guaranteed order each frame.
        // This is the same pattern combat.rs uses: each system builds on the state
        // that the previous one set up. Without .chain(), Bevy could run them in
        // any order (or in parallel), which would cause bugs — e.g. execute_merge
        // could try to despawn entities before merge_walk has moved them.
        app.add_systems(
            Update,
            (
                check_merge_system,
                merge_walk_system,
                execute_merge_system,
                cancel_merge_system,
            )
                .chain(),
        );
    }
}

// ── Components ──────────────────────────────────────────────────────────────

/// Temporary marker component — both merging slimes get this, pointing at each other.
/// This follows the same pattern as `ActiveAttack`: a temporary component whose
/// presence means "this entity is busy doing something" and whose absence means
/// "this entity is available for normal behavior."
///
/// `meeting_point` is stored on each entity rather than in a shared resource because
/// in ECS, data lives on the entities it describes. There's no central "MergeController"
/// object orchestrating things — each entity carries its own state, and systems observe
/// that state and react. This is a key difference from OOP, where you'd likely have a
/// MergeManager class holding references to both slimes.
#[derive(Component)]
pub struct Merging {
    /// The other slime participating in this merge.
    pub partner: Entity,
    /// The midpoint where both slimes will walk to before merging.
    /// Computed once at merge initiation so both slimes converge on a fixed
    /// point instead of chasing a moving target.
    pub meeting_point: Vec3,
}

/// Permanent marker — prevents a merged slime from merging again.
/// This is a "tag" component: it has no data, just presence/absence.
/// We query `Without<MergedSlime>` to filter out entities that have already merged.
#[derive(Component)]
pub struct MergedSlime;

/// Resource — repeating timer that gates how often we roll the dice for merges.
/// Without this, we'd check every single frame (60+ times per second), which would
/// make merges happen almost instantly and waste CPU on the distance checks.
#[derive(Resource)]
pub struct MergeCheckTimer(pub Timer);

// ── Systems ─────────────────────────────────────────────────────────────────

/// System 1: Periodically checks if any same-team slime pairs should merge.
///
/// Every 0.5s (gated by MergeCheckTimer), iterates all eligible slimes and checks
/// every same-team pair. If two slimes are within 100 units, rolls a 0.5% chance
/// for them to start merging.
///
/// "Eligible" means: not inert, not dying, not attacking, not already merging,
/// and not a MergedSlime. All these exclusions use query filters — this is how
/// ECS handles complex conditions. Instead of `if (!entity.isInert && !entity.isDying ...)`
/// like in OOP, we declare the filters in the query signature and Bevy automatically
/// skips non-matching entities.
///
/// The probability math: with 5 same-team slimes, there are C(5,2) = 10 pairs.
/// Maybe 3-5 are within range at any time. Over a ~20s fight with checks every 0.5s,
/// that's ~40 checks × ~4 qualifying pairs × 0.005 = ~0.8 expected merges per game.
fn check_merge_system(
    mut timer: ResMut<MergeCheckTimer>,
    time: Res<Time>,
    // This query has a LOT of filters. Each `Without<X>` removes entities that have
    // component X. This is more efficient than checking at runtime — Bevy's query
    // system uses archetype-based filtering, so entities that don't match are never
    // even iterated over.
    eligible: Query<
        (Entity, &Team, &Transform),
        (
            Without<Inert>,
            Without<Dying>,
            Without<ActiveAttack>,
            Without<Merging>,
            Without<MergedSlime>,
            With<Health>,
        ),
    >,
    mut commands: Commands,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let mut rng = rand::thread_rng();

    // Collect into a Vec so we can do nested iteration (comparing every pair).
    // We can't nest .iter() calls on the same query because Rust's borrow checker
    // won't allow two simultaneous immutable borrows of the query iterator state.
    // Collecting to a Vec is the standard ECS workaround for pair-wise comparisons.
    let candidates: Vec<(Entity, &Team, Vec3)> = eligible
        .iter()
        .map(|(e, team, t)| (e, team, t.translation))
        .collect();

    // Track which entities we've already paired this tick, so one slime
    // doesn't get matched with multiple partners simultaneously.
    let mut already_paired: Vec<Entity> = Vec::new();

    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            let (entity_a, team_a, pos_a) = candidates[i];
            let (entity_b, team_b, pos_b) = candidates[j];

            // Same-team only — cross-team merging doesn't make gameplay sense
            if team_a != team_b {
                continue;
            }

            // Skip if either entity was already paired this tick
            if already_paired.contains(&entity_a) || already_paired.contains(&entity_b) {
                continue;
            }

            let distance = pos_a.distance(pos_b);
            if distance > 100.0 {
                continue;
            }

            // Roll the dice — 0.5% chance per qualifying pair per tick
            if rng.gen::<f32>() > 0.005 {
                continue;
            }

            // They're merging! Compute the meeting point as the midpoint between them.
            // Using a fixed point (computed once) means both slimes walk toward the same
            // spot. If we used a moving target (like each other's current position),
            // they'd overshoot and oscillate.
            let meeting_point = (pos_a + pos_b) / 2.0;

            // Insert Merging on both entities. This marks them as "busy" so other
            // systems (pick_target, pick_attack) will skip them.
            // Also remove TargetEntity so they stop chasing enemies.
            commands.entity(entity_a).insert(Merging {
                partner: entity_b,
                meeting_point,
            });
            commands.entity(entity_a).remove::<TargetEntity>();

            commands.entity(entity_b).insert(Merging {
                partner: entity_a,
                meeting_point,
            });
            commands.entity(entity_b).remove::<TargetEntity>();

            already_paired.push(entity_a);
            already_paired.push(entity_b);
        }
    }
}

/// System 2: Moves merging slimes toward their meeting point.
///
/// This is simpler than the normal movement system because we're walking toward
/// a fixed point (stored in the Merging component) rather than chasing another entity.
/// We reuse the entity's Speed component so merged-to-be slimes walk at their normal pace.
fn merge_walk_system(
    mut query: Query<(&Merging, &mut Transform, &Speed), Without<Dying>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for (merging, mut transform, speed) in query.iter_mut() {
        let diff = merging.meeting_point - transform.translation;

        // Stop moving when close enough to the meeting point.
        // 15 units is close enough that they'll overlap visually.
        if diff.length() < 15.0 {
            continue;
        }

        // Move toward the meeting point at the entity's normal speed.
        // normalize() gives us a unit vector pointing toward the meeting point,
        // then we scale by speed and delta time for frame-rate-independent movement.
        let direction = diff.normalize();
        transform.translation += direction * speed.0 * delta;
    }
}

/// System 3: When both partners are close enough, despawn them and spawn the merged slime.
///
/// This checks if the two merging partners are within 40 units of each other.
/// We use a slightly larger threshold than merge_walk's 15-unit "stop moving" threshold
/// to ensure both slimes have time to arrive before we trigger the merge.
///
/// The merged slime is a brand new entity — the original two are despawned entirely.
/// This is the ECS way: rather than "transforming" an existing entity, we destroy
/// the old ones and create a new one with the properties we want. It's cleaner than
/// trying to modify one entity and delete the other, because the merged slime has
/// fundamentally different stats.
fn execute_merge_system(
    query: Query<(Entity, &Merging, &Transform, &Team), Without<Dying>>,
    mut commands: Commands,
) {
    // Track which entities we've already processed this frame to avoid
    // trying to despawn the same entity twice (both partners would match).
    let mut already_merged: Vec<Entity> = Vec::new();

    for (entity, merging, transform, team) in query.iter() {
        if already_merged.contains(&entity) {
            continue;
        }

        // Check if our partner still exists and has Merging
        let Ok((partner_entity, _, partner_transform, _)) = query.get(merging.partner) else {
            continue;
        };

        // Are both partners close enough to each other?
        let distance = transform
            .translation
            .distance(partner_transform.translation);
        if distance > 40.0 {
            continue;
        }

        // Merge! Despawn both originals and spawn the new merged slime.
        let midpoint = (transform.translation + partner_transform.translation) / 2.0;

        // Mark both as processed so we don't try to merge them again
        already_merged.push(entity);
        already_merged.push(partner_entity);

        commands.entity(entity).despawn();
        commands.entity(partner_entity).despawn();

        // The merged slime's scale depends on team — player slimes are 1x normally
        // (so 2x merged), enemy slimes are 2x normally (so 4x merged).
        // let merged_scale = match team {
        //     Team::Player => 2.0,
        //     Team::Enemy => 4.0,
        // };
        let merged_scale = 2.0;

        // Spawn the merged slime with boosted stats.
        // Uses "BigSlime" animation variants — these use the same sprite sheets as
        // normal slimes but with 0.3s frame duration (3x slower). This makes the
        // merged slime look heavy and lumbering. The slower attack animation also
        // inherently makes the attack cycle ~3x longer.
        //
        // No SpriteModification (the spawn scale animation targets 1.0, which would
        // fight our 2x scale). No Inert either — it spawns ready to fight.
        commands.spawn((
            AnimationType::BigSlimeJumpIdle,
            Transform::from_translation(midpoint).with_scale(Vec3::splat(merged_scale)),
            *team,
            PickTargetStrategy::Close,
            Sprite {
                flip_y: *team == Team::Enemy,
                ..default()
            },
            DeathAnimation(AnimationType::BigSlimeDeath),
            // 4x health of a normal slime (normal = 10)
            Health(40),
            // Same movement speed — the "lumbering" look comes from the slower animation
            Speed(125.0),
            // 4x damage of a normal slime (normal = 2), same range
            KnownAttacks(vec![Attack {
                animation: AnimationType::BigSlimeAttack,
                hit_frame: 3,
                on_hit_effect: AttackEffect {
                    damage: 8,
                    knockback: 0.0,
                },
                range: 60.0,
            }]),
            // Permanent marker — prevents this slime from merging again
            MergedSlime,
        ));
    }
}

/// System 4: Cancels a merge if the partner dies or gets despawned.
///
/// If a merging slime's partner is no longer alive (despawned or dying), we remove
/// the Merging component so the survivor goes back to normal combat behavior.
/// Without this, a slime whose partner died mid-merge would walk to the meeting point
/// and then stand there forever, never re-entering combat.
fn cancel_merge_system(
    query: Query<(Entity, &Merging)>,
    // A separate query to check if the partner still exists and isn't dying.
    // We use a separate query (instead of checking within the first) because the
    // partner might not have a Merging component anymore if it was already cleaned up.
    alive_check: Query<Entity, Without<Dying>>,
    mut commands: Commands,
) {
    for (entity, merging) in query.iter() {
        // If the partner doesn't exist at all (despawned) or is dying, cancel the merge.
        if alive_check.get(merging.partner).is_err() {
            commands.entity(entity).remove::<Merging>();
        }
    }
}
