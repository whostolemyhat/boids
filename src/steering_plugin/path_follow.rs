use std::f32;

use avian2d::prelude::*;
use bevy::color::palettes::css::*;

use bevy::prelude::*;

use crate::steering_plugin::{Ship, seek};
use crate::utils::set_magnitude;

#[derive(Resource)]
pub struct Path {
    pub points: Vec<Vec2>,
    pub radius: f32,
}
#[derive(Component)]
pub struct PathPoint;

pub fn on_start_path(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut path: ResMut<Path>,
) {
    let point_radius = 10.;
    let circle = Circle::new(point_radius);
    let yellow: Color = YELLOW_GREEN.into();

    let new_path = Path {
        points: vec![
            Vec2::new(-161., -160.),
            Vec2::new(162., -160.),
            Vec2::new(163., 160.),
            Vec2::new(-164., 160.),
        ],
        radius: 20.,
    };

    for point in &new_path.points {
        commands.spawn((
            Mesh2d(meshes.add(circle)),
            MeshMaterial2d(materials.add(ColorMaterial::from(yellow))),
            Transform::from_xyz(point.x, point.y, 0.),
            PathPoint,
        ));
    }

    *path = new_path;
}

fn get_normal_point(pos: Vec2, a: Vec2, b: Vec2) -> Vec2 {
    // vec pointing from a to pos
    let vec_a = pos - a;

    // vec pointing from a to b
    let mut vec_b = b - a;

    vec_b = vec_b.normalize();
    let dot = vec_a.dot(vec_b);

    vec_b *= dot;

    a + vec_b
}

pub fn path_follow_system(
    ship: Single<(&Position, &MaxLinearSpeed, &mut LinearVelocity), With<Ship>>,
    path: Res<Path>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    path.points.windows(2).for_each(|slice| {
        gizmos.line_2d(slice[0], slice[1], RED);
    });
    // join end to start
    gizmos.line_2d(path.points[path.points.len() - 1], path.points[0], RED);

    let (position, max_linear_speed, mut velocity) = ship.into_inner();

    let distance_ahead = 15.;
    let mut future = velocity.0;
    future = set_magnitude(future, distance_ahead);
    future += position.0;

    let mut biggest_gap = f32::INFINITY;
    let mut normal;
    let mut target = Vec2::new(0., 0.);

    for i in 0..path.points.len() {
        // wrap around
        let mut a = path.points[i];
        let mut b = path.points[(i + 1) % path.points.len()];

        let mut normal_point = get_normal_point(future, a, b);
        let mut dir = b - a;

        // if normal not in line segment, set to end point
        if normal_point.x < a.x.min(b.x)
            || normal_point.x > a.x.max(b.x)
            || normal_point.y < a.y.min(b.y)
            || normal_point.y > a.y.max(b.y)
        {
            normal_point = b;

            // get next line segment
            a = path.points[(i + 1) % path.points.len()];
            b = path.points[(i + 2) % path.points.len()];
            dir = b - a;
        }

        let distance = future.distance(normal_point);
        if distance < biggest_gap {
            biggest_gap = distance;
            normal = normal_point;

            let adjusted_dir = set_magnitude(dir, distance_ahead);
            target = normal + adjusted_dir;
        }
    }

    if biggest_gap > path.radius {
        gizmos.circle_2d(target, 15., TEAL);

        let steer = seek(&target, &velocity, max_linear_speed.0, position);
        velocity.0 += steer * time.delta_secs();
    }
}

pub fn clean_up_path(mut commands: Commands, query: Query<Entity, With<PathPoint>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}
