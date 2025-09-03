use avian2d::prelude::*;
use bevy::color::palettes::css::{YELLOW, YELLOW_GREEN};
use bevy::prelude::ops::{atan2, cos, sin};
use bevy::prelude::*;
use bevy_rand::prelude::*;
use rand::Rng;

use crate::steering_plugin::{PursueOffset, PursueTarget, Ship, WrapEdges, seek};
use crate::utils::set_magnitude;

// call on ship/target collision
#[allow(clippy::complexity)]
pub fn reset_pursue_target(
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

pub fn clean_up_pursue(
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

#[allow(clippy::type_complexity)]
pub fn pursue_system(
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

pub fn on_start_pursue(
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
