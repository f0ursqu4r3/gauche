// wander behavior

use std::vec;

use glam::{IVec2, Vec2};
use rand::random_range;

use crate::{
    audio::{Audio, SoundEffect},
    entity::{self, swap_step_sound, EntityState, EntityType, StepSound, VID},
    entity_templates::init_as_train,
    particle::{ParticleData, ParticleLayer},
    particle_templates::{blood_puddle, blood_splatter},
    sprite::Sprite,
    stage::TileData,
    state::{get_adjacent_entities, State},
    step::{entity_step_sound_lookup, lean_entity, TIMESTEP},
    tile::{is_tile_occupied, tile_shake_area_at, Tile},
};

pub fn wander(state: &mut State, audio: &mut Audio, vid: VID) {
    // check if exists
    if state.entity_manager.get_entity(vid).is_none() {
        return; // Entity not found, exit early
    }

    // check mood is wandering
    if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
        if entity.mood != crate::entity::Mood::Wander {
            return; // Entity is not in a wandering mood, exit early
        }
    }

    // check if entity is wandering, if is, move to random position
    if ready_to_move(state, vid) {
        let current_tile_pos = state.entity_manager.get_entity(vid).unwrap().pos.as_ivec2();
        let wants_to_move_to = pick_random_adjacent_tile_position_include_center(current_tile_pos);
        if wants_to_move_to != current_tile_pos {
            move_entity_on_grid(
                state,
                audio,
                vid,
                wants_to_move_to,
                false, // Do not ignore tile collision for zombies
                false, // reset move cooldown
                false, // ignore entity collision
            );

            // if zombie, put him into his base sprite
            // get entity type, match on it
            let entity_type = state.entity_manager.get_entity(vid).unwrap().type_;
            if entity_type == crate::entity::EntityType::Zombie {
                // set sprite to zombie base sprite
                if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
                    entity.sprite = Some(Sprite::Zombie);
                    entity.state = EntityState::Idle; // Reset state to idle after wandering
                }
            }
        }
    }
}

