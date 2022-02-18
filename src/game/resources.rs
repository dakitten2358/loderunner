use bevy::{math::IVec2, prelude::*};

use crate::assets::{
    LevelAsset::{LevelDataAsset, TileType},
    PlaylistAsset,
};

#[derive(Clone, Copy, PartialEq)]
pub enum EffectiveTileType {
    None,
    Blocker,
    Ladder,
    Rope,
}

impl Default for EffectiveTileType {
    fn default() -> Self {
        EffectiveTileType::None
    }
}

#[derive(Clone, Copy, Default)]
pub struct LevelTile {
    pub entity: Option<Entity>,
    pub behaviour: EffectiveTileType,
}

impl LevelTile {
    pub const NONE: Self = Self {
        entity: None,
        behaviour: EffectiveTileType::None,
    };
    pub const BLOCKER: Self = Self {
        entity: None,
        behaviour: EffectiveTileType::Blocker,
    };
}

pub struct LevelResource {
    tiles: Vec<LevelTile>,
    width: i32,
    height: i32,
    treasures: u32,
}

#[derive(Clone, Copy)]
pub struct TilesAround {
    pub above: Tile,
    pub below: Tile,

    pub on: Tile,

    pub left: Tile,
    pub right: Tile,

    pub below_left: Tile,
    pub below_right: Tile,
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub entity: Option<Entity>,
    pub pos: IVec2,
    pub behaviour: EffectiveTileType,
}

impl Tile {
    pub fn new(pos: IVec2, tile: &LevelTile) -> Self {
        Self {
            entity: tile.entity,
            pos,
            behaviour: tile.behaviour,
        }
    }
}

impl LevelResource {
    pub fn from_asset(level_asset: &LevelDataAsset) -> Self {
        let size = (level_asset.width * level_asset.height) as usize;
        let mut new_resource = Self {
            tiles: vec![LevelTile { ..Default::default() }; size],
            width: level_asset.width,
            height: level_asset.height,
            treasures: 0,
        };

        for tile in &level_asset.tiles {
            let index = new_resource.to_index(tile.position);
            new_resource.tiles[index].behaviour = match tile.behaviour {
                TileType::Brick | TileType::SolidBrick => EffectiveTileType::Blocker,
                TileType::Ladder => EffectiveTileType::Ladder,
                TileType::Rope => EffectiveTileType::Rope,
                _ => EffectiveTileType::None,
            };

            if tile.behaviour == TileType::Gold {
                new_resource.treasures += 1;
            }
        }

        new_resource
    }

    pub fn around(&self, pos: IVec2) -> TilesAround {
        let above = IVec2::new(0, 1);
        let below = IVec2::new(0, -1);
        let left = IVec2::new(-1, 0);
        let right = IVec2::new(1, 0);

        let below_left = IVec2::new(-1, -1);
        let below_right = IVec2::new(1, -1);

        TilesAround {
            above: Tile::new(pos + above, &self.at(pos + above)),
            below: Tile::new(pos + below, &self.at(pos + below)),
            left: Tile::new(pos + left, &self.at(pos + left)),
            right: Tile::new(pos + right, &self.at(pos + right)),
            on: Tile::new(pos, &self.at(pos)),
            below_left: Tile::new(pos + below_left, &self.at(pos + below_left)),
            below_right: Tile::new(pos + below_right, &self.at(pos + below_right)),
        }
    }

    fn at(&self, pos: IVec2) -> LevelTile {
        if !self.is_in_bounds(pos) {
            if pos.y >= self.height {
                return LevelTile::NONE;
            }
            return LevelTile::BLOCKER;
        }

        self.tiles[self.to_index(pos)]
    }

    pub fn set(&mut self, pos: IVec2, effective_tile: EffectiveTileType) {
        if self.is_in_bounds(pos) {
            let index = self.to_index(pos);
            self.tiles[index].behaviour = effective_tile;
        }
    }

    pub fn set_entity(&mut self, pos: IVec2, entity: Entity) {
        if self.is_in_bounds(pos) {
            let index = self.to_index(pos);
            self.tiles[index].entity = Some(entity);
        }
    }

    fn to_index(&self, pos: IVec2) -> usize {
        (pos.y * self.width + pos.x) as usize
    }

    fn is_in_bounds(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
    }

    pub fn treasure_count(&self) -> u32 {
        self.treasures
    }
}

#[derive(Default)]
pub struct LevelState {
    pub should_complete: bool,
    pub completed: bool,
}

pub struct PlaylistState {
    index: usize,
    playlist_handle: Handle<PlaylistAsset>,
}

impl PlaylistState {
    pub fn new(playlist: Handle<PlaylistAsset>) -> Self {
        Self {
            index: 0,
            playlist_handle: playlist,
        }
    }

    pub fn current_level<'a>(&self, playlists: &'a Res<Assets<PlaylistAsset>>) -> &'a str {
        let playlist = playlists.get(&self.playlist_handle).unwrap();
        playlist.levels[self.index].as_str()
    }

    pub fn next_level(&mut self, playlists: &Res<Assets<PlaylistAsset>>) {
        let playlist = playlists.get(&self.playlist_handle).unwrap();
        self.index = (self.index + 1) % playlist.levels.len();
    }
}
