use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    input_plugin::MousePos,
    utils::{adjust_magnitude, heading, set_magnitude},
};

#[derive(PartialEq, Debug, Hash, Eq, Clone, States, Default)]
pub enum Behaviour {
    #[default]
    Seek,
    Arrive,
}

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                seek_system.run_if(in_state(Behaviour::Seek)),
                arrive_system.run_if(in_state(Behaviour::Arrive)),
                rotate_system,
            ),
        );
    }
}

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct ShipController;

fn rotate_system(
    mut query: Query<(&LinearVelocity, &MaxAngularSpeed, &mut Rotation), With<Ship>>,
    time: Res<Time>,
) {
    for (velocity, max_angular_speed, mut rotation) in &mut query {
        // rotate to face direction of travel
        *rotation = rotation.slerp(
            Rotation::radians(heading(velocity.0)),
            max_angular_speed.0 * time.delta_secs(),
        );
    }
}

// https://natureofcode.com/autonomous-agents/#example-51-seeking-a-target
fn seek_system(
    mut query: Query<(&mut LinearVelocity, &MaxLinearSpeed, &Position), With<Ship>>,
    mouse_pos: Res<MousePos>,
    time: Res<Time>,
) {
    for (mut velocity, max_linear_speed, position) in &mut query {
        let mut to_cursor = mouse_pos.0 - position.0;
        to_cursor = set_magnitude(to_cursor, max_linear_speed.0);

        let steer = to_cursor - velocity.0;

        velocity.0 += steer * time.delta_secs();
    }
}

fn arrive_system(
    mut query: Query<(&mut LinearVelocity, &MaxLinearSpeed, &Position), With<Ship>>,
    target: Res<MousePos>,
) {
    for (mut velocity, max_linear_speed, position) in &mut query {
        let mut desired = target.0 - position.0;
        let d = desired.length();

        // TODO arrival radius
        let arrival_radius = 60.;
        if d < arrival_radius {
            // val, original min, original max, new range min, new range max
            let adjusted_magnitude = adjust_magnitude(d, 0., max_linear_speed.0, 0., 100.);
            desired = set_magnitude(desired, adjusted_magnitude);
        } else {
            desired = set_magnitude(desired, max_linear_speed.0);
        }

        let steer = desired - velocity.0;

        velocity.0 += steer;
    }
}