pub fn growl_sometimes(state: &mut State, audio: &mut Audio, vid: VID) {
    // check if exists
    if state.entity_manager.get_entity(vid).is_none() {
        return; // Entity not found, exit early
    }

    // if entity is player, just return
    if let Some(entity) = state.entity_manager.get_entity(vid) {
        if entity.type_ == crate::entity::EntityType::Player {
            return; // Player should not growl
        }
    }

    // if entity does not have a growl sound, return
    if let Some(entity) = state.entity_manager.get_entity(vid) {
        if entity.growl.is_none() {
            return;
        }
    }

    pub const GROWL_CHANCE: f32 = 0.0001;
    if random_range(0.0..1.0) < GROWL_CHANCE {
        let pos = state.entity_manager.get_entity(vid).unwrap().pos;
        // play growl sound effect
        // loudness based on distance to player
        let sound_loudness =
            calc_sound_loudness_from_player_dist_falloff(state, pos, BASE_SOUND_HEAR_DISTANCE);
        if sound_loudness > 0.0 {
            if let Some(entity) = state.entity_manager.get_entity(vid) {
                if let Some(growl_sound) = entity.growl {
                    audio.play_sound_effect_scaled(growl_sound, sound_loudness * 0.3);
                }
            }
        }
        // shake the entity a little
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            entity.shake = 0.4; // Set shake to a larger value for growl
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    FistPunch,
    ZombieScratch,
}

pub fn attack_sprite_lookup(attack_type: AttackType) -> Sprite {
    match attack_type {
        AttackType::FistPunch => Sprite::Fist,
        AttackType::ZombieScratch => Sprite::ZombieScratch1,
    }
}

pub fn attack_sound_lookup(attack_type: AttackType) -> SoundEffect {
    match attack_type {
        AttackType::FistPunch => SoundEffect::Punch1, // Using fist punch sound as attack sound
        AttackType::ZombieScratch => SoundEffect::ZombieScratch1, // Using scratch sound as attack sound
    }
}

/// stub
pub fn attack(
    state: &mut State,
    audio: &mut Audio,
    attacker: &VID,
    attacked: &VID,
    attack_type: AttackType,
) {
    // check if exists
    if state.entity_manager.get_entity(*attacker).is_none()
        || state.entity_manager.get_entity(*attacked).is_none()
    {
        return; // Entity not found, exit early
    }

    // if attacked is not attackable, return
    if let Some(attacked_entity) = state.entity_manager.get_entity(*attacked) {
        if !attacked_entity.attackable {
            return;
        }
    }

    // play sound effect based on attack type, scale with distance to player if there is a player
    let attacker_pos = state.entity_manager.get_entity(*attacker).unwrap().pos;
    let sound_loudness =
        calc_sound_loudness_from_player_dist_falloff(state, attacker_pos, BASE_SOUND_HEAR_DISTANCE);
    if sound_loudness > 0.0 {
        audio.play_sound_effect_scaled(attack_sound_lookup(attack_type), sound_loudness);
    }

    // get strength of attack, lets say zombie scratch is 1
    let attack_strength = match attack_type {
        AttackType::FistPunch => 10,    // Fist punch deals 10 damage
        AttackType::ZombieScratch => 5, // Zombie scratch deals 1 damage
    };
    let attacker_pos = state.entity_manager.get_entity(*attacker).unwrap().pos;
    let attackee_pos = state.entity_manager.get_entity(*attacked).unwrap().pos;
    if let Some(attacked_entity) = state.entity_manager.get_entity_mut(*attacked) {
        if attacked_entity.health >= attack_strength {
            attacked_entity.health -= attack_strength;
        } else {
            attacked_entity.health = 0;
        }
        // make them shake a little
        attacked_entity.shake += 0.1; // Set shake to a moderate value for

        // lean attacker towards attackee at 45 degree angle if attacker is to left or right
        // if attacker is above, become 0 rot, if below, become 180 rot
        // attacked will lean in the opposite way
        pub const ATTACK_LEAN: f32 = 45.0; // Leaning angle
        if attacker_pos.x < attackee_pos.x {
            // Attacker is to the left of the attacked
            state.entity_manager.get_entity_mut(*attacker).unwrap().rot = ATTACK_LEAN;
            state.entity_manager.get_entity_mut(*attacked).unwrap().rot = ATTACK_LEAN;
        } else if attacker_pos.x > attackee_pos.x {
            // Attacker is to the right of the attacked
            state.entity_manager.get_entity_mut(*attacker).unwrap().rot = -ATTACK_LEAN;
            state.entity_manager.get_entity_mut(*attacked).unwrap().rot = -ATTACK_LEAN;
        } else if attacker_pos.y < attackee_pos.y {
            // Attacker is above the attacked
            state.entity_manager.get_entity_mut(*attacker).unwrap().rot = 180.0;
            state.entity_manager.get_entity_mut(*attacked).unwrap().rot = 180.0;
        } else {
            // Attacker is below the attacked
            state.entity_manager.get_entity_mut(*attacker).unwrap().rot = 0.0;
            state.entity_manager.get_entity_mut(*attacked).unwrap().rot = 0.0;
        }
    }

    // spawn a particle at the attacked entitys position, slightly offset towards the attacker
    let particle_offset = if attacker_pos.x < attackee_pos.x {
        Vec2::new(-0.2, 0.0) // Offset to the left
    } else if attacker_pos.x > attackee_pos.x {
        Vec2::new(0.2, 0.0) // Offset to the right
    } else if attacker_pos.y < attackee_pos.y {
        Vec2::new(0.0, -0.2) // Offset upwards
    } else {
        Vec2::new(0.0, 0.2) // Offset downwards
    };
    let particle_pos = attackee_pos + particle_offset;

    let sprite = attack_sprite_lookup(attack_type);
    state.particles.spawn_static(ParticleData::new(
        particle_pos,
        Vec2::new(16.0, 16.0),
        random_range(-45.0..45.0),
        1.0,
        30,
        sprite,
        ParticleLayer::Foreground,
    ));

    // spawn a blood splatter effect
    let base_direction = (attacker_pos - attackee_pos).normalize_or_zero();
    let magnitude = 0.1; // Adjust this value to control the intensity of the splatter
    blood_splatter(state, audio, particle_pos, base_direction, magnitude);

    // calculate the feet position of attacked entity
    let attacked_entity = state.entity_manager.get_entity_mut(*attacked).unwrap();
    let attacked_feet_pos = attacked_entity.pos + Vec2::new(0.0, 0.5); // Offset to the feet position
                                                                       // spawn a blood puddle at the feet position
    blood_puddle(&mut state.particles, attacked_feet_pos, magnitude);
}

/// check adjacent tiles, if any of them are occupied by an entity with player alignment, attack them.
pub fn indiscriminately_attack_nearby(state: &mut State, audio: &mut Audio, vid: VID) {
    // check if exists
    if state.entity_manager.get_entity(vid).is_none() {
        return; // Entity not found, exit early
    }

    // if not zombie, return
    if let Some(entity) = state.entity_manager.get_entity(vid) {
        if entity.type_ != crate::entity::EntityType::Zombie {
            return; // Only zombies should attack indiscriminately
        }
    }

    // check if entity is ready to attack
    if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
        if entity.attack_cooldown_countdown > 0.0 {
            return; // Not ready to attack yet
        }
    }

    let pos = state.entity_manager.get_entity(vid).unwrap().pos.as_ivec2();
    let own_alignment = state.entity_manager.get_entity(vid).unwrap().alignment;
    let adjacent_vids = get_adjacent_entities(state, pos);
    let vid_of_adjacent_entity = adjacent_vids.iter().find(|&&adj_vid| {
        if let Some(adj_entity) = state.entity_manager.get_entity(adj_vid) {
            adj_entity.alignment != own_alignment
        } else {
            false // Entity not found, treat as not a player
        }
    });

    let attacker_vid = &vid;
    if let Some(attackee_vid) = vid_of_adjacent_entity {
        attack(
            state,
            audio,
            attacker_vid,
            attackee_vid,
            AttackType::ZombieScratch,
        );
        // reset attack cooldown
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            entity.attack_cooldown_countdown = entity.attack_cooldown; // Reset cooldown countdown
        }
    }
}

