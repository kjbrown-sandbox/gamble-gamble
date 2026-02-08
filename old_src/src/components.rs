// components.rs - All component definitions for our game
// Components are the data containers in our ECS architecture.
// Each entity is made up of multiple components.

use bevy::prelude::*;

/// Health component - represents how much health an entity has.
/// Every soldier will have this component.
/// This is per-entity state: each soldier has its own Health value.
#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(amount: i32) -> Self {
        Health {
            current: amount,
            max: amount,
        }
    }

    /// Take damage. Returns true if the entity died.
    pub fn take_damage(&mut self, damage: i32) -> bool {
        self.current = (self.current - damage).max(0);
        self.current <= 0
    }
}

/// Team component - identifies which team/army this soldier belongs to.
/// Used to distinguish player soldiers from enemy soldiers.
#[derive(Component, PartialEq, Eq)]
pub struct Team {
    pub is_player: bool,
}

/// AttackCooldown component - tracks when this soldier can next attack.
/// Without this, soldiers would attack every single frame (60+ times per second!).
/// This ensures they attack roughly once per second.
#[derive(Component)]
pub struct AttackCooldown {
    /// How many seconds until this soldier can attack again
    pub timer: f32,
    /// The cooldown duration in seconds
    pub cooldown_duration: f32,
}

impl AttackCooldown {
    pub fn new(duration: f32) -> Self {
        AttackCooldown {
            timer: 0.0, // Ready to attack immediately
            cooldown_duration: duration,
        }
    }

    /// Update the timer. Returns true if the cooldown is finished.
    pub fn update(&mut self, delta_time: f32) -> bool {
        if self.timer > 0.0 {
            self.timer -= delta_time;
            false
        } else {
            true
        }
    }

    /// Reset the cooldown timer
    pub fn reset(&mut self) {
        self.timer = self.cooldown_duration;
    }
}

/// Soldier component - identifies this entity as a soldier and stores their available attacks.
///
/// WHY STORE ATTACKS HERE?
/// Previously, attacks were pre-spawned as child entities. But we want a soldier to only
/// be able to have ONE attack "in flight" at a time. Now:
/// - available_attacks: List of AttackIds this soldier CAN use (just data, not entities)
/// - When attacking: we spawn a temporary AttackInstance child
/// - While that child exists: soldier cannot attack (it's "busy")
/// - When cooldown finishes: the AttackInstance is despawned
/// - No children = soldier can pick and use another attack
#[derive(Component)]
pub struct Soldier {
    /// The attacks this soldier can choose from when ready to attack.
    /// These are just IDs referencing the AttackDatabase - no entities spawned until used.
    pub available_attacks: Vec<AttackId>,
}

/// HealthDisplay component - marks a UI text element that displays a soldier's health.
/// This is used to link a UI element to a specific soldier entity.
#[derive(Component)]
pub struct HealthDisplay {
    /// The entity of the soldier whose health this displays
    pub soldier_entity: Entity,
    /// Which team this display is for (for labeling)
    pub is_player: bool,
}

/// GameOverText component - marks the UI text element that displays the game over message.
#[derive(Component)]
pub struct GameOverText;

/// Animation components - track animation state for sprite-based entities
/// 
/// AnimationType enum - defines which animation to play
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    JumpIdle,      // Idle/waiting animation
    Attack,        // Standard attack animation
    MoveSmallJump, // Small jump attack animation
    Hurt,          // Taking damage animation
    Death,         // Death animation
}

/// AnimationState - tracks current animation frame and timing
#[derive(Component)]
pub struct AnimationState {
    pub animation_type: AnimationType,
    pub current_frame: usize,
    pub frame_timer: f32,
    pub frame_duration: f32, // Seconds per frame
    pub total_frames: usize,
    pub looping: bool,       // Whether animation should loop
    pub finished: bool,      // True when non-looping animation completes
}

impl AnimationState {
    pub fn new(animation_type: AnimationType, total_frames: usize, frame_duration: f32, looping: bool) -> Self {
        AnimationState {
            animation_type,
            current_frame: 0,
            frame_timer: 0.0,
            frame_duration,
            total_frames,
            looping,
            finished: false,
        }
    }

