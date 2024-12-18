use bevy::{ecs::batching::BatchingStrategy, prelude::*, window::PrimaryWindow};

use crate::{
    camera::{FocusableEntity, FocusedEntity},
    grid::{Grid, GridEntity},
    utils::{window_to_world, ViewCone},
    ANT_COUNT, ANT_ROTATION_SPEED, ANT_SIZE, ANT_SPEED, ANT_VIEW_ANGLE, ANT_VIEW_DISTANCE,
    DEBUG_ANT_VIEW_COLOR, DEBUG_ANT_VIEW_COLOR_ALERT, DEBUG_ANT_VIEW_RADIUS_COLOR, NEST_COLOR,
    NEST_POSITION, NEST_SIZE, PHEROMONE_DECAY,
};

pub struct AntPlugin;

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AntSettings>()
            .add_systems(Startup, spawn_ants)
            .add_systems(Update, move_ants)
            .add_systems(Update, check_mouse)
            .add_systems(Update, ant_focused)
            .add_systems(Update, draw_nest)
            .add_systems(Update, ant_sees_other_ant);
    }
}

#[derive(Resource)]
pub struct AntSettings {
    pub view_distance: f32,
    pub view_angle: f32,
    pub speed: f32,
    pub n_ants: usize,
    pub nest_size: f32,
    pub nest_position: Vec2,
}

impl Default for AntSettings {
    fn default() -> Self {
        Self {
            view_distance: ANT_VIEW_DISTANCE,
            view_angle: ANT_VIEW_ANGLE,
            speed: ANT_SPEED,
            n_ants: ANT_COUNT,
            nest_size: NEST_SIZE,
            nest_position: NEST_POSITION.into(),
        }
    }
}

#[derive(Debug)]
pub enum DesiredTarget {
    PHEROMONE,
    FOOD,
    NOTHING,
}

pub enum Pheromones {
    LookingForFood,
    LookingForHome,
}

impl Pheromones {
    pub fn decay(&self, pheromone: f32) -> f32 {
        match self {
            _ => pheromone * PHEROMONE_DECAY,
        }
    }

