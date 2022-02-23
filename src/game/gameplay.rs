use crate::assets::playlist_asset::PlaylistAsset;
use crate::assets::AnimAsset;
use crate::assets::LevelAsset::*;
use crate::assets::LevelDataAsset;
use crate::game::ai::{test_pathfind, NavMesh};
use crate::game::PlaylistState;
use crate::game::{bundles::*, components::*, resources::*};
use crate::AppStates;
use crate::CoreAssets;
use crate::{MAP_SIZE_HALF_WIDTH, MAP_SIZE_HEIGHT, MAP_SIZE_WIDTH, TILE_SIZE_HEIGHT, TILE_SIZE_WIDTH};
use bevy::prelude::*;

pub struct SpawnableResources {
    pub fire_left: SpriteEffectBundle,
    pub fire_right: SpriteEffectBundle,
}

pub fn init_gameplay(
    mut commands: Commands,
    core_assets: Res<CoreAssets>,
    playlist_state: Res<PlaylistState>,
    level_datas: Res<Assets<LevelDataAsset>>,
    playlists: Res<Assets<PlaylistAsset>>,
    animations: Res<Assets<AnimAsset>>,
) {
    let level_path = playlist_state.current_level(&playlists);
    let level_data = level_datas.get(level_path).unwrap();
    let mut level = LevelResource::from_asset(level_data);
    spawn_level_entities(&mut commands, &core_assets, level_data, &animations, &mut level);
    let level_navmesh = NavMesh::from_level(&level);
    test_pathfind(IVec2::new(1, 1), IVec2::new(1, 6), &level_navmesh);
    commands.insert_resource(level_navmesh);
    commands.insert_resource(level);
    commands.insert_resource(LevelState { ..Default::default() });

    let fire_atlas = &core_assets.hole_atlas;
    let fire_anim = &animations.get_handle("anims/fire.anim");
    let spawnables = SpawnableResources {
        fire_left: SpriteEffectBundle::new(fire_atlas, fire_anim, Vec3::ZERO, "left"),
        fire_right: SpriteEffectBundle::new(fire_atlas, fire_anim, Vec3::ZERO, "right"),
    };
    commands.insert_resource(spawnables);
}

#[derive(Component)]
pub struct LevelSpecificComponent;

fn spawn_level_entities(
    commands: &mut Commands,
    core_assets: &Res<CoreAssets>,
    level_data: &LevelDataAsset,
    animations: &Res<Assets<AnimAsset>>,
    level: &mut LevelResource,
) {
    let tiles_atlas = &core_assets.tiles_atlas;
    let guard_atlas = &core_assets.guard_atlas;
    let guard_anim = &animations.get_handle("anims/guard.anim");
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
            TileType::Guard => commands.spawn_bundle(GuardBundle::new(guard_atlas, guard_anim, pos, level_offset)),
            TileType::HiddenLadder => commands.spawn_bundle(HiddenLadderBundle::new(tiles_atlas, pos, level_offset)),
            TileType::Ladder => commands.spawn_bundle(LadderBundle::new(tiles_atlas, pos)),
            TileType::Player => commands.spawn_bundle(PlayerBundle::new(runner_atlas, runner_anim, pos, level_offset)),
            TileType::Rope => commands.spawn_bundle(RopeBundle::new(tiles_atlas, pos)),
            TileType::SolidBrick => commands.spawn_bundle(SolidBrickBundle::new(tiles_atlas, pos)),
        }
        .insert(LevelSpecificComponent)
        .id();
        level.set_entity(tile.position, tile_id);
    }

    for x in 0..MAP_SIZE_WIDTH {
        let pos = Vec3::new(x as f32 * TILE_SIZE_WIDTH, MAP_SIZE_HEIGHT as f32 * TILE_SIZE_HEIGHT, 0.0) + level_offset;
        commands.spawn_bundle(VictoryTileBundle::new(pos)).insert(LevelSpecificComponent);
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

        runner.wants_to_burn_left = keyboard_input.pressed(KeyCode::Z);
        runner.wants_to_burn_right = keyboard_input.pressed(KeyCode::X);
    }
}

pub fn start_burns(
    time: Res<Time>,
    level: Res<LevelResource>,
    mut commands: Commands,
    spawnables: Res<SpawnableResources>,
    mut runners: Query<(&GridTransform, &mut Runner, &mut Transform, &mut Movement), Without<Falling>>,
    mut all_burnables: Query<&mut Burnable>,
) {
    for (transform, mut runner, mut world_transform, mut movement) in runners.iter_mut() {
        // check to see if we should start a burn
        if !runner.is_burning() {
            let tiles = level.around(transform.translation);
            if runner.wants_to_burn_left && start_burn(&tiles.below_left, &mut all_burnables) {
                let mut fire = spawnables.fire_left.clone();
                fire.transform.translation = transform.to_world(tiles.left.pos);
                commands.spawn_bundle(fire).insert(LevelSpecificComponent);
                runner.burning_left = true;
                runner.burn_time = 0.0;
                world_transform.translation.x = transform.snap(world_transform.translation).x;
            } else if runner.wants_to_burn_right && start_burn(&tiles.below_right, &mut all_burnables) {
                let mut fire = spawnables.fire_right.clone();
                fire.transform.translation = transform.to_world(tiles.right.pos);
                commands.spawn_bundle(fire).insert(LevelSpecificComponent);
                runner.burning_right = true;
                runner.burn_time = 0.0;
                world_transform.translation.x = transform.snap(world_transform.translation).x;
            }
        }
        // we're in a burn already, so don't do anything except tick a timer
        else {
            // clear any movement
            movement.consume();

            runner.burn_time += time.delta_seconds();
            if runner.burn_time >= (10.0 / 22.0) {
                runner.burning_left = false;
                runner.burning_right = false;
                runner.burn_time = 0.0;
            }
        }
    }
}

