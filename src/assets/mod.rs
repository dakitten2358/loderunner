#[allow(non_snake_case)]
pub mod AssetLoading;
#[allow(non_snake_case)]
pub mod LevelAsset;
pub mod animations;

pub use animations::{AnimAsset, AnimAssetPlugin};
pub use AssetLoading::{AssetsLoading, LoadingLabel, LoadingPlugin, Progress};
pub use LevelAsset::{LevelDataAsset, LevelDataAssetPlugin};
