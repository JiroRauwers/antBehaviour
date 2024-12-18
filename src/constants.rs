use std::f64::consts::PI;

pub const DEGREES_90: f32 = (PI as f32) / 2.0;
pub const DEGREES_180: f32 = PI as f32;
pub const DEGREES_270: f32 = (PI as f32) * 1.5;
pub const DEGREES_360: f32 = (PI as f32) * 2.0;

pub const GRID_RESOLUTION: f32 = 50.0;
pub const GRID_AREA_SIZE: f32 = 5000.0;
pub const DEBUG_GRID_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 0.04];

pub const CAMERA_MAX_ZOOM: f32 = 0.5;
pub const CAMERA_MIN_ZOOM: f32 = 5.0;
pub const CAMERA_FOCUS_RANGE: f32 = 20.0;

pub const SPRITE_ANT: &str = "ant.png";

pub const NEST_SIZE: f32 = 100.0;
pub const NEST_POSITION: (f32, f32) = (0.0, 0.0);
pub const NEST_COLOR: [f32; 4] = [1.0, 0.65, 0.0, 1.0];

pub const ANT_COUNT: usize = 10000;
pub const ANT_VIEW_DISTANCE: f32 = 150.0;
pub const ANT_SIZE: f32 = 25.0;
pub const ANT_VIEW_ANGLE: f32 = (PI / 2.0) as f32;
pub const ANT_SPEED: f32 = 100.0;
pub const ANT_ROTATION_SPEED: f32 = 1.0;

pub const DEBUG_ANT_VIEW_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 0.5];
pub const DEBUG_ANT_VIEW_RADIUS_COLOR: [f32; 4] = [0.55, 0.55, 0.55, 0.2];
pub const DEBUG_ANT_VIEW_COLOR_ALERT: [f32; 4] = [1.0, 0.0, 0.0, 0.5];

pub const PHEROMONE_DECAY: f32 = 0.99;
pub const PHEROMONE_MAX: f32 = 1.0;
