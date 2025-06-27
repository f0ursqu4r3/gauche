use glam::*;
use rand::random_range;
use raylib::prelude::*;

pub const FRAMES_PER_SECOND: u32 = 60;
pub const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

use crate::{
    audio::{Audio, SoundEffect},
    entity::{self, Entity, EntityType, StepSound, VID},
    entity_behavior::{
        die_if_health_zero, growl_sometimes, indiscriminately_attack_nearby, move_entity_on_grid,
        ready_to_move, step_attack_cooldown, step_inventory_item_cooldowns, step_move_cooldown,
        step_rail_layer, step_train, wander,
    },
    entity_templates::init_as_item,
    graphics::Graphics,
    item::Item,
    item_use,
    particle_templates::spawn_weather_clouds,
    render::TILE_SIZE,
    stage::{flip_stage_tiles, TileData},
    state::{Mode, State},
    tile::{self, can_build_on, Tile},
};

pub const PLACE_TILE_COOLDOWN: f32 = 0.05; // Cooldown for placing tiles in seconds

pub fn step(
    rl: &mut RaylibHandle,
    _rlt: &mut RaylibThread,
    state: &mut State,
    audio: &mut Audio,
    graphics: &mut Graphics,
    dt: f32,
) {
    state.time_since_last_update += rl.get_frame_time();

    /* FYI: while loop makes step spin until catchup if we are behind some frames. This is on purpose*/
    while state.time_since_last_update > TIMESTEP {
        state.time_since_last_update -= TIMESTEP;

        if state.frame_pause > 0 {
            state.frame_pause -= 1;
            return;
        }

        match state.mode {
            Mode::Title => step_title(state, audio),
            Mode::Playing => step_playing(state, audio, graphics),
            _ => {} // Other modes
        }

        state.particles.step();
        state.scene_frame = state.scene_frame.saturating_add(1);

        if state.frame == u32::MAX {
            state.frame = 0;
        } else {
            state.frame += 1;
        }
    }

    // step sound effect cooldowns
    audio.step_sound_effect_cooldowns(dt);
}

fn step_title(state: &mut State, _audio: &mut Audio) {
    if state.menu_inputs.confirm {
        // NOTE: The actual transition now happens in `process_input_title`
        // to ensure `init_playing_state` is called.
        // This remains for potential future menu logic.
    }
}

