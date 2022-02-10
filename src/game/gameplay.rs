use crate::assets::AnimAsset;
use crate::assets::LevelAsset::*;
use crate::assets::LevelDataAsset;
use crate::game::{bundles::*, components::*, resources::*};
use crate::CoreAssets;
use crate::{MAP_SIZE_HALF_WIDTH, TILE_SIZE_HEIGHT, TILE_SIZE_WIDTH};
use bevy::prelude::*;

pub fn init_gameplay(
    mut commands: Commands,
    core_assets: Res<CoreAssets>,
    level_datas: Res<Assets<LevelDataAsset>>,
    animations: Res<Assets<AnimAsset>>,
) {
    let level_data = level_datas.get("levels/debug/debug.level").unwrap();
    let mut level = LevelResource::from_asset(level_data);
    spawn_level_entities(&mut commands, core_assets, level_data, &animations, &mut level);
    commands.insert_resource(level)
}

#[derive(Component)]
pub struct LevelSpecificComponent;

fn spawn_level_entities(
    commands: &mut Commands,
    core_assets: Res<CoreAssets>,
    level_data: &LevelDataAsset,
    animations: &Res<Assets<AnimAsset>>,
    level: &mut LevelResource,
) {
    let tiles_atlas = &core_assets.tiles_atlas;
    let guard_atlas = &core_assets.guard_atlas;
    let hole_atlas = &core_assets.hole_atlas;
    let hole_anim = &animations.get_handle("anims/brick.anim");
    let runner_atlas = &core_assets.runner_atlas;
    let runner_anim = &animations.get_handle("anims/runner.anim");

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

        let tile_id = match tile.behaviour {
            TileType::Brick => commands.spawn_bundle(BrickBundle::new(hole_atlas, hole_anim, pos, level_offset)),
            TileType::FalseBrick => commands.spawn_bundle(FalseBrickBundle::new(tiles_atlas, pos)),
            TileType::Gold => commands.spawn_bundle(GoldBundle::new(tiles_atlas, pos)),
            TileType::Guard => commands.spawn_bundle(GuardBundle::new(guard_atlas, pos)),
            TileType::HiddenLadder => commands.spawn_bundle(HiddenLadderBundle::new(tiles_atlas, pos)),
            TileType::Ladder => commands.spawn_bundle(LadderBundle::new(tiles_atlas, pos)),
            TileType::Player => commands.spawn_bundle(PlayerBundle::new(runner_atlas, runner_anim, pos, level_offset)),
            TileType::Rope => commands.spawn_bundle(RopeBundle::new(tiles_atlas, pos)),
            TileType::SolidBrick => commands.spawn_bundle(SolidBrickBundle::new(tiles_atlas, pos)),
        }
        .insert(LevelSpecificComponent)
        .id();
        level.set_entity(tile.position, tile_id);
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

pub fn player_input(keyboard_input: Res<Input<KeyCode>>, mut players: Query<(&mut Movement, &mut Runner), With<LocalPlayerInput>>) {
    // movement
    for (mut player_movement, mut runner) in players.iter_mut() {
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

        runner.burn_left = keyboard_input.pressed(KeyCode::Z);
        runner.burn_right = keyboard_input.pressed(KeyCode::X);
    }
}

pub fn start_burns(level: Res<LevelResource>, mut runners: Query<(&GridTransform, &mut Runner), Without<Falling>>, mut all_burnables: Query<&mut Burnable>) {
    for (transform, mut runner) in runners.iter_mut() {
        let tiles = level.around(transform.translation);
        if runner.burn_left && start_burn(&tiles.below_left, &mut all_burnables) {
            println!("left burn");
        } else if runner.burn_right && start_burn(&tiles.below_right, &mut all_burnables) {
            println!("right burn");
        }
    }
}

fn start_burn(tile: &Tile, all_burnables: &mut Query<&mut Burnable>) -> bool {
    if let Some(ent) = tile.entity {
        if let Ok(mut burnable) = all_burnables.get_mut(ent) {
            burnable.start_burn();
            return true;
        }
    }
    false
}

pub fn apply_burnables(time: Res<Time>, mut level: ResMut<LevelResource>, mut query: Query<(&mut Burnable, &GridTransform)>) {
    use BurnState::*;

    for (mut burnable, transform) in query.iter_mut() {
        burnable.burn_time += time.delta_seconds();

        match burnable.get_state() {
            StartingBurn => burnable.set_state(Burning),
            Burning => {
                if burnable.burn_time > 0.5 {
                    burnable.set_state(Burnt);
                }
            }
            Burnt => {
                level.set(transform.translation, EffectiveTileType::None);
                if burnable.burn_time > 4.5 {
                    burnable.set_state(Rebuilding);
                }
            }
            Rebuilding => {
                if burnable.burn_time > 5.0 {
                    burnable.set_state(NotBurning);
                    level.set(transform.translation, EffectiveTileType::Blocker);
                    //TODO(jm): kill stuff?  spawn killer here?
                }
            }
            _ => burnable.burn_time = 0.0,
        }
    }
}

pub fn pending_despawns(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut DespawnAfter)>) {
    for (entity, mut despawn_after) in query.iter_mut() {
        despawn_after.time_remaining -= time.delta_seconds();
        if despawn_after.time_remaining <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn exit_gameplay(mut commands: Commands, to_despawn: Query<Entity, With<LevelSpecificComponent>>) {
    for entity in to_despawn.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<LevelResource>();
}
