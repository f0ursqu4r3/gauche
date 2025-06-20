use glam::IVec2;

use crate::state::State;

#[derive(Debug, Clone, Copy)]
pub enum Tile {
    None,
    Grass,
    Wall,
    Ruin,
    Water,
}

pub fn walkable(tile: Tile) -> bool {
    matches!(tile, Tile::None | Tile::Grass)
}

/// Helper function to check if a tile is occupied by an impassable entity.
pub fn is_tile_occupied(state: &State, tile_coords: IVec2) -> bool {
    // Check grid bounds first
    if tile_coords.x < 0
        || tile_coords.y < 0
        || tile_coords.x as usize >= state.spatial_grid.len()
        || tile_coords.y as usize >= state.spatial_grid[0].len()
    {
        return true; // Treat out-of-bounds as occupied
    }

    // Look up entities in the target cell of the spatial grid.
    let entities_in_cell = &state.spatial_grid[tile_coords.x as usize][tile_coords.y as usize];

    // Check if any of them are impassable.
    for vid in entities_in_cell {
        if let Some(entity) = state.entity_manager.get_entity(*vid) {
            if entity.impassable {
                return true; // Found an impassable entity, tile is occupied.
            }
        }
    }

    false // No impassable entities found.
}
