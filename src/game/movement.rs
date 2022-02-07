use crate::game::{components::*, resources::*};
use crate::{TILE_SIZE_HEIGHT, TILE_SIZE_WIDTH};
use bevy::prelude::*;

const HORIZONTAL_MOVEMENT_SPEED: f32 = TILE_SIZE_WIDTH * 7.0;
const FALL_SPEED: f32 = TILE_SIZE_HEIGHT * 7.0;
const CLIMB_SPEED: f32 = TILE_SIZE_HEIGHT * 7.0;

pub fn apply_movement(time: Res<Time>, level: Res<LevelResource>, mut query: Query<(&mut Movement, &mut Transform, &GridTransform)>) {
    use EffectiveTileType::*;

    let delta_time = time.delta().as_secs_f32();
    for (mut movement, mut transform, grid_transform) in query.iter_mut() {
        let mut desired_position = transform.translation;
        let tiles = level.around(grid_transform.translation);

        let desired_direction = movement.consume();

        if should_start_falling(&movement, &tiles) {
            movement.start_falling(grid_transform.translation);
        } else if movement.is_falling() {
            let mut desired_movement = delta_time * FALL_SPEED;
            if tiles.below.behaviour == Blocker || (tiles.on.behaviour == Rope && tiles.on.pos != movement.fall_start_pos()) {
                let blocking_tile_y = grid_transform.to_world(tiles.below.pos).y;
                let (is_overlapping, movable_distance) = is_range_overlapping(blocking_tile_y, desired_position.y, TILE_SIZE_HEIGHT);
                if is_overlapping {
                    desired_movement = 0.0;
                    movement.stop_falling();
                } else {
                    desired_movement = f32::min(movable_distance, desired_movement)
                }
            }

            desired_position.y -= desired_movement;
            desired_position.x += drift_towards(grid_transform.snap(desired_position).x, desired_position.x, delta_time * FALL_SPEED);
        }
        // dropping from rope?
        else if desired_direction.y < 0.0
            && tiles.on.behaviour == Rope
            && (tiles.below.behaviour == None || tiles.below.behaviour == Rope)
        {
            movement.start_falling(grid_transform.translation);
        }
        // top of ladder?
        else if desired_direction.y > 0.0 && (tiles.on.behaviour == None || tiles.on.behaviour == Rope) && tiles.below.behaviour == Ladder
        {
            let mut desired_movement = delta_time * CLIMB_SPEED;
            let blocking_tile_y = grid_transform.to_world(tiles.above.pos).y;
            let (_, movable_distance) = is_range_overlapping(blocking_tile_y, desired_position.y, TILE_SIZE_HEIGHT);
            desired_movement = f32::min(movable_distance, desired_movement);

            desired_position.y += desired_movement;
        }
        // trying to move up ladder, and _can_
        else if desired_direction.y > 0.0 && tiles.on.behaviour == Ladder {
            let desired_movement = delta_time * CLIMB_SPEED;
            desired_position.y += desired_movement;
            desired_position.x += drift_towards(
                grid_transform.snap(desired_position).x,
                desired_position.x,
                delta_time * HORIZONTAL_MOVEMENT_SPEED,
            );
        }
        // trying to move down ladder, and _can
        else if desired_direction.y < 0.0
            && (tiles.on.behaviour == Ladder
                || ((tiles.on.behaviour == None || tiles.on.behaviour == Rope) && tiles.below.behaviour == Ladder))
        {
            let mut desired_movement = delta_time * -CLIMB_SPEED;
            if tiles.below.behaviour == Blocker {
                let blocking_tile_y = grid_transform.to_world(tiles.below.pos).y;
                let (is_overlapping, movable_distance) = is_range_overlapping(blocking_tile_y, desired_position.y, TILE_SIZE_HEIGHT);
                desired_movement = f32::min(movable_distance, desired_movement.abs()) * -1.0;
                if is_overlapping {
                    movement.stop_falling();
                }
            }

            if desired_movement != 0.0 {
                desired_position.y += desired_movement;
                desired_position.x += drift_towards(
                    grid_transform.snap(desired_position).x,
                    desired_position.x,
                    delta_time * HORIZONTAL_MOVEMENT_SPEED,
                );
            }
        }

        // if we haven't moved up or down, let's try horizontal?
        if desired_position == transform.translation && desired_direction.x != 0.0 {
            let mut desired_movement = delta_time * HORIZONTAL_MOVEMENT_SPEED * useful_sign(desired_direction.x);
            let relevant_tile = get_horizontal_tile_from_sign(desired_direction.x, &tiles);
            if relevant_tile.behaviour == Blocker {
                let blocking_tile_x = grid_transform.to_world(relevant_tile.pos).x;
                let (_, movable_distance) = is_range_overlapping(blocking_tile_x, desired_position.x, TILE_SIZE_WIDTH);
                desired_movement = f32::min(movable_distance, desired_movement.abs()) * useful_sign(desired_direction.x);
            }

            desired_position.x += desired_movement;
            if tiles.on.behaviour != Ladder {
                desired_position.y += drift_towards(
                    grid_transform.snap(desired_position).y,
                    desired_position.y,
                    delta_time * HORIZONTAL_MOVEMENT_SPEED,
                );
            }
        }

        // feed velocity back into movement
        let velocity = desired_position - transform.translation;
        movement.velocity = velocity;
        transform.translation = desired_position;
    }
}

fn drift_towards(target: f32, current: f32, speed: f32) -> f32 {
    let signed_distance_between = target - current;
    let direction = useful_sign(signed_distance_between);
    if signed_distance_between.abs() < speed {
        signed_distance_between
    } else {
        speed * direction
    }
}

fn get_horizontal_tile_from_sign(horizontal_direction: f32, tiles: &TilesAround) -> &Tile {
    if horizontal_direction > 0.0 {
        &tiles.right
    } else {
        &tiles.left
    }
}

#[allow(clippy::needless_bool, clippy::if_same_then_else)] // i want to keep the conditions separate for readability
fn should_start_falling(movement: &Movement, tiles: &TilesAround) -> bool {
    use EffectiveTileType::*;

    // already falling?
    if movement.is_falling() {
        false
    }
    // nothing below us, and not on a rope
    else if tiles.below.behaviour == None && tiles.on.behaviour != Rope {
        true
    }
    // we're not on a rope, and there's a rope below us
    else if tiles.on.behaviour == None && tiles.below.behaviour == Rope {
        true
    }
    // no need to start falling apparently,
    else {
        false
    }
}

fn is_range_overlapping(a: f32, b: f32, size: f32) -> (bool, f32) {
    let delta = (b - a).abs();
    if delta <= size {
        (true, 0.0)
    } else {
        (false, delta - size)
    }
}

fn useful_sign(num: f32) -> f32 {
    if num > 0.0 {
        1.0
    } else if num < 0.0 {
        -1.0
    } else {
        0.0
    }
}
