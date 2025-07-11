use glam::{IVec2, Vec2};

use crate::{
    audio::{Audio, SoundEffect},
    entity::{Alignment, DamageType, VID},
    entity_behavior::{attack, AttackType},
    entity_templates::init_as_rail_layer,
    graphics::Graphics,
    item::{Item, ItemType},
    render::TILE_SIZE,
    stage::TileData,
    state::State,
    tile::{self, damage_tile, tile_shake_area_at, Tile},
    utils::new_york_dist,
};

////////////////////////////////////////////        BASE USE LOGIC        ////////////////////////////////////////////

/// The primary entry point for using an item.
/// It checks for usability and cooldowns, calls the specific item logic,
/// and then applies consumption and cooldown effects if the use was successful.
pub fn use_item(
    state: &mut State,
    graphics: &Graphics,
    audio: &mut Audio,
    user_vid: Option<VID>,
    item: &mut Item,
) -> bool {
    // An item can be used if it's 'usable' (like a medkit) or 'placeable' (like a wall).
    let can_be_attempted = item.usable;
    if !can_be_attempted || item.use_cooldown_countdown > 0.0 || item.count == 0 {
        return false;
    }

    // Attempt to use the item by calling the specific logic function.
    let success = use_item_internal_lookup(state, graphics, audio, user_vid, item);

    if success {
        // If the action was successful, apply cooldown and consumption.
        item.use_cooldown_countdown = item.use_cooldown;

        if item.consume_on_use {
            if item.count > 0 {
                item.count -= 1;
            }
            if item.count == 0 {
                item.marked_for_destruction = true;
            }
        }
    }

    success
}

//////////////////////////////////////////// ITEM LOGIC LUT ////////////////////////////////////////////

/// Calls the correct specific-item-use function based on the item's type.
fn use_item_internal_lookup(
    state: &mut State,
    graphics: &Graphics,
    audio: &mut Audio,
    user_vid: Option<VID>,
    item: &mut Item,
) -> bool {
    // Returns true on successful use
    match item.type_ {
        ItemType::Wall => use_wall(state, graphics, audio, user_vid, item),
        ItemType::Medkit => use_medkit(state, audio, user_vid, item),
        ItemType::Bandage => use_bandage(state, audio, user_vid, item),
        ItemType::Bandaid => use_bandaid(state, audio, user_vid, item),
        ItemType::Fist => use_fist(state, graphics, audio, user_vid, item),
        ItemType::ConductorHat => use_conductor_hat(state, audio, user_vid, item),
    }
}

//////////////////////////////////////////// SPECIFIC ITEM LOGIC ////////////////////////////////////////////
/// Places a wall tile at the mouse cursor location if within range and on a valid tile.
pub fn use_wall(
    state: &mut State,
    graphics: &Graphics,
    audio: &mut Audio,
    user_vid: Option<VID>,
    item: &Item,
) -> bool {
    let user = match user_vid.and_then(|vid| state.entity_manager.get_entity(vid)) {
        Some(e) => e,
        None => return false,
    };

    let target_tile_pos = match get_item_use_pos(state, graphics) {
        Some(tile) => tile,
        None => return false, // Invalid tile position
    };
    let user_tile_pos = user.pos.as_ivec2();
    let distance = new_york_dist(user_tile_pos, target_tile_pos);

    // Check if the target tile is within the item's range and can be built on.
    if distance >= item.min_range as i32
        && distance <= item.range as i32
        && tile::can_build_on(state, target_tile_pos)
    {
        // Place the wall.
        let mut tile = TileData::default();
        tile.tile = Tile::Wall;
        tile.hp = 100;
        tile.max_hp = 100;
        tile.breakable = true;

        state
            .stage
            .set_tile(target_tile_pos.x as usize, target_tile_pos.y as usize, tile);

        audio.play_sound_effect(SoundEffect::BlockLand);
        return true; // Success
    }

    audio.play_sound_effect(SoundEffect::CantUse);

    false // Use failed
}

/// Heals the user for a fixed amount if their health is not full.
pub fn use_medkit(
    state: &mut State,
    audio: &mut Audio,
    user_vid: Option<VID>,
    _item: &Item,
) -> bool {
    if let Some(vid) = user_vid {
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            const HEAL_AMOUNT: u32 = 100;

            // Use the entity's own max_hp value
            if entity.health < entity.max_hp {
                entity.health = (entity.health + HEAL_AMOUNT).min(entity.max_hp);
                audio.play_sound_effect(SoundEffect::ClothRip);
                return true; // Success
            }
        }
    }
    audio.play_sound_effect(SoundEffect::CantUse);

    false
}

/// bandage is like medkit but only heals 10 HP
pub fn use_bandage(
    state: &mut State,
    audio: &mut Audio,
    user_vid: Option<VID>,
    _item: &Item,
) -> bool {
    if let Some(vid) = user_vid {
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            const HEAL_AMOUNT: u32 = 10;

            // Use the entity's own max_hp value
            if entity.health < entity.max_hp {
                entity.health = (entity.health + HEAL_AMOUNT).min(entity.max_hp);
                audio.play_sound_effect(SoundEffect::ClothRip);
                return true; // Success
            }
        }
    }
    audio.play_sound_effect(SoundEffect::CantUse);

    false
}

