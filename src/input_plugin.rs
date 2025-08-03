use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePos(Vec2::new(0.0, 0.0)))
            .add_systems(Update, (mouse_cursor_system, move_target_system));
    }
}

#[derive(Resource, Default)]
pub struct MousePos(pub Vec2);

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Target;

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

fn move_target_system(mut query: Query<&mut Transform, With<Target>>, mouse_pos: Res<MousePos>) {
    for mut transform in query.iter_mut() {
        transform.translation = Vec3::new(mouse_pos.0.x, mouse_pos.0.y, 0.);
    }
}
