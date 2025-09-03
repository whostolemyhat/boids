use avian2d::prelude::*;
use bevy::prelude::ops::{atan2, cos, sin};
use bevy::prelude::*;
use std::ops::Mul;

use crate::input_plugin::MousePos;
use crate::steering_plugin::{PursueOffset, PursueTarget, Ship, seek};
use crate::utils::set_magnitude;

#[allow(clippy::complexity)]
pub fn evade_system(
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

pub fn flee_system(
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
