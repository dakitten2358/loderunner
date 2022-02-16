use crate::{assets::AnimAsset, game::components::*};
use bevy::prelude::*;

#[derive(Bundle, Clone, Default)]
pub struct BrickBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    blocker: Blocker,
    pub burnable: Burnable,
    pub grid_transform: GridTransform,
    pub anim_data: Handle<AnimAsset>,
    pub sprite_anim: SpriteAnimator,
}

impl BrickBundle {
    pub fn new(texture: &Handle<TextureAtlas>, anim: &Handle<AnimAsset>, position: Vec3, offset: Vec3) -> Self {
        Self {
            sprite: TextureAtlasSprite::new(35),
            texture_atlas: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(position),
            grid_transform: GridTransform {
                offset,
                ..Default::default()
            },
            anim_data: anim.clone(),
            ..Default::default()
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct FalseBrickBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

impl FalseBrickBundle {
    pub fn new(texture: &Handle<TextureAtlas>, position: Vec3) -> Self {
        Self {
            sprite: TextureAtlasSprite::new(7),
            texture_atlas: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(position),
            ..Default::default()
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct GoldBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

impl GoldBundle {
    pub fn new(texture: &Handle<TextureAtlas>, position: Vec3) -> Self {
        Self {
            sprite: TextureAtlasSprite::new(4),
            texture_atlas: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(position),
            ..Default::default()
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct GuardBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub movement: Movement,
}

impl GuardBundle {
    pub fn new(texture: &Handle<TextureAtlas>, position: Vec3) -> Self {
        Self {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(position),
            ..Default::default()
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct HiddenLadderBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

impl HiddenLadderBundle {
    pub fn new(texture: &Handle<TextureAtlas>, position: Vec3) -> Self {
        Self {
            sprite: TextureAtlasSprite::new(2),
            texture_atlas: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(position),
            ..Default::default()
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct LadderBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

impl LadderBundle {
    pub fn new(texture: &Handle<TextureAtlas>, position: Vec3) -> Self {
        Self {
            sprite: TextureAtlasSprite::new(5),
            texture_atlas: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(position),
            ..Default::default()
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct PlayerBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub player_input: LocalPlayerInput,
    pub grid_transform: GridTransform,
    pub movement: Movement,
    pub anim_data: Handle<AnimAsset>,
    pub sprite_anim: SpriteAnimator,
    pub runner: Runner,
}

impl PlayerBundle {
    pub fn new(texture: &Handle<TextureAtlas>, anim: &Handle<AnimAsset>, position: Vec3, offset: Vec3) -> Self {
        Self {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(position),
            grid_transform: GridTransform {
                offset,
                ..Default::default()
            },
            anim_data: anim.clone(),
            ..Default::default()
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct RopeBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

impl RopeBundle {
    pub fn new(texture: &Handle<TextureAtlas>, position: Vec3) -> Self {
        Self {
            sprite: TextureAtlasSprite::new(6),
            texture_atlas: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(position),
            ..Default::default()
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct SolidBrickBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

impl SolidBrickBundle {
    pub fn new(texture: &Handle<TextureAtlas>, position: Vec3) -> Self {
        Self {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(position),
            ..Default::default()
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct SpriteEffectBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub anim_data: Handle<AnimAsset>,
    pub sprite_anim: SpriteAnimator,
    pub spawn_after: DespawnAfter,
}

impl SpriteEffectBundle {
    pub fn new(texture: &Handle<TextureAtlas>, anim: &Handle<AnimAsset>, position: Vec3, starting_anim: &str) -> Self {
        Self {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: texture.clone(),
            anim_data: anim.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(position),
            spawn_after: DespawnAfter::new(0.5),
            sprite_anim: SpriteAnimator::new(starting_anim),
            ..Default::default()
        }
    }
}
