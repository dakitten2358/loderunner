use crate::assets::LevelAsset::*;
use crate::assets::LevelDataAsset;
use crate::game::{bundles::*, components::*, resources::*};
use crate::CoreAssets;
use crate::{MAP_SIZE_HALF_WIDTH, TILE_SIZE_HEIGHT, TILE_SIZE_WIDTH};
use bevy::prelude::*;

const HORIZONTAL_MOVEMENT_SPEED: f32 = TILE_SIZE_WIDTH * 5.0;
const FALL_SPEED: f32 = TILE_SIZE_HEIGHT * 5.0;
const CLIMB_SPEED: f32 = TILE_SIZE_HEIGHT * 5.0;

pub fn init_gameplay(mut commands: Commands, core_assets: Res<CoreAssets>, level_datas: Res<Assets<LevelDataAsset>>) {
    let level_data = level_datas.get("levels/debug/debug.level").unwrap();
    spawn_level_entities(&mut commands, core_assets, level_data);
    commands.insert_resource(LevelResource::from_asset(level_data))
}

#[derive(Component)]
pub struct LevelSpecificComponent;

fn spawn_level_entities(commands: &mut Commands, core_assets: Res<CoreAssets>, level_data: &LevelDataAsset) {
    let tiles_atlas = &core_assets.tiles_atlas;
    let guard_atlas = &core_assets.guard_atlas;
    let runner_atlas = &core_assets.runner_atlas;

    let level_offset = Vec3::new(
        MAP_SIZE_HALF_WIDTH as f32 * TILE_SIZE_WIDTH * -1.0 + (TILE_SIZE_WIDTH / 2.0),
        TILE_SIZE_HEIGHT / 2.0,
        0.0,
    );

    for tile in &level_data.tiles {
        let pos = Vec3::new(
            tile.position.x as f32 * TILE_SIZE_WIDTH,
            tile.position.y as f32 * TILE_SIZE_HEIGHT,
            0.0,
        ) + level_offset;

        match tile.behaviour {
            TileType::Brick => commands.spawn_bundle(BrickBundle::new(tiles_atlas, pos)),
            TileType::FalseBrick => commands.spawn_bundle(FalseBrickBundle::new(tiles_atlas, pos)),
            TileType::Gold => commands.spawn_bundle(GoldBundle::new(tiles_atlas, pos)),
            TileType::Guard => commands.spawn_bundle(GuardBundle::new(guard_atlas, pos)),
            TileType::HiddenLadder => commands.spawn_bundle(HiddenLadderBundle::new(tiles_atlas, pos)),
            TileType::Ladder => commands.spawn_bundle(LadderBundle::new(tiles_atlas, pos)),
            TileType::Player => commands.spawn_bundle(PlayerBundle::new(runner_atlas, pos, level_offset)),
            TileType::Rope => commands.spawn_bundle(RopeBundle::new(tiles_atlas, pos)),
            TileType::SolidBrick => commands.spawn_bundle(SolidBrickBundle::new(tiles_atlas, pos)),
        }
        .insert(LevelSpecificComponent);
    }
}

pub fn update_grid_transforms(mut query: Query<(&Transform, &mut GridTransform)>) {
    for (transform, mut grid_transform) in query.iter_mut() {
        let p = transform.translation - grid_transform.offset;
        let x = (p.x / TILE_SIZE_WIDTH).round() as i32;
        let y = (p.y / TILE_SIZE_HEIGHT).round() as i32;
        grid_transform.translation = IVec2::new(x, y);
    }
}

pub fn player_input(keyboard_input: Res<Input<KeyCode>>, mut players: Query<&mut Movement, With<LocalPlayerInput>>) {
    // movement
    for mut player_movement in players.iter_mut() {
        if keyboard_input.pressed(KeyCode::Right) {
            player_movement.add_move_right();
        }
        if keyboard_input.pressed(KeyCode::Left) {
            player_movement.add_move_left();
        }
        if keyboard_input.pressed(KeyCode::Up) {
            player_movement.add_move_up();
        }
        if keyboard_input.pressed(KeyCode::Down) {
            player_movement.add_move_down();
        }
    }

    // burns
}

