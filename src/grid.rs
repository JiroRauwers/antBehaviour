use bevy::{
    ecs::component::{ComponentId, Components},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{utils::window_to_world, DEBUG_GRID_COLOR, GRID_AREA_SIZE, GRID_RESOLUTION};

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
    size: UVec2,                           // Number of cells (width, height)
    cell_size: Vec2,                       // Size of each cell
    items: Vec<Vec<(GridEntity, Entity)>>, // Entities stored in each cell
    offset: Vec2,                          // Offset to align the grid with (0, 0) at the center
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
            items: vec![vec![]; num_cells], // Initialize with empty lists
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
    pub fn get_grid_pos(&self, world_pos: Vec2) -> UVec2 {
        // Adjust world position to grid-relative position
        let relative_pos = world_pos - self.offset;

        // Calculate grid indices
        let x = (relative_pos.x / self.cell_size.x).floor() as u32;
        let y = (relative_pos.y / self.cell_size.y).floor() as u32;

        UVec2::new(x, y).clamp(UVec2::ZERO, self.size - 1)
    }

    pub fn add_entity(&mut self, entity: (&GridEntity, Entity)) -> Result<(), ()> {
        // Ensure indices are within bounds
        let curr_pos = entity.0.current_position;
        if curr_pos.x >= self.size.x || curr_pos.y >= self.size.y {
            return Err(()); // Out of bounds
        }

        // Compute the flattened index
        let index = ((curr_pos.x) + (curr_pos.y) * self.size.x) as usize;

        // Add the entity to the corresponding cell
        self.items[index].push((*entity.0, entity.1));
        Ok(())
    }
    pub fn lazy_remove(&mut self, entity: (&GridEntity, Entity)) {
        for entities in self.items.iter_mut() {
            if let Some(index) = entities.iter().position(|e| e.1 == entity.1) {
                entities.remove(index);
                return;
            }
        }
    }
    pub fn has_entity(&self, pos: UVec2, entity: (&GridEntity, Entity)) -> bool {
        // Ensure indices are within bounds
        if pos.x >= self.size.x || pos.y >= self.size.y {
            return false; // Out of bounds
        }

        // Compute the flattened index
        let index = ((pos.x) + (pos.y) * self.size.x) as usize;

        // Check if the entity is in the corresponding cell
        self.items[index].iter().any(|e| e.1 == entity.1)
    }
    pub fn remove_from(&mut self, pos: UVec2, entity: (&GridEntity, Entity)) {
        // Remove the entity from the corresponding cell
        if self.has_entity(pos, entity) {
            let index = ((pos.x) + (pos.y) * self.size.x) as usize;
            self.items[index].retain(|e| e.1 != entity.1);
        }
    }

    pub fn draw_cell<C>(&self, gizmos: &mut Gizmos, pos: UVec2, color: C)
    where
        C: Into<Color>,
    {
        let cell_position = self.offset
            + Vec2::new(pos.x as f32, pos.y as f32) * self.cell_size
            + (self.cell_size * 0.5);
        gizmos.circle_2d(
            Isometry2d {
                translation: cell_position,
                ..Default::default()
            },
            10.,
            Color::srgb(1., 0., 0.),
        );
        gizmos.rect_2d(
            Isometry2d {
                translation: cell_position,
                ..Default::default()
            },
            self.cell_size,
            color.into(),
        );
    }
}

fn draw_grid(
    grid: Res<Grid>,
    mut gizmos: Gizmos,
    mut last_cursor_pos: Local<UVec2>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
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
    grid.draw_cell(
        &mut gizmos,
        *last_cursor_pos,
        LinearRgba::default().with_red(1.),
    );

    // Draw highlighted cells
    for (i, entities) in grid.items.iter().enumerate() {
        if entities.len() > 0 {
            let x = (i % grid.size.x as usize) as u32;
            let y = (i / grid.size.x as usize) as u32;

            grid.draw_cell(
                &mut gizmos,
                UVec2::new(x, y),
                LinearRgba::from_f32_array([1.0, 0.0, 0.0, 1.0]),
            );
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

        if !grid.has_entity(g_entity.0.current_position, g_entity) {
            if !grid.has_entity(g_entity.0.last_position, g_entity) {
                println!("Entity not in grid {:?}", g_entity.0);
            } else {
                grid.remove_from(g_entity.0.last_position, g_entity);
            }

            if let Err(_) = grid.add_entity(g_entity) {
                println!("grid position out of bounds entity {:?}", g_entity);
                // println!("Error adding entity to grid");
            }
            continue;
        }
    }
}
