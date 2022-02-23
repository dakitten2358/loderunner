use crate::game::{components::*, resources::*};
use crate::{TILE_SIZE_HEIGHT, TILE_SIZE_WIDTH};
use bevy::prelude::*;

const FALL_SPEED: f32 = TILE_SIZE_HEIGHT * 7.0;

#[derive(Component, Default, Clone)]
pub struct Movement {
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,

    is_falling: bool,
    fall_direction: f32,
    start_fall_position: IVec2,

    pub horizontal_speed: f32,
    pub climb_speed: f32,

    pub velocity: Vec3,
}

impl Movement {
    pub fn new(horizontal_speed: f32, climb_speed: f32) -> Self {
        Self {
            horizontal_speed,
            climb_speed,
            ..Default::default()
        }
    }

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
pub struct Falling;

#[allow(clippy::type_complexity)]
pub fn apply_movement(
    mut commands: Commands,
    time: Res<Time>,
    level: Res<LevelResource>,
    mut query: Query<(Entity, &mut Movement, &mut Transform, &GridTransform), (Without<Falling>, Without<Killed>, Without<Stunned>)>,
) {
    use EffectiveTileType::*;

    let delta_time = time.delta().as_secs_f32();
    for (entity, mut movement, mut transform, grid_transform) in query.iter_mut() {
        let mut desired_position = transform.translation;
        let tiles = level.around(grid_transform.translation);
        let previous_velocity = movement.velocity;

        let desired_direction = movement.consume();

        if should_start_falling(&tiles) || wants_to_drop_from_rope(desired_direction, tiles) {
            movement.start_falling(grid_transform.translation, previous_velocity.x);
            commands.entity(entity).insert(Falling {});
        } else if drop_from_ladder_bottom(desired_direction, tiles) {
            movement.start_falling(tiles.above.pos, 0.0);
            commands.entity(entity).insert(Falling {});
        }
        // top of ladder?
        else if desired_direction.y > 0.0 && (tiles.on.behaviour == None || tiles.on.behaviour == Rope) && tiles.below.behaviour == Ladder
        {
            let mut desired_movement = delta_time * movement.climb_speed;
            let blocking_tile_y = grid_transform.to_world(tiles.above.pos).y;
            let (_, movable_distance) = is_range_overlapping(blocking_tile_y, desired_position.y, TILE_SIZE_HEIGHT);
            desired_movement = f32::min(movable_distance, desired_movement);

            desired_position.y += desired_movement;
        }
        // trying to move up ladder, and _can_
        else if desired_direction.y > 0.0 && tiles.on.behaviour == Ladder {
            let mut desired_movement = delta_time * movement.climb_speed;

            if tiles.above.behaviour == Blocker {
                let blocking_tile_y = grid_transform.to_world(tiles.above.pos).y;
                let (_, movable_distance) = is_range_overlapping(blocking_tile_y, desired_position.y, TILE_SIZE_HEIGHT);
                desired_movement = f32::min(movable_distance, desired_movement.abs());
            }

            if desired_movement != 0.0 {
                desired_position.y += desired_movement;
                desired_position.x += drift_towards(
                    grid_transform.snap(desired_position).x,
                    desired_position.x,
                    delta_time * movement.horizontal_speed,
                );
            }
        }
        // trying to move down ladder, and _can
        else if desired_direction.y < 0.0
            && (tiles.on.behaviour == Ladder
                || ((tiles.on.behaviour == None || tiles.on.behaviour == Rope) && tiles.below.behaviour == Ladder))
        {
            let mut desired_movement = delta_time * -movement.climb_speed;
            if tiles.below.behaviour == Blocker {
                let blocking_tile_y = grid_transform.to_world(tiles.below.pos).y;
                let (_, movable_distance) = is_range_overlapping(blocking_tile_y, desired_position.y, TILE_SIZE_HEIGHT);
                desired_movement = f32::min(movable_distance, desired_movement.abs()) * -1.0;
            }

            if desired_movement != 0.0 {
                desired_position.y += desired_movement;
                desired_position.x += drift_towards(
                    grid_transform.snap(desired_position).x,
                    desired_position.x,
                    delta_time * movement.horizontal_speed,
                );
            }
        }

        // if we haven't moved up or down, let's try horizontal?
        if desired_position == transform.translation && desired_direction.x != 0.0 {
            let mut desired_movement = delta_time * movement.horizontal_speed * useful_sign(desired_direction.x);
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
                    delta_time * movement.horizontal_speed,
                );
            }
        }

        // feed velocity back into movement
        movement.velocity = desired_position - transform.translation;
        transform.translation = desired_position;
    }
}