    /// Update animation timer and advance frame if needed
    /// Returns true if a non-looping animation just finished this frame
    pub fn update(&mut self, delta_time: f32) -> bool {
        if self.finished {
            return false;
        }

        self.frame_timer += delta_time;

        if self.frame_timer >= self.frame_duration {
            self.frame_timer = 0.0;
            self.current_frame += 1;

            if self.current_frame >= self.total_frames {
                if self.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.total_frames - 1;
                    self.finished = true;
                    return true; // Animation just finished
                }
            }
        }
        false
    }

    /// Change to a different animation type, resetting all state.
    ///
    /// ANIMATION FRAME COUNTS (from sprite sheets):
    /// - JumpIdle: 6 frames, looping
    /// - Attack: 5 frames, non-looping
    /// - MoveSmallJump: 6 frames, non-looping
    /// - Hurt: 3 frames, non-looping
    /// - Death: 6 frames, non-looping
    pub fn change_to(&mut self, animation_type: AnimationType) {
        // Don't restart if already playing this animation
        if self.animation_type == animation_type && !self.finished {
            return;
        }

        self.animation_type = animation_type;
        self.current_frame = 0;
        self.frame_timer = 0.0;
        self.finished = false;

        // Set frame count and looping based on animation type
        match animation_type {
            AnimationType::JumpIdle => {
                self.total_frames = 6;
                self.frame_duration = 0.1;
                self.looping = true;
            }
            AnimationType::Attack => {
                self.total_frames = 5;
                self.frame_duration = 0.08; // Faster attack animation
                self.looping = false;
            }
            AnimationType::MoveSmallJump => {
                self.total_frames = 6;
                self.frame_duration = 0.08;
                self.looping = false;
            }
            AnimationType::Hurt => {
                self.total_frames = 3;
                self.frame_duration = 0.1;
                self.looping = false;
            }
            AnimationType::Death => {
                self.total_frames = 6;
                self.frame_duration = 0.15; // Slower death for dramatic effect
                self.looping = false;
            }
        }
    }
}

/// Dying component - marks entities that are playing death animation.
/// Prevents them from being despawned until animation completes.
/// Added when health <= 0, removed after death animation finishes.
#[derive(Component)]
pub struct Dying;

// =============================================================================
// ATTACK SYSTEM
// =============================================================================
// This section defines a data-driven attack system where:
// - Attack DEFINITIONS live in a Resource (like a database)
// - Attack INSTANCES are child entities of soldiers (with runtime state)
// - EFFECTS are enums that describe what happens on hit/miss
//
// This separation lets us:
// - Define attacks once, reuse across many soldiers
// - Give each soldier their own cooldown state per attack
// - Easily add new effects without changing core combat code

/// AttackId is a "newtype" wrapper around usize.
///
/// WHY USE A NEWTYPE?
/// In Rust, a newtype is a tuple struct with one field: `struct Foo(Bar);`
/// Benefits:
/// 1. Type safety: Can't accidentally pass a random usize where AttackId is expected
/// 2. Semantics: The type name documents what this number means
/// 3. Methods: Can add methods specific to this ID type
///
/// The derives:
/// - Clone, Copy: Can be copied without .clone() (it's just a number)
/// - Debug: Can print it with {:?}
/// - PartialEq, Eq: Can compare with ==
/// - Hash: Can use as HashMap key
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AttackId(pub usize);

/// Effect represents something that happens as a result of an attack.
///
/// This is an enum with variants that hold different data.
/// Using an enum lets us:
/// - Have a fixed set of effect types (compiler catches typos)
/// - Store different data per effect type
/// - Match exhaustively (compiler warns if we miss a case)
///
/// Clone is needed because we'll copy effects when applying them.
#[derive(Clone, Debug)]
pub enum Effect {
    /// Deal damage to the attack's target
    DamageTarget(i32),

    /// Deal damage to the attacker (self-harm, like a risky attack)
    DamageSelf(i32),

    /// Heal the attacker
    HealSelf(i32),

    // FUTURE EFFECTS (not implemented yet, but showing the pattern):
    // ApplyBuff { stat: Stat, amount: f32, duration: f32 },
    // ApplyDebuff { target: EffectTarget, stat: Stat, amount: f32, duration: f32 },
    // Stun { duration: f32 },
}

/// Groups effects by when they trigger.
///
/// This lets attacks have different outcomes based on hit/miss:
/// - on_success: Effects applied when the attack hits
/// - on_fail: Effects applied when the attack misses
/// - on_use: Effects applied regardless of hit/miss (always happens)
#[derive(Clone, Debug, Default)]
pub struct AttackEffects {
    pub on_success: Vec<Effect>,
    pub on_fail: Vec<Effect>,
    pub on_use: Vec<Effect>,
}

/// The static definition of an attack type.
///
/// This is DEFINITION data - it doesn't change during gameplay.
/// Think of it like a template or blueprint.
/// The actual runtime state (cooldowns) lives in AttackInstance.
#[derive(Clone, Debug)]
pub struct AttackDefinition {
    /// Human-readable name for UI/debugging
    pub name: String,

    /// Probability of hitting (0.0 = never, 1.0 = always)
    /// This affects which effects trigger (on_success vs on_fail)
    pub hit_chance: f32,

    /// Seconds between uses of this attack
    pub cooldown: f32,

    /// What happens when this attack is used
    pub effects: AttackEffects,
}

/// Resource containing all attack definitions.
///
/// This acts as a "database" of attacks that soldiers can reference.
/// Using a Resource means:
/// - One copy exists globally (not per-entity)
/// - Any system can read it with Res<AttackDatabase>
/// - Soldiers just store AttackId references, not full definitions
///
/// Vec is used for simplicity; HashMap<AttackId, AttackDefinition>
/// would be better for large numbers of attacks.
#[derive(Resource, Default)]
pub struct AttackDatabase {
    pub attacks: Vec<AttackDefinition>,
}

impl AttackDatabase {
    /// Add an attack and return its ID
    pub fn add(&mut self, attack: AttackDefinition) -> AttackId {
        let id = AttackId(self.attacks.len());
        self.attacks.push(attack);
        id
    }

