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