fn step_playing(state: &mut State, audio: &mut Audio, graphics: &mut Graphics) {
    // game over if no player
    if state.player_vid.is_none() {
        state.mode = Mode::GameOver;
        state.game_over = true;
        return;
    };

    // set inventory index from numpad
    set_inventory_index_from_numpad(state);

    // --- Player Movement ---
    if let Some(player_vid) = state.player_vid {
        if ready_to_move(state, player_vid) {
            let player_tile_pos = state
                .entity_manager
                .get_entity(player_vid)
                .unwrap()
                .pos
                .as_ivec2();

            let wants_to_move_to = if state.playing_inputs.left {
                player_tile_pos + IVec2::new(-1, 0)
            } else if state.playing_inputs.right {
                player_tile_pos + IVec2::new(1, 0)
            } else if state.playing_inputs.up {
                player_tile_pos + IVec2::new(0, -1)
            } else if state.playing_inputs.down {
                player_tile_pos + IVec2::new(0, 1)
            } else {
                player_tile_pos // No movement input
            };

            if wants_to_move_to != player_tile_pos {
                move_entity_on_grid(
                    state,
                    audio,
                    player_vid,
                    wants_to_move_to,
                    false, // Do not ignore tile collision for player
                    false, // reset move cooldown
                    false, // do not ignore entity collision for player
                );
            }
        }

        let target_cam_pos = state.entity_manager.get_entity(player_vid).unwrap().pos * TILE_SIZE;
        graphics.play_cam.pos = graphics.play_cam.pos.lerp(target_cam_pos, 0.1);
    }

    // player item drop logic
    /*
       if no item in the selected slot, do nothing
       if theres an item in the players inventory in the selected slot
       check the tile to see if it has an entity on it
       go through the entities on the tile to see if any of them are item type
       if yes, cant drop
       else, drop the item, and remove it from the inventory
    */
    let mut item_to_try_to_drop: Option<Item> = None;
    let mut drop_location: Option<IVec2> = None;
    if state.playing_inputs.drop {
        if let Some(player_vid) = state.player_vid {
            if let Some(player) = state.entity_manager.get_entity_mut(player_vid) {
                // Check if the player has an item in the selected slot
                if player.inventory.has_selected_entry() {
                    let selected_index = player.inventory.selected_index;
                    if let Some(entry) = player.inventory.get(selected_index) {
                        // make a copy of the item to drop
                        if entry.item.droppable {
                            // Check if the tile is empty or has no item entities
                            let tile_pos = player.pos.as_ivec2();
                            item_to_try_to_drop = Some(entry.item);
                            drop_location = Some(tile_pos);
                        } else {
                            audio.play_sound_effect(SoundEffect::CantUse);
                        }
                    }
                }
            }
        }
    }
    if let (Some(item), Some(location)) = (item_to_try_to_drop, drop_location) {
        // Check if the tile is empty or has no item entities
        let tile_pos = location;
        let entities_on_tile = state.spatial_grid[tile_pos.x as usize][tile_pos.y as usize]
            .iter()
            .filter_map(|vid| state.entity_manager.get_entity(*vid))
            .collect::<Vec<&Entity>>();

        if entities_on_tile.iter().all(|e| e.type_ != EntityType::Item) {
            // make a new entity
            if let Some(new_entity_vid) = state.entity_manager.new_entity() {
                if let Some(entity) = state.entity_manager.get_entity_mut(new_entity_vid) {
                    // Initialize the new entity as an item with the dropped item
                    init_as_item(entity, item);
                    entity.pos =
                        Vec2::new(tile_pos.x as f32, tile_pos.y as f32) + Vec2::new(0.5, 0.5);
                    state.add_entity_to_grid(new_entity_vid, tile_pos);

                    // remove item from player inventory
                    if let Some(player_vid) = state.player_vid {
                        if let Some(player) = state.entity_manager.get_entity_mut(player_vid) {
                            player.inventory.remove_selected_entry();
                        }
                    }
                }
            }
        } else {
            audio.play_sound_effect(SoundEffect::CantUse);
        }
    }

    // player item pickup logic
    /*
       if item already in the selected slot, do nothing
       if theres no item in the players inventory in the selected slot
       check the tile to see if it has an entity on it
       go through the entities on the tile to see if any of them are item type
       if yes, add that item to the inventory
       delete the item entity from the grid
       else, do nothing
    */
    let mut item_to_try_to_pickup: Option<Item> = None;
    let mut entity_of_item: Option<VID> = None;
    let mut location_to_pickup: Option<IVec2> = None;
    if state.playing_inputs.pick_up {
        if let Some(player_vid) = state.player_vid {
            if let Some(player) = state.entity_manager.get_entity_mut(player_vid) {
                // Check if the tile has an item entity
                let tile_pos = player.pos.as_ivec2();
                let entities_on_tile = state.spatial_grid[tile_pos.x as usize][tile_pos.y as usize]
                    .iter()
                    .filter_map(|vid| state.entity_manager.get_entity(*vid))
                    .collect::<Vec<&Entity>>();
                // Find the first item entity on the tile
                if let Some(item_entity) = entities_on_tile
                    .iter()
                    .find(|e| e.type_ == EntityType::Item)
                {
                    item_to_try_to_pickup = item_entity.item;
                    entity_of_item = Some(item_entity.vid);
                    location_to_pickup = Some(tile_pos);
                } else {
                    // No item entity found on the tile, play can't use sound
                    audio.play_sound_effect(SoundEffect::CantUse);
                }
            }
        }
    }
    if let (Some(item), Some(location), Some(entity_of_item_vid)) =
        (item_to_try_to_pickup, location_to_pickup, entity_of_item)
    {
        // Check if the item can be added to the inventory
        if let Some(player_vid) = state.player_vid {
            if let Some(player) = state.entity_manager.get_entity_mut(player_vid) {
                let item_to_put_on_ground = player.inventory.insert(item);
                // remove the item entity from the grid, and deactivate it
                if state
                    .entity_manager
                    .get_entity_mut(entity_of_item_vid)
                    .is_some()
                {
                    state.remove_entity_from_grid(entity_of_item_vid, location);
                    state.entity_manager.set_inactive_vid(entity_of_item_vid);
                }

                // if an item was swapped out, drop it on the ground
                if let Some(item_to_drop_put_on_ground) = item_to_put_on_ground {
                    // Check if the tile is empty or has no item entities
                    let tile_pos = location;
                    let entities_on_tile = state.spatial_grid[tile_pos.x as usize]
                        [tile_pos.y as usize]
                        .iter()
                        .filter_map(|vid| state.entity_manager.get_entity(*vid))
                        .collect::<Vec<&Entity>>();

                    if entities_on_tile.iter().all(|e| e.type_ != EntityType::Item) {
                        // Create a new entity for the dropped item
                        if let Some(new_entity_vid) = state.entity_manager.new_entity() {
                            if let Some(entity) =
                                state.entity_manager.get_entity_mut(new_entity_vid)
                            {
                                init_as_item(entity, item_to_drop_put_on_ground);
                                entity.pos = Vec2::new(tile_pos.x as f32, tile_pos.y as f32)
                                    + Vec2::new(0.5, 0.5);
                                state.add_entity_to_grid(new_entity_vid, tile_pos);
                            }
                        }
                    } else {
                        // print a swapped item was deleted...
                        audio.play_sound_effect(SoundEffect::CantUse);
                    }
                }
            }
        }
    }

    // --- Player Item Use Logic ---
    let use_item = state.playing_inputs.use_center
        || state.playing_inputs.use_down
        || state.playing_inputs.use_up
        || state.playing_inputs.use_left
        || state.playing_inputs.use_right
        || state.mouse_inputs.left;

    // do item use
    if use_item {
        if let Some(player_vid) = state.player_vid {
            let mut temp_item: Option<Item> = None;
            let mut selected_index = 0;

            // Scope 1: Get the item out of the inventory.
            // This borrows `state` mutably, but the borrow ends when the scope does.
            if let Some(player) = state.entity_manager.get_entity_mut(player_vid) {
                selected_index = player.inventory.selected_index;
                if let Some(entry) = player.inventory.get_mut(selected_index) {
                    // Temporarily replace the item in the inventory with a dummy placeholder.
                    // We take ownership of the real item we want to use.
                    temp_item = Some(std::mem::replace(
                        &mut entry.item,
                        Item::new(crate::item::ItemType::Fist), // A dummy item
                    ));
                }
            } // Mutable borrow of `state` via `player` ends here.

            // Scope 2: Use the item.
            // We can now borrow `state` again because the previous borrow is gone.
            if let Some(mut item_to_use) = temp_item {
                // Call the use function with the item we took.
                let used =
                    item_use::use_item(state, graphics, audio, Some(player_vid), &mut item_to_use);
                // if used, determine which keys/mouse button triggered it and set the input

                // Scope 3: Put the item back (or handle its destruction).
                // This is another, separate mutable borrow of `state`.
                if let Some(player) = state.entity_manager.get_entity_mut(player_vid) {
                    if item_to_use.marked_for_destruction {
                        // The item was used up, so we remove its entry from the inventory.
                        player
                            .inventory
                            .entries
                            .retain(|e| e.index != selected_index);
                    } else {
                        // The item was not used up, so find the entry (which now holds the dummy)
                        // and put the real item back.
                        if let Some(entry) = player.inventory.get_mut(selected_index) {
                            entry.item = item_to_use;
                        }
                    }
                }
            }
        }
    }
    // // Consume the click so it doesn't trigger again next frame.
    // state.mouse_inputs.left = false;

    // --- AI / Other Entity Logic ---
    for vid in state.entity_manager.get_active_vids() {
        step_move_cooldown(state, vid);
        wander(state, audio, vid);
        entity_shake_attenuation(state, vid);
        growl_sometimes(state, audio, vid);
        indiscriminately_attack_nearby(state, audio, vid);
        die_if_health_zero(state, audio, vid);
        step_attack_cooldown(state, vid);
        step_inventory_item_cooldowns(state, vid);
        step_rail_layer(state, audio, vid);
        step_train(state, audio, vid);
    }

    // flip tile variants
    flip_stage_tiles(state);

    spawn_weather_clouds(state, graphics, state.cloud_density);

    // --- Entity Cleanup ("Sweep" Phase) ---
    // At the very end of the step, we remove all entities that were marked for destruction.
    let vids_to_remove: Vec<(VID, IVec2)> = state
        .entity_manager
        .iter()
        .filter(|e| e.marked_for_destruction && e.active)
        .map(|e| (e.vid, e.pos.as_ivec2()))
        .collect();

    for (vid, pos) in vids_to_remove {
        // Remove from the spatial grid to prevent ghost collisions
        state.remove_entity_from_grid(vid, pos);
        // Deactivate the entity in the manager, freeing up its ID
        state.entity_manager.set_inactive_vid(vid);

        // If the player died, update the game state accordingly
        if let Some(player_vid) = state.player_vid {
            if player_vid == vid {
                state.player_vid = None;
            }
        }
    }
}

