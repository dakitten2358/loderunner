use super::super::{TILE_SIZE_HEIGHT, TILE_SIZE_WIDTH};
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
pub struct LocalPlayerInput {}

#[derive(Component, Default, Clone)]
pub struct Runner {
    pub wants_to_burn_left: bool,
    pub wants_to_burn_right: bool,

    pub burning_left: bool,
    pub burning_right: bool,
    pub burn_time: f32,
}

impl Runner {
    pub fn is_burning(&self) -> bool {
        self.burning_left || self.burning_right
    }
}

#[derive(Component, Clone)]
pub struct Overlaps {
    pub entities: Vec<Entity>,
    pub width: f32,
    pub height: f32,
}

impl Default for Overlaps {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            height: 22.0,
            width: 18.0, // width is normally 20, but we'll move it in a bit to ensure we don't accidentally overlap when in the column next to it
        }
    }
}

#[derive(Component, Clone)]
pub struct GoldPickup {
    pub count: u32,
    pub max: u32,
}

impl Default for GoldPickup {
    fn default() -> Self {
        Self { count: 0, max: 1 }
    }
}

#[derive(Component, Default, Clone)]
pub struct Blocker {}

#[derive(PartialEq, Copy, Clone)]
pub enum BurnState {
    NotBurning,
    StartingBurn,
    Burning,
    Burnt,
    Rebuilding,
}

#[derive(Component, Clone)]
pub struct Burnable {
    burn_state: BurnState,
    pub burn_time: f32,
}

impl Burnable {
    pub fn start_burn(&mut self) {
        use BurnState::*;

        if self.burn_state == NotBurning {
            self.burn_state = StartingBurn;
            self.burn_time = 0.0;
        }
    }

    pub fn get_state(&self) -> BurnState {
        self.burn_state
    }

    pub fn set_state(&mut self, new_state: BurnState) {
        self.burn_state = new_state;
    }

    pub fn is_burning(&self) -> bool {
        self.burn_state != BurnState::NotBurning
    }
}

impl Default for Burnable {
    fn default() -> Self {
        Self {
            burn_state: BurnState::NotBurning,
            burn_time: 0.0,
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct GridTransform {
    pub offset: Vec3,
    pub translation: IVec2,
}

impl GridTransform {
    pub fn snap(&self, pos: Vec3) -> Vec3 {
        let p = pos - self.offset;
        let x = (p.x / TILE_SIZE_WIDTH).round() as i32;
        let y = (p.y / TILE_SIZE_HEIGHT).round() as i32;

        Vec3::new(x as f32 * TILE_SIZE_WIDTH, y as f32 * TILE_SIZE_HEIGHT, pos.z) + self.offset
    }

    pub fn to_world(&self, pos: IVec2) -> Vec3 {
        let x = pos.x as f32 * TILE_SIZE_WIDTH;
        let y = pos.y as f32 * TILE_SIZE_HEIGHT;

        Vec3::new(x, y, 0.0) + self.offset
    }
}

#[derive(Component, Default, Clone)]
pub struct Movement {
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,

    is_falling: bool,
    fall_direction: f32,
    start_fall_position: IVec2,

    pub velocity: Vec3,
}

impl Movement {
    pub fn add_move_left(&mut self) {
        self.move_left = true;
    }

    pub fn add_move_right(&mut self) {
        self.move_right = true;
    }

    pub fn add_move_up(&mut self) {
        self.move_up = true;
    }

    pub fn add_move_down(&mut self) {
        self.move_down = true;
    }

    pub fn start_falling(&mut self, start_pos: IVec2, direction: f32) {
        self.is_falling = true;
        self.start_fall_position = start_pos;
        self.fall_direction = direction;
    }

    pub fn is_falling(&self) -> bool {
        self.is_falling
    }

    pub fn get_fall_direction(&self) -> f32 {
        self.fall_direction
    }

    pub fn fall_start_pos(&self) -> IVec2 {
        self.start_fall_position
    }

    pub fn stop_falling(&mut self) {
        self.is_falling = false;
    }

    pub fn consume(&mut self) -> Vec2 {
        let mut directions = Vec2::ZERO;

        // horiz
        if self.move_left && !self.move_right {
            directions.x = -1.0;
        } else if self.move_right && !self.move_left {
            directions.x = 1.0;
        }

        // vert
        if self.move_up && !self.move_down {
            directions.y = 1.0;
        } else if self.move_down && !self.move_up {
            directions.y = -1.0;
        }

        self.move_left = false;
        self.move_right = false;

        self.move_up = false;
        self.move_down = false;

        directions
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct Falling {}

#[derive(Component, Debug, Clone, Default)]
pub struct HiddenLadder {}

#[derive(Component, Debug, Clone)]
pub struct SpriteAnimator {
    pub frame_index: usize,
    pub animation_name: Option<String>,
    pub elapsed: f32,
    pub active: bool,
}

impl SpriteAnimator {
    pub fn new(starting_anim: &str) -> Self {
        Self {
            frame_index: 0,
            animation_name: Some(starting_anim.to_owned()),
            elapsed: 0.0,
            active: true,
        }
    }

    pub fn switch(&mut self, anim: &str) {
        let new_anim = Some(anim.to_string());
        if new_anim != self.animation_name {
            self.frame_index = 0;
            self.animation_name = new_anim;
            self.elapsed = 0.0;
        }
    }
}

impl Default for SpriteAnimator {
    fn default() -> Self {
        Self {
            frame_index: 0,
            animation_name: None,
            elapsed: 0.0,
            active: false,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct DespawnAfter {
    pub time_remaining: f32,
}

impl DespawnAfter {
    pub fn new(time: f32) -> Self {
        Self { time_remaining: time }
    }
}

impl Default for DespawnAfter {
    fn default() -> Self {
        DespawnAfter::new(1.0)
    }
}
