mod assets;
mod camera;
mod game;

use assets::LevelAsset::TileType;
use assets::{AssetLoading::ProgressCounter, AssetsLoading, LevelDataAsset, LevelDataAssetPlugin, LoadingPlugin};
use bevy::asset::Handle;
use bevy::{asset::AssetServerSettings, prelude::*};
use camera::*;
use game::bundles::*;

const TILE_SIZE_WIDTH: f32 = 20.0;
const TILE_SIZE_HEIGHT: f32 = 22.0;
const TILE_PADDING_WIDTH: f32 = 3.0;
const TILE_PADDING_HEIGHT: f32 = 3.0;
const MAP_SIZE_WIDTH: i32 = 28;
const MAP_SIZE_HALF_WIDTH: i32 = MAP_SIZE_WIDTH / 2;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Component)]
pub enum AppStates {
    InitialLoading,
    Testing,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum StartupSystems {
    Boot,
    LoadCoreAssets,
    SetupTest,
}

#[derive(Clone, Default)]
pub struct CoreAssets {
    pub tiles_atlas: Handle<TextureAtlas>,
    pub guard_atlas: Handle<TextureAtlas>,
    pub runner_atlas: Handle<TextureAtlas>,
}

#[derive(Component)]
pub struct AnimationTimer(Timer);

fn main() {
    use AppStates::*;
    use StartupSystems::*;

    let mut window_descriptor = WindowDescriptor {
        title: "Loderunner".to_string(),
        width: 1280.,
        height: 720.,
        vsync: true,
        mode: bevy::window::WindowMode::BorderlessFullscreen,
        ..Default::default()
    };

    let args: Vec<String> = std::env::args().collect();
    if args.contains(&String::from("-window")) {
        window_descriptor.mode = bevy::window::WindowMode::Windowed;
    }

    let assets_directory = get_assets_directory();
    let mut app_builder = App::new();
    app_builder
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(window_descriptor)
        .insert_resource(AssetServerSettings {
            asset_folder: assets_directory.into_os_string().into_string().unwrap(),
        })
        .insert_resource(CoreAssets { ..Default::default() })
        .add_plugins(DefaultPlugins)
        .add_plugin(LevelDataAssetPlugin)
        .add_plugin(ScalableOrthographicCameraPlugin)
        .add_plugin(LoadingPlugin {
            loading_state: InitialLoading,
            next_state: Testing,
        })
        .add_state(InitialLoading)
        .add_startup_system(boot.label(Boot))
        .add_system_set(
            SystemSet::on_enter(InitialLoading)
                .with_system(load_core_assets)
                .with_system(core_asset_loading_onenter),
        )
        .add_system_set(SystemSet::on_update(InitialLoading).with_system(core_asset_loading))
        .add_system_set(SystemSet::on_exit(InitialLoading).with_system(core_asset_loading_onexit))
        .add_system_set(SystemSet::on_enter(Testing).with_system(setup_test))
        .add_system_set(
            SystemSet::on_update(Testing)
                .with_system(animate_sprite_system)
                .with_system(game::test_input)
        );

    app_builder.run();
}

fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

fn get_assets_directory() -> std::path::PathBuf {
    let mut assets_dir = std::env::current_dir().expect("failed to get cwd");
    assets_dir.push("assets");
    assets_dir
}

fn boot(mut commands: Commands) {
    commands.spawn_bundle(ScalableOrthographicCameraBundle::new(640.0, 360.0));
}

fn load_core_assets(
    asset_server: Res<AssetServer>,
    mut core_assets: ResMut<CoreAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut loading: ResMut<AssetsLoading>,
) {
    // tiles
    let texture_handle = asset_server.load("tiles.png");
    loading.add(&texture_handle);

    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle,
        Vec2::new(TILE_SIZE_WIDTH, TILE_SIZE_HEIGHT),
        3,
        3,
        Vec2::new(TILE_PADDING_WIDTH, TILE_PADDING_HEIGHT),
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    loading.add(&texture_atlas_handle);
    core_assets.tiles_atlas = texture_atlas_handle;

    // guard
    let guard_texture_handle = asset_server.load("guard.png");
    loading.add(&guard_texture_handle);
    let guard_texture_atlas = TextureAtlas::from_grid(guard_texture_handle, Vec2::new(20.0, 22.0), 11, 2);
    let guard_texture_atlas_handle = texture_atlases.add(guard_texture_atlas);
    loading.add(&guard_texture_atlas_handle);
    core_assets.guard_atlas = guard_texture_atlas_handle;

    // runner
    let runner_texture_handle = asset_server.load("runner.png");
    loading.add(&runner_texture_handle);
    let runner_texture_atlas = TextureAtlas::from_grid(runner_texture_handle, Vec2::new(20.0, 22.0), 10, 2);
    let runner_texture_atlas_handle = texture_atlases.add(runner_texture_atlas);
    loading.add(&runner_texture_atlas_handle);
    core_assets.runner_atlas = runner_texture_atlas_handle;

    let level_data_handle: Handle<LevelDataAsset> = asset_server.load("levels/classic/001.level");
    loading.add(&level_data_handle);
}

#[derive(Component)]
struct LoadingScreenComponent;

#[derive(Component)]
struct LoadingScreenProgressBar;

fn core_asset_loading_onenter(mut commands: Commands, asset_server: Res<AssetServer>) {
    let progress_position = Vec3::new(0.0, TILE_SIZE_HEIGHT * 2.0, 0.0);

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("progress_bar_bg.png"),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(progress_position),
            ..Default::default()
        })
        .insert(LoadingScreenComponent);

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("progress_bar_fg.png"),
            transform: Transform::from_scale(Vec3::new(0.0, 1.0, 1.0)).with_translation(progress_position),
            ..Default::default()
        })
        .insert(LoadingScreenComponent)
        .insert(LoadingScreenProgressBar);
}

