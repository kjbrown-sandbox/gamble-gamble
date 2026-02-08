use bevy::prelude::*;
// use crate::components::{AnimationState, AnimationType, DamageEvent, Dying, Health, Soldier};
// use crate::resources::SpriteSheets;

/* Animation needs to be handle a few things:
- Updating the sprite frames
- Allow the ability to loop an animation
- Probably needs a way to know when it's done playing
- Perhaps fire an event when done animating?

*/

#[derive(Component)]
pub struct AnimationState {
    //  pub animation_type: AnimationType,
    pub current_frame: usize,
    pub frame_timer: f32,
    pub frame_duration: f32, // milliseconds per frame
    total_frames: usize,
    pub looping: bool,  // Whether animation should loop
    pub finished: bool, // True when non-looping animation completes
}

impl AnimationState {
    pub fn new(
        frame_duration: f32,
        spritesheet_layout: &TextureAtlasLayout,
        looping: bool,
    ) -> Self {
        AnimationState {
            current_frame: 0,
            frame_timer: 0.0,
            frame_duration,
            looping,
            finished: false,
            total_frames: spritesheet_layout.len(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.finished {
            return;
        }

        self.frame_timer += delta_time;

        // Check if it's time to advance to the next frame
        if self.frame_timer >= self.frame_duration {
            self.frame_timer -= self.frame_duration;
            self.current_frame += 1;

            if self.current_frame >= self.total_frames {
                if self.looping {
                    self.current_frame = 0; // Loop back to first frame
                } else {
                    self.current_frame = self.total_frames - 1; // Stay on last frame
                    self.finished = true;
                }
            }
        }
    }
}