pub fn step_attack_cooldown(state: &mut State, vid: VID) {
    // check if exists
    if state.entity_manager.get_entity(vid).is_none() {
        return; // Entity not found, exit early
    }

    // check if entity has attack cooldown
    if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
        if entity.attack_cooldown_countdown > 0.0 {
            entity.attack_cooldown_countdown -= TIMESTEP;
        }
    }
}

/// Step inventory item cooldowns for an entity.
pub fn step_inventory_item_cooldowns(state: &mut State, vid: VID) {
    // check if exists
    if state.entity_manager.get_entity(vid).is_none() {
        return; // Entity not found, exit early
    }

    // Step through the entity's inventory and reduce cooldowns
    if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
        for item in &mut entity.inventory.iter_mut_items() {
            if item.use_cooldown_countdown > 0.0 {
                item.use_cooldown_countdown -= TIMESTEP;
            }
        }
    }
}

/// Checks if an entity is ready to move based on its cooldown.
pub fn ready_to_move(state: &mut State, vid: VID) -> bool {
    if let Some(entity) = state.entity_manager.get_entity(vid) {
        if entity.move_cooldown_countdown > 0.0 {
            return false; // Not ready to move yet
        }
        return true; // Ready to move
    }
    false // Entity not found
}

pub fn step_move_cooldown(state: &mut State, vid: VID) {
    // check if exists
    if state.entity_manager.get_entity(vid).is_none() {
        return; // Entity not found, exit early
    }

    // Step the move cooldown countdown for the entity
    if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
        if entity.move_cooldown_countdown > 0.0 {
            entity.move_cooldown_countdown -= TIMESTEP;
        } else {
            entity.move_cooldown_countdown = 0.0;
        }
    }
}

pub fn reset_move_cooldown(state: &mut State, vid: VID) {
    if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
        entity.move_cooldown_countdown = entity.move_cooldown;
    }
}

pub const BASE_SOUND_HEAR_DISTANCE: f32 = 16.0;
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
    ignore_entity_collision: bool,
    dont_reset_move_cooldown: bool,
) -> bool {
    // Get the grid representation of the target position

    // Check if the terrain is walkable
    let terrain_is_walkable = state
        .stage
        .get_tile_type(target_grid_pos.x as usize, target_grid_pos.y as usize)
        .is_some_and(|t| t.walkable());

    // Check if the tile is already occupied by another impassable entity
    let tile_is_unoccupied = !is_tile_occupied(state, target_grid_pos);
    let mut moved = false;
    if (terrain_is_walkable || ignore_tile_collision)
        && (tile_is_unoccupied || ignore_entity_collision)
    {
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
            let footprint_sprite: Option<Sprite> = match entity.type_ {
                EntityType::Player => Some(Sprite::PlayerFootprint),
                EntityType::Zombie => Some(Sprite::ZombieFootprint),
                _ => None,
            };
            if footprint_sprite.is_none() {
                return moved; // No footprint sprite for this entity type
            }
            let footprint_sprite = footprint_sprite.unwrap();

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
            state.particles.spawn_static(ParticleData::new(
                footprint_pos,
                Vec2::new(8.0, 8.0),
                random_range(-10.0..10.0),
                0.1, // alpha
                60,  // lifetime
                footprint_sprite,
                ParticleLayer::Background,
            ));
        }
    }

    moved
}