pub fn apply_movement(time: Res<Time>, level: Res<LevelResource>, mut query: Query<(&mut Movement, &mut Transform, &GridTransform)>) {
    use EffectiveTileType::*;

    let delta_time = time.delta().as_secs_f32();
    for (mut movement, mut transform, grid_transform) in query.iter_mut() {
        let mut desired_position = transform.translation;
        let tiles = level.around(grid_transform.translation);

        let desired_direction = movement.consume();

        // should we start falling?
        if !movement.is_falling() && tiles.below.behaviour == None && tiles.on.behaviour != Rope {
            movement.start_falling(grid_transform.translation);
            
        } else if !movement.is_falling() && tiles.on.behaviour == None && tiles.below.behaviour == Rope {
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

            let snapped_x = grid_transform.snap(desired_position).x;
            let direction = if (snapped_x - desired_position.x) > 0.0 { 1.0 } else if (snapped_x - desired_position.x) < 0.0 { -1.0 } else {0.0};

            let mut horiz_move_amount = delta_time * FALL_SPEED * direction;
            if (snapped_x - desired_position.x).abs() < horiz_move_amount.abs() {
                horiz_move_amount = snapped_x - desired_position.x;
            }
            desired_position.x += horiz_move_amount;
        }
        // dropping from rope?
        else if desired_direction.y < 0.0 && tiles.on.behaviour == Rope && tiles.below.behaviour == None {
            movement.start_falling(grid_transform.translation);
        }
        // top of ladder?
        else if desired_direction.y > 0.0 && (tiles.on.behaviour == None || tiles.on.behaviour == Rope) && tiles.below.behaviour == Ladder
        {
            let mut desired_movement = delta_time * CLIMB_SPEED;

            let blocking_tile_y = grid_transform.to_world(tiles.above.pos).y;
            let (is_overlapping, movable_distance) = is_range_overlapping(blocking_tile_y, desired_position.y, TILE_SIZE_HEIGHT);
            if is_overlapping {
                desired_movement = 0.0;
            } else {
                desired_movement = f32::min(movable_distance, desired_movement)
            }

            desired_position.y += desired_movement;
        }
        // trying to move up ladder, and _can_
        else if desired_direction.y > 0.0 && tiles.on.behaviour == Ladder {
            let desired_movement = delta_time * CLIMB_SPEED;
            desired_position.y += desired_movement;
            desired_position.x = grid_transform.snap(desired_position).x;
        }
        // trying to move down ladder, and _can
        else if desired_direction.y < 0.0
            && (tiles.on.behaviour == Ladder || (tiles.on.behaviour == None && tiles.below.behaviour == Ladder))
        {
            let mut desired_movement = delta_time * CLIMB_SPEED;

            if tiles.below.behaviour == Blocker {
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
            desired_position.x = grid_transform.snap(desired_position).x;
        }
        // trying to move left
        else if desired_direction.x < 0.0 {
            let mut desired_movement = delta_time * HORIZONTAL_MOVEMENT_SPEED;
            if tiles.left.behaviour == Blocker {
                let blocking_tile_x = grid_transform.to_world(tiles.left.pos).x;
                let (is_overlapping, movable_distance) = is_range_overlapping(blocking_tile_x, desired_position.x, TILE_SIZE_WIDTH);
                desired_movement = if is_overlapping {
                    0.0
                } else {
                    f32::min(movable_distance, desired_movement)
                };
            }

            desired_position.x -= desired_movement;
            desired_position.y = grid_transform.snap(desired_position).y;
        }
        // trying to move right
        else if desired_direction.x > 0.0 {
            let mut desired_movement = delta_time * HORIZONTAL_MOVEMENT_SPEED;
            if tiles.right.behaviour == Blocker {
                let blocking_tile_x = grid_transform.to_world(tiles.right.pos).x;
                let (is_overlapping, movable_distance) = is_range_overlapping(blocking_tile_x, desired_position.x, TILE_SIZE_WIDTH);
                desired_movement = if is_overlapping {
                    0.0
                } else {
                    f32::min(movable_distance, desired_movement)
                };
            }

            desired_position.x += desired_movement;
            desired_position.y = grid_transform.snap(desired_position).y;
        }

        // feed velocity back into movement
        let velocity = desired_position - transform.translation;
        movement.velocity = velocity;
        transform.translation = desired_position;
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

pub fn exit_gameplay(mut commands: Commands, to_despawn: Query<Entity, With<LevelSpecificComponent>>) {
    for entity in to_despawn.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<LevelResource>();
}
