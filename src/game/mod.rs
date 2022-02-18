use bevy::prelude::*;

mod bundles;
mod components;
mod resources;

mod animations;
mod gameplay;
mod movement;

use crate::BevyState;
use animations::{animate_sprites, animgraph_brick, animgraph_runner};
use gameplay::*;
use movement::{apply_falling, apply_movement, build_overlaps};
pub use resources::PlaylistState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum GameplaySystem {
    Input,
    Movement,
    Overlaps,
    Animation,
}

pub struct GameplayPlugin<S: BevyState> {
    pub for_state: S,
}

impl<S: BevyState> Plugin for GameplayPlugin<S> {
    fn build(&self, app: &mut App) {
        use GameplaySystem::*;

        app.add_system_set(SystemSet::on_enter(self.for_state.clone()).with_system(init_gameplay));
        app.add_system_set(
            SystemSet::on_update(self.for_state.clone())
                .with_system(update_grid_transforms.before(Input))
                .with_system(player_input.label(Input))
                .with_system(start_burns.after(Input).before(Movement))
                .with_system(apply_burnables.after(Input).before(Movement))
                .with_system(apply_falling.before(Movement).after(Input))
                .with_system(apply_movement.label(Movement).after(Input))
                .with_system(build_overlaps.label(Overlaps).after(Movement))
                .with_system(gold_pickups.after(Overlaps))
                .with_system(animgraph_runner.before(Animation).after(Movement))
                .with_system(animgraph_brick.before(Animation).after(Movement))
                .with_system(animate_sprites.label(Animation).after(Movement))
                .with_system(pending_despawns.after(Input).after(Movement).after(Animation))
                .with_system(completion.after(Input).after(Movement).after(Overlaps).after(Animation))
                .with_system(next_level.after(Input).after(Movement).after(Overlaps).after(Animation)),
        );
        app.add_system_set(SystemSet::on_exit(self.for_state.clone()).with_system(exit_gameplay));
    }
}
