use avian2d::prelude::*;
use bevy::prelude::ops::{cos, sin};
use bevy::prelude::*;
use bevy_rand::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use bevy::color::palettes::css::{YELLOW, YELLOW_GREEN};

use crate::steering_plugin::path_follow::clean_up_path;
use crate::{
    input_plugin::MousePos,
    utils::{adjust_magnitude, heading, set_magnitude},
};

mod evade;
mod path_follow;
mod pursue;

use evade::{evade_system, flee_system};
use path_follow::{Path, on_start_path, path_follow_system};
use pursue::{clean_up_pursue, on_start_pursue, pursue_system, reset_pursue_target};

#[derive(PartialEq, Debug, Hash, Eq, Clone, States, Default, Component)]
pub enum Behaviour {
    #[default]
    Seek,
    Arrive,
    Wander,
    Pursue,
    Flee,
    Evade,
    PathFollow,
}

#[derive(Resource)]
struct Debug(bool);

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EntropyPlugin::<WyRand>::default())
            .insert_resource(Debug(false))
            .insert_resource(Theta(PI / 2.))
            .insert_resource(Path {
                points: vec![],
                radius: 10.,
            })
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    seek_system.run_if(in_state(Behaviour::Seek)),
                    arrive_system.run_if(in_state(Behaviour::Arrive)),
                    wander_system.run_if(in_state(Behaviour::Wander)),
                    pursue_system.run_if(in_state(Behaviour::Pursue)),
                    flee_system.run_if(in_state(Behaviour::Flee)),
                    evade_system.run_if(in_state(Behaviour::Evade)),
                    path_follow_system.run_if(in_state(Behaviour::PathFollow)),
                    reset_pursue_target,
                    rotate_system,
                ),
            )
            .add_systems(OnEnter(Behaviour::Pursue), on_start_pursue)
            .add_systems(OnExit(Behaviour::Pursue), clean_up_pursue)
            .add_systems(OnEnter(Behaviour::Evade), on_start_pursue)
            .add_systems(OnExit(Behaviour::Evade), clean_up_pursue)
            .add_systems(OnEnter(Behaviour::PathFollow), on_start_path)
            .add_systems(OnExit(Behaviour::PathFollow), clean_up_path);
    }
}

#[derive(Resource)]
struct Theta(f32);

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct WanderTarget;

#[derive(Component)]
struct WanderRadius;

#[derive(Component)]
struct PursueTarget;

#[derive(Component)]
struct PursueOffset;

#[derive(Component)]
pub struct WrapEdges;

fn setup(
    mut commands: Commands,
    debug: Res<Debug>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if debug.0 {
        let wander_radius = 50.;
        let wander_circle = Circle::new(wander_radius);
        let yellowish: Color = YELLOW_GREEN.into();

        commands.spawn((
            Mesh2d(meshes.add(wander_circle)),
            MeshMaterial2d(materials.add(ColorMaterial::from(yellowish))),
            Transform::from_xyz(0., 0., 0.),
            WanderRadius,
            RigidBody::Kinematic,
        ));

        let target_radius = 5.;
        let circle = Circle::new(target_radius);
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
        let steer = seek(&mouse_pos.0, &velocity.0, max_linear_speed.0, &position.0);
        velocity.0 += steer * time.delta_secs();
    }
}

fn seek(target: &Vec2, velocity: &Vec2, max_linear_speed: f32, position: &Vec2) -> Vec2 {
    let mut to_cursor = target - position;
    to_cursor = set_magnitude(to_cursor, max_linear_speed);

    to_cursor - velocity
}

fn arrive_system(
    mut query: Query<(&mut LinearVelocity, &MaxLinearSpeed, &Position), With<Ship>>,
    target: Res<MousePos>,
) {
    for (mut velocity, max_linear_speed, position) in &mut query {
        let mut desired = target.0 - position.0;
        let d = desired.length();

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

#[allow(clippy::type_complexity)]
fn wander_system(
    query: Query<(&mut LinearVelocity, &MaxLinearSpeed, &Position), With<Ship>>,
    time: Res<Time>,
    mut rng: GlobalEntropy<WyRand>,
    debug: Res<Debug>,
    mut debug_query: Query<&mut Position, (With<WanderTarget>, Without<Ship>)>,
    mut debug_radius: Query<
        &mut Position,
        (With<WanderRadius>, Without<WanderTarget>, Without<Ship>),
    >,
    mut wander_theta: ResMut<Theta>,
) {
    let distance_ahead = 100.;
    let wander_radius = 50.;
    let displace = rng.random_range(-0.3..0.3);

    for (mut velocity, max_linear_speed, position) in query {
        let mut circle_pos = set_magnitude(velocity.0, distance_ahead);
        circle_pos += position.0;

        let heading = heading(velocity.0);
        let theta = wander_theta.0 + heading;

        // polar-cartesian conversion
        let circle_offset = Vec2 {
            x: wander_radius * cos(theta),
            y: wander_radius * sin(theta),
        };

        let target = circle_pos + circle_offset;

        // seek
        let steer = seek(&target, &velocity, max_linear_speed.0, position);
        velocity.0 += steer * time.delta_secs();

        // if debug draw circles
        if debug.0 {
            for mut circle_position in debug_query.iter_mut() {
                circle_position.0 = target;
            }
            for mut outer_circle in debug_radius.iter_mut() {
                outer_circle.0 = circle_pos;
            }
        }
        wander_theta.0 += displace;
    }
}

#[cfg(test)]
mod test {
    use crate::steering_plugin::seek;
    use bevy::prelude::*;

    #[test]
    fn seek_should_return_vec2() {
        let target = Vec2::new(10.0, -12.0);
        let velocity = Vec2::new(180.0, -123.0);
        let max_speed = 14.0;
        let position = Vec2::new(14.0, 18.0);

        assert_eq!(
            seek(&target, &velocity, max_speed, &position),
            Vec2::new(-181.8503, 109.12281)
        );

        let target = Vec2::new(-102.0, 130.0);

        assert_eq!(
            seek(&target, &velocity, max_speed, &position),
            Vec2::new(-190.07162, 132.72432)
        );

        let target = Vec2::new(10.0, -12.0);
        let max_speed = 2.0;

        assert_eq!(
            seek(&target, &velocity, max_speed, &position),
            Vec2::new(-180.26433, 121.01755)
        );
    }
}
