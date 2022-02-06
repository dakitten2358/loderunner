use crate::assets::LevelAsset::*;
use crate::assets::LevelDataAsset;
use crate::game::{bundles::*, components::*};
use crate::CoreAssets;
use crate::{MAP_SIZE_HALF_WIDTH, MAP_SIZE_HEIGHT, TILE_SIZE_HEIGHT, TILE_SIZE_WIDTH};
use bevy::prelude::*;

const TILE_SIZE_HALF_WIDTH: f32 = TILE_SIZE_WIDTH / 2.0;
const HORIZONTAL_MOVEMENT_SPEED: f32 = TILE_SIZE_WIDTH * 5.0;

pub struct GameplayState {}
pub struct ColliderState {
    pub colliders: Vec<Vec3>,
}

pub fn init_gameplay(mut commands: Commands, core_assets: Res<CoreAssets>, level_datas: Res<Assets<LevelDataAsset>>) {
    commands.insert_resource(GameplayState {});
    commands.insert_resource(ColliderState { colliders: Vec::new() });

    let level_data = level_datas.get("levels/debug/debug.level").unwrap();
    spawn_level_entities(&mut commands, core_assets, level_data);

    /*
    for y in (MAP_SIZE_HEIGHT-1)..0 {
        let mut tiles_in_row: Vec<&LevelTile> = level_data.tiles.iter().filter(|tile| tile.position.y == y).collect();
        tiles_in_row.sort_by(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap());

        let mut current_range: Option<std::ops::Range<i32>> = None;
        for tile in tiles_in_row {
            //if (tile.behaviour == )
        }
    }
    */
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

pub fn update_collision_map(mut collision_map: ResMut<ColliderState>, query: Query<&Transform, With<Blocker>>) {
    collision_map.colliders.clear();
    for collider_position in query.iter() {
        collision_map.colliders.push(collider_position.translation);
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

pub fn apply_movement(time: Res<Time>, collision_map: Res<ColliderState>, mut query: Query<(&mut Movement, &mut Transform)>) {
    for (mut movement, mut transform) in query.iter_mut() {
        let mut desired_position = transform.translation;

        let desired_direction = movement.consume();

        // trying to move up, and _can_
        // trying to move down, and _can
        // trying to move left
        if desired_direction.x < 0.0 {
            desired_position.x -= time.delta().as_secs_f32() * HORIZONTAL_MOVEMENT_SPEED;
        }
        // trying to move right
        else if desired_direction.x > 0.0 {
            desired_position.x += time.delta().as_secs_f32() * HORIZONTAL_MOVEMENT_SPEED;
        }

        // overlapping anything?
        for collider in &collision_map.colliders {
            if is_overlapping(&desired_position, collider) {
                desired_position = transform.translation;
            }
        }

        // clamp stuff
        let horizontal_edge = (TILE_SIZE_WIDTH * MAP_SIZE_HALF_WIDTH as f32) - TILE_SIZE_HALF_WIDTH;
        if desired_position.x >= horizontal_edge {
            desired_position.x = horizontal_edge;
        }
        if desired_position.x <= -horizontal_edge {
            desired_position.x = -horizontal_edge
        }

        // feed velocity back into movement
        let velocity = desired_position - transform.translation;
        movement.velocity = velocity;
        transform.translation = desired_position;
    }
}

fn is_overlapping(a: &Vec3, b: &Vec3) -> bool {
    is_range_overlapping(a.x, b.x, TILE_SIZE_WIDTH) && is_range_overlapping(a.y, b.y, TILE_SIZE_HEIGHT)
}

fn is_range_overlapping(a: f32, b: f32, size: f32) -> bool {
    (b - a).abs() < size
}

pub fn exit_gameplay(mut commands: Commands, to_despawn: Query<Entity, With<LevelSpecificComponent>>) {
    for entity in to_despawn.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<GameplayState>();
    commands.remove_resource::<ColliderState>();
}
