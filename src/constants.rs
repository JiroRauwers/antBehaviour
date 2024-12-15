use std::f64::consts::PI;

pub const GRID_RESOLUTION: f32 = 100.0;
pub const GRID_AREA_SIZE: f32 = 1000.0;
pub const DEBUG_GRID_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 0.5];

pub const CAMERA_MAX_ZOOM: f32 = 0.5;
pub const CAMERA_MIN_ZOOM: f32 = 5.0;
pub const CAMERA_FOCUS_RANGE: f32 = 20.0;

pub const SPRITE_ANT: &str = "ant.png";

pub const ANT_VIEW_DISTANCE: f32 = 100.0;
pub const ANT_VIEW_ANGLE: f32 = (PI / 2.0) as f32;
pub const ANT_SPEED: f32 = 1.0;

pub const DEBUG_ANT_VIEW_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 0.5];
pub const DEBUG_ANT_VIEW_RADIUS_COLOR: [f32; 4] = [0.55, 0.55, 0.55, 0.2];
pub const DEBUG_ANT_VIEW_COLOR_ALERT: [f32; 4] = [1.0, 0.0, 0.0, 0.5];
