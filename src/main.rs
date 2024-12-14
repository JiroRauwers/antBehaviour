use ant_behaviour::{
    camera::{CameraPlugin, FocusableEntity},
    utils::{window_to_world, ViewCone},
    ANT_SPEED, ANT_VIEW_ANGLE, ANT_VIEW_DISTANCE, DEBUG_ANT_VIEW_COLOR_ALERT,
    DEBUG_ANT_VIEW_RADIUS_COLOR,
};
use bevy::{prelude::*, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resizable: false,
                        focused: true,
                        title: "Ants".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_resource::<AntsSettings>()
        .add_plugins(CameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (draw_grid, move_ant_system))
        .run();
}

#[derive(Debug, Component)]
struct Ant;

#[derive(Resource)]
struct AntsSettings {
    view_distance: f32,
    view_angle: f32,
    // speed: f32,
}

impl Default for AntsSettings {
    fn default() -> Self {
        Self {
            view_distance: ANT_VIEW_DISTANCE,
            view_angle: ANT_VIEW_ANGLE,
            // speed: ANT_SPEED,
        }
    }
}

#[derive(Resource)]
struct Grid {
    size: UVec2,
    cell_size: Vec2,
    items: Vec<Vec<Entity>>,
}

#[derive(Component)]
struct AntMovementTimer(Timer);

fn move_ant_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut AntMovementTimer), With<Ant>>,
) {
    for (mut transform, mut timer) in query.iter_mut() {
        // Update the timer
        timer.0.tick(time.delta());

        // Check if moving up or down
        let direction = if timer.0.elapsed_secs() < 1.0 {
            1.0 // Moving up
        } else {
            -1.0 // Moving down
        };

        // Move the ant
        transform.translation.y += direction * 100.0 * time.delta_secs();

        // Reset the timer after 2 seconds (1 second up + 1 second down)
        if timer.0.elapsed_secs() >= 2.0 {
            timer.0.reset();
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle = asset_server.load("ant.png");
    commands.spawn((
        Sprite {
            image: texture_handle,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(0., 0., 0.1)),
        Ant,
        AntMovementTimer(Timer::from_seconds(2.0, TimerMode::Repeating)),
        FocusableEntity::default(),
    ));
}

fn draw_grid(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_transform_query: Query<&Transform, With<Camera2d>>,
    mut gizmos: Gizmos,
    ants: Query<&Transform, With<Ant>>,
    ants_settings: Res<AntsSettings>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    let window = window_query.single();
    let camera_transform = camera_transform_query.single();

    gizmos
        .grid_2d(
            Isometry2d {
                translation: Vec2::splat(0.),
                ..Default::default()
            },
            UVec2::splat(10),
            Vec2::splat(40.),
            LinearRgba::gray(0.05),
        )
        .outer_edges();

    let button_pressed = mouse_button.pressed(MouseButton::Left);
    for ant in ants.iter() {
        // Draw a circle around the ant "view"
        gizmos.circle_2d(
            ant.translation.truncate(),
            ants_settings.view_distance,
            LinearRgba::from_f32_array(DEBUG_ANT_VIEW_RADIUS_COLOR),
        );

        let mut view_cone = ViewCone::new(
            ant.translation.truncate(),
            ants_settings.view_distance,
            ants_settings.view_angle,
        );

        if let Some(cursor_position) = window.cursor_position() {
            if button_pressed {
                let click_pos = window_to_world(cursor_position, window, camera_transform);
                gizmos.circle_2d(
                    Isometry2d {
                        translation: click_pos,
                        ..Default::default()
                    },
                    20.0 * camera_transform.scale.x,
                    LinearRgba::from_f32_array(DEBUG_ANT_VIEW_RADIUS_COLOR),
                );

                if view_cone.contains(click_pos) {
                    view_cone.color(LinearRgba::from_f32_array(DEBUG_ANT_VIEW_COLOR_ALERT));
                }
            }
        }

        view_cone.draw(&mut gizmos);
    }
}
