use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

#[derive(Debug, TypeUuid, Deserialize)]
#[uuid = "d973d6ed-c61e-4de1-bacc-63960676240e"]
pub struct PlaylistAsset {
    pub levels: Vec<String>,
}

#[derive(Default)]
pub struct PlaylistAssetLoader;

impl AssetLoader for PlaylistAssetLoader {
    fn load<'a>(&'a self, bytes: &'a [u8], load_context: &'a mut LoadContext) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            // random sleep for testing the loading screen
            //std::thread::sleep(std::time::Duration::from_millis((1+(rand::random::<u64>() % 24)) * 1000));
            let loaded_data = serde_json::de::from_slice::<PlaylistAsset>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(loaded_data));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["playlist"]
    }
}

pub struct PlaylistAssetPlugin;

impl Plugin for PlaylistAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<PlaylistAsset>();
        app.init_asset_loader::<PlaylistAssetLoader>();
    }
}
