use bevy::prelude::*;

pub mod bundles;
pub mod components;
pub mod resources;

pub use bundles::*;
pub use components::*;
pub use resources::*;

mod gameplay;

use crate::BevyState;
use gameplay::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum GameplaySystem {
    Input,
}

pub struct GameplayPlugin<S: BevyState> {
    pub for_state: S,
}

impl<S: BevyState> Plugin for GameplayPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(self.for_state.clone()).with_system(init_gameplay));
        app.add_system_set(
            SystemSet::on_update(self.for_state.clone())
                .with_system(update_grid_transforms.before(GameplaySystem::Input))
                .with_system(player_input.label(GameplaySystem::Input))
                .with_system(apply_movement.after(GameplaySystem::Input)),
        );
        app.add_system_set(SystemSet::on_exit(self.for_state.clone()).with_system(exit_gameplay));
    }
}