fn core_asset_loading(counter: Res<ProgressCounter>, mut progress_bars: Query<&mut Transform, With<LoadingScreenProgressBar>>) {
    let progress = counter.progress();
    let progress_percent: f32 = progress.into();

    for mut progress_bar in progress_bars.iter_mut() {
        progress_bar.scale.x = progress_percent.clamp(0.0, 1.0);
    }
}

fn core_asset_loading_onexit(mut commands: Commands, to_despawn: Query<Entity, With<LoadingScreenComponent>>) {
    for entity in to_despawn.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_test(commands: Commands, core_assets: Res<CoreAssets>, level_datas: Res<Assets<LevelDataAsset>>) {
    let level_data = level_datas.get("levels/classic/001.level").unwrap();
    spawn_level_entities(commands, core_assets, level_data);
}

#[derive(Component)]
pub struct LevelSpecificComponent;

fn spawn_level_entities(mut commands: Commands, core_assets: Res<CoreAssets>, level_data: &LevelDataAsset) {
    let tiles_atlas = &core_assets.tiles_atlas;
    let guard_atlas = &core_assets.guard_atlas;
    let runner_atlas = &core_assets.runner_atlas;

    let level_offset = Vec3::new(MAP_SIZE_HALF_WIDTH as f32 * TILE_SIZE_WIDTH * -1.0, TILE_SIZE_HEIGHT / 2.0, 0.0);
    for tile in &level_data.tiles {
        let pos = Vec3::new(
            tile.position.x as f32 * TILE_SIZE_WIDTH,
            tile.position.y as f32 * TILE_SIZE_HEIGHT,
            0.0,
        ) + level_offset;

        match tile.behaviour {
            TileType::Brick => commands.spawn_bundle(BrickBundle::new(tiles_atlas, pos)),
            TileType::FalseBrick => commands.spawn_bundle(FalseBrickBundle::new(tiles_atlas, pos)),
            TileType::Gold => commands.spawn_bundle(GoldBundle::new(tiles_atlas, pos)),
            TileType::Guard => commands.spawn_bundle(GuardBundle::new(guard_atlas, pos)),
            TileType::HiddenLadder => commands.spawn_bundle(HiddenLadderBundle::new(tiles_atlas, pos)),
            TileType::Ladder => commands.spawn_bundle(LadderBundle::new(tiles_atlas, pos)),
            TileType::Player => commands.spawn_bundle(PlayerBundle::new(runner_atlas, pos)),
            TileType::Rope => commands.spawn_bundle(RopeBundle::new(tiles_atlas, pos)),
            TileType::SolidBrick => commands.spawn_bundle(SolidBrickBundle::new(tiles_atlas, pos)),
        }
        .insert(LevelSpecificComponent);
    }
}
