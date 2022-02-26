use crate::{
    game::resources::LevelResource, CoreAssets, MAP_SIZE_HALF_WIDTH, MAP_SIZE_HEIGHT, MAP_SIZE_WIDTH, TILE_SIZE_HEIGHT, TILE_SIZE_WIDTH,
};
use bevy::prelude::*;

use super::{
    components::{GridTransform, Runner},
    movement::Movement,
    resources::{EffectiveTileType, Tile},
};
use std::collections::HashMap;

#[derive(Component, Debug, Default, Clone)]
pub struct AiController {
    pub path: Vec<IVec2>,
    pub path_time_remaining: f32,
}

#[allow(clippy::comparison_chain)]
pub fn run_ai_guards(
    time: Res<Time>,
    navmesh: Res<NavMesh>,
    mut guards: Query<(&Transform, &GridTransform, &mut AiController, &mut Movement), With<AiController>>,
    players: Query<&GridTransform, With<Runner>>,
) {
    for (transform, grid_transform, mut ai, mut movement) in guards.iter_mut() {
        ai.path_time_remaining -= time.delta_seconds();

        if ai.path.is_empty() || ai.path_time_remaining <= 0.0 {
            let ai_pos2 = grid_transform.translation;
            if let Some(runner_transform) = find_nearest_runner(grid_transform.translation, &players) {
                let runner_pos = runner_transform.translation;
                if let Ok(mut path) = astar_pathfind(ai_pos2, runner_pos, &navmesh) {
                    ai.path.clear();
                    ai.path.append(&mut path);
                    ai.path.remove(0); // we don't need the first one, since that's our starting tile
                    ai.path_time_remaining += 1.0 / 5.0;
                    //println!("repath");
                }
            }
        } else if !ai.path.is_empty() {
            let ai_pos = transform.translation;
            let t = grid_transform.to_world(ai.path[0]);
            let d = Vec3::distance(ai_pos, t);
            /*println!(
                "vel: {},{} ({}), tar: {},{} (distance={})",
                movement.velocity.x,
                movement.velocity.y,
                movement.velocity.length(),
                t.x,
                t.y,
                d
            );
            */
            if d < 5.0 {
                //println!("pop");
                ai.path.remove(0);
            }

            if !ai.path.is_empty() {
                let target_pos = grid_transform.to_world(ai.path[0]);
                let delta = target_pos - ai_pos;

                if delta.x.abs() > delta.y.abs() {
                    if target_pos.x < ai_pos.x {
                        movement.add_move_left();
                    } else if target_pos.x > ai_pos.x {
                        movement.add_move_right();
                    }
                } else if target_pos.y > ai_pos.y {
                    movement.add_move_up();
                } else if target_pos.y < ai_pos.y {
                    movement.add_move_down();
                }
            }
        }
    }
}

fn find_nearest_runner<'a>(to_position: IVec2, runners: &'a Query<&GridTransform, With<Runner>>) -> Option<&'a GridTransform> {
    let distance = |t: &GridTransform| (t.translation - to_position).as_vec2().length();

    let nearest_distance = f32::MAX;
    let mut nearest_runner = None;
    for transform in runners.iter() {
        if distance(transform) < nearest_distance {
            nearest_runner = Some(transform);
        }
    }
    nearest_runner
}

#[allow(dead_code)]
pub fn debug_navmesh(commands: &mut Commands, core_assets: &Res<CoreAssets>, navmesh: &NavMesh) {
    for tile in &navmesh.tiles {
        commands.spawn_bundle(DebugNavTile::new(&core_assets.debug_atlas, tile, navmesh));
    }
}

#[derive(Debug, Default, Clone)]
pub struct NavMesh {
    pub tiles: Vec<NavTile>,
}