/// bandaid is like medkit but only heals 1 HP
pub fn use_bandaid(
    state: &mut State,
    audio: &mut Audio,
    user_vid: Option<VID>,
    _item: &Item,
) -> bool {
    if let Some(vid) = user_vid {
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            const HEAL_AMOUNT: u32 = 1;

            // Use the entity's own max_hp value
            if entity.health < entity.max_hp {
                entity.health = (entity.health + HEAL_AMOUNT).min(entity.max_hp);
                audio.play_sound_effect(SoundEffect::ClothRip);
                return true; // Success
            }
        }
    }
    audio.play_sound_effect(SoundEffect::CantUse);

    false
}

/// Attacks an entity or damages a tile at the mouse cursor location.
pub fn use_fist(
    state: &mut State,
    graphics: &Graphics,
    audio: &mut Audio,
    user_vid: Option<VID>,
    item: &Item,
) -> bool {
    let user_vid = match user_vid {
        Some(vid) => vid,
        None => return false,
    };
    // Get user position separately to avoid borrow issues
    let user_pos = match state.entity_manager.get_entity(user_vid) {
        Some(e) => e.pos,
        None => return false,
    };

    let target_tile_pos = match get_item_use_pos(state, graphics) {
        Some(tile) => tile,
        None => return false, // Invalid tile position
    };
    let user_tile_pos = user_pos.as_ivec2();
    let distance = new_york_dist(user_tile_pos, target_tile_pos);

    if distance >= item.min_range as i32 && distance <= item.range as i32 {
        // --- 1. Prioritize attacking entities ---
        if let Some(vids_in_cell) = state
            .spatial_grid
            .get(target_tile_pos.x as usize)
            .and_then(|col| col.get(target_tile_pos.y as usize))
        {
            // if theres even one, just attack the first one
            if let Some(&attackee_vid) = vids_in_cell.first() {
                // Perform the attack
                attack(
                    state,
                    audio,
                    &user_vid,
                    &attackee_vid,
                    AttackType::FistPunch,
                );
                return true; // Successfully attacked an entity
            }
        }

        // --- 2. If no entity, try to damage a tile ---
        const FIST_DAMAGE: u8 = 10;
        if damage_tile(
            state,
            audio,
            target_tile_pos,
            FIST_DAMAGE,
            DamageType::Punch,
            user_pos,
        ) {
            tile_shake_area_at(
                state,
                target_tile_pos,
                0.3, //shake amount
                2.0, //dist
            );
            return true; // damage_tile returns true if it successfully dealt damage.
        }
    }

    audio.play_sound_effect(SoundEffect::CantUse);
    false
}

pub fn use_conductor_hat(
    state: &mut State,
    audio: &mut Audio,
    user_vid: Option<VID>,
    _item: &Item,
) -> bool {
    // get the user position
    // calculate the right most position on the map at the players y coordinate
    // spawn a train at that position

    let user_vid = match user_vid {
        Some(vid) => vid,
        None => return false,
    };

    let user_pos = match state.entity_manager.get_entity(user_vid) {
        Some(e) => e.pos,
        None => return false,
    };

    // Calculate the rightmost position on the map at the user's y coordinate
    let stage_width = state.stage.get_width() as f32;
    let rightmost_x = stage_width;
    let user_y = user_pos.y;
    let rail_layer_pos = Vec2::new(rightmost_x, user_y.floor());
    // Spawn the train entity
    if let Some(vid) = state.entity_manager.new_entity() {
        if let Some(rail_layer) = state.entity_manager.get_entity_mut(vid) {
            init_as_rail_layer(rail_layer);
            rail_layer.pos = rail_layer_pos;
            rail_layer.direction = IVec2::new(-1, 0); // Rail layer moves left
        }
    }

    audio.play_sound_effect(SoundEffect::DistantTrainSound);
    true
}

pub fn get_item_use_pos(state: &State, graphics: &Graphics) -> Option<IVec2> {
    if state.playing_inputs.use_down
        || state.playing_inputs.use_up
        || state.playing_inputs.use_left
        || state.playing_inputs.use_right
    {
        // Use the tile in the direction of the arrow keys
        if let Some(player_vid) = state.player_vid {
            if let Some(player) = state.entity_manager.get_entity(player_vid) {
                let player_pos = player.pos.as_ivec2();
                // do left right up down, but also diagonal if combined
                let item_use_offset =
                    if state.playing_inputs.use_down && state.playing_inputs.use_right {
                        IVec2::new(1, 1)
                    } else if state.playing_inputs.use_down && state.playing_inputs.use_left {
                        IVec2::new(-1, 1)
                    } else if state.playing_inputs.use_up && state.playing_inputs.use_right {
                        IVec2::new(1, -1)
                    } else if state.playing_inputs.use_up && state.playing_inputs.use_left {
                        IVec2::new(-1, -1)
                    } else if state.playing_inputs.use_down {
                        IVec2::new(0, 1)
                    } else if state.playing_inputs.use_up {
                        IVec2::new(0, -1)
                    } else if state.playing_inputs.use_left {
                        IVec2::new(-1, 0)
                    } else if state.playing_inputs.use_right {
                        IVec2::new(1, 0)
                    } else {
                        return None; // No valid direction
                    };

                return Some(player_pos + item_use_offset);
            }
            return None;
        }
        return None; // No player to use item
    } else if state.mouse_inputs.left {
        // Use the tile under the mouse cursor
        Some(
            graphics
                .screen_to_world(state.mouse_inputs.pos.as_vec2())
                .as_ivec2(),
        )
    // or space for use on self
    } else if state.playing_inputs.use_center {
        if let Some(player_vid) = state.player_vid {
            if let Some(player) = state.entity_manager.get_entity(player_vid) {
                return Some(player.pos.as_ivec2());
            }
        }
        None // No player to use item on
    } else {
        None // No item use action
    }
}