/// Spawns death effects (corpse, blood, sound) for a dying entity.
pub fn on_entity_death(state: &mut State, audio: &mut Audio, vid: VID) {
    let mut corpse_sprite = None;
    let mut entity_pos = glam::Vec2::ZERO;
    let mut entity_rot = 0.0;
    let mut should_spawn_effects = false;

    // --- Scope 1: Read Data (Immutable Borrow) ---
    // We get all the info we need from the entity and store it in local variables.
    if let Some(entity) = state.entity_manager.get_entity(vid) {
        corpse_sprite = match entity.type_ {
            crate::entity::EntityType::Player => Some(Sprite::PlayerDead),
            crate::entity::EntityType::Zombie => Some(Sprite::ZombieDead),
            _ => None,
        };
        entity_pos = entity.pos;
        entity_rot = entity.rot;
        should_spawn_effects = true;
    }
    // The immutable borrow of `state` (via `entity`) ends here.

    // --- Scope 2: Apply Effects (Mutable Borrows) ---
    // Now that the immutable borrow is gone, we can safely mutate `state`.
    if should_spawn_effects {
        // 1. Spawn a static particle for the corpse.
        if let Some(corpse_sprite) = corpse_sprite {
            let corpse_data = ParticleData::new(
                entity_pos,
                glam::Vec2::splat(16.0), // TILE_SIZE
                entity_rot,              // Inherit the entity's final rotation
                1.0,                     // Start fully opaque
                60 * 15,                 // Lasts for 15 seconds
                corpse_sprite,
                ParticleLayer::Background, // Render behind living entities
            );
            state.particles.spawn_static(corpse_data);
        }

        // 2. Play a death sound effect.
        let death_sound_effect = match state.entity_manager.get_entity(vid).unwrap().type_ {
            EntityType::None => SoundEffect::BoxBreak,
            EntityType::Player => SoundEffect::AnimalCrush1,
            EntityType::Zombie => SoundEffect::AnimalCrush2,
            EntityType::Chicken => SoundEffect::AnimalCrush2,
            EntityType::RailLayer => SoundEffect::BoxBreak,
            EntityType::Train => SoundEffect::BoxBreak,
            EntityType::Item => SoundEffect::BoxBreak,
        };
        let sound_loudness = calc_sound_loudness_from_player_dist_falloff(
            state,
            entity_pos,
            BASE_SOUND_HEAR_DISTANCE,
        );
        if sound_loudness > 0.0 {
            audio.play_sound_effect_scaled(death_sound_effect, sound_loudness);
        }

        // 3. Spawn blood and gore effects.
        blood_splatter(
            state,
            audio,
            entity_pos,
            glam::Vec2::new(0.0, -1.0), // Splatter moves generally upwards
            0.8,                        // A good amount of splatter
        );
        blood_puddle(&mut state.particles, entity_pos, 1.0);
    }
}

/// Checks if an entity's health is zero and, if so, marks it for destruction.
pub fn die_if_health_zero(state: &mut State, audio: &mut Audio, vid: VID) {
    let mut should_die = false;
    if let Some(entity) = state.entity_manager.get_entity(vid) {
        // Check if health is 0 AND it hasn't already been marked for death.
        if entity.health == 0 && !entity.marked_for_destruction {
            should_die = true;
        }
    }

    if should_die {
        // Trigger all the death effects (sound, particles, corpse).
        on_entity_death(state, audio, vid);

        // Mark the entity for cleanup at the end of the frame.
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            entity.marked_for_destruction = true;
            entity.state = EntityState::Dead; // Set state for clarity
        }
    }
}

