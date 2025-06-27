use glam::{IVec2, Vec2};

use crate::{
    audio::{Audio, SoundEffect},
    entity::DamageType,
    particle_templates::debris_splatter,
    sprite::Sprite,
    stage::TileData,
    state::State,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    None,
    Grass,
    Wall,
    Ruin,
    Water,
    Rail,
}

impl Tile {
    pub fn walkable(self) -> bool {
        matches!(self, Tile::None | Tile::Grass | Tile::Ruin | Tile::Rail)
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
        Tile::Rail => vec![Sprite::Rail],
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

/// Determines if a given tile can be damaged by a given damage type.
/// This allows for future expansion (e.g., some tiles immune to punch, weak to explosion).
pub fn can_damage_tile(tile_data: &TileData, damage_type: DamageType) -> bool {
    if !tile_data.breakable {
        return false;
    }

    // For now, let's say Punches can damage any breakable tile.
    match damage_type {
        DamageType::Punch => true,
        DamageType::Scratch => true, // Zombies should also be able to claw at walls
        _ => false,
    }
}

/// Called when a tile's HP is reduced to 0.
/// Handles changing the tile, dropping items, or other break effects.
pub fn on_tile_break(state: &mut State, audio: &mut Audio, tile_pos: IVec2, tile_data: &TileData) {
    // Behavior depends on the type of tile that broke.
    match tile_data.tile {
        Tile::Wall => {
            // A broken wall becomes a non-breakable 'Ruin' tile.
            let mut tile = TileData::default();
            tile.tile = Tile::Ruin;
            state
                .stage
                .set_tile(tile_pos.x as usize, tile_pos.y as usize, tile);
            // TODO: In the future, you could drop a "stone" item here.
        }
        _ => {
            // By default, most broken tiles just become empty space.
            state.stage.set_tile(
                tile_pos.x as usize,
                tile_pos.y as usize,
                TileData::default(),
            );
        }
    }

    // Play the appropriate break sound effect.
    let sound_effect = tile_break_sound_lookup(&tile_data.tile);
    audio.play_sound_effect(sound_effect);
}

/// Called when a tile takes damage but is not yet broken.
/// Handles sound and particle effects.
pub fn on_tile_damage(state: &mut State, audio: &mut Audio, tile_pos: IVec2, attacker_pos: Vec2) {
    let tile_sprite: Option<Sprite> = if let Some(td) = state
        .stage
        .get_tile(tile_pos.x as usize, tile_pos.y as usize)
    {
        get_tile_sprite(&td)
    } else {
        return;
    };
    // If we don't have a sprite, we can't do anything.
    if tile_sprite.is_none() {
        return;
    }
    let tile_sprite = tile_sprite.unwrap();

    // Play a generic "thud" sound.
    if let Some(tile_type) = state
        .stage
        .get_tile_type(tile_pos.x as usize, tile_pos.y as usize)
    {
        audio.play_sound_effect(tile_damage_sound_lookup(&tile_type));
    }

    // Calculate effect positions and directions.
    let tile_center_world_pos = tile_pos.as_vec2() + Vec2::splat(0.5);
    let direction_from_attacker = (tile_center_world_pos - attacker_pos).normalize_or_zero();

    // Spawn debris particles of the tile's sprite flying away from the attacker.
    debris_splatter(
        &mut state.particles,
        tile_center_world_pos,
        direction_from_attacker,
        tile_sprite,
    );
}

/// The main public function for dealing damage to a tile.
/// It checks for breakability, applies damage, and triggers effects.
/// Returns true if damage was successfully dealt.
pub fn damage_tile(
    state: &mut State,
    audio: &mut Audio,
    tile_pos: IVec2,
    damage: u8,
    damage_type: DamageType,
    attacker_pos: Vec2,
) -> bool {
    let tile_data = match state
        .stage
        .get_tile(tile_pos.x as usize, tile_pos.y as usize)
    {
        Some(td) => td,
        None => return false, // Tile doesn't exist.
    };

    // Check if the tile can be damaged by this attack type.
    if !can_damage_tile(&tile_data, damage_type) {
        return false;
    }

    // Trigger the visual/audio "hit" effect.
    on_tile_damage(state, audio, tile_pos, attacker_pos);

    // Get a mutable copy of the tile data to work with.
    let mut tile_data_mut = tile_data;
    let new_hp = tile_data_mut.hp.saturating_sub(damage);
    tile_data_mut.hp = new_hp;

    // Check if the tile broke.
    if new_hp == 0 {
        // The tile is destroyed. on_tile_break handles setting the new tile (e.g., Ruin).
        // We do not need to set the tile again after this.
        on_tile_break(state, audio, tile_pos, &tile_data_mut);
    } else {
        // The tile was damaged but not destroyed. Write the updated data back to the stage.
        state
            .stage
            .set_tile(tile_pos.x as usize, tile_pos.y as usize, tile_data_mut);
    }

    true
}

pub fn tile_break_sound_lookup(tile: &Tile) -> SoundEffect {
    match tile {
        _ => SoundEffect::BoxBreak,
    }
}

pub fn tile_damage_sound_lookup(tile: &Tile) -> SoundEffect {
    match tile {
        _ => SoundEffect::Explosion1,
    }
}