    pub fn get_color(&self) -> [f32; 4] {
        match self {
            Pheromones::LookingForFood => [1.0, 0.0, 0.0, 1.0],
            Pheromones::LookingForHome => [0.0, 0.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Component)]
pub struct Ant {
    desired_direction: Vec2,
    desired_target: DesiredTarget,
}

impl Ant {
    pub fn new() -> Self {
        Self {
            desired_direction: Vec2::ZERO,
            desired_target: DesiredTarget::NOTHING,
        }
    }

    pub fn get_view_cone(&self, transform: &Transform, settings: &AntSettings) -> ViewCone {
        ViewCone::new(
            transform.translation.truncate(),
            settings.view_distance,
            settings.view_angle,
            transform.rotation.to_euler(EulerRot::XYZ).2,
        )
    }

    pub fn debug_view(
        &self,
        transform: &Transform,
        gizmos: &mut Gizmos,
        settings: &AntSettings,
    ) -> &Self {
        // Draw a circle around the ant "view"
        gizmos.circle_2d(
            transform.translation.truncate(),
            settings.view_distance,
            LinearRgba::from_f32_array(DEBUG_ANT_VIEW_RADIUS_COLOR),
        );

        // Draw a circle around the ant "size"
        gizmos.circle_2d(
            transform.translation.truncate(),
            ANT_SIZE / 2.0,
            LinearRgba::from_f32_array(DEBUG_ANT_VIEW_COLOR),
        );

        // draw desired direction
        let desired_position =
            transform.translation.truncate() + self.desired_direction * settings.view_distance;
        gizmos.circle_2d(
            desired_position,
            5.0,
            LinearRgba::from_f32_array(DEBUG_ANT_VIEW_COLOR),
        );

        // draw view cone
        let view_cone = ViewCone::new(
            transform.translation.truncate(),
            settings.view_distance,
            settings.view_angle,
            transform.rotation.to_euler(EulerRot::XYZ).2,
        );
        view_cone.draw(gizmos);
        self
    }
}

fn spawn_ants(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    grid: ResMut<Grid>,
    ant_settings: Res<AntSettings>,
) {
    let texture_handle = asset_server.load("ant.png");

    (0..ant_settings.n_ants).into_iter().for_each(|_| {
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let distance = rand::random::<f32>() * NEST_SIZE * 1.6;
        let translation = Vec3::new(
            NEST_POSITION.0 + distance * angle.cos(),
            NEST_POSITION.1 + distance * angle.sin(),
            0.1,
        );
        let rotation = Quat::from_rotation_z(rand::random::<f32>() * std::f32::consts::TAU);
        commands.spawn((
            Sprite {
                image: texture_handle.clone(),
                ..Default::default()
            },
            Transform {
                translation,
                rotation,
                ..Default::default()
            },
            Ant::new(),
            FocusableEntity::default(),
            GridEntity::new(grid.get_grid_pos(translation.truncate())),
        ));
    });
}

fn ant_sees_other_ant(
    ants: Query<(&Transform, Entity, &Ant, &GridEntity), With<Ant>>,
    ants_settings: Res<AntSettings>,
    grid: Res<Grid>,
) {
    ants.par_iter()
        .batching_strategy(BatchingStrategy {
            batch_size_limits: 1..500,
            ..Default::default()
        })
        .for_each(|(ant_transform, ant_entity, ant, _)| {
            let ant_position = ant_transform.translation.truncate();
            let cells_in_area =
                grid.get_cells_in_area_from_world(ant_position, ants_settings.view_distance);
            let _view_cone = ant.get_view_cone(ant_transform, &ants_settings);

            cells_in_area
                .into_iter()
                .for_each(|(_, (ants_grid_entities, _, _))| {
                    ants_grid_entities.into_iter().for_each(|(_, entitie)| {
                        if entitie != ant_entity {
                            if let Ok((other_ant_transform, _, _, _)) = ants.get(entitie) {
                                let _other_ant_position =
                                    other_ant_transform.translation.truncate();
                                // if view_cone.contains(other_ant_position, ANT_SIZE / 2.) {
                                //     // Ant sees another ant
                                // }
                            }
                        }
                    })
                });
        });
}

fn ant_focused(
    ants: Query<(&Transform, &Ant, Entity), With<Ant>>,
    focused_entity: Res<FocusedEntity>,
    mut gizmos: Gizmos,
    grid: Res<Grid>,
    ants_settings: Res<AntSettings>,
) {
    if let Some(focused_entity) = focused_entity.0 {
        if let Ok((transform, ant, ant_entity)) = ants.get(focused_entity) {
            ant.debug_view(transform, &mut gizmos, &ants_settings);

            let ant_position = transform.translation.truncate();
            let cells_in_area =
                grid.get_cells_in_area_from_world(ant_position, ants_settings.view_distance);
            let view_cone = ant.get_view_cone(transform, &ants_settings);

            // (pos ,(ants, pheromones, food))
            cells_in_area
                .into_iter()
                .for_each(|(grid_cells, (ants_grid_entities, _, _))| {
                    grid.draw_cell(
                        &mut gizmos,
                        grid_cells,
                        LinearRgba::from_f32_array([0.5, 0.5, 0.5, 0.04]),
                    );
                    ants_grid_entities.into_iter().for_each(|(_, entitie)| {
                        if entitie != ant_entity {
                            if let Ok((other_ant_transform, _, _)) = ants.get(entitie) {
                                let other_ant_position = other_ant_transform.translation.truncate();
                                if view_cone.contains(other_ant_position, ANT_SIZE / 2.) {
                                    // Ant sees another ant
                                    // draw a redline beetween the two ants
                                    gizmos.line_2d(
                                        ant_position,
                                        other_ant_position,
                                        LinearRgba::from_f32_array([0.0, 1.0, 0.0, 1.0]),
                                    );
                                } else {
                                    //  draw a gray line
                                    gizmos.line_2d(
                                        ant_position,
                                        other_ant_position,
                                        LinearRgba::from_f32_array([1.0, 0.0, 0.0, 1.0]),
                                    );
                                }
                            }
                        }
                    })
                });
        }
    }
}
fn move_ants(
    mut ants: Query<(&mut Transform, &mut Ant), With<Ant>>,
    ants_settings: Res<AntSettings>,
    time: Res<Time>,
    grid: Res<Grid>,
) {
    let (min, max) = grid.get_boundaries();
    let border_threshold = ants_settings.view_distance * 1.5;
    let delta_secs = time.delta_secs();

    ants.par_iter_mut()
        .batching_strategy(BatchingStrategy {
            batch_size_limits: 1..500,
            ..Default::default()
        })
        .for_each(|(mut ant_transform, mut ant)| {
            let ant_position = ant_transform.translation.truncate();

            // If the ant has no specific target, it will randomly steer
            if let DesiredTarget::NOTHING = ant.desired_target {
                // Steer away from borders if close enough
                if ant_position.x < min.x + border_threshold {
                    ant.desired_direction.x = ant.desired_direction.x + 1.0;
                } else if ant_position.x > max.x - border_threshold {
                    ant.desired_direction.x = ant.desired_direction.x - 1.0;
                }
                ant.desired_direction.y =
                    ant.desired_direction.y + (rand::random::<f32>() - 0.5) * 0.4;
                if ant_position.y < min.y + border_threshold {
                    ant.desired_direction.y = ant.desired_direction.y + 1.0;
                } else if ant_position.y > max.y - border_threshold {
                    ant.desired_direction.y = ant.desired_direction.y - 1.0;
                }
                ant.desired_direction.x =
                    ant.desired_direction.x + (rand::random::<f32>() - 0.5) * 0.4;
                ant.desired_direction = ant.desired_direction.normalize_or_zero();
            }

            // Current forward direction of the ant
            let current_direction = ant_transform.rotation * Vec3::Y;

            // Calculate the angle between the current direction and the desired direction
            let angle = current_direction.angle_between(ant.desired_direction.extend(0.0));

            // Calculate the rotation step based on the ant's rotation speed and the elapsed time
            let rotation_speed = match ant.desired_target {
                DesiredTarget::FOOD => ANT_ROTATION_SPEED * 3.0,
                // case searching for pheromone
                _ => match (
                    ant_position.x < min.x + ants_settings.view_distance,
                    ant_position.x > max.x - ants_settings.view_distance,
                    ant_position.y < min.y + ants_settings.view_distance,
                    ant_position.y > max.y - ants_settings.view_distance,
                ) {
                    (true, _, _, _) | (_, true, _, _) | (_, _, true, _) | (_, _, _, true) => {
                        // Increase turning speed when steering from borders
                        ANT_ROTATION_SPEED * 5.0
                    }
                    _ => ANT_ROTATION_SPEED,
                },
            };
            let rotation_step = rotation_speed * delta_secs;

            // Determine the new rotation
            let new_rotation = if angle < rotation_step {
                Quat::from_rotation_arc(current_direction, ant.desired_direction.extend(0.0))
            } else {
                Quat::from_rotation_arc(
                    current_direction,
                    current_direction
                        .lerp(ant.desired_direction.extend(0.0), rotation_step / angle),
                )
            };

            // Apply the new rotation to the ant
            ant_transform.rotation = new_rotation * ant_transform.rotation;

            // Move the ant forward in the direction it is facing
            let forward_movement =
                ant_transform.rotation * Vec3::Y * ants_settings.speed * delta_secs;
            ant_transform.translation += forward_movement;

            // Constrain the ant to the grid space
            ant_transform.translation.x = ant_transform.translation.x.clamp(min.x, max.x);
            ant_transform.translation.y = ant_transform.translation.y.clamp(min.y, max.y);
        });
}

fn check_mouse(
    mut ants: Query<(&Transform, &mut Ant), With<Ant>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    ants_settings: Res<AntSettings>,
    mut gizmos: Gizmos,
    _grid: Res<Grid>,
    camera_transform: Query<&Transform, With<Camera>>,
    focused_entity: Res<FocusedEntity>,
) {
    let window = windows.single();
    let camera_transform = camera_transform.single();
    if buttons.pressed(MouseButton::Left) {
        if let Some(focused_entity) = focused_entity.0 {
            if let Ok((ant_transform, mut ant)) = ants.get_mut(focused_entity) {
                let ant_position = ant_transform.translation.truncate();
                let view_cone = ViewCone::new(
                    ant_position,
                    ants_settings.view_distance,
                    ants_settings.view_angle,
                    ant_transform.rotation.to_euler(EulerRot::XYZ).2,
                );

                if let Some(cursor_position) = window.cursor_position() {
                    let cursor_world_position =
                        window_to_world(cursor_position, window, camera_transform);

                    // Draw a blue dot at the mouse click position
                    gizmos.circle_2d(
                        cursor_world_position,
                        5.0,
                        LinearRgba::new(0.0, 0.0, 1.0, 1.0),
                    );

                    if view_cone.contains(cursor_world_position, 0.0) {
                        ant.desired_target = DesiredTarget::FOOD;
                        ant.desired_direction = (cursor_world_position - ant_position).normalize();
                        println!(
                                "Ant at position {:?} sees the mouse click as food and sets direction towards it",
                                ant_position
                            );

                        // Draw a green dot at the ant's desired direction
                        let desired_position =
                            ant_position + ant.desired_direction * ants_settings.view_distance;
                        gizmos.circle_2d(
                            desired_position,
                            5.0,
                            LinearRgba::from_f32_array(DEBUG_ANT_VIEW_COLOR_ALERT),
                        );
                    }
                }
            }
        }
    } else {
        if let Some(focused_entity) = focused_entity.0 {
            if let Ok((_, mut ant)) = ants.get_mut(focused_entity) {
                ant.desired_target = DesiredTarget::NOTHING;
            }
        }
    }
}

fn draw_nest(mut gizmos: Gizmos, ant_settings: Res<AntSettings>) {
    // Draw the nest
    gizmos.circle_2d(
        ant_settings.nest_position,
        ant_settings.nest_size,
        LinearRgba::from_f32_array(NEST_COLOR),
    );
}