    /// Look up an attack by ID
    pub fn get(&self, id: AttackId) -> Option<&AttackDefinition> {
        self.attacks.get(id.0)
    }
}

/// Component for attack instances attached to soldiers.
///
/// This is a TEMPORARY child entity that exists while an attack is "in progress."
/// It's spawned when a soldier uses an attack, and despawned when the cooldown finishes.
///
/// WHY SPAWN/DESPAWN INSTEAD OF PRE-SPAWNING?
/// We want a soldier to only have ONE attack active at a time. By making the
/// AttackInstance a temporary child:
/// - Presence of child = soldier is busy (can't attack)
/// - No children = soldier is ready to pick a new attack
/// - Simple check: if children.is_empty() { can_attack() }
///
/// This is cleaner than tracking a separate "is_attacking" boolean or cooldown
/// on the Soldier component itself.
#[derive(Component)]
pub struct AttackInstance {
    /// Which attack definition this instance uses (for debugging/effects)
    pub attack_id: AttackId,

    /// Current cooldown timer (counts down to 0, then this entity is despawned)
    pub cooldown_remaining: f32,
}

impl AttackInstance {
    /// Create a new attack instance with cooldown already started.
    /// This is spawned when the soldier uses an attack.
    pub fn new(attack_id: AttackId, cooldown: f32) -> Self {
        Self {
            attack_id,
            cooldown_remaining: cooldown,
        }
    }

    /// Check if this attack's cooldown is finished (ready to be despawned)
    pub fn is_finished(&self) -> bool {
        self.cooldown_remaining <= 0.0
    }

    /// Tick down the cooldown timer
    pub fn tick(&mut self, delta: f32) {
        if self.cooldown_remaining > 0.0 {
            self.cooldown_remaining -= delta;
        }
    }
}

// =============================================================================
// EVENTS
// =============================================================================
// Events are Bevy's way of communicating between systems without tight coupling.
// One system sends an event, and any number of other systems can listen for it.
// This is the "Observer" or "Pub/Sub" pattern built into Bevy's ECS.
//
// Events are stored in a ring buffer and cleared every frame (by default).
// Systems read events using EventReader<T> and send using EventWriter<T>.

/// DamageEvent - fired whenever an entity takes damage.
///
/// This event allows other systems (like audio, particles, UI) to react to damage
/// without the combat system needing to know about them. This is called "loose coupling"
/// and makes the code more modular and testable.
///
/// The #[derive(Event)] macro is required for any struct used as an event.
#[derive(Event)]
pub struct DamageEvent {
    /// The entity that took damage (useful for visual effects at their position)
    pub target: Entity,
    /// How much damage was dealt (useful for scaling effects or showing damage numbers)
    pub amount: i32,
}

// =============================================================================
// DAMAGE POPUP COMPONENT
// =============================================================================
// Damage popups are floating numbers that appear when entities take damage.
// They use world-space text (Text2d) rather than UI text, so they exist
// in the game world and can be positioned relative to the damaged entity.

/// DamagePopup - component for floating damage numbers.
///
/// When an entity takes damage, we spawn a Text2d entity with this component.
/// The popup floats upward and fades out over time, then despawns.
///
/// KEY CONCEPTS:
/// - Uses Timer for lifecycle management (how long before despawn)
/// - Uses Timer::fraction() for animation progress (returns 0.0 → 1.0)
/// - Uses lerp (linear interpolation) to smoothly animate position and opacity
///
/// WHAT IS LERP?
/// Lerp stands for "linear interpolation". Given two values and a percentage,
/// it returns the value at that percentage between them.
///   lerp(start, end, 0.0) = start
///   lerp(start, end, 0.5) = halfway between start and end
///   lerp(start, end, 1.0) = end
///
/// Combined with Timer::fraction(), this creates smooth animations:
///   y = start_y + fraction * float_distance
/// As fraction goes 0.0→1.0, y smoothly increases by float_distance.
#[derive(Component)]
pub struct DamagePopup {
    /// Timer controlling popup lifetime. When finished, popup despawns.
    pub timer: Timer,

    /// Starting Y position (captured when spawned at damaged entity's location).
    pub start_y: f32,

    /// How far up the popup floats over its lifetime (in world units).
    pub float_distance: f32,
}

// =============================================================================
// MENU UI COMPONENTS
// =============================================================================
// These marker components identify UI elements so we can query and despawn them.
// Using marker components (structs with no fields) is a common Bevy pattern.

/// Marker component for the main menu UI container.
/// Used to despawn all menu UI when exiting the Menu state.
#[derive(Component)]
pub struct MenuUI;

/// Marker component for the "Fight!" button.
/// Used to detect clicks and transition to Battle state.
#[derive(Component)]
pub struct FightButton;

/// Marker component for the game over UI container.
/// Used to despawn game over UI when returning to menu.
#[derive(Component)]
pub struct GameOverUI;

/// Marker component for the "Play Again" button.
/// Used to detect clicks and transition back to Menu state.
#[derive(Component)]
pub struct PlayAgainButton;

