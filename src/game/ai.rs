use crate::{game::resources::LevelResource, MAP_SIZE_HEIGHT, MAP_SIZE_WIDTH};
use bevy::prelude::*;

use super::resources::{EffectiveTileType, Tile};
use std::collections::HashMap;

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
                    None | Rope => {
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

    fn add_nonblocking_raw_tile(tile: &Tile, raw_tile: &mut RawNavTile) {
        use EffectiveTileType::*;
        if tile.behaviour != Blocker {
            raw_tile.connections.push(tile.pos);
        }
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

pub fn test_pathfind(start_pos: IVec2, end_pos: IVec2, navmesh: &NavMesh) {
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
                // rebuild the path
                let mut path = vec![current_index];
                let mut c = current_index;
                while came_from.contains_key(&c) {
                    c = came_from[&c];
                    path.insert(0, c);
                }

                // print the path out
                for (index, p) in path.iter().enumerate() {
                    let p_tile = navmesh.get_tile_by_index(*p).unwrap();
                    println!("{}: ({},{})", index, p_tile.pos.x, p_tile.pos.y);
                }

                return;
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
        println!("failed to find path");
    }
}
