use crate::assets::AnimAsset;
use bevy::prelude::*;

use super::{components::*, resources::LevelResource};

pub fn animgraph_runner(level: Res<LevelResource>, mut runners: Query<(&Movement, &GridTransform, &mut SpriteAnimator), With<Runner>>) {
    use crate::game::resources::EffectiveTileType::*;

    for (movement, transform, mut animator) in runners.iter_mut() {
        // initialized?
        if animator.animation_name == Option::None {
            animator.animation_name = Some("runRight".to_string());
        }

        // animations only active when we're moving, otherwise we stay at whatever frame we were at
        animator.active = movement.velocity != Vec3::ZERO;
        if animator.active {
            let tiles = level.around(transform.translation);

            if movement.is_falling() {
                if movement.get_fall_direction() >= 0.0 {
                    animator.switch("fallRight")
                } else {
                    animator.switch("fallLeft")
                }
            } else if tiles.on.behaviour == Rope && movement.velocity.x > 0.0 {
                animator.switch("ropeRight")
            } else if tiles.on.behaviour == Rope && movement.velocity.x < 0.0 {
                animator.switch("ropeLeft")
            } else if movement.velocity.x > 0.0 {
                animator.switch("runRight")
            } else if movement.velocity.x < 0.0 {
                animator.switch("runLeft")
            } else if movement.velocity.y != 0.0 {
                animator.switch("runUpDown")
            }
        }
    }
}

pub fn animgraph_brick(mut query: Query<(&Burnable, &mut SpriteAnimator)>) {
    use BurnState::*;
    for (burnable, mut animator) in query.iter_mut() {
        animator.active = true;
        match burnable.get_state() {
            Burning => animator.switch("burning"),
            Rebuilding => animator.switch("rebuilding"),
            NotBurning => animator.switch("default"),
            _ => {}
        }
    }
}

pub fn animate_sprites(
    time: Res<Time>,
    animations: Res<Assets<AnimAsset>>,
    mut animated_sprites: Query<(&mut TextureAtlasSprite, &mut SpriteAnimator, &Handle<AnimAsset>)>,
) {
    for (mut sprite, mut anim, anim_handle) in animated_sprites.iter_mut() {
        if !anim.active {
            continue;
        }

        let anim_data = animations.get(anim_handle).unwrap();
        let frame_time = 1.0 / anim_data.fps;

        if let Some(animation_name) = &anim.animation_name {
            let anim_sequence = &anim_data.sequence[animation_name];
            anim.elapsed += time.delta_seconds();

            while anim.elapsed > frame_time {
                anim.frame_index = anim_sequence.next_frame(anim.frame_index);
                anim.elapsed -= frame_time;
                sprite.index = anim_sequence.frames[anim.frame_index];
            }
        }
    }
}
