use bevy::prelude::*;
use super::super::{TILE_SIZE_HEIGHT, TILE_SIZE_WIDTH};

#[derive(Component, Default, Clone)]
pub struct LocalPlayerInput {}

#[derive(Component, Default, Clone)]
pub struct Blocker {}

#[derive(Component, Default, Clone)]
pub struct GridTransform {
    pub offset: Vec3,
    pub translation: IVec2,
}

impl GridTransform {
	pub fn snap(&self, pos: Vec3) -> Vec3{
		let p = pos - self.offset;
        let x = (p.x / TILE_SIZE_WIDTH).round() as i32;
        let y = (p.y / TILE_SIZE_HEIGHT).round() as i32;

		Vec3::new(x as f32 * TILE_SIZE_WIDTH, y as f32 * TILE_SIZE_HEIGHT, pos.z) + self.offset
	}
}

#[derive(Component, Default, Clone)]
pub struct Movement {
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,

	pub is_falling: bool,

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
