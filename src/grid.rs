use bevy::{prelude::*, window::PrimaryWindow};
use rayon::prelude::*;

use crate::{
    camera::FocusedEntity, ui::UiControls, utils::window_to_world, ANT_VIEW_DISTANCE,
    DEBUG_ANT_VIEW_RADIUS_COLOR, DEBUG_GRID_COLOR, GRID_AREA_SIZE, GRID_RESOLUTION,
};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Grid>()
            .add_systems(Update, draw_grid)
            .add_systems(Update, update_grid_entities_grid)
            .add_systems(Update, update_grid_entities_self_pos);
        // .add_system(update_grid.system());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridEntityType {
    Ant,
    Food,
    Pheromone,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct GridEntity {
    last_position: UVec2,
    current_position: UVec2,
}

impl GridEntity {
    pub fn new(position: UVec2) -> Self {
        Self {
            last_position: position,
            current_position: position,
        }
    }
}

impl Default for GridEntity {
    fn default() -> Self {
        Self {
            last_position: UVec2::ZERO,
            current_position: UVec2::ZERO,
        }
    }
}

#[derive(Resource)]
pub struct Grid {
    size: UVec2,     // Number of cells (width, height)
    cell_size: Vec2, // Size of each cell
    // 2D array of cells, each containing lists of different entities
    pheromones: Vec<Vec<(GridEntity, Entity)>>,
    ants: Vec<Vec<(GridEntity, Entity)>>,
    food: Vec<Vec<(GridEntity, Entity)>>,
    offset: Vec2, // Offset to align the grid with (0, 0) at the center
}

impl Default for Grid {
    fn default() -> Self {
        let num_cells = (GRID_AREA_SIZE / GRID_RESOLUTION).powi(2) as usize; // Total number of cells
        let size = UVec2::splat((GRID_AREA_SIZE / GRID_RESOLUTION) as u32); // Grid dimensions
        let cell_size = Vec2::splat(GRID_RESOLUTION); // Each cell is `GRID_RESOLUTION` x `GRID_RESOLUTION`

        let grid_dimensions = size.as_vec2() * cell_size; // Full dimensions of the grid
        let offset = -grid_dimensions / 2.0; // Center the grid around (0, 0)

        Self {
            size,
            cell_size,
            ants: vec![vec![]; num_cells],
            food: vec![vec![]; num_cells],
            pheromones: vec![vec![]; num_cells],
            offset,
        }
    }
}

impl Grid {
    pub fn draw_grid(&self, gizmos: &mut Gizmos) {
        // Draw the full grid
        gizmos
            .grid_2d(
                Isometry2d {
                    translation: Vec2::splat(0.),
                    ..Default::default()
                },
                self.size,
                self.cell_size,
                LinearRgba::from_f32_array(DEBUG_GRID_COLOR),
            )
            .outer_edges();
    }

    pub fn get_boundaries(&self) -> (Vec2, Vec2) {
        let min = self.offset;
        let max = self.offset + self.size.as_vec2() * self.cell_size;
        (min, max)
    }

    pub fn get_grid_pos(&self, world_pos: Vec2) -> UVec2 {
        // Adjust world position to grid-relative position
        let relative_pos = world_pos - self.offset;

        // Calculate grid indices
        let x = (relative_pos.x / self.cell_size.x).floor() as u32;
        let y = (relative_pos.y / self.cell_size.y).floor() as u32;

        UVec2::new(x, y).clamp(UVec2::ZERO, self.size - 1)
    }

