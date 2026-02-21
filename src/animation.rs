use bevy::{prelude::*, render::render_resource::Texture};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_sprite_sheets)
            .add_systems(Update, (switch_animation_system, animation_system).chain());
    }
}

/// Stores which animation an entity should return to when "at rest"
#[derive(Component, Copy, Clone)]
pub struct IdleAnimation(pub AnimationType);

#[derive(Component, Copy, Clone)]
pub struct VictoryAnimation(pub AnimationType);

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
#[require(AnimationState, Sprite)]
pub enum AnimationType {
    SlimeJumpIdle,
    SlimeAttack,
    SlimeMoveSmallJump,
    SlimeHurt,
    SlimeDeath,
    // "Big" variants use the same sprite sheets as their normal counterparts,
    // but with a 0.3s frame duration (3x slower). This makes merged slimes look
    // heavy and lumbering. The slower attack animation also inherently makes
    // the attack cycle ~3x longer, so merged slimes attack less frequently.
    BigSlimeJumpIdle,
    BigSlimeAttack,
    BigSlimeDeath,
    // Red variants for enemy slimes — same frame counts and timing,
    // just different sprite sheet images (hue-shifted to purple/red).
    EnemySlimeJumpIdle,
    EnemySlimeAttack,
    EnemySlimeMoveSmallJump,
    EnemySlimeHurt,
    EnemySlimeDeath,
    EnemyBigSlimeJumpIdle,
    EnemyBigSlimeAttack,
    EnemyBigSlimeDeath,
}

fn default_animated_sprite() -> Sprite {
    Sprite {
        texture_atlas: Some(TextureAtlas::default()),
        ..default()
    }
}

#[derive(Component, Default)]
pub struct AnimationState {
    //  pub animation_type: AnimationType,
    pub frame_index: usize,
    pub frame_timer: f32,
    pub frame_duration: f32, // seconds per frame
    total_frames: usize,
    pub looping: bool,  // Whether animation should loop
    pub finished: bool, // True when non-looping animation completes
}

/// This prevents loading the same assets multiple times
#[derive(Resource)]
pub struct SpriteSheets {
    pub slime_jump_idle: Handle<Image>,
    pub slime_attack: Handle<Image>,
    pub slime_move_small_jump: Handle<Image>,
    pub slime_hurt: Handle<Image>,
    pub slime_death: Handle<Image>,

    // Red/enemy variants — same layouts, different images
    pub enemy_slime_jump_idle: Handle<Image>,
    pub enemy_slime_attack: Handle<Image>,
    pub enemy_slime_move_small_jump: Handle<Image>,
    pub enemy_slime_hurt: Handle<Image>,
    pub enemy_slime_death: Handle<Image>,

    pub jump_idle_layout: Handle<TextureAtlasLayout>,
    pub attack_layout: Handle<TextureAtlasLayout>,
    pub move_small_jump_layout: Handle<TextureAtlasLayout>,
    pub hurt_layout: Handle<TextureAtlasLayout>,
    pub death_layout: Handle<TextureAtlasLayout>,
}

impl AnimationState {
    pub fn new(frame_duration: f32, total_frames: usize, looping: bool) -> Self {
        AnimationState {
            frame_index: 0,
            frame_timer: 0.0,
            frame_duration,
            looping,
            finished: false,
            total_frames,
        }
    }

    pub fn update(&mut self, delta_time: f32) -> bool {
        if self.finished || self.total_frames == 0 {
            return false;
        }

        self.frame_timer += delta_time;

        // Check if it's time to advance to the next frame
        if self.frame_timer >= self.frame_duration {
            self.frame_timer -= self.frame_duration;
            self.frame_index += 1;

            if self.frame_index >= self.total_frames {
                if self.looping {
                    self.frame_index = 0; // Loop back to first frame
                } else {
                    self.frame_index = self.total_frames - 1; // Stay on last frame
                    self.finished = true;
                    return true;
                }
            }
        }
        false
    }
}

pub fn animation_system(
    mut query: Query<(&mut AnimationState, &mut Sprite, &AnimationType, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut anim_state, mut sprite, animation_type, entity) in query.iter_mut() {
        // Send an event when a non-looping animation finishes
        let finished = anim_state.update(time.delta_secs());
        if finished {
            commands.trigger(AnimationFinishedEvent {
                entity: entity,
                animation_type: *animation_type,
            });
        }

        // Update the sprite's index to match the current animation frame index
        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = anim_state.frame_index;
        }
    }
}

