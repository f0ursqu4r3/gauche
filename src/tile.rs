use glam::IVec2;

use crate::{sprite::Sprite, stage::TileData, state::State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    None,
    Grass,
    Wall,
    Ruin,
    Water,
}

impl Tile {
    pub fn walkable(self) -> bool {
        matches!(self, Tile::None | Tile::Grass)
    }

    pub fn empty(self) -> bool {
        matches!(self, Tile::None)
    }

    pub fn can_build_on(self) -> bool {
        matches!(self, Tile::None | Tile::Grass)
    }
}

/// Check if a tile is walkable and unoccupied by impassable entities.
pub fn is_tile_walkable(state: &State, tile_coords: IVec2) -> bool {
    // Check grid bounds first
    if tile_coords.x < 0
        || tile_coords.y < 0
        || tile_coords.x as usize >= state.spatial_grid.len()
        || tile_coords.y as usize >= state.spatial_grid[0].len()
    {
        return false; // Treat out-of-bounds as not buildable.
    }

    let tile_walkable = match state
        .stage
        .get_tile_type(tile_coords.x as usize, tile_coords.y as usize)
    {
        Some(tile) => tile.walkable(),
        None => false, // If the tile doesn't exist, treat it as not walkable.
    };

    tile_walkable && !is_tile_occupied(state, tile_coords)
}

/// Check if a tile is unoccupied by impassable entities and can be built on.
pub fn can_build_on(state: &State, tile_coords: IVec2) -> bool {
    // Check grid bounds first
    if tile_coords.x < 0
        || tile_coords.y < 0
        || tile_coords.x as usize >= state.spatial_grid.len()
        || tile_coords.y as usize >= state.spatial_grid[0].len()
    {
        return false; // Treat out-of-bounds as not buildable.
    }

    let tile_buildable_upon = match state
        .stage
        .get_tile_type(tile_coords.x as usize, tile_coords.y as usize)
    {
        Some(tile) => tile.can_build_on(),
        None => false, // If the tile doesn't exist, treat it as not buildable.
    };
    tile_buildable_upon && !is_tile_occupied(state, tile_coords)
}

/// Check if tile is an empty tile and unoccupied by any impassable entities.
pub fn is_tile_empty(state: &State, tile_coords: IVec2) -> bool {
    // Check grid bounds first
    if tile_coords.x < 0
        || tile_coords.y < 0
        || tile_coords.x as usize >= state.spatial_grid.len()
        || tile_coords.y as usize >= state.spatial_grid[0].len()
    {
        return false; // Treat out-of-bounds as not empty.
    }
    let tile_empty = match state
        .stage
        .get_tile_type(tile_coords.x as usize, tile_coords.y as usize)
    {
        Some(tile) => tile.empty(),
        None => false, // If the tile doesn't exist, treat it as occupied.
    };
    !is_tile_occupied(state, tile_coords) && tile_empty
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

pub fn get_tile_variants(tile_data: &TileData) -> Vec<Sprite> {
    match tile_data.tile {
        Tile::Grass => vec![Sprite::Grass],
        Tile::Wall => vec![Sprite::Wall],
        Tile::Ruin => vec![Sprite::Ruin],
        Tile::Water => vec![Sprite::Water3, Sprite::Water4],
        _ => vec![],
    }
}

// take in a tile data, fetch variants, and lookup the current variant
pub fn get_tile_sprite(tile_data: &TileData) -> Option<Sprite> {
    let variants = get_tile_variants(tile_data);
    if (tile_data.variant as usize) < variants.len() {
        Some(variants[tile_data.variant as usize])
    } else {
        None // Return None if the index is out of bounds
    }
}
