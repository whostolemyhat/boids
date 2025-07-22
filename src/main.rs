use avian2d::prelude::*;
use bevy::color::palettes::css::{BLUE, RED};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);
    }
}

fn main() {
    App::new()
        .add_plugins(AppPlugin)
        .add_systems(Startup, setup)
        .insert_resource(MousePos(Vec2::new(0.0, 0.0)))
        .add_systems(Update, (mouse_cursor_system, move_target))
        .run();
}

#[derive(Resource, Default)]
struct MousePos(Vec2);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Target;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d, MainCamera));

    let circle = Circle::new(30.);
    let red: Color = RED.into();

    let square = Rectangle::new(20., 30.);
    let blue: Color = BLUE.into();

    commands.spawn((
        Mesh2d(meshes.add(circle)),
        MeshMaterial2d(materials.add(ColorMaterial::from(red))),
        Transform::from_xyz(-150., 0., 0.),
        Target,
    ));
    commands.spawn((
        Mesh2d(meshes.add(square)),
        MeshMaterial2d(materials.add(ColorMaterial::from(blue))),
        Transform::from_xyz(0., -150., 0.),
        RigidBody::Kinematic,
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
        println!("{},{}", world_pos.x, world_pos.y);
    }
}

fn move_target(mut query: Query<&mut Transform, With<Target>>, mouse_pos: Res<MousePos>) {
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::new(mouse_pos.0.x, mouse_pos.0.y, 0.);
    }
}