#[allow(clippy::type_complexity)]
pub fn apply_falling(
    mut commands: Commands,
    time: Res<Time>,
    level: Res<LevelResource>,
    mut query: Query<(Entity, &mut Movement, &mut Transform, &GridTransform), (With<Falling>, With<Runner>)>,
) {
    use EffectiveTileType::*;

    let delta_time = time.delta().as_secs_f32();
    for (entity, mut movement, mut transform, grid_transform) in query.iter_mut() {
        let mut desired_position = transform.translation;
        let tiles = level.around(grid_transform.translation);

        movement.consume();

        let mut desired_movement = delta_time * FALL_SPEED;
        if (tiles.below.behaviour == Blocker || tiles.below.behaviour == Ladder)
            || (tiles.on.behaviour == Rope && tiles.on.pos != movement.fall_start_pos())
        {
            let blocking_tile_y = grid_transform.to_world(tiles.below.pos).y;
            let (is_overlapping, movable_distance) = is_range_overlapping(blocking_tile_y, desired_position.y, TILE_SIZE_HEIGHT);
            if is_overlapping {
                desired_movement = 0.0;
                movement.stop_falling();
                commands.entity(entity).remove::<Falling>();
            } else {
                desired_movement = f32::min(movable_distance, desired_movement)
            }
        }

        desired_position.y -= desired_movement;
        desired_position.x += drift_towards(grid_transform.snap(desired_position).x, desired_position.x, delta_time * FALL_SPEED);

        // feed velocity back into movement
        movement.velocity = desired_position - transform.translation;
        transform.translation = desired_position;
    }
}

#[allow(clippy::type_complexity)]
pub fn apply_falling_guard(
    mut commands: Commands,
    time: Res<Time>,
    mut level: ResMut<LevelResource>,
    mut query: Query<(Entity, &mut Movement, &mut Transform, &GridTransform), (With<Falling>, Without<Runner>, Without<Killed>)>,
    bricks: Query<Entity, With<Burnable>>,
) {
    use EffectiveTileType::*;

    let delta_time = time.delta().as_secs_f32();
    for (entity, mut movement, mut transform, grid_transform) in query.iter_mut() {
        let mut desired_position = transform.translation;
        let tiles = level.around(grid_transform.translation);

        movement.consume();

        let mut desired_movement = delta_time * FALL_SPEED;
        if (tiles.below.behaviour == Blocker || tiles.below.behaviour == Ladder)
            || (tiles.on.behaviour == Rope && tiles.on.pos != movement.fall_start_pos())
            || (tiles.on.entity.is_some() && bricks.get(tiles.on.entity.unwrap()).is_ok())
        {
            let blocking_tile_y = grid_transform.to_world(tiles.below.pos).y;
            let (is_overlapping, movable_distance) = is_range_overlapping(blocking_tile_y, desired_position.y, TILE_SIZE_HEIGHT);
            if is_overlapping {
                desired_movement = 0.0;
                movement.stop_falling();
                commands.entity(entity).remove::<Falling>();

                // mark it as stunned
                if tiles.on.entity.is_some() && bricks.get(tiles.on.entity.unwrap()).is_ok() {
                    commands.entity(entity).insert(Stunned {});
                    level.set_override(grid_transform.translation, EffectiveTileType::Blocker);
                }
            } else {
                desired_movement = f32::min(movable_distance, desired_movement)
            }
        }

        desired_position.y -= desired_movement;
        desired_position.x += drift_towards(grid_transform.snap(desired_position).x, desired_position.x, delta_time * FALL_SPEED);

        // feed velocity back into movement
        movement.velocity = desired_position - transform.translation;
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
fn should_start_falling(tiles: &TilesAround) -> bool {
    use EffectiveTileType::*;

    // if we're on a ladder, we dont' fall
    if tiles.on.behaviour == Ladder {
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

fn wants_to_drop_from_rope(desired_direction: Vec2, tiles: TilesAround) -> bool {
    use EffectiveTileType::*;
    desired_direction.y < 0.0 && tiles.on.behaviour == Rope && (tiles.below.behaviour == None || tiles.below.behaviour == Rope)
}

fn drop_from_ladder_bottom(desired_direction: Vec2, tiles: TilesAround) -> bool {
    use EffectiveTileType::*;
    desired_direction.y < 0.0 && (tiles.on.behaviour == None || tiles.on.behaviour == Rope) && tiles.above.behaviour == Ladder
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

pub fn build_overlaps(mut query: Query<(Entity, &Transform, &mut Overlaps)>) {
    for (_, _, mut overlap) in query.iter_mut() {
        overlap.entities.clear();
    }

    let mut combinations = query.iter_combinations_mut();
    while let Some([(a_entity, a_transform, mut a_overlap), (b_entity, b_transform, mut b_overlap)]) = combinations.fetch_next() {
        // if one of them is inactive, no overlap can occur
        if !a_overlap.is_active || !b_overlap.is_active {
            continue;
        }

        // are these overlapping?
        if is_overlapping((a_transform, &a_overlap), (b_transform, &b_overlap)) {
            mark_overlap((a_entity, &mut a_overlap), (b_entity, &mut b_overlap))
        }
    }
}

fn is_overlapping(a: (&Transform, &Overlaps), b: (&Transform, &Overlaps)) -> bool {
    is_range_overlapping(a.0.translation.x, b.0.translation.x, f32::min(a.1.width, b.1.width)).0
        && is_range_overlapping(a.0.translation.y, b.0.translation.y, f32::min(a.1.height, b.1.height)).0
}

fn mark_overlap(a: (Entity, &mut Overlaps), b: (Entity, &mut Overlaps)) {
    a.1.entities.push(b.0);
    b.1.entities.push(a.0);
}
