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

/// Soldier component - a marker component that identifies this entity as a soldier.
/// Marker components have no data - they just "tag" entities for identification.
/// Useful for queries like "give me all soldiers" without needing to list all their components.
#[derive(Component)]
pub struct Soldier;

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
/// This is the INSTANCE data - it changes during gameplay.
/// Each soldier has their own AttackInstance entities as children,
/// each tracking its own cooldown state.
///
/// WHY CHILD ENTITIES?
/// - Each attack can have its own components (cooldown, buffs, etc.)
/// - Systems can query attacks directly: Query<&AttackInstance>
/// - Easy to add/remove attacks from soldiers at runtime
/// - Follows ECS composition pattern
#[derive(Component)]
pub struct AttackInstance {
    /// Which attack definition this instance uses
    pub attack_id: AttackId,

    /// Current cooldown timer (counts down to 0)
    pub current_cooldown: f32,
}

impl AttackInstance {
    pub fn new(attack_id: AttackId) -> Self {
        Self {
            attack_id,
            current_cooldown: 0.0, // Ready to use immediately
        }
    }

    /// Check if this attack is ready to use
    pub fn is_ready(&self) -> bool {
        self.current_cooldown <= 0.0
    }

    /// Tick down the cooldown timer
    pub fn tick(&mut self, delta: f32) {
        if self.current_cooldown > 0.0 {
            self.current_cooldown -= delta;
        }
    }

    /// Start the cooldown (call after using the attack)
    pub fn start_cooldown(&mut self, duration: f32) {
        self.current_cooldown = duration;
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

