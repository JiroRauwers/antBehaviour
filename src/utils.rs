use bevy::{
    color::{Color, ColorToComponents, LinearRgba},
    math::{Mat2, Vec2},
    prelude::{Gizmos, Transform},
    window::Window,
};

use crate::DEBUG_ANT_VIEW_COLOR;

#[derive(Debug, Clone, Copy)]
pub struct ViewCone {
    center: Vec2,
    rotation: f32,
    radius: f32,
    view_angle: f32,
    segments: usize,
    color: Color,
    direction: Vec2,
}

impl ViewCone {
    pub fn new(center: Vec2, radius: f32, view_angle: f32, rotation: f32) -> Self {
        Self {
            center,
            radius,
            view_angle,
            segments: 50,
            color: LinearRgba::from_f32_array(DEBUG_ANT_VIEW_COLOR).into(),
            direction: Vec2::Y,
            rotation,
        }
    }
    pub fn color<C>(&mut self, color: C) -> &Self
    where
        C: Into<Color>,
    {
        self.color = color.into();
        self
    }
    pub fn segments(&mut self, segments: usize) -> &Self {
        self.segments = segments;
        self
    }
    pub fn rotate_to(&mut self, direction: Vec2) -> &Self {
        self.direction = direction.normalize();
        self
    }

    /// Draw the view cone using gizmos
    pub fn draw(&self, gizmos: &mut Gizmos) {
        let half_angle = self.view_angle / 2.0;

        // Calculate step size for the arc
        let step = self.view_angle / self.segments as f32;

        // Calculate start and end directions with rotation
        let start_direction = Mat2::from_angle(self.rotation - half_angle) * self.direction;

        // Draw the arc
        let mut prev_point = self.center + self.radius * start_direction;
        for i in 1..=self.segments {
            let angle = self.rotation - half_angle + i as f32 * step;
            let rotated_dir = Mat2::from_angle(angle) * self.direction;
            let next_point = self.center + self.radius * rotated_dir;

            // Only draw line segments between consecutive points
            gizmos.line_2d(prev_point, next_point, self.color);

            prev_point = next_point;
        }

        // Draw lines from the center to the start and end of the arc
        let start_point = self.center
            + self.radius * Mat2::from_angle(self.rotation - half_angle) * self.direction;
        let end_point = self.center
            + self.radius * Mat2::from_angle(self.rotation + half_angle) * self.direction;

        gizmos.line_2d(self.center, start_point, self.color); // Left edge
        gizmos.line_2d(self.center, end_point, self.color); // Right edge

        // draw line from center to direction
        gizmos.line_2d(
            self.center,
            self.center + Mat2::from_angle(self.rotation) * self.direction * 50.,
            self.color,
        );
    }

    /// Check if a given point is inside the view cone
    pub fn contains(&self, point: Vec2, area: f32) -> bool {
        // Calculate the vector from the cone's center to the point
        let to_point = point - self.center;

        // Check if the point is within the radius plus the area
        if to_point.length_squared() > (self.radius + area) * (self.radius + area) {
            return false; // Point is outside the radius plus the area
        }

        // Normalize the direction vector
        let direction_norm = Mat2::from_angle(self.rotation) * self.direction;

        // Calculate the angle between the direction and the point
        let angle_to_point = direction_norm.angle_to(to_point);

        // Check if the point's angle is within the cone's view angle
        angle_to_point.abs() <= self.view_angle / 2.0
    }
}

// Convert screen coordinates to world coordinates
pub fn window_to_world(
    cursor_position: Vec2,
    window: &Window,
    camera_transform: &Transform,
) -> Vec2 {
    // Get the window size
    let window_size = Vec2::new(window.width(), window.height());

    // Normalize the cursor position to screen space [-1, 1]
    let normalized_cursor = (cursor_position / window_size) * 2.0 - Vec2::ONE;

    // Invert the Y-axis (screen to world)
    let inverted_cursor = Vec2::new(normalized_cursor.x, -normalized_cursor.y);

    // Adjust for the camera's scale (zoom level)
    let scaled_cursor = Vec2::new(
        inverted_cursor.x * (window_size.x / 2.0) * camera_transform.scale.x,
        inverted_cursor.y * (window_size.y / 2.0) * camera_transform.scale.y,
    );

    // Apply the camera's translation (pan)
    let world_position = scaled_cursor + camera_transform.translation.truncate();

    world_position
}

pub fn square<T>(x: T) -> T
where
    T: std::ops::Mul<Output = T> + Copy,
{
    x * x
}
