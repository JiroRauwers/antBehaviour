use ant_behaviour::{
    camera::{CameraPlugin, FocusableEntity},
    grid::{Grid, GridEntity, GridPlugin},
    utils::{window_to_world, ViewCone},
    ANT_SIZE, ANT_VIEW_ANGLE, ANT_VIEW_DISTANCE, DEBUG_ANT_VIEW_COLOR, DEBUG_ANT_VIEW_COLOR_ALERT,
    DEBUG_ANT_VIEW_RADIUS_COLOR, DEGREES_90,
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
        .add_systems(Update, draw)
        .add_systems(Update, move_ant_system)
        .add_systems(Update, ant_sees_other_ant)
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
    mut query: Query<(&mut Transform, &mut AntMovementTimer), With<Ant>>,
) {
    for (index, (mut transform, mut timer)) in query.iter_mut().enumerate() {
        // Update the timer
        timer.0.tick(time.delta());
        // let initial_position = transform.translation.truncate();

        // Check if moving up or down
        let direction = if timer.0.elapsed_secs() < 1.0 {
            1.0 // Moving up
        } else {
            -1.0 // Moving down
        };

        // Move the ant
        if index % 2 == 0 {
            transform.translation.y += direction * 300.0 * time.delta_secs();
        } else {
            transform.translation.y -= direction * 300.0 * time.delta_secs();
        }

        // Reset the timer after 2 seconds (1 second up + 1 second down)
        if timer.0.elapsed_secs() >= 2.0 {
            timer.0.reset();
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, grid: ResMut<Grid>) {
    let texture_handle = asset_server.load("ant.png");

    // ant 1
    let pos_1 = Vec3::new(115.0, 15.0, 0.1);
    commands.spawn((
        Sprite {
            image: texture_handle.clone(),
            ..Default::default()
        },
        Transform::from_translation(pos_1).with_rotation(Quat::from_rotation_z(DEGREES_90)),
        Ant,
        // AntMovementTimer(Timer::from_seconds(10.0, TimerMode::Repeating)),
        FocusableEntity::default(),
        GridEntity::new(grid.get_grid_pos(pos_1.truncate())),
    ));

    // ant 2
    let pos_2 = Vec3::new(-45.0, 15.0, 0.1);
    commands.spawn((
        Sprite {
            image: texture_handle.clone(),
            ..Default::default()
        },
        Transform::from_translation(pos_2),
        Ant,
        // AntMovementTimer(Timer::from_seconds(10.0, TimerMode::Repeating)),
        FocusableEntity::default(),
        GridEntity::new(grid.get_grid_pos(pos_2.truncate())),
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

        // Draw a circle around the ant "size"
        gizmos.circle_2d(
            ant.translation.truncate(),
            ANT_SIZE / 2.0,
            LinearRgba::from_f32_array(DEBUG_ANT_VIEW_COLOR),
        );

        // calculate the view cone
        let mut view_cone = ViewCone::new(
            ant.translation.truncate(),
            ants_settings.view_distance,
            ants_settings.view_angle,
            ant.rotation.to_euler(EulerRot::XYZ).2,
        );

        // check if the mouse is inside the view cone
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

                if view_cone.contains(click_pos, 0.0) {
                    view_cone.color(LinearRgba::from_f32_array(DEBUG_ANT_VIEW_COLOR_ALERT));
                }
            }
        }

        // Draw the view cone
        view_cone.draw(&mut gizmos);
    }
}

fn ant_sees_other_ant(
    ants: Query<(&Transform, &GridEntity, Entity), With<Ant>>,
    ants_settings: Res<AntsSettings>,
    grid: Res<Grid>,
) {
    for (ant_transform, _, ant_entity) in ants.iter() {
        let ant_position = ant_transform.translation.truncate();
        let cells_in_area =
            grid.get_cells_in_area_from_world(ant_position, ants_settings.view_distance);
        let view_cone = ViewCone::new(
            ant_position,
            ants_settings.view_distance,
            ants_settings.view_angle,
            ant_transform.rotation.to_euler(EulerRot::XYZ).2,
        );

        for (_, entities) in cells_in_area {
            for (_, entitie) in entities.iter() {
                if *entitie != ant_entity {
                    if let Ok((other_ant_transform, _, _)) = ants.get(*entitie) {
                        let other_ant_position = other_ant_transform.translation.truncate();
                        if view_cone.contains(other_ant_position, ANT_SIZE / 2.) {
                            // Ant sees another ant
                            println!("ant: {} sees ant {}", ant_entity.index(), entitie.index());
                        }
                    }
                }
            }
        }
    }
}