#[derive(Debug, Default, Clone)]
pub struct NavTile {
    pub pos: IVec2,
    pub connections: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
struct RawNavTile {
    pub pos: IVec2,
    pub connections: Vec<IVec2>,
}

impl RawNavTile {
    pub fn new(pos: IVec2) -> Self {
        Self {
            pos,
            connections: Vec::new(),
        }
    }
}

impl NavMesh {
    pub fn from_level(level: &LevelResource) -> Self {
        use EffectiveTileType::*;

        // build raw tiles
        let mut raw_nav_tiles: Vec<RawNavTile> = Vec::new();
        for y in 0..MAP_SIZE_HEIGHT {
            for x in 0..MAP_SIZE_WIDTH {
                let pos = IVec2::new(x, y);
                let tiles = level.around(pos);

                match tiles.on.behaviour {
                    Blocker => {}
                    None => {
                        let mut raw_tile = RawNavTile::new(pos);
                        NavMesh::add_nonblocking_raw_tile(&tiles.below, &mut raw_tile);
                        if tiles.below.behaviour == Blocker || tiles.below.behaviour == Ladder {
                            NavMesh::add_nonblocking_raw_tile(&tiles.left, &mut raw_tile);
                        }
                        if tiles.below.behaviour == Blocker || tiles.below.behaviour == Ladder {
                            NavMesh::add_nonblocking_raw_tile(&tiles.right, &mut raw_tile);
                        }
                        raw_nav_tiles.push(raw_tile);
                    }
                    Rope => {
                        let mut raw_tile = RawNavTile::new(pos);
                        NavMesh::add_nonblocking_raw_tile(&tiles.left, &mut raw_tile);
                        NavMesh::add_nonblocking_raw_tile(&tiles.right, &mut raw_tile);
                        NavMesh::add_nonblocking_raw_tile(&tiles.below, &mut raw_tile);
                        raw_nav_tiles.push(raw_tile);
                    }
                    Ladder => {
                        let mut raw_tile = RawNavTile::new(pos);
                        NavMesh::add_nonblocking_raw_tile(&tiles.left, &mut raw_tile);
                        NavMesh::add_nonblocking_raw_tile(&tiles.right, &mut raw_tile);
                        NavMesh::add_nonblocking_raw_tile(&tiles.above, &mut raw_tile);
                        NavMesh::add_nonblocking_raw_tile(&tiles.below, &mut raw_tile);
                        raw_nav_tiles.push(raw_tile);
                    }
                }
            }
        }

        // convert to actual nav tiles
        let mut nav_tiles: Vec<NavTile> = Vec::new();
        for raw_nav_tile in &raw_nav_tiles {
            let mut nav_tile = NavTile {
                pos: raw_nav_tile.pos,
                ..Default::default()
            };
            for connected_to_pos in &raw_nav_tile.connections {
                if let Some(connected_to_index) = raw_nav_tiles.iter().position(|x| x.pos == *connected_to_pos) {
                    nav_tile.connections.push(connected_to_index);
                }
            }
            nav_tiles.push(nav_tile);
        }

        Self { tiles: nav_tiles }
    }

    fn add_nonblocking_raw_tile(tile: &Tile, raw_tile: &mut RawNavTile) -> bool {
        use EffectiveTileType::*;
        if tile.behaviour != Blocker {
            raw_tile.connections.push(tile.pos);
            return true;
        }
        false
    }

    pub fn get_tile_index_by_pos(&self, pos: IVec2) -> Option<usize> {
        for (index, tile) in self.tiles.iter().enumerate() {
            if tile.pos == pos {
                return Some(index);
            }
        }

        Option::None
    }