    pub fn add_entity(
        &mut self,
        entity_type: GridEntityType,
        entity: (&GridEntity, Entity),
    ) -> Result<(), ()> {
        // Ensure indices are within bounds
        let curr_pos = entity.0.current_position;
        if curr_pos.x >= self.size.x || curr_pos.y >= self.size.y {
            return Err(()); // Out of bounds
        }

        // Compute the flattened index
        let index = ((curr_pos.x) + (curr_pos.y) * self.size.x) as usize;

        // Add the entity to the corresponding cell
        match entity_type {
            GridEntityType::Ant => self.ants[index].push((*entity.0, entity.1)),
            GridEntityType::Food => self.food[index].push((*entity.0, entity.1)),
            GridEntityType::Pheromone => self.pheromones[index].push((*entity.0, entity.1)),
            #[allow(unreachable_patterns)]
            _ => return Err(()),
        }
        Ok(())
    }
    pub fn lazy_remove(
        &mut self,
        entity_type: GridEntityType,
        entity: (&GridEntity, Entity),
    ) -> Result<(), ()> {
        let entity_id = entity.1;
        let entity_type_vec = match entity_type {
            GridEntityType::Ant => &mut self.ants,
            GridEntityType::Food => &mut self.food,
            GridEntityType::Pheromone => &mut self.pheromones,
            #[allow(unreachable_patterns)]
            _ => return Err(()),
        };

        entity_type_vec.par_iter_mut().for_each(|entities| {
            if let Some(index) = entities.iter().position(|e| e.1 == entity_id) {
                entities.remove(index);
            }
        });
        Ok(())
    }
    pub fn has_entity(
        &self,
        pos: UVec2,
        entity_type: GridEntityType,
        entity: (&GridEntity, Entity),
    ) -> Result<bool, ()> {
        // Ensure indices are within bounds
        if pos.x >= self.size.x || pos.y >= self.size.y {
            return Ok(false); // Out of bounds
        }

        // Compute the flattened index
        let index = ((pos.x) + (pos.y) * self.size.x) as usize;

        // Check if the entity is in the corresponding cell
        match entity_type {
            GridEntityType::Ant => Ok(self.ants[index].iter().any(|e| e.1 == entity.1)),
            GridEntityType::Food => Ok(self.food[index].iter().any(|e| e.1 == entity.1)),
            GridEntityType::Pheromone => Ok(self.pheromones[index].iter().any(|e| e.1 == entity.1)),
            #[allow(unreachable_patterns)]
            _ => return Err(()),
        }
    }
    pub fn remove_from(
        &mut self,
        entity_type: GridEntityType,
        pos: UVec2,
        entity: (&GridEntity, Entity),
    ) -> Result<(), ()> {
        // Remove the entity from the corresponding cell
        match self.has_entity(pos, entity_type, entity) {
            Ok(true) => {
                let index = ((pos.x) + (pos.y) * self.size.x) as usize;
                match entity_type {
                    GridEntityType::Ant => Ok(self.ants[index].retain(|e| e.1 != entity.1)),
                    GridEntityType::Food => Ok(self.food[index].retain(|e| e.1 != entity.1)),
                    GridEntityType::Pheromone => {
                        Ok(self.pheromones[index].retain(|e| e.1 != entity.1))
                    }
                    #[allow(unreachable_patterns)]
                    _ => return Err(()),
                }
            }
            #[allow(unreachable_patterns)]
            _ => return Err(()),
        }
    }

    pub fn draw_cell<C>(&self, gizmos: &mut Gizmos, pos: UVec2, color: C)
    where
        C: Into<Color> + Copy,
    {
        let cell_position = self.offset
            + Vec2::new(pos.x as f32, pos.y as f32) * self.cell_size
            + (self.cell_size * 0.5);
        let color_converted = color.into();
        gizmos.circle_2d(
            Isometry2d {
                translation: cell_position,
                ..Default::default()
            },
            10.,
            color_converted,
        );
        gizmos.rect_2d(
            Isometry2d {
                translation: cell_position,
                ..Default::default()
            },
            self.cell_size,
            color_converted,
        );
    }

    pub fn get_cells_in_area_from_grid(&self, grid_pos: UVec2, radius: f32) -> Vec<UVec2> {
        let radius = (radius / GRID_RESOLUTION).ceil() as u32;
        let mut cells = Vec::new();

        for y in (grid_pos.y.saturating_sub(radius))..=(grid_pos.y.saturating_add(radius)) {
            for x in (grid_pos.x.saturating_sub(radius))..=(grid_pos.x.saturating_add(radius)) {
                let pos = UVec2::new(x, y);
                if pos.x < self.size.x && pos.y < self.size.y {
                    cells.push(pos);
                }
            }
        }
        cells
    }

