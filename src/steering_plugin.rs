use avian2d::prelude::*;
use bevy::prelude::ops::{cos, sin};
use bevy::prelude::*;
use bevy_rand::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use bevy::color::palettes::css::{BLUE, RED, YELLOW, YELLOW_GREEN};

use crate::{
    input_plugin::MousePos,
    utils::{adjust_magnitude, heading, set_magnitude},
};

#[derive(PartialEq, Debug, Hash, Eq, Clone, States, Default, Component)]
pub enum Behaviour {
    Seek,
    Arrive,
    #[default]
    Wander,
}

#[derive(Resource)]
struct Debug(bool);

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EntropyPlugin::<WyRand>::default())
            .insert_resource(Debug(true))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    seek_system.run_if(in_state(Behaviour::Seek)),
                    arrive_system.run_if(in_state(Behaviour::Arrive)),
                    wander_system.run_if(in_state(Behaviour::Wander)),
                    rotate_system,
                ),
            );
    }
}

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct ShipController;

#[derive(Component)]
struct WanderTarget;

fn setup(
    mut commands: Commands,
    debug: Res<Debug>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if debug.0 {
        let wander_radius = 25.;
        let circle = Circle::new(wander_radius);
        let yellow: Color = YELLOW.into();
        commands.spawn((
            Mesh2d(meshes.add(circle)),
            MeshMaterial2d(materials.add(ColorMaterial::from(yellow))),
            Transform::from_xyz(0., 0., 0.),
            WanderTarget,
            RigidBody::Kinematic,
        ));
    }
}

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

fn wander_system(
    query: Query<(&mut LinearVelocity, &MaxLinearSpeed, &Position), With<Ship>>,
    time: Res<Time>,
    mut rng: GlobalEntropy<WyRand>,
    debug: Res<Debug>,
    mut debug_query: Query<&mut Position, (With<WanderTarget>, Without<Ship>)>,
) {
    let distance_ahead = 80.;
    let wander_radius = 25.;
    let wander_theta = rng.random_range(-0.3..0.3);

    for (mut velocity, max_linear_speed, position) in query {
        let mut circle_pos: Vec2 = position.0;
        circle_pos = set_magnitude(circle_pos, distance_ahead);
        circle_pos += position.0;

        let heading = heading(velocity.0);
        // polar-cartesian conversion
        let circle_offset = Vec2 {
            x: wander_radius * cos(wander_theta + heading),
            y: wander_radius * sin(wander_theta + heading),
        };
        let target = circle_pos + circle_offset;

        println!("{heading}");

        // seek
        let mut steer = target - velocity.0;
        steer = set_magnitude(steer, max_linear_speed.0);
        velocity.0 += steer * time.delta_secs();

        // if debug draw circles
        if debug.0 {
            for mut position in debug_query.iter_mut() {
                position.0 = target;
            }
        }
    }
}

// TODO polar
// TODO alter angle for polar
// TODO convert to cartesian
