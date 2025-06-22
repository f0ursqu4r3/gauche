// wander behavior

use glam::{IVec2, Vec2};
use rand::{random_range, Rng};

use crate::{
    audio::{Audio, SoundEffect},
    entity::{swap_step_sound, StepSound, VID},
    particle::ParticleData,
    sprite::Sprite,
    state::State,
    step::{entity_step_sound_lookup, lean_entity, FRAMES_PER_SECOND, TIMESTEP},
    tile::{self, is_tile_occupied},
};

pub fn wander(state: &mut State, vid: VID) {
    // check if entity is wandering, if is, move to random position
}

/// Checks if an entity is ready to move based on its cooldown.
/// Also resets the move cooldown countdown if the entity is ready to move.
pub fn ready_to_move(state: &mut State, vid: VID) -> bool {
    if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
        if entity.move_cooldown_countdown > 0.0 {
            entity.move_cooldown_countdown -= TIMESTEP;
            return false; // Not ready to move yet
        }
        return true;
    }
    false // Entity not found
}

pub fn reset_move_cooldown(state: &mut State, vid: VID) {
    if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
        entity.move_cooldown_countdown = entity.move_cooldown; // Reset cooldown countdown
    }
}

pub const BASE_SOUND_HEAR_DISTANCE: f32 = 64.0;
pub const STEP_SOUND_HEAR_DISTANCE: f32 = 8.0;

pub fn calc_sound_loudness_from_player_dist_falloff(
    state: &State,
    sound_pos: Vec2,
    hear_distance: f32,
) -> f32 {
    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity(player_vid) {
            let distance = sound_pos.distance(player.pos);
            if distance < hear_distance {
                // Volume falls off linearly with distance
                return 1.0 - (distance / hear_distance);
            }
        }
    }
    0.0 // Sound is too far to be heard
}

/// Attempts to move an entity to a target position.
/// Returns `true` if the move was successful, `false` otherwise.
/// This function checks for walkable terrain and entity collisions.
/// If the move is successful, it updates the entity's position, plays a step sound
/// based on distance to the player, and updates the spatial grid.
pub fn move_entity_on_grid(
    state: &mut State,
    audio: &mut Audio,
    vid: VID,
    target_grid_pos: IVec2,
    ignore_tile_collision: bool,
    dont_reset_move_cooldown: bool,
) -> bool {
    // Get the grid representation of the target position

    // Check if the terrain is walkable
    let terrain_is_walkable = state
        .stage
        .get_tile(target_grid_pos.x as usize, target_grid_pos.y as usize)
        .is_some_and(|t| tile::walkable(*t));

    // Check if the tile is already occupied by another impassable entity
    let tile_is_unoccupied = !is_tile_occupied(state, target_grid_pos);
    let mut moved = false;
    if (terrain_is_walkable || ignore_tile_collision) && tile_is_unoccupied {
        // If the move is valid, get the entity to update it
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            let old_grid_pos = entity.pos.as_ivec2();
            entity.pos = target_grid_pos.as_vec2() + Vec2::splat(0.5); // Center the entity on the grid tile

            // Update the entity's location in the spatial grid for collision detection
            state.move_entity_in_grid(vid, old_grid_pos, target_grid_pos);
            moved = true;
        }
    } else {
        // fail to move sound, scale with dist // currently only if player
        if let Some(entity) = state.entity_manager.get_entity(vid) {
            if entity.type_ == crate::entity::EntityType::Player {
                // Calculate the sound loudness based on distance to the player
                let sound_loudness = calc_sound_loudness_from_player_dist_falloff(
                    state,
                    entity.pos,
                    BASE_SOUND_HEAR_DISTANCE,
                );
                if sound_loudness > 0.0 {
                    // Play a sound effect indicating the move failed
                    audio.play_sound_effect_scaled(SoundEffect::HitBlock1, sound_loudness);
                }
            }
        }

        // reset move cooldown if the entity failed to move
        if !dont_reset_move_cooldown {
            reset_move_cooldown(state, vid);
        }
    }

    // move sound
    if moved {
        let entity_position: Option<Vec2> = state.entity_manager.get_entity(vid).map(|e| e.pos);

        if let Some(entity_position) = entity_position {
            // Calculate the sound loudness based on distance to the player
            let sound_loudness = calc_sound_loudness_from_player_dist_falloff(
                state,
                entity_position,
                STEP_SOUND_HEAR_DISTANCE,
            );
            if sound_loudness > 0.0 {
                if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
                    // Play the step sound effect with the calculated volume
                    let sound_effect = entity_step_sound_lookup(entity);
                    audio.play_sound_effect_scaled(sound_effect, sound_loudness);
                    swap_step_sound(entity);
                }
            }
        }

        // lean the entity
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            lean_entity(entity);
        }

        if !dont_reset_move_cooldown {
            reset_move_cooldown(state, vid);
        }

        // pub struct ParticleData {
        //     pub pos: Vec2,
        //     pub size: Vec2,
        //     pub rot: f32,
        //     pub alpha: f32,
        //     pub lifetime: u32,
        //     pub initial_lifetime: u32, // Used for age-based calculations (e.g., animations)
        //     pub sprite: Sprite,
        // }

        // put a footprint based on entity type
        // put ZombieGib1 slightly offset from entity position
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            let footprint_sprite = match entity.type_ {
                crate::entity::EntityType::Player => Sprite::PlayerFootprint,
                crate::entity::EntityType::Zombie => Sprite::ZombieFootprint,
                _ => Sprite::NoSprite, // No footprint for other entities
            };

            // Place a footprint at the entity's position
            let footprint_pos = entity.pos + Vec2::new(0.0, 0.5);
            //  be 0.2 down, but randomly 0.2 left or right (like different feet)
            pub const FEET_OFFSET: f32 = 0.2; // Offset for the footprint
            let offset_x = if entity.step_sound == StepSound::Step1 {
                -FEET_OFFSET // Left foot
            } else {
                FEET_OFFSET // Right foot
            };
            let footprint_pos = Vec2::new(footprint_pos.x + offset_x, footprint_pos.y);
            state.particles.spawn_static(ParticleData {
                pos: footprint_pos,
                size: Vec2::new(8.0, 8.0),
                rot: random_range(-10.0..10.0),
                alpha: 0.1,
                lifetime: FRAMES_PER_SECOND * 32,
                initial_lifetime: 60,
                sprite: footprint_sprite,
            });
        }
    }

    moved
}