fn start_burn(tile: &Tile, all_burnables: &mut Query<&mut Burnable>) -> bool {
    if let Some(ent) = tile.entity {
        if let Ok(mut burnable) = all_burnables.get_mut(ent) {
            if !burnable.is_burning() {
                burnable.start_burn();
                return true;
            }
        }
    }
    false
}

pub fn apply_burnables(
    mut commands: Commands,
    time: Res<Time>,
    mut level: ResMut<LevelResource>,
    mut query: Query<(&mut Burnable, &GridTransform, &Overlaps)>,
    killables: Query<&Killable>,
) {
    use BurnState::*;

    for (mut burnable, transform, overlaps) in query.iter_mut() {
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

                    for overlapping_entity in &overlaps.entities {
                        if killables.get(*overlapping_entity).is_ok() {
                            commands.entity(*overlapping_entity).insert(Killed {});
                        }
                    }
                }
            }
            _ => burnable.burn_time = 0.0,
        }
    }
}

pub fn gold_pickups(
    mut commands: Commands,
    level: Res<LevelResource>,
    mut state: ResMut<LevelState>,
    mut players: Query<(&mut GoldPickup, &Overlaps)>,
    treasures: Query<Entity, With<Treasure>>,
) {
    for (mut pickup, overlap) in players.iter_mut() {
        for entity in &overlap.entities {
            // make sure it's gold (overlap could be anything)
            if let Ok(gold_entity) = treasures.get(*entity) {
                // make sure we can pick it up
                if pickup.count >= pickup.max {
                    break;
                }

                // pick it up and destroy it
                pickup.count += 1;
                commands.entity(gold_entity).despawn_recursive();
            }
        }

        if pickup.count >= level.treasure_count() {
            state.should_complete = true;
        }
    }
}

pub fn show_exit_ladders(
    mut state: ResMut<LevelState>,
    mut level: ResMut<LevelResource>,
    mut hidden_ladders: Query<(Entity, &mut Visibility, &GridTransform), With<HiddenLadder>>,
) {
    if state.should_complete && !state.completed {
        state.completed = true;

        for (hidden_ladder, mut ladder_visibility, transform) in hidden_ladders.iter_mut() {
            ladder_visibility.is_visible = true;
            level.set(transform.translation, EffectiveTileType::Ladder);
            level.set_entity(transform.translation, hidden_ladder);
        }
    }
}

pub fn next_level(
    mut app_state: ResMut<State<AppStates>>,
    mut playlist_state: ResMut<PlaylistState>,
    playlists: Res<Assets<PlaylistAsset>>,
    players: Query<&Overlaps, With<Runner>>,
    victory_tiles: Query<Entity, With<Victory>>,
) {
    for player_overlap in players.iter() {
        for overlapping_entity in &player_overlap.entities {
            if victory_tiles.get(*overlapping_entity).is_ok() {
                playlist_state.next_level(&playlists);
                app_state.set(AppStates::ChangeLevel).expect("failed to change state");
                break;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn start_guard_respawn(
    mut commands: Commands,
    mut level: ResMut<LevelResource>,
    mut new_dead_guards: Query<
        (
            Entity,
            &mut Respawnable,
            &mut Visibility,
            &mut Transform,
            &mut Overlaps,
            &GridTransform,
        ),
        (With<Guard>, Added<Killed>),
    >,
) {
    for (entity, mut respawn, mut visibility, mut transform, mut overlaps, grid_transform) in new_dead_guards.iter_mut() {
        respawn.timer = 0.0;
        respawn.position = level.get_random_respawn();

        visibility.is_visible = false;
        overlaps.is_active = false;
        // probably don't need to move it, but let's do so anyways
        transform.translation = Vec3::new(-100.0, -100.0, 0.0);
        commands.entity(entity).remove::<Stunned>();
        level.reset_override(grid_transform.translation);
    }
}

#[allow(clippy::type_complexity)]
pub fn respawn_guard(
    mut commands: Commands,
    time: Res<Time>,
    mut dead_guards: Query<
        (
            Entity,
            &mut Respawnable,
            &mut Visibility,
            &mut Transform,
            &mut Overlaps,
            &GridTransform,
        ),
        (With<Guard>, With<Killed>),
    >,
) {
    for (guard_entity, mut respawn, mut visibility, mut transform, mut overlaps, grid_transform) in dead_guards.iter_mut() {
        respawn.timer += time.delta_seconds();
        if respawn.timer > 2.5 {
            overlaps.is_active = true;
            respawn.timer = 0.0;
            commands.entity(guard_entity).remove::<Killed>();
        } else if respawn.timer > 2.0 {
            visibility.is_visible = true;
            transform.translation = grid_transform.to_world(respawn.position);
        }
    }
}

pub fn restart_level(mut app_state: ResMut<State<AppStates>>, dead_players: Query<&Runner, With<Killed>>) {
    if !dead_players.is_empty() {
        app_state.set(AppStates::ChangeLevel).expect("failed to change state");
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
    commands.remove_resource::<LevelState>();
    commands.remove_resource::<SpawnableResources>();
}