// step as rail layer
/*
    if the entity is a rail layer, move it in the direction it is facing if its move cooldown is ready.
    ignore all tile collision no matter what
    convert the tile you move onto into a rail tile
    keep moving until you hit the edge of the map
    once you hit the edge of the map, mark yourself for destruction
*/
pub fn step_rail_layer(state: &mut State, audio: &mut Audio, vid: VID) {
    // check if exists
    if state.entity_manager.get_entity(vid).is_none() {
        return; // Entity not found, exit early
    }

    // check if entity is a rail layer
    if let Some(entity) = state.entity_manager.get_entity(vid) {
        if entity.type_ != crate::entity::EntityType::RailLayer {
            return; // Only rail layers should step this way
        }
    }

    // check if entity is ready to move
    if !ready_to_move(state, vid) {
        return; // Not ready to move yet
    }

    // get current position and direction
    let current_pos = state.entity_manager.get_entity(vid).unwrap().pos.as_ivec2();
    let direction = state.entity_manager.get_entity(vid).unwrap().direction;

    // calculate new position based on direction
    let new_pos = current_pos + direction;

    // check if new position is within bounds of the stage
    let mut went_out_of_bounds: bool = false;
    if !state.stage.in_bounds(new_pos) {
        // mark for destruction if out of bounds
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            entity.marked_for_destruction = true;
            println!(
                "Rail layer out of bounds, marking for destruction: {:?}",
                vid
            );
            went_out_of_bounds = true;
        }
    }
    if went_out_of_bounds {
        // print
        println!(
            "Rail layer {:?} went out of bounds at position {:?}",
            vid, new_pos
        );
        // print the direction it was going
        println!(
            "Rail layer {:?} was going in direction {:?}",
            vid, direction
        );
        // print the direction it came from
        let opposite_direction = -direction;
        println!(
            "Rail layer {:?} came from direction {:?}",
            vid, opposite_direction
        );

        // we will spawn a train at the start position on the first rail that was placed
        // calculate your start position by going in the opposite direction to the edge of the map
        let mut start_pos = new_pos + opposite_direction;
        while state.stage.in_bounds(start_pos) {
            start_pos += opposite_direction;
        }
        start_pos -= opposite_direction; // Step back to the last valid position

        // kill all entities in the start position instantly, just set health to 0
        // get all vids in the spatial grid at the start position
        if !state.stage.in_bounds(start_pos) {
            return;
        }
        let vids_in_start_pos = &state.spatial_grid[start_pos.x as usize][start_pos.y as usize];

        for entity_vid in vids_in_start_pos {
            if *entity_vid != vid {
                if let Some(entity) = state.entity_manager.get_entity_mut(*entity_vid) {
                    entity.health = 0; // Set health to 0 to simulate instant death
                }
            }
        }

        // spawn a train at the start position
        if let Some(new_entity_vid) = state.entity_manager.new_entity() {
            if let Some(entity) = state.entity_manager.get_entity_mut(new_entity_vid) {
                init_as_train(entity);
                entity.pos = start_pos.as_vec2() + Vec2::splat(0.5); // Center the train on the grid tile
                                                                     // set the direction to be the same as the rail layer
                entity.direction = direction;
                // set target position to be its starting position
                entity.target_pos = Some(start_pos.as_vec2() + Vec2::splat(0.5));
                // do int 5-20, cast to float
                let train_length: f32 = (random_range(5..=20) as f32).floor();
                entity.counter_a = train_length;
            }
        }
        return;
    }

    // convert the tile at new position into a rail tile
    // #[derive(Debug, Clone, Copy, PartialEq)]
    // pub struct TileData {
    //     pub tile: Tile,
    //     pub hp: u8,
    //     pub max_hp: u8,
    //     pub breakable: bool,
    //     pub variant: u8,
    //     pub flip_speed: u16,
    // }
    let mut tile = TileData::default();
    tile.tile = Tile::Rail; // Set the tile to Rail
    tile.breakable = false; // Rail tiles are not breakable
    tile.rot = 90.0; // No rotation for rail tiles

    state
        .stage
        .set_tile(new_pos.x as usize, new_pos.y as usize, tile);

    // move the entity to the new position
    move_entity_on_grid(state, audio, vid, new_pos, true, true, false);
}