/// given position, pick random position to the left, right, up, or down
pub fn pick_random_adjacent_tile_position(pos: IVec2) -> IVec2 {
    let direction = random_range(0..4);
    match direction {
        0 => IVec2::new(pos.x - 1, pos.y), // Left
        1 => IVec2::new(pos.x + 1, pos.y), // Right
        2 => IVec2::new(pos.x, pos.y - 1), // Up
        _ => IVec2::new(pos.x, pos.y + 1), // Down
    }
}

pub fn pick_random_adjacent_tile_position_include_center(pos: IVec2) -> IVec2 {
    let direction = random_range(0..5);
    match direction {
        0 => IVec2::new(pos.x - 1, pos.y), // Left
        1 => IVec2::new(pos.x + 1, pos.y), // Right
        2 => IVec2::new(pos.x, pos.y - 1), // Up
        3 => IVec2::new(pos.x, pos.y + 1), // Down
        _ => pos,                          // Stay in place
    }
}

/// given position, pick random adjacent position with diagonals
pub fn pick_random_adjacent_tile_position_with_diagonals(pos: IVec2) -> IVec2 {
    let direction = random_range(0..8);
    match direction {
        0 => IVec2::new(pos.x - 1, pos.y),     // Left
        1 => IVec2::new(pos.x + 1, pos.y),     // Right
        2 => IVec2::new(pos.x, pos.y - 1),     // Up
        3 => IVec2::new(pos.x, pos.y + 1),     // Down
        4 => IVec2::new(pos.x - 1, pos.y - 1), // Up-Left
        5 => IVec2::new(pos.x + 1, pos.y - 1), // Up-Right
        6 => IVec2::new(pos.x - 1, pos.y + 1), // Down-Left
        _ => IVec2::new(pos.x + 1, pos.y + 1), // Down-Right
    }
}

/// given position, pick random adjacent position with diagonals, including the center
pub fn pick_random_adjacent_tile_position_with_diagonals_include_center(pos: IVec2) -> IVec2 {
    let direction = random_range(0..9);
    match direction {
        0 => IVec2::new(pos.x - 1, pos.y),     // Left
        1 => IVec2::new(pos.x + 1, pos.y),     // Right
        2 => IVec2::new(pos.x, pos.y - 1),     // Up
        3 => IVec2::new(pos.x, pos.y + 1),     // Down
        4 => IVec2::new(pos.x - 1, pos.y - 1), // Up-Left
        5 => IVec2::new(pos.x + 1, pos.y - 1), // Up-Right
        6 => IVec2::new(pos.x - 1, pos.y + 1), // Down-Left
        7 => IVec2::new(pos.x + 1, pos.y + 1), // Down-Right
        _ => pos,                              // Stay in place
    }
}

/// pick a random tile position in a radius around the given position
pub fn pick_random_tile_position_in_radius(pos: IVec2, radius: i32) -> IVec2 {
    let x_offset = random_range(-radius..=radius);
    let y_offset = random_range(-radius..=radius);
    IVec2::new(pos.x + x_offset, pos.y + y_offset)
}

/// pick a random tile position in a radius around the given position, including the center
pub fn pick_random_tile_position_in_radius_include_center(pos: IVec2, radius: i32) -> IVec2 {
    let x_offset = random_range(-radius..=radius);
    let y_offset = random_range(-radius..=radius);
    if x_offset == 0 && y_offset == 0 {
        pos // Stay in place
    } else {
        IVec2::new(pos.x + x_offset, pos.y + y_offset)
    }
}
