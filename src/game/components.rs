use super::super::{TILE_SIZE_HEIGHT, TILE_SIZE_WIDTH};
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
pub struct LocalPlayerInput {}

#[derive(Component, Default, Clone)]
pub struct Runner {}

#[derive(Component, Default, Clone)]
pub struct Blocker {}

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

    pub fn start_falling(&mut self, start_pos: IVec2) {
        self.is_falling = true;
        self.start_fall_position = start_pos;
    }

    pub fn is_falling(&self) -> bool {
        self.is_falling
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

#[derive(Component, Debug, Clone)]
pub struct SpriteAnimator {
    pub frame_index: usize,
    pub animation_name: Option<String>,
    pub elapsed: f32,
    pub active: bool,
}

impl SpriteAnimator {
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