/// now a step train
/*
    a train should move in the direction is is facing if that target tile is a rail tile
    when a train moves, it should immediatly do 1000 damage to all entities in the target tile (except trains)
    when a trains next move will put it off the map, mark itself for destruction
    trains ignore entity collision, but first check to make sure the target position doesnt have a train before trying to move


    if the target tile is not a rail tile, it should set its own hp to 0

    later: (do not implement now)
        and spawn a fire and a bunch of smoke particles
*/
pub fn step_train(state: &mut State, audio: &mut Audio, vid: VID) {
    // check if exists
    if state.entity_manager.get_entity(vid).is_none() {
        return; // Entity not found, exit early
    }

    // check if entity is a train
    if let Some(entity) = state.entity_manager.get_entity(vid) {
        if entity.type_ != crate::entity::EntityType::Train {
            return; // Only trains should step this way
        }
    }

    // check if entity is ready to move
    if !ready_to_move(state, vid) {
        return; // Not ready to move yet
    }

    // get current position and direction
    let current_pos = state.entity_manager.get_entity(vid).unwrap().pos.as_ivec2();
    let direction = state.entity_manager.get_entity(vid).unwrap().direction;

    // calculate new position based on direction
    let new_pos = current_pos + direction;

    // check if new position is within bounds of the stage
    if !state.stage.in_bounds(new_pos) {
        // mark for destruction if out of bounds
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            entity.marked_for_destruction = true;
        }
        return;
    }

    // check if the target tile is a rail tile
    let target_tile = state
        .stage
        .get_tile_type(new_pos.x as usize, new_pos.y as usize);
    if target_tile.is_none() || target_tile.unwrap() != Tile::Rail {
        // set own hp to 0 and mark for destruction
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            entity.health = 0;
            println!(
                "Train hit non-rail tile, marking for destruction: {:?}",
                vid
            );
        }
        return;
    }

    // check for other trains in the target position
    // pub spatial_grid: Vec<Vec<Vec<VID>>>,

    let other_trains_in_target = state.spatial_grid[new_pos.x as usize][new_pos.y as usize]
        .iter()
        .any(|&other_vid| {
            if let Some(other_entity) = state.entity_manager.get_entity(other_vid) {
                other_entity.type_ == EntityType::Train && other_vid != vid
            } else {
                false // Entity not found, treat as not a train
            }
        });

    if other_trains_in_target {
        // do not move
        return;
    }

    // move the entity to the new position
    let moved = move_entity_on_grid(state, audio, vid, new_pos, true, true, false);
    let mut hit_entities: Vec<VID> = vec![];
    if moved {
        // get the target tile data
        // damage all entities in the target tile (except trains)
        for entity in state.spatial_grid[new_pos.x as usize][new_pos.y as usize]
            .iter()
            .filter_map(|&vid| state.entity_manager.get_entity(vid))
        {
            if entity.type_ != crate::entity::EntityType::Train {
                hit_entities.push(entity.vid);
            }
        }

        tile_shake_area_at(state, new_pos, 2.0, 2.0);

        // fetch a target position, if none, dont do this part

        pub struct NewTrain {
            pub pos: Vec2,
            pub direction: IVec2,
            pub sprite: Option<Sprite>,
        }
        let mut new_train: Option<NewTrain> = None;
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            if let Some(target_pos) = entity.target_pos {
                // if counter_a is 0, spawn a train at the target position
                // if counter_a is 1, spawn a caboose at the target position
                // if counter_a is 2-inf, spawn a traincar at the target position
                if entity.counter_a >= 2.0 {
                    // spawn a traincar
                    new_train = Some(NewTrain {
                        pos: target_pos,
                        direction: entity.direction,
                        sprite: Some(Sprite::TrainCarA),
                    });
                } else if entity.counter_a > 0.0 {
                    // spawn a caboose
                    new_train = Some(NewTrain {
                        pos: target_pos,
                        direction: entity.direction,
                        sprite: Some(Sprite::Caboose),
                    });
                }
                // decrement the counter_a
                entity.counter_a -= 1.0;
                entity.counter_a = entity.counter_a.max(0.0);
            }
        }

        // if we have a new train, spawn it
        if let Some(new_train) = new_train {
            if let Some(new_entity_vid) = state.entity_manager.new_entity() {
                if let Some(entity) = state.entity_manager.get_entity_mut(new_entity_vid) {
                    init_as_train(entity);
                    entity.pos = new_train.pos; // Set the position to the target position
                    entity.direction = new_train.direction; // Set the direction to the same as the train
                    entity.sprite = new_train.sprite; // Set the sprite to the train car sprite
                }
            }
        }
    }

    // apply damage to all hit entities
    for hit_vid in hit_entities {
        if let Some(hit_entity) = state.entity_manager.get_entity_mut(hit_vid) {
            // apply 1000 damage
            hit_entity.health = hit_entity.health.saturating_sub(1000);
        }
    }
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
