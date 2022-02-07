use bevy::math::IVec2;

use crate::assets::LevelAsset::{LevelDataAsset, TileType};

#[derive(Clone, Copy, PartialEq)]
pub enum EffectiveTileType {
    None,
    Blocker,
    Ladder,
    Rope,
}

pub struct LevelResource {
    tiles: Vec<EffectiveTileType>,
    width: i32,
    height: i32,
}

#[derive(Clone, Copy)]
pub struct TilesAround {
    pub above: Tile,
    pub below: Tile,

    pub on: Tile,

    pub left: Tile,
    pub right: Tile,
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub pos: IVec2,
    pub behaviour: EffectiveTileType,
}

impl Tile {
    pub fn new(pos: IVec2, behaviour: EffectiveTileType) -> Self {
        Self { pos, behaviour }
    }
}

impl LevelResource {
    pub fn from_asset(level_asset: &LevelDataAsset) -> Self {
        let size = (level_asset.width * level_asset.height) as usize;
        let mut new_resource = Self {
            tiles: vec![EffectiveTileType::None; size],
            width: level_asset.width,
            height: level_asset.height,
        };

        for tile in &level_asset.tiles {
            let index = new_resource.to_index(tile.position);
            new_resource.tiles[index] = match tile.behaviour {
                TileType::Brick | TileType::SolidBrick => EffectiveTileType::Blocker,
                TileType::Ladder => EffectiveTileType::Ladder,
                TileType::Rope => EffectiveTileType::Rope,
                _ => EffectiveTileType::None,
            }
        }

        new_resource
    }

    pub fn around(&self, pos: IVec2) -> TilesAround {
        let above: IVec2 = IVec2::new(0, 1);
        let below: IVec2 = IVec2::new(0, -1);
        let left: IVec2 = IVec2::new(-1, 0);
        let right: IVec2 = IVec2::new(1, 0);

        TilesAround {
            above: Tile::new(pos + above, self.at(pos + above)),
            below: Tile::new(pos + below, self.at(pos + below)),
            left: Tile::new(pos + left, self.at(pos + left)),
            right: Tile::new(pos + right, self.at(pos + right)),
            on: Tile::new(pos, self.at(pos)),
        }
    }

    fn at(&self, pos: IVec2) -> EffectiveTileType {
        if !self.is_in_bounds(pos) {
            if pos.y >= self.height {
                return EffectiveTileType::None;
            }
            return EffectiveTileType::Blocker;
        }
        self.tiles[self.to_index(pos)]
    }

    #[allow(dead_code)]
    fn set(&mut self, pos: IVec2, effective_tile: EffectiveTileType) {
        if self.is_in_bounds(pos) {
            let index = self.to_index(pos);
            self.tiles[index] = effective_tile;
        }
    }

    fn to_index(&self, pos: IVec2) -> usize {
        (pos.y * self.width + pos.x) as usize
    }

    fn is_in_bounds(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
    }
}
