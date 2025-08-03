mod game_plugin;
mod input_plugin;
mod steering_plugin;
mod utils;

use bevy::prelude::*;

use crate::game_plugin::GamePlugin;
use crate::input_plugin::InputPlugin;
use crate::steering_plugin::SteeringPlugin;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((GamePlugin, InputPlugin, SteeringPlugin));
    }
}

fn main() {
    App::new().add_plugins(AppPlugin).run();
}
