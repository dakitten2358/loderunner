use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, TypeUuid, Deserialize)]
#[uuid = "cf714ee8-e1ae-4ace-8467-a1ba1cf357ab"]
pub struct AnimAsset {
    pub fps: f32,
    pub sequence: HashMap<String, Vec<usize>>,
}

impl Default for AnimAsset {
    fn default() -> Self {
        Self {
            fps: 1.0,
            sequence: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct AnimAssetLoader;

impl AssetLoader for AnimAssetLoader {
    fn load<'a>(&'a self, bytes: &'a [u8], load_context: &'a mut LoadContext) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            // random sleep for testing the loading screen
            //std::thread::sleep(std::time::Duration::from_millis((1+(rand::random::<u64>() % 24)) * 1000));
            let loaded_data = serde_json::de::from_slice::<AnimAsset>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(loaded_data));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["anim"]
    }
}

pub struct AnimAssetPlugin;

impl Plugin for AnimAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<AnimAsset>();
        app.init_asset_loader::<AnimAssetLoader>();
    }
}
