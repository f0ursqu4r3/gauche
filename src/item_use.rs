use crate::{
    audio::{Audio, SoundEffect},
    entity::{DamageType, VID},
    entity_behavior::{attack, AttackType},
    graphics::Graphics,
    item::{Item, ItemType},
    stage::TileData,
    state::State,
    tile::{self, damage_tile},
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
    let can_be_attempted = item.usable || item.can_be_placed;
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
        ItemType::Fist => use_fist(state, graphics, audio, user_vid, item),
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

    let target_tile_pos = graphics.screen_to_tile(state.mouse_inputs.pos.as_vec2());
    let user_tile_pos = user.pos.as_ivec2();
    let distance = new_york_dist(user_tile_pos, target_tile_pos);

    // Check if the target tile is within the item's range and can be built on.
    if distance >= item.min_range as i32
        && distance <= item.range as i32
        && tile::can_build_on(state, target_tile_pos)
    {
        // Place the wall.
        state.stage.set_tile(
            target_tile_pos.x as usize,
            target_tile_pos.y as usize,
            TileData {
                tile: tile::Tile::Wall,
                hp: 100, // Example: full health for the block
                max_hp: 100,
                breakable: true,
                variant: 0,
                flip_speed: 0,
            },
        );

        audio.play_sound_effect(SoundEffect::BlockLand);
        return true; // Success
    }

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
            const HEAL_AMOUNT: u32 = 25;

            // Use the entity's own max_hp value
            if entity.health < entity.max_hp {
                entity.health = (entity.health + HEAL_AMOUNT).min(entity.max_hp);
                audio.play_sound_effect(SoundEffect::ClothRip);
                return true; // Success
            }
        }
    }
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

    let target_tile_pos = graphics.screen_to_tile(state.mouse_inputs.pos.as_vec2());
    let user_tile_pos = user_pos.as_ivec2();
    let distance = new_york_dist(user_tile_pos, target_tile_pos);

    if distance >= item.min_range as i32 && distance <= item.range as i32 {
        // --- 1. Prioritize attacking entities ---
        if let Some(cell) = state
            .spatial_grid
            .get(target_tile_pos.x as usize)
            .and_then(|col| col.get(target_tile_pos.y as usize))
        {
            if let Some(&attackee_vid) = cell.iter().find(|&&vid| {
                state
                    .entity_manager
                    .get_entity(vid)
                    .is_some_and(|e| e.alignment == crate::entity::Alignment::Enemy)
            }) {
                attack(
                    state,
                    audio,
                    &user_vid,
                    &attackee_vid,
                    AttackType::FistPunch,
                );
                return true;
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
            return true; // damage_tile returns true if it successfully dealt damage.
        }
    }

    false
}