    pub fn get_cells_in_area_from_world(
        &self,
        world_pos: Vec2,
        radius: f32,
    ) -> Vec<(
        UVec2,
        (
            Vec<(GridEntity, Entity)>, // Ants
            Vec<(GridEntity, Entity)>, // Food
            Vec<(GridEntity, Entity)>, // Pheromones
        ),
    )> {
        let mut cells = Vec::new();
        let min_x = ((world_pos.x - radius - self.offset.x) / self.cell_size.x).floor() as i32;
        let max_x = ((world_pos.x + radius - self.offset.x) / self.cell_size.x).ceil() as i32;
        let min_y = ((world_pos.y - radius - self.offset.y) / self.cell_size.y).floor() as i32;
        let max_y = ((world_pos.y + radius - self.offset.y) / self.cell_size.y).ceil() as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x >= 0 && y >= 0 {
                    let pos = UVec2::new(x as u32, y as u32);
                    if pos.x < self.size.x && pos.y < self.size.y {
                        let cell_min =
                            self.offset + Vec2::new(pos.x as f32, pos.y as f32) * self.cell_size;
                        let cell_max = cell_min + self.cell_size;

                        let corners = [
                            cell_min,
                            Vec2::new(cell_min.x, cell_max.y),
                            Vec2::new(cell_max.x, cell_min.y),
                            cell_max,
                        ];

                        if corners
                            .iter()
                            .any(|&corner| corner.distance(world_pos) <= radius)
                        {
                            cells.push((
                                pos,
                                (
                                    self.ants[((pos.x) + (pos.y) * self.size.x) as usize].clone(),
                                    self.food[((pos.x) + (pos.y) * self.size.x) as usize].clone(),
                                    self.pheromones[((pos.x) + (pos.y) * self.size.x) as usize]
                                        .clone(),
                                ),
                            ));
                        }
                    }
                }
            }
        }
        cells
    }
}

fn draw_grid(
    grid: Res<Grid>,
    mut gizmos: Gizmos,
    mut last_cursor_pos: Local<UVec2>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<&Transform, With<Camera2d>>,
    entity_query: Query<(&Transform, Entity), With<GridEntity>>,
    focused_entity: Res<FocusedEntity>,
    ui_controlss: Res<UiControls>,
) {
    if ui_controlss.show_grid == false {
        return;
    }
    let window = windows.single();
    let camera_transform = camera_query.single(); // Immutable access to the camera transform

    grid.draw_grid(&mut gizmos);

    if let Some(cursor_position) = window.cursor_position() {
        let grid_pos =
            grid.get_grid_pos(window_to_world(cursor_position, window, camera_transform));
        if *last_cursor_pos != grid_pos {
            last_cursor_pos.clone_from(&grid_pos);
        }
    }
    // Draw the last cursor position
    grid.draw_cell(
        &mut gizmos,
        *last_cursor_pos,
        LinearRgba::default().with_red(1.),
    );

    for (transform, entity) in entity_query.iter() {
        if focused_entity.0 == Some(entity) {
            // Draw cells around the focused entity
            let cells = grid
                .get_cells_in_area_from_world(transform.translation.truncate(), ANT_VIEW_DISTANCE);
            for (pos, _items) in cells {
                grid.draw_cell(
                    &mut gizmos,
                    pos,
                    LinearRgba::from_f32_array(DEBUG_ANT_VIEW_RADIUS_COLOR),
                );
            }
        }
    }
}

fn update_grid_entities_self_pos(
    grid: Res<Grid>,
    mut entities: Query<(&mut GridEntity, &Transform)>,
) {
    for (mut g_entity, transform) in entities.iter_mut() {
        let new_pos = grid.get_grid_pos(transform.translation.truncate());
        g_entity.last_position = g_entity.current_position;
        g_entity.current_position = new_pos;
    }
}

fn update_grid_entities_grid(mut grid: ResMut<Grid>, entities: Query<(&GridEntity, Entity)>) {
    for g_entity in entities.iter() {
        // check if the entity is in the grid
        match grid.has_entity(g_entity.0.current_position, GridEntityType::Ant, g_entity) {
            Ok(false) => {
                match grid.has_entity(g_entity.0.last_position, GridEntityType::Ant, g_entity) {
                    Ok(false) => {
                        println!("Entity not in grid {:?}", g_entity.0);
                    }
                    Ok(true) => {
                        if let Err(_) = grid.remove_from(
                            GridEntityType::Ant,
                            g_entity.0.last_position,
                            g_entity,
                        ) {
                            println!("Error removing entity from grid");
                        }
                    }
                    Err(_) => {
                        println!("Error checking entity in grid");
                    }
                }

                if let Err(_) = grid.add_entity(GridEntityType::Ant, g_entity) {
                    println!("grid position out of bounds entity {:?}", g_entity);
                }
            }
            Ok(true) => {}
            Err(_) => {
                println!("Error checking entity in grid");
            }
        }
    }
}
