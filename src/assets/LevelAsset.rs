use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

#[derive(Debug, TypeUuid)]
#[uuid = "4b05438e-39d9-4b57-9a3a-061ed489b5c9"]
pub struct LevelDataAsset {
    pub tiles: Vec<LevelTile>,
}

impl LevelDataAsset {
    pub fn new() -> Self {
        Self { tiles: Vec::new() }
    }
}

#[derive(Debug, Deserialize)]
pub struct LevelDataDiskAsset {
    pub rows: Vec<String>,
}

#[derive(Debug)]
pub struct LevelTile {
    pub position: IVec2,
    pub behaviour: TileType,
}

impl LevelTile {
    pub fn new(tile_type: TileType, position: IVec2) -> Self {
        Self {
            position,
            behaviour: tile_type,
        }
    }
}

#[derive(Debug)]
pub enum TileType {
    Brick,
    SolidBrick,
    Ladder,
    Rope,
    FalseBrick,
    HiddenLadder,
    Gold,
    Guard,
    Player,
}

#[derive(Default)]
pub struct LevelDataAssetLoader;

impl AssetLoader for LevelDataAssetLoader {
    fn load<'a>(&'a self, bytes: &'a [u8], load_context: &'a mut LoadContext) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            // random sleep for testing the loading screen
            //std::thread::sleep(std::time::Duration::from_millis((1+(rand::random::<u64>() % 24)) * 1000));
            let loaded_data = serde_json::de::from_slice::<LevelDataDiskAsset>(bytes)?;
            let mut level_data = LevelDataAsset::new();

            let mut y = loaded_data.rows.len() as i32 - 1;
            for row_data in &loaded_data.rows {
                for (x, ch) in row_data.chars().enumerate() {
                    let p = IVec2::new(x as i32, y);

                    match ch {
                        '#' => level_data.tiles.push(LevelTile::new(TileType::Brick, p)),
                        '@' => level_data.tiles.push(LevelTile::new(TileType::SolidBrick, p)),
                        'H' => level_data.tiles.push(LevelTile::new(TileType::Ladder, p)),
                        '-' => level_data.tiles.push(LevelTile::new(TileType::Rope, p)),
                        'X' => level_data.tiles.push(LevelTile::new(TileType::FalseBrick, p)),
                        'S' => level_data.tiles.push(LevelTile::new(TileType::HiddenLadder, p)),
                        '$' => level_data.tiles.push(LevelTile::new(TileType::Gold, p)),
                        '0' => level_data.tiles.push(LevelTile::new(TileType::Guard, p)),
                        '&' => level_data.tiles.push(LevelTile::new(TileType::Player, p)),
                        ' ' => {}
                        _ => {
                            println!("WARNING:  unexpected tile type: {} found!", ch);
                        }
                    }
                }
                y -= 1;
            }

            load_context.set_default_asset(LoadedAsset::new(level_data));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level"]
    }
}

pub struct LevelDataAssetPlugin;

impl Plugin for LevelDataAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<LevelDataAsset>();
        app.init_asset_loader::<LevelDataAssetLoader>();
    }
}