    // nb: this used to be:
    //
    //pub fn get_tile_by_index<'a>(&'a self, index: usize) -> Option<&'a NavTile> {
    //
    // but apparently if the lifetime is tied to self, it can be ellided completely
    pub fn get_tile_by_index(&self, index: usize) -> Option<&NavTile> {
        if index < self.tiles.len() {
            return Some(&self.tiles[index]);
        }
        Option::None
    }
}

enum PathfindingError {
    Failed,
}

fn astar_pathfind(start_pos: IVec2, end_pos: IVec2, navmesh: &NavMesh) -> Result<Vec<IVec2>, PathfindingError> {
    let h = |index: usize| {
        let tile = navmesh.get_tile_by_index(index).unwrap();
        (end_pos - tile.pos).as_vec2().length()
    };

    if let Some(start_index) = navmesh.get_tile_index_by_pos(start_pos) {
        let mut came_from: HashMap<usize, usize> = HashMap::new();

        let mut open_set = vec![start_index];

        let mut gscore: HashMap<usize, f32> = HashMap::new();
        for i in 0..navmesh.tiles.len() {
            gscore.insert(i, f32::MAX);
        }
        gscore.insert(start_index, 0.0);

        let mut fscore: HashMap<usize, f32> = HashMap::new();
        for i in 0..navmesh.tiles.len() {
            fscore.insert(i, f32::MAX);
        }
        fscore.insert(start_index, h(start_index));

        while !open_set.is_empty() {
            open_set.sort_by(|a, b| {
                let dist_a = fscore[a];
                let dist_b = fscore[b];
                dist_a.partial_cmp(&dist_b).unwrap()
            });

            let current_index = open_set.swap_remove(0);
            let current_tile = navmesh.get_tile_by_index(current_index).unwrap();

            // are we at the end?
            if current_tile.pos == end_pos {
                let mut tile = navmesh.get_tile_by_index(current_index).unwrap();

                // rebuild the path
                let mut path = vec![tile.pos];
                let mut c = current_index;
                while came_from.contains_key(&c) {
                    c = came_from[&c];
                    tile = navmesh.get_tile_by_index(c).unwrap();
                    path.insert(0, tile.pos);
                }

                return Ok(path);
            }

            // add neighbors that have a better score
            // to the open_set (unless it's there already)
            for n in &current_tile.connections {
                let tentative_gscore = gscore[&current_index] + 1.0;
                if tentative_gscore < gscore[n] {
                    came_from.insert(*n, current_index);
                    gscore.insert(*n, tentative_gscore);
                    fscore.insert(*n, tentative_gscore + h(*n));
                    if !open_set.contains(n) {
                        open_set.push(*n);
                    }
                }
            }
        }
    }

    Err(PathfindingError::Failed)
}

#[derive(Bundle, Clone, Default)]
pub struct DebugNavTile {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

impl DebugNavTile {
    pub fn new(texture: &Handle<TextureAtlas>, tile: &NavTile, navmesh: &NavMesh) -> Self {
        let level_offset = Vec3::new(
            MAP_SIZE_HALF_WIDTH as f32 * TILE_SIZE_WIDTH * -1.0 + (TILE_SIZE_WIDTH / 2.0),
            TILE_SIZE_HEIGHT / 2.0,
            0.1,
        );

        let pos = Vec3::new(tile.pos.x as f32 * TILE_SIZE_WIDTH, tile.pos.y as f32 * TILE_SIZE_HEIGHT, 0.0) + level_offset;

        let mut top = false;
        let mut right = false;
        let mut down = false;
        let mut left = false;

        for connection in &tile.connections {
            let connected_tile = &navmesh.tiles[*connection];
            let delta = connected_tile.pos - tile.pos;

            let r = IVec2::new(1, 0);
            let l = IVec2::new(-1, 0);
            let t = IVec2::new(0, 1);
            let d = IVec2::new(0, -1);

            if delta == r {
                right = true
            } else if delta == l {
                left = true
            } else if delta == t {
                top = true
            } else if delta == d {
                down = true
            }
        }

        let index = DebugNavTile::map_to_index(top, right, down, left);

        Self {
            sprite: TextureAtlasSprite::new(index),
            texture_atlas: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(pos),
            ..Default::default()
        }
    }

    fn map_to_index(top: bool, right: bool, down: bool, left: bool) -> usize {
        let mut v: usize = 0;
        v |= if top { 1 } else { 0 };
        v |= if right { 2 } else { 0 };
        v |= if down { 4 } else { 0 };
        v |= if left { 8 } else { 0 };

        match v {
            // NONE
            0 => 15,
            // TOP
            1 => 0,
            // RIGHT
            2 => 1,
            // DOWN
            4 => 2,
            // LEFT
            8 => 3,
            // TOP | RIGHT
            3 => 4,
            // RIGHT | DOWN
            6 => 5,
            // DOWN | LEFT
            12 => 6,
            // LEFT | TOP
            9 => 7,
            // TOP | DOWN
            5 => 8,
            // RIGHT | LEFT
            10 => 9,
            // TOP | LEFT | DOWN | RIGHT
            15 => 10,
            // TOP | RIGHT | DOWN
            7 => 11,
            // RIGHT | DOWN | LEFT
            14 => 12,
            // TOP | DOWN | LEFT,
            13 => 13,
            // TOP | RIGHT | LEFT
            11 => 14,
            _ => 15,
        }
    }
}
