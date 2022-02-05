use bevy::prelude::*;

pub mod bundles;
pub mod components;
pub use bundles::*;
pub use components::*;

pub fn test_input(keyboard_input: Res<Input<KeyCode>>, gamepads: Res<Gamepads>, axes: Res<Axis<GamepadAxis>>, mut players: Query<&mut Transform, With<LocalPlayerInput>>) {
	if keyboard_input.pressed(KeyCode::D)
	{
		for mut player in players.iter_mut() {
			player.translation = player.translation + Vec3::new(1.0, 0.0,0.0);
		}
	}
}
