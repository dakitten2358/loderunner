use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, TypeUuid)]
#[uuid="4b05438e-39d9-4b57-9a3a-061ed489b5c9"]
pub struct LevelDataAsset
{
	pub rows: Vec<String>,
}

#[derive(Default)]
pub struct LevelDataAssetLoader;

impl AssetLoader for LevelDataAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
			let custom_asset = serde_json::de::from_slice::<LevelDataAsset>(&bytes)?;
            load_context.set_default_asset(LoadedAsset::new(custom_asset));
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
