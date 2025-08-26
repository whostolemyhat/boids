use avian2d::prelude::*;
use bevy::prelude::ops::{atan2, cos, sin};
use bevy::prelude::*;
use bevy_rand::prelude::*;
use rand::Rng;
use std::f32::consts::PI;
use std::ops::Mul;

use bevy::color::palettes::css::{YELLOW, YELLOW_GREEN};

use crate::{
    input_plugin::MousePos,
    utils::{adjust_magnitude, heading, set_magnitude},
};

#[derive(PartialEq, Debug, Hash, Eq, Clone, States, Default, Component)]
pub enum Behaviour {
    #[default]
    Seek,
    Arrive,
    Wander,
    Pursue,
    Flee,
    Evade,
}

#[derive(Resource)]
struct Debug(bool);

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EntropyPlugin::<WyRand>::default())
            .insert_resource(Debug(false))
            .insert_resource(Theta(PI / 2.))
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
                    reset_pursue_target,
                    rotate_system,
                ),
            )
            .add_systems(OnEnter(Behaviour::Pursue), on_start_pursue)
            .add_systems(OnExit(Behaviour::Pursue), clean_up_pursue)
            .add_systems(OnEnter(Behaviour::Evade), on_start_pursue)
            .add_systems(OnExit(Behaviour::Evade), clean_up_pursue);
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

fn on_start_pursue(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: GlobalEntropy<WyRand>,
    ship: Single<&mut MaxLinearSpeed, With<Ship>>,
) {
    // add target
    let target_radius = 15.;
    let circle = Circle::new(target_radius);
    let yellow: Color = YELLOW_GREEN.into();
    let random_x = rng.random_range(-20.0..20.);
    let random_y = rng.random_range(-20.0..20.);

    let offset_radius = 5.;
    let offset_circle = Circle::new(offset_radius);
    let offset_colour: Color = YELLOW.into();

    commands.spawn((
        Mesh2d(meshes.add(circle)),
        MeshMaterial2d(materials.add(ColorMaterial::from(yellow))),
        Transform::from_xyz(0., 0., 0.),
        PursueTarget,
        RigidBody::Kinematic,
        LinearVelocity(Vec2::new(random_x * 10., random_y * 10.)),
        WrapEdges,
        MaxLinearSpeed(200.0),
        CollisionEventsEnabled,
        Collider::circle(target_radius),
    ));

    commands.spawn((
        Mesh2d(meshes.add(offset_circle)),
        MeshMaterial2d(materials.add(ColorMaterial::from(offset_colour))),
        Transform::from_xyz(0., 0., 0.),
        PursueOffset,
        RigidBody::Kinematic,
    ));

    let mut ship = ship.into_inner();
    ship.0 = 300.;
}

#[allow(clippy::type_complexity)]
fn pursue_system(
    ship_query: Single<
        (&mut LinearVelocity, &MaxLinearSpeed, &Position),
        (With<Ship>, Without<PursueTarget>),
    >,
    target_query: Single<(&Position, &LinearVelocity), (With<PursueTarget>, Without<Ship>)>,
    offset_query: Single<&mut Position, (With<PursueOffset>, Without<Ship>, Without<PursueTarget>)>,
    time: Res<Time>,
) {
    let (mut velocity, max_speed, position) = ship_query.into_inner();
    let (target_pos, target_velocity) = target_query.into_inner();

    // adjust this based on speed of target
    let distance_ahead = 25.;

    // target = circle
    // offset = place ship needs to aim for
    // heading() ends up 90deg off for target
    let target_heading = atan2(target_velocity.0.y, target_velocity.0.x);
    let mut offset_pos = set_magnitude(target_velocity.0, distance_ahead);
    offset_pos += target_pos.0;

    let mut target_offset = Vec2 {
        x: distance_ahead * cos(target_heading),
        y: distance_ahead * sin(target_heading),
    };
    target_offset += offset_pos;
    let mut offset = offset_query.into_inner();
    offset.0 = target_offset;

    let to_target = target_offset - position.0;
    let steer = seek(&to_target, &velocity, max_speed.0, position);
    velocity.0 += steer * time.delta_secs();
}

fn clean_up_pursue(
    mut commands: Commands,
    target_query: Single<Entity, With<PursueTarget>>,
    offset_query: Single<Entity, With<PursueOffset>>,
    ship: Single<&mut MaxLinearSpeed, With<Ship>>,
) {
    let target = target_query.into_inner();
    commands.entity(target).despawn();
    let offset = offset_query.into_inner();
    commands.entity(offset).despawn();

    // reset max speed
    let mut ship = ship.into_inner();
    ship.0 = 200.;
}

// call on ship/target collision
#[allow(clippy::complexity)]
fn reset_pursue_target(
    mut collision_event_reader: EventReader<CollisionStarted>,
    target_query: Single<(&mut Position, &mut LinearVelocity), (With<PursueTarget>, Without<Ship>)>,
    mut rng: GlobalEntropy<WyRand>,
) {
    let (mut position, mut velocity) = target_query.into_inner();

    for CollisionStarted(_first, _second) in collision_event_reader.read() {
        println!("Caught!");

        let random_x = rng.random_range(-20.0..20.);
        let random_y = rng.random_range(-20.0..20.);

        let pos_x = rng.random_range(-300.0..300.);
        let pos_y = rng.random_range(-300.0..300.);

        velocity.0 = Vec2::new(random_x * 10., random_y * 10.);
        position.0 = Vec2::new(pos_x, pos_y);
    }
}

#[allow(clippy::complexity)]
fn evade_system(
    ship_query: Single<
        (&mut LinearVelocity, &MaxLinearSpeed, &Position),
        (With<Ship>, Without<PursueTarget>),
    >,
    target_query: Single<(&Position, &LinearVelocity), (With<PursueTarget>, Without<Ship>)>,
    offset_query: Single<&mut Position, (With<PursueOffset>, Without<Ship>, Without<PursueTarget>)>,
    time: Res<Time>,
) {
    // TODO refactor
    // pursuit, but * -1
    let (mut velocity, max_speed, position) = ship_query.into_inner();
    let (target_pos, target_velocity) = target_query.into_inner();

    // adjust this based on speed of target
    let distance_ahead = 25.;

    // target = circle
    // offset = place ship needs to aim for
    // heading() ends up 90deg off for target
    let target_heading = atan2(target_velocity.0.y, target_velocity.0.x);
    let mut offset_pos = set_magnitude(target_velocity.0, distance_ahead);
    offset_pos += target_pos.0;

    let mut target_offset = Vec2 {
        x: distance_ahead * cos(target_heading),
        y: distance_ahead * sin(target_heading),
    };
    target_offset += offset_pos;
    let mut offset = offset_query.into_inner();
    offset.0 = target_offset;

    let to_target = target_offset - position.0;
    let steer = seek(&to_target, &velocity, max_speed.0, position).mul(-1.);
    velocity.0 += steer * time.delta_secs();
}

fn flee_system(
    mut query: Query<(&mut LinearVelocity, &MaxLinearSpeed, &Position), With<Ship>>,
    mouse_pos: Res<MousePos>,
    time: Res<Time>,
) {
    // seek but in opposite direction
    for (mut velocity, max_linear_speed, position) in &mut query {
        let to_cursor = mouse_pos.0 - position.0;

        let steer = seek(&to_cursor, &velocity, max_linear_speed.0, position).mul(-1.);

        velocity.0 += steer * time.delta_secs();
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
