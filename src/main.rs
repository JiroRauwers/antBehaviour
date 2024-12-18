use ant_behaviour::{ant::AntPlugin, camera::CameraPlugin, grid::GridPlugin, ui::UiPlugin};
use bevy::prelude::*;

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
        .add_plugins((CameraPlugin, UiPlugin))
        .add_plugins((GridPlugin, AntPlugin))
        .run();
}
