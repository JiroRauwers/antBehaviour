use ant_behaviour::{
    camera::{CameraPlugin, FocusableEntity},
    grid::{Grid, GridEntity, GridPlugin},
    utils::{square, window_to_world, ViewCone},
    ANT_SPEED, ANT_VIEW_ANGLE, ANT_VIEW_DISTANCE, DEBUG_ANT_VIEW_COLOR_ALERT,
    DEBUG_ANT_VIEW_RADIUS_COLOR, GRID_AREA_SIZE, GRID_RESOLUTION,
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
        .add_plugins(GridPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (draw))
        .add_systems(Update, move_ant_system)
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

#[derive(Component)]
struct AntMovementTimer(Timer);

fn move_ant_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut AntMovementTimer, Entity), With<Ant>>,
    mut grid: ResMut<Grid>,
) {
    for (mut transform, mut timer, entity) in query.iter_mut() {
        // Update the timer
        timer.0.tick(time.delta());
        let initial_position = transform.translation.truncate();

        // Check if moving up or down
        let direction = if timer.0.elapsed_secs() < 1.0 {
            1.0 // Moving up
        } else {
            -1.0 // Moving down
        };

        // Move the ant
        transform.translation.y += direction * 300.0 * time.delta_secs();

        // Reset the timer after 2 seconds (1 second up + 1 second down)
        if timer.0.elapsed_secs() >= 2.0 {
            timer.0.reset();
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, grid: ResMut<Grid>) {
    let texture_handle = asset_server.load("ant.png");

    let pos = Vec3::new(115.0, 15.0, 0.1);
    commands.spawn((
        Sprite {
            image: texture_handle,
            ..Default::default()
        },
        Transform::from_translation(pos),
        Ant,
        AntMovementTimer(Timer::from_seconds(10.0, TimerMode::Repeating)),
        FocusableEntity::default(),
        GridEntity::new(grid.get_grid_pos(pos.truncate())),
    ));
}

fn draw(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_transform_query: Query<&Transform, With<Camera2d>>,
    mut gizmos: Gizmos,
    ants: Query<&Transform, With<Ant>>,
    ants_settings: Res<AntsSettings>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    let window = window_query.single();
    let camera_transform = camera_transform_query.single();

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