// This is what allows an entity to change its animation and sprite
pub fn switch_animation_system(
    mut query: Query<(&mut AnimationState, &mut Sprite, &AnimationType), Changed<AnimationType>>,
    sprite_sheets: Res<SpriteSheets>,
    assets: Res<Assets<TextureAtlasLayout>>,
) {
    for (mut anim_state, mut sprite, animation_type) in query.iter_mut() {
        let mut wip_texture_atlas = TextureAtlas::default();
        match animation_type {
            AnimationType::SlimeJumpIdle => {
                sprite.image = sprite_sheets.slime_jump_idle.clone();
                wip_texture_atlas.layout = sprite_sheets.jump_idle_layout.clone();
                *anim_state = AnimationState::new(
                    0.1,
                    assets.get(&sprite_sheets.jump_idle_layout).unwrap().len(),
                    true,
                );
            }
            AnimationType::SlimeAttack => {
                sprite.image = sprite_sheets.slime_attack.clone();
                wip_texture_atlas.layout = sprite_sheets.attack_layout.clone();
                *anim_state = AnimationState::new(
                    0.1,
                    assets.get(&sprite_sheets.attack_layout).unwrap().len(),
                    false,
                );
            }
            AnimationType::SlimeMoveSmallJump => {
                sprite.image = sprite_sheets.slime_move_small_jump.clone();
                wip_texture_atlas.layout = sprite_sheets.move_small_jump_layout.clone();
                *anim_state = AnimationState::new(
                    0.1,
                    assets
                        .get(&sprite_sheets.move_small_jump_layout)
                        .unwrap()
                        .len(),
                    true,
                );
            }
            AnimationType::SlimeHurt => {
                sprite.image = sprite_sheets.slime_hurt.clone();
                wip_texture_atlas.layout = sprite_sheets.hurt_layout.clone();
                *anim_state = AnimationState::new(
                    0.1,
                    assets.get(&sprite_sheets.hurt_layout).unwrap().len(),
                    false,
                );
            }
            AnimationType::SlimeDeath => {
                sprite.image = sprite_sheets.slime_death.clone();
                wip_texture_atlas.layout = sprite_sheets.death_layout.clone();
                *anim_state = AnimationState::new(
                    0.1,
                    assets.get(&sprite_sheets.death_layout).unwrap().len(),
                    false,
                );
            }
            // Big variants reuse the same sprite sheets but run at 0.3s per frame.
            // This is the simplest way to make merged slimes animate differently —
            // no new assets needed, just a different frame_duration in the same match.
            AnimationType::BigSlimeJumpIdle => {
                sprite.image = sprite_sheets.slime_jump_idle.clone();
                wip_texture_atlas.layout = sprite_sheets.jump_idle_layout.clone();
                *anim_state = AnimationState::new(
                    0.2,
                    assets.get(&sprite_sheets.jump_idle_layout).unwrap().len(),
                    true,
                );
            }
            AnimationType::BigSlimeAttack => {
                sprite.image = sprite_sheets.slime_attack.clone();
                wip_texture_atlas.layout = sprite_sheets.attack_layout.clone();
                *anim_state = AnimationState::new(
                    0.2,
                    assets.get(&sprite_sheets.attack_layout).unwrap().len(),
                    false,
                );
            }
            AnimationType::BigSlimeDeath => {
                sprite.image = sprite_sheets.slime_death.clone();
                wip_texture_atlas.layout = sprite_sheets.death_layout.clone();
                *anim_state = AnimationState::new(
                    0.2,
                    assets.get(&sprite_sheets.death_layout).unwrap().len(),
                    false,
                );
            }
            // Enemy (red) variants — same layouts and timing as their green counterparts,
            // just swapping the image handle for the red sprite sheet.
            AnimationType::EnemySlimeJumpIdle => {
                sprite.image = sprite_sheets.enemy_slime_jump_idle.clone();
                wip_texture_atlas.layout = sprite_sheets.jump_idle_layout.clone();
                *anim_state = AnimationState::new(
                    0.1,
                    assets.get(&sprite_sheets.jump_idle_layout).unwrap().len(),
                    true,
                );
            }
            AnimationType::EnemySlimeAttack => {
                sprite.image = sprite_sheets.enemy_slime_attack.clone();
                wip_texture_atlas.layout = sprite_sheets.attack_layout.clone();
                *anim_state = AnimationState::new(
                    0.1,
                    assets.get(&sprite_sheets.attack_layout).unwrap().len(),
                    false,
                );
            }
            AnimationType::EnemySlimeMoveSmallJump => {
                sprite.image = sprite_sheets.enemy_slime_move_small_jump.clone();
                wip_texture_atlas.layout = sprite_sheets.move_small_jump_layout.clone();
                *anim_state = AnimationState::new(
                    0.1,
                    assets
                        .get(&sprite_sheets.move_small_jump_layout)
                        .unwrap()
                        .len(),
                    true,
                );
            }
            AnimationType::EnemySlimeHurt => {
                sprite.image = sprite_sheets.enemy_slime_hurt.clone();
                wip_texture_atlas.layout = sprite_sheets.hurt_layout.clone();
                *anim_state = AnimationState::new(
                    0.1,
                    assets.get(&sprite_sheets.hurt_layout).unwrap().len(),
                    false,
                );
            }
            AnimationType::EnemySlimeDeath => {
                sprite.image = sprite_sheets.enemy_slime_death.clone();
                wip_texture_atlas.layout = sprite_sheets.death_layout.clone();
                *anim_state = AnimationState::new(
                    0.1,
                    assets.get(&sprite_sheets.death_layout).unwrap().len(),
                    false,
                );
            }
            AnimationType::EnemyBigSlimeJumpIdle => {
                sprite.image = sprite_sheets.enemy_slime_jump_idle.clone();
                wip_texture_atlas.layout = sprite_sheets.jump_idle_layout.clone();
                *anim_state = AnimationState::new(
                    0.2,
                    assets.get(&sprite_sheets.jump_idle_layout).unwrap().len(),
                    true,
                );
            }
            AnimationType::EnemyBigSlimeAttack => {
                sprite.image = sprite_sheets.enemy_slime_attack.clone();
                wip_texture_atlas.layout = sprite_sheets.attack_layout.clone();
                *anim_state = AnimationState::new(
                    0.2,
                    assets.get(&sprite_sheets.attack_layout).unwrap().len(),
                    false,
                );
            }
            AnimationType::EnemyBigSlimeDeath => {
                sprite.image = sprite_sheets.enemy_slime_death.clone();
                wip_texture_atlas.layout = sprite_sheets.death_layout.clone();
                *anim_state = AnimationState::new(
                    0.2,
                    assets.get(&sprite_sheets.death_layout).unwrap().len(),
                    false,
                );
            }
        }
        sprite.texture_atlas = Some(wip_texture_atlas);
    }
}

