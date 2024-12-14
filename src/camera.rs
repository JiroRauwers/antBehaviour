use bevy::{
    ecs::query::{self, QueryData},
    input::mouse::MouseWheel,
    prelude::*,
    window::PrimaryWindow,
};

use crate::{utils::window_to_world, CAMERA_FOCUS_RANGE, CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM};

pub struct CameraPlugin;

#[derive(Component, Default, Clone)]
pub struct FocusableEntity {
    pub zoom: f32,
}

impl FocusableEntity {
    pub fn new(zoom: f32) -> Self {
        Self { zoom }
    }
}

#[derive(Default, Resource)]
pub struct FocusedEntity(pub Option<Entity>);

#[derive(Default, Resource)]
pub struct CameraTarget(pub Option<Vec3>);

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FocusedEntity::default())
            .insert_resource(CameraTarget::default())
            .add_systems(Startup, setup)
            .add_systems(Update, pan_camera_system)
            .add_systems(Update, zoom_camera_system)
            .add_systems(Update, focus_on_entity_system)
            .add_systems(Update, move_camera_system);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

/// System to pan the camera, taking scale into account
fn pan_camera_system(
    mut query: Query<&mut Transform, With<Camera2d>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut previous_cursor_position: Local<Option<Vec2>>,
    mut focused_entity: ResMut<FocusedEntity>,
    mut camera_target: ResMut<CameraTarget>,
) {
    let mut camera_transform = query.single_mut();

    // Only pan if Space is held and Left Mouse Button is pressed
    if key_input.pressed(KeyCode::Space) && mouse_input.pressed(MouseButton::Left) {
        focused_entity.0 = None; // Unfocus the entity when panning starts
        camera_target.0 = None; // Clear the camera target

        if let Some(cursor_event) = cursor_moved_events.read().last() {
            let current_cursor_position = cursor_event.position;

            if let Some(prev_position) = *previous_cursor_position {
                let delta = current_cursor_position - prev_position;

                // Adjust panning by camera scale
                camera_transform.translation.x -= delta.x * camera_transform.scale.x;
                camera_transform.translation.y += delta.y * camera_transform.scale.y;
                // Flip Y
            }

            *previous_cursor_position = Some(current_cursor_position);
        }
    } else {
        *previous_cursor_position = None;
    }
}

/// System to zoom the camera in and out
fn zoom_camera_system(
    mut query: Query<&mut Transform, With<Camera2d>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();
    let mut camera_transform = query.single_mut();

    // Handle keyboard zoom (centered on camera)
    if key_input.pressed(KeyCode::Minus) {
        camera_transform.scale *= Vec3::new(1.1, 1.1, 1.0); // Zoom out
    } else if key_input.pressed(KeyCode::Equal) {
        camera_transform.scale *= Vec3::new(0.9, 0.9, 1.0); // Zoom in
    }

    // Clamp zoom level using the constants
    camera_transform.scale.x = camera_transform
        .scale
        .x
        .clamp(CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM);
    camera_transform.scale.y = camera_transform
        .scale
        .y
        .clamp(CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM);

    if camera_transform.scale.x == CAMERA_MIN_ZOOM || camera_transform.scale.x == CAMERA_MAX_ZOOM {
        return;
    }

    // Handle mouse scroll zoom (toward cursor)
    if let Some(cursor_screen_pos) = window.cursor_position() {
        for scroll_event in scroll_events.read() {
            let zoom_factor = if scroll_event.y > 0.0 { 0.9 } else { 1.1 };

            // Convert cursor position to world coordinates
            let cursor_world_pos = window_to_world(cursor_screen_pos, window, &camera_transform);

            // Compute the offset between the cursor and the camera
            let pre_zoom_offset = cursor_world_pos - camera_transform.translation.truncate();

            // Apply zoom
            camera_transform.scale *= Vec3::new(zoom_factor, zoom_factor, 1.0);

            // Clamp zoom level using the constants
            camera_transform.scale.x = camera_transform
                .scale
                .x
                .clamp(CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM);
            camera_transform.scale.y = camera_transform
                .scale
                .y
                .clamp(CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM);

            // Adjust the camera position to zoom toward the cursor
            let post_zoom_offset = pre_zoom_offset * zoom_factor;
            camera_transform.translation -= Vec3::new(
                post_zoom_offset.x - pre_zoom_offset.x,
                post_zoom_offset.y - pre_zoom_offset.y,
                0.0,
            );
        }
    }
}

/// System to focus and follow an entity with the camera
fn focus_on_entity_system(
    entity_query: Query<(Entity, &Transform, &FocusableEntity), With<FocusableEntity>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut focused_entity: ResMut<FocusedEntity>,
    mut camera_target: ResMut<CameraTarget>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    let window = windows.single();
    let camera_transform = camera_query.single(); // Immutable access to the camera transform

    // Handle mouse click to focus on an entity
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_screen_pos) = window.cursor_position() {
            let cursor_world_pos = window_to_world(cursor_screen_pos, window, camera_transform);

            for (entity, entity_transform, focusable) in entity_query.iter() {
                if cursor_world_pos.distance(entity_transform.translation.truncate())
                    < CAMERA_FOCUS_RANGE
                {
                    focused_entity.0 = Some(entity);
                    camera_target.0 = Some(Vec3::new(
                        entity_transform.translation.x,
                        entity_transform.translation.y,
                        focusable.zoom,
                    ));
                    return;
                }
            }
        }
    }

    // Update the camera target for the currently focused entity
    if let Some(entity) = focused_entity.0 {
        if let Ok((_, entity_transform, focusable)) = entity_query.get(entity) {
            camera_target.0 = Some(Vec3::new(
                entity_transform.translation.x,
                entity_transform.translation.y,
                focusable.zoom,
            ));
        }
    } else {
        camera_target.0 = None;
    }
}
/// System to move the camera toward the target
fn move_camera_system(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    camera_target: Res<CameraTarget>,
    time: Res<Time>,
) {
    let mut camera_transform = camera_query.single_mut();

    if let Some(target) = camera_target.0 {
        // Separate position and zoom
        let target_position = Vec3::new(target.x, target.y, camera_transform.translation.z);
        let target_zoom = target.z;

        // Smoothly move the camera toward the target position
        let direction = target_position - camera_transform.translation;
        camera_transform.translation += direction * time.delta_secs() * 5.0;

        // Smoothly adjust the zoom level
        let zoom_diff = target_zoom - camera_transform.scale.x;
        camera_transform.scale += Vec3::splat(zoom_diff * time.delta_secs() * 5.0);

        // Snap to the target position if close enough
        if direction.length() < 0.1 {
            camera_transform.translation = target_position;
        }

        // Snap zoom level if close enough
        if zoom_diff.abs() < 0.01 {
            camera_transform.scale = Vec3::splat(target_zoom);
        }

        // Clamp zoom level
        camera_transform.scale.x = camera_transform
            .scale
            .x
            .clamp(CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM);
        camera_transform.scale.y = camera_transform
            .scale
            .y
            .clamp(CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM);
    }
}
