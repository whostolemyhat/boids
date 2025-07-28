use avian2d::prelude::*;
use bevy::color::palettes::css::{BLUE, RED};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPlugins, PhysicsPlugins::default()));
    }
}

fn main() {
    App::new()
        .add_plugins(AppPlugin)
        .add_systems(Startup, setup)
        .insert_resource(MousePos(Vec2::new(0.0, 0.0)))
        .add_systems(
            Update,
            (mouse_cursor_system, move_target, move_ship, clamp_edges),
        )
        .run();
}

#[derive(Resource, Default)]
struct MousePos(Vec2);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Target;

#[derive(Component)]
struct Ship;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d, MainCamera));

    let target_size = 15.0;
    let circle = Circle::new(target_size);
    let red: Color = RED.into();

    let ship_size = 15.0;
    let blue: Color = BLUE.into();
    let triangle = Triangle2d::new(
        Vec2::Y * ship_size,
        Vec2::new(-ship_size, -ship_size),
        Vec2::new(ship_size, -ship_size),
    );

    commands.spawn((
        Mesh2d(meshes.add(circle)),
        MeshMaterial2d(materials.add(ColorMaterial::from(red))),
        Transform::from_xyz(-150., 0., 0.),
        Target,
    ));
    commands.spawn((
        Mesh2d(meshes.add(triangle)),
        MeshMaterial2d(materials.add(ColorMaterial::from(blue))),
        Transform::from_xyz(0., -150., 0.),
        RigidBody::Kinematic,
        MaxLinearSpeed(350.0),
        MaxAngularSpeed(30.0),
        ShipController {
            acceleration: 0.,
            max_speed: 50.0,
        },
        Ship,
    ));
}

fn mouse_cursor_system(
    mut mouse_pos: ResMut<MousePos>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = *camera;
    if let Some(world_pos) = window
        .cursor_position()
        .map(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.unwrap().origin.truncate())
    {
        mouse_pos.0 = world_pos;
    }
}

fn move_target(mut query: Query<&mut Transform, With<Target>>, mouse_pos: Res<MousePos>) {
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::new(mouse_pos.0.x, mouse_pos.0.y, 0.);
    }
}

#[derive(Component)]
struct ShipController {
    acceleration: f32,
    // maxlinearspeed
    max_speed: f32,
}

// https://natureofcode.com/autonomous-agents/#example-51-seeking-a-target
fn move_ship(
    mut query: Query<
        (
            &mut ShipController,
            &mut LinearVelocity,
            &mut Rotation,
            &mut MaxLinearSpeed,
            &Position,
        ),
        With<Ship>,
    >,
    mouse_pos: Res<MousePos>,
    time: Res<Time>,
) {
    for (mut ship, mut velocity, mut rotation, mut max_linear_speed, position) in &mut query {
        let mut to_cursor = mouse_pos.0 - position.0;
        to_cursor = set_magnitude(to_cursor, max_linear_speed.0);

        let steer = to_cursor - velocity.0;

        velocity.x += steer.x * time.delta_secs();
        velocity.y += steer.y * time.delta_secs();
        // if velocity.x < ship.max_speed {
        // velocity.x += acceleration * time.delta_secs();
        //     *rotation = rotation.add_angle(0.1);
        // }
    }
}

fn set_magnitude(vector: Vec2, scale: f32) -> Vec2 {
    vector.normalize() * scale
}

fn clamp_edges(mut query: Query<&mut Transform, With<Ship>>) {
    let half_max_width = 400.;
    let half_max_height = 300.;

    // bevy screen centre is 0,0
    for mut transform in &mut query {
        if transform.translation.x > half_max_width {
            transform.translation.x = -(half_max_width);
        } else if transform.translation.x < (-half_max_width) {
            transform.translation.x = half_max_width;
        }

        if transform.translation.y > half_max_height {
            transform.translation.y = -(half_max_height);
        } else if transform.translation.y < (-half_max_height) {
            transform.translation.y = half_max_height;
        }
    }
}