pub fn load_sprite_sheets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load all sprite sheet images
    let slime_jump_idle = asset_server.load("sprites/slimes/Jump-Idle/Slime_Jump_Spritesheet.png");
    let slime_attack = asset_server.load("sprites/slimes/Attack/Slime_Attack_Spritesheet.png");
    let slime_move_small_jump =
        asset_server.load("sprites/slimes/Move-Small Jump/Slime_Move_Spritesheet.png");
    let slime_hurt = asset_server.load("sprites/slimes/Hurt/Slime_Hurt_Spritesheet.png");
    let slime_death = asset_server.load("sprites/slimes/Death/Slime_Death_Spritesheet.png");

    // Red/enemy variants — same sprite sheets, hue-shifted to purple/red
    let enemy_slime_jump_idle =
        asset_server.load("sprites/slimes/Jump-Idle-Red/Slime_Jump_Spritesheet.png");
    let enemy_slime_attack =
        asset_server.load("sprites/slimes/Attack-Red/Slime_Attack_Spritesheet.png");
    let enemy_slime_move_small_jump =
        asset_server.load("sprites/slimes/Move-Small Jump-Red/Slime_Move_Spritesheet.png");
    let enemy_slime_hurt = asset_server.load("sprites/slimes/Hurt-Red/Slime_Hurt_Spritesheet.png");
    let enemy_slime_death =
        asset_server.load("sprites/slimes/Death-Red/Slime_Death_Spritesheet.png");

    // Create texture atlas layouts (define frame grid)
    let jump_idle_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(60, 99),
        6,
        1,
        None,
        None,
    ));

    let attack_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(60, 99),
        5,
        1,
        None,
        None,
    ));

    let move_small_jump_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(60, 99),
        6,
        1,
        None,
        None,
    ));

    let hurt_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(60, 99),
        3,
        1,
        None,
        None,
    ));

    let death_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(60, 99),
        6,
        1,
        None,
        None,
    ));

    commands.insert_resource(SpriteSheets {
        slime_jump_idle,
        slime_attack,
        slime_move_small_jump,
        slime_hurt,
        slime_death,

        enemy_slime_jump_idle,
        enemy_slime_attack,
        enemy_slime_move_small_jump,
        enemy_slime_hurt,
        enemy_slime_death,

        jump_idle_layout,
        attack_layout,
        move_small_jump_layout,
        hurt_layout,
        death_layout,
    });
}

#[derive(Event)]
pub struct AnimationFinishedEvent {
    pub entity: Entity,
    pub animation_type: AnimationType,
}
