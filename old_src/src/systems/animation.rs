// systems/animation.rs - Animation system
// Handles sprite animation updates and animation triggers

use bevy::prelude::*;
use crate::components::{AnimationState, AnimationType, DamageEvent, Dying, Health, Soldier};
use crate::resources::SpriteSheets;

/// Animation system - updates sprite frames based on animation state
/// Runs every frame to advance animations
pub fn animation_system(
    mut query: Query<(&mut AnimationState, &mut Sprite)>,
    time: Res<Time>,
) {
    for (mut anim_state, mut sprite) in query.iter_mut() {
        // Update animation timer and check if frame changed
        anim_state.update(time.delta_secs());

        // Update sprite atlas index to current frame
        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = anim_state.current_frame;
        }
    }
}

/// System to change animation when AnimationType changes
/// This handles switching sprite sheets and frame counts when animation type changes
pub fn animation_switcher_system(
    mut query: Query<(&mut AnimationState, &mut Sprite), Changed<AnimationState>>,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (anim_state, mut sprite) in query.iter_mut() {
        // When animation type changes, update sprite sheet and layout
        match anim_state.animation_type {
            AnimationType::JumpIdle => {
                sprite.image = sprite_sheets.slime_jump_idle.clone();
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.layout = sprite_sheets.jump_idle_layout.clone();
                }
            }
            AnimationType::Attack => {
                sprite.image = sprite_sheets.slime_attack.clone();
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.layout = sprite_sheets.attack_layout.clone();
                }
            }
            AnimationType::MoveSmallJump => {
                sprite.image = sprite_sheets.slime_move_small_jump.clone();
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.layout = sprite_sheets.move_small_jump_layout.clone();
                }
            }
            AnimationType::Hurt => {
                sprite.image = sprite_sheets.slime_hurt.clone();
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.layout = sprite_sheets.hurt_layout.clone();
                }
            }
            AnimationType::Death => {
                sprite.image = sprite_sheets.slime_death.clone();
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.layout = sprite_sheets.death_layout.clone();
                }
            }
        }
    }
}

/// Observer that triggers hurt animation when a soldier takes damage.
///
/// OBSERVERS IN BEVY:
/// Observers react to triggered events (via commands.trigger()).
/// Unlike regular systems that run every frame, observers only run
/// when their event is triggered. This is perfect for one-shot reactions
/// like playing a hurt animation.
///
/// The On<T> wrapper:
/// - Contains the triggered event data
/// - Dereferences to &T via `*trigger` or direct field access
/// - Registered with `.add_observer(function)` on the App
///
/// We check if the entity is not dying (health > 0) because dead entities
/// should play the death animation, not hurt.
pub fn on_damage_animation(
    trigger: On<DamageEvent>,
    mut query: Query<(&mut AnimationState, &Health), Without<Dying>>,
) {
    // Only play hurt animation if entity is still alive and not dying
    if let Ok((mut anim_state, health)) = query.get_mut(trigger.target) {
        if health.current > 0 {
            anim_state.change_to(AnimationType::Hurt);
        }
    }
}

/// System to return soldiers to idle animation after non-looping animations finish.
///
/// This checks for finished non-looping animations (Attack, MoveSmallJump, Hurt)
/// and returns them to the JumpIdle animation. Death animations are excluded
/// because dying entities should stay on their final death frame.
pub fn animation_finished_system(
    mut query: Query<&mut AnimationState, (With<Soldier>, Without<Dying>)>,
) {
    for mut anim_state in query.iter_mut() {
        // Only handle finished non-looping animations that aren't Death
        if anim_state.finished && anim_state.animation_type != AnimationType::Death {
            anim_state.change_to(AnimationType::JumpIdle);
        }
    }
}
