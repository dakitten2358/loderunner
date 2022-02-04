use bevy::{asset::AssetServerSettings, prelude::*};

mod camera;
use camera::*;

mod assets;
use assets::{LevelDataAsset, LevelDataAssetPlugin};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum StartupSystems {
    Boot,
    LoadCoreAssets,
    SetupTest,
}

#[derive(Clone, Default)]
pub struct CoreAssets {
    pub tiles_atlas: Handle<TextureAtlas>,
}

fn main() {
    use StartupSystems::*;

    let assets_directory = get_assets_directory();
    let mut app_builder = App::new();
    app_builder
        .insert_resource(WindowDescriptor {
            title: "Loderunner".to_string(),
            width: 1920.,
            height: 1080.,
            vsync: true,
            ..Default::default()
        })
        .insert_resource(AssetServerSettings {
            asset_folder: assets_directory.into_os_string().into_string().unwrap(),
        })
        .insert_resource(CoreAssets { ..Default::default() })
        .add_plugins(DefaultPlugins)
        .add_plugin(LevelDataAssetPlugin)
        .add_plugin(ScalableOrthographicCameraPlugin)
        .add_startup_system(boot.label(Boot))
        .add_startup_system(load_core_assets.label(LoadCoreAssets).after(Boot))
        .add_startup_system(setup_test.label(SetupTest).after(LoadCoreAssets))
        .add_system(animate_sprite_system);
    app_builder.run();
}

fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
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
    mut level_datas: ResMut<Assets<LevelDataAsset>>,
) {
    let texture_handle = asset_server.load("tiles.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(22.0, 20.0), 3, 3);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    core_assets.tiles_atlas = texture_atlas_handle;

    let level_data_handle: Handle<LevelDataAsset> = asset_server.load("levels/classic/001.level");
    let level_data = level_datas.get(level_data_handle).unwrap();
    for row in &level_data.rows {
        for ch in row.chars() {
            match ch {
                '$' => {
                    println!("found money");
                }
                '0' => {
                    println!("found enemy");
                }
                ' ' => {
                    
                }
                'H' => {
                    println!("found ladder");
                }
                'S' => {
                    println!("found secret ladder");
                }
                '#' => {
                    println!("found breakable floor");
                }
                '@' => {
                    println!("found solid floor");
                }
                '&' => {
                    println!("found player");
                }
                '-' => {
                    println!("found rope");
                }
                _ => {
                    println!("found unexpected {}", ch);
                }
            }
        }
    }
}

fn setup_test(mut commands: Commands, core_assets: Res<CoreAssets>) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: core_assets.tiles_atlas.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true));

    let corner_positions = vec![
        Vec3::new(22.0 * 13.0, 20.0 * 16.0, 0.0),
        Vec3::new(22.0 * -13.0, 20.0 * 16.0, 0.0),
        Vec3::new(22.0 * 13.0, 20.0 * 0.0, 0.0),
        Vec3::new(22.0 * -13.0, 20.0 * 0.0, 0.0),
    ];
    for corner_position in corner_positions {
        commands.spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(1),
            texture_atlas: core_assets.tiles_atlas.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(corner_position),
            ..Default::default()
        });
    }
}