/// Sets entity rotation from -15 to 15 degrees randomly
pub fn lean_entity(entity: &mut Entity) {
    entity.rot = random_range(-15.0..=15.0);
}

pub fn entity_step_sound_lookup(entity: &Entity) -> SoundEffect {
    // TODO: different step sounds based on entity type or state
    match entity.type_ {
        EntityType::RailLayer => SoundEffect::RailPlace,
        _ => match entity.step_sound {
            StepSound::Step1 => SoundEffect::Step1,
            StepSound::Step2 => SoundEffect::Step2,
        },
    }
}

pub fn entity_shake_attenuation(state: &mut State, vid: VID) {
    let entity = state.entity_manager.get_entity_mut(vid).unwrap();
    pub const SHAKE_ATTENUATION_RATE: f32 = 0.01;
    if entity.shake > SHAKE_ATTENUATION_RATE {
        entity.shake -= SHAKE_ATTENUATION_RATE
    } else {
        entity.shake = 0.0
    }
}

pub fn drop_item(state: &mut State, audio: &mut Audio, item: Item, pos: IVec2) -> Option<VID> {
    // Check if the tile is empty or has no item entities
    let entities_on_tile = state.spatial_grid[pos.x as usize][pos.y as usize]
        .iter()
        .filter_map(|vid| state.entity_manager.get_entity(*vid))
        .collect::<Vec<&Entity>>();

    if entities_on_tile.iter().any(|e| e.type_ == EntityType::Item) {
        audio.play_sound_effect(SoundEffect::CantUse);
        return None; // Cannot drop item, tile is occupied by an item entity
    }

    // Create a new entity for the dropped item
    if let Some(new_entity_vid) = state.entity_manager.new_entity() {
        if let Some(entity) = state.entity_manager.get_entity_mut(new_entity_vid) {
            init_as_item(entity, item);
            entity.pos = Vec2::new(pos.x as f32, pos.y as f32) + Vec2::new(0.5, 0.5);
            state.add_entity_to_grid(new_entity_vid, pos);
            return Some(new_entity_vid);
        }
    }
    None
}

pub fn set_inventory_index_from_numpad(state: &mut State) {
    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity_mut(player_vid) {
            if state.playing_inputs.select_inventory_index_0 {
                player.inventory.selected_index = 0;
            } else if state.playing_inputs.select_inventory_index_1 {
                player.inventory.selected_index = 1;
            } else if state.playing_inputs.select_inventory_index_2 {
                player.inventory.selected_index = 2;
            } else if state.playing_inputs.select_inventory_index_3 {
                player.inventory.selected_index = 3;
            } else if state.playing_inputs.select_inventory_index_4 {
                player.inventory.selected_index = 4;
            } else if state.playing_inputs.select_inventory_index_5 {
                player.inventory.selected_index = 5;
            } else if state.playing_inputs.select_inventory_index_6 {
                player.inventory.selected_index = 6;
            } else if state.playing_inputs.select_inventory_index_7 {
                player.inventory.selected_index = 7;
            } else if state.playing_inputs.select_inventory_index_8 {
                player.inventory.selected_index = 8;
            } else if state.playing_inputs.select_inventory_index_9 {
                player.inventory.selected_index = 9;
            }
        }
    }
}
