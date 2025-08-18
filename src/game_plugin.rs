use avian2d::prelude::*;
use bevy::color::palettes::css::{BLUE, RED};
use bevy::prelude::*;

use crate::input_plugin::{MainCamera, Target};
use crate::steering_plugin::{Behaviour, Ship, WrapEdges};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPlugins, PhysicsPlugins::default()))
            .init_state::<Behaviour>()
            .add_systems(Startup, setup)
            .add_systems(Update, (clamp_edges_system, button_handler_system));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d, MainCamera));

    let target_size = 15.0;
    let circle = Circle::new(target_size);
    let red: Color = RED.into();

    let ship_height = 15.0;
    let ship_width = 10.0;
    let blue: Color = BLUE.into();
    let triangle = Triangle2d::new(
        Vec2::Y * ship_height,
        Vec2::new(-ship_width, -ship_width),
        Vec2::new(ship_width, -ship_width),
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
        MaxLinearSpeed(250.0),
        MaxAngularSpeed(10.0),
        Ship,
        WrapEdges,
    ));

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            ..default()
        },
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                // not automatic, make sure this matches #[default] in Behaviour enum
                create_selected_button("Seek"),
                create_normal_button("Arrive"),
                create_normal_button("Wander"),
                create_normal_button("Pursue"),
            ],
        )],
    ));
}

#[derive(Component)]
struct SelectedOption;

const NORMAL_BUTTON: Color = Color::srgb(0.95, 0.75, 0.35);
const HOVERED_BUTTON: Color = Color::srgb(1., 0.83, 0.52);
const HOVERED_SELECTED_BUTTON: Color = Color::srgb(1., 0.54, 0.8);
const SELECTED_BUTTON: Color = Color::srgb(0.96, 0.05, 0.7);

fn button(btn_text: &str) -> impl Bundle + use<> {
    let behaviour = match btn_text {
        "Arrive" => Behaviour::Arrive,
        "Wander" => Behaviour::Wander,
        "Pursue" => Behaviour::Pursue,
        _ => Behaviour::Seek,
    };

    (
        Button,
        Node {
            width: Val::Px(100.0),
            height: Val::Px(35.0),
            border: UiRect::all(Val::Px(3.0)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            margin: UiRect {
                top: Val::Px(15.),
                ..default()
            },
            ..default()
        },
        behaviour,
        BorderRadius::MAX,
        children![(
            Text::new(btn_text),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.24, 0.21, 0.19)),
            TextShadow {
                color: Color::srgb(0.64, 0.61, 0.59),
                offset: Vec2 { x: 0.5, y: 0.5 }
            },
        )],
    )
}
fn create_normal_button(btn_text: &str) -> impl Bundle + use<> {
    (
        button(btn_text),
        BackgroundColor(NORMAL_BUTTON),
        BorderColor(Color::BLACK),
    )
}

fn create_selected_button(btn_text: &str) -> impl Bundle + use<> {
    (
        button(btn_text),
        BackgroundColor(SELECTED_BUTTON),
        SelectedOption,
        BorderColor(Color::WHITE),
    )
}

// TODO highlight current behaviour
#[allow(clippy::type_complexity)]
fn button_handler_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<Behaviour>>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Behaviour,
        ),
        (Changed<Interaction>, With<Button>, Without<SelectedOption>),
    >,
    current_selected: Single<
        (Entity, &mut BackgroundColor, &mut BorderColor),
        With<SelectedOption>,
    >,
    mode: Res<State<Behaviour>>,
) {
    let (previous_button, mut previous_button_color, mut previous_button_border) =
        current_selected.into_inner();

    for (entity, interaction, mut bg, mut border, behaviour) in &mut interaction_query {
        if *interaction == Interaction::Pressed && *behaviour != **mode {
            *previous_button_color = NORMAL_BUTTON.into();
            *previous_button_border = Color::BLACK.into();
            commands.entity(previous_button).remove::<SelectedOption>();
            commands.entity(entity).insert(SelectedOption);
            next_state.set(behaviour.clone());
            *bg = SELECTED_BUTTON.into();
        }

        let is_selected = *behaviour == **mode;
        match (interaction, is_selected) {
            (Interaction::Hovered, true) => {
                border.0 = Color::WHITE;
                *bg = HOVERED_SELECTED_BUTTON.into();
            }
            (Interaction::Hovered, false) => {
                border.0 = Color::WHITE;
                *bg = HOVERED_BUTTON.into();
            }
            (Interaction::None, true) => {
                *bg = SELECTED_BUTTON.into();
                border.0 = Color::BLACK;
            }
            (Interaction::None, false) => {
                *bg = NORMAL_BUTTON.into();
                border.0 = Color::BLACK;
            }
            _ => {}
        }
    }
}

// keep in middle of screen
fn clamp_edges_system(mut query: Query<&mut Position, With<WrapEdges>>) {
    let half_max_width = 400.;
    let half_max_height = 300.;

    // bevy screen centre is 0,0
    for mut position in &mut query {
        if position.x > half_max_width {
            position.x = -(half_max_width);
        } else if position.x < (-half_max_width) {
            position.x = half_max_width;
        }

        if position.y > half_max_height {
            position.y = -(half_max_height);
        } else if position.y < (-half_max_height) {
            position.y = half_max_height;
        }
    }
}
