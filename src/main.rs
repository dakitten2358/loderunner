mod assets;
mod camera;
mod game;

use assets::PlaylistAsset;
use assets::{
    AnimAsset, AnimAssetPlugin, AssetLoading::ProgressCounter, AssetsLoading, LevelDataAsset, LevelDataAssetPlugin, LoadingPlugin,
    PlaylistAssetPlugin,
};
use bevy::asset::Handle;
use bevy::{asset::AssetServerSettings, prelude::*};
use camera::*;
use game::PlaylistState;
use std::fmt::Debug;
use std::hash::Hash;

pub const TILE_SIZE_WIDTH: f32 = 20.0;
pub const TILE_SIZE_HEIGHT: f32 = 22.0;
pub const TILE_PADDING_WIDTH: f32 = 3.0;
pub const TILE_PADDING_HEIGHT: f32 = 3.0;
pub const MAP_SIZE_WIDTH: i32 = 28;
pub const MAP_SIZE_HALF_WIDTH: i32 = MAP_SIZE_WIDTH / 2;
pub const MAP_SIZE_HEIGHT: i32 = 16;

pub trait BevyState: Component + Debug + Clone + Eq + Hash {}
impl<T: Component + Debug + Clone + Eq + Hash> BevyState for T {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Component)]
pub enum AppStates {
    InitialLoading,
    Testing,
    ChangeLevel,
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
    pub hole_atlas: Handle<TextureAtlas>,
    pub debug_atlas: Handle<TextureAtlas>,

    pub map_handles: Vec<Handle<LevelDataAsset>>,
    pub playlist_handles: Vec<Handle<PlaylistAsset>>,
    pub anim_handles: Vec<Handle<AnimAsset>>,
}

#[derive(Clone)]
pub struct StartupSettings {
    window_mode: bevy::window::WindowMode,
    playlist: String,
    asset_directory: String,
}

impl Default for StartupSettings {
    fn default() -> Self {
        Self {
            window_mode: bevy::window::WindowMode::BorderlessFullscreen,
            playlist: "playlists/classic.playlist".to_owned(),
            asset_directory: Default::default(),
        }
    }
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

    let startup_settings = get_startup_settings();
    window_descriptor.mode = startup_settings.window_mode;

    let mut app_builder = App::new();
    app_builder
        .insert_resource(startup_settings.clone())
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(window_descriptor)
        .insert_resource(AssetServerSettings {
            asset_folder: startup_settings.asset_directory,
        })
        .insert_resource(CoreAssets { ..Default::default() })
        .add_plugins(DefaultPlugins)
        .add_plugin(LevelDataAssetPlugin)
        .add_plugin(AnimAssetPlugin)
        .add_plugin(PlaylistAssetPlugin)
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
        .add_system_set(
            SystemSet::on_exit(InitialLoading)
                .with_system(core_asset_loading_onexit)
                .with_system(core_asset_loading_setup_playlist),
        )
        .add_plugin(game::GameplayPlugin { for_state: Testing })
        .add_system_set(SystemSet::on_enter(ChangeLevel).with_system(change_level));

    app_builder.run();
}

fn get_startup_settings() -> StartupSettings {
    let mut startup_settings = StartupSettings { ..Default::default() };
    let args: Vec<String> = std::env::args().collect();

    // asset dir
    let mut assets_dir = std::env::current_dir().expect("failed to get cwd");
    assets_dir.push("assets");
    startup_settings.asset_directory = assets_dir.into_os_string().into_string().unwrap();

    // window modes
    if args.contains(&String::from("-window")) {
        startup_settings.window_mode = bevy::window::WindowMode::Windowed;
    }
    if args.contains(&String::from("-fullscreen")) {
        startup_settings.window_mode = bevy::window::WindowMode::Fullscreen;
    }

    // startup playlist
    if args.contains(&String::from("-debug")) {
        startup_settings.playlist = "playlists/debug.playlist".to_owned();
    }

    startup_settings
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
    core_assets.tiles_atlas = texture_atlas_handle;

    // guard
    let guard_texture_handle = asset_server.load("guard.png");
    loading.add(&guard_texture_handle);
    let guard_texture_atlas = TextureAtlas::from_grid(guard_texture_handle, Vec2::new(20.0, 22.0), 11, 2);
    let guard_texture_atlas_handle = texture_atlases.add(guard_texture_atlas);
    core_assets.guard_atlas = guard_texture_atlas_handle;

    // runner
    let runner_texture_handle = asset_server.load("runner.png");
    loading.add(&runner_texture_handle);
    let runner_texture_atlas = TextureAtlas::from_grid(runner_texture_handle, Vec2::new(20.0, 22.0), 9, 2);
    let runner_texture_atlas_handle = texture_atlases.add(runner_texture_atlas);
    core_assets.runner_atlas = runner_texture_atlas_handle;

    // hole
    let hole_texture_handle = asset_server.load("hole.png");
    loading.add(&hole_texture_handle);
    let hole_texture_atlas = TextureAtlas::from_grid(hole_texture_handle, Vec2::new(20.0, 22.0), 9, 4);
    let hole_texture_atlas_handle = texture_atlases.add(hole_texture_atlas);
    core_assets.hole_atlas = hole_texture_atlas_handle;

    // load all the maps
    for level_data_handle in asset_server.load_folder("levels").expect("failed to load levels") {
        loading.add(&level_data_handle);
        core_assets.map_handles.push(level_data_handle.typed());
    }

    // load all the playlists
    for playlist_data_handle in asset_server.load_folder("playlists").expect("failed ot load playlists") {
        loading.add(&playlist_data_handle);
        core_assets.playlist_handles.push(playlist_data_handle.typed());
    }

    // load all the anims
    for anim_data_handle in asset_server.load_folder("anims").expect("fialed to load anims") {
        loading.add(&anim_data_handle);
        core_assets.anim_handles.push(anim_data_handle.typed());
    }

    // debug
    let debug_texture_handle = asset_server.load("debug.png");
    loading.add(&debug_texture_handle);
    let debug_texture_atlas = TextureAtlas::from_grid(debug_texture_handle, Vec2::new(20.0, 22.0), 11, 2);
    let debug_texture_atlas_handle = texture_atlases.add(debug_texture_atlas);
    core_assets.debug_atlas = debug_texture_atlas_handle;
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

fn core_asset_loading_setup_playlist(
    mut commands: Commands,
    startup_settings: Res<StartupSettings>,
    playlists: Res<Assets<PlaylistAsset>>,
) {
    let playlist_handle = playlists.get_handle(startup_settings.playlist.as_str());
    commands.insert_resource(PlaylistState::new(playlist_handle));
}

fn change_level(mut state: ResMut<State<AppStates>>) {
    state.set(AppStates::Testing).expect("failed to return to game state");
}
