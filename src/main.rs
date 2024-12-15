use ant_behaviour::{
    ant::{Ant, AntPlugin, AntSettings},
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
        .add_plugins(CameraPlugin)
        .add_plugins((GridPlugin, AntPlugin))
        .run();
}

fn draw(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_transform_query: Query<&Transform, With<Camera2d>>,
    mut gizmos: Gizmos,
    ants: Query<&Transform, With<Ant>>,
    ants_settings: Res<AntSettings>,
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
