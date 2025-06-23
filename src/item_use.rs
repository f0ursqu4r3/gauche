use crate::{
    audio::Audio,
    entity::VID,
    item::{Item, ItemType},
    state::State,
};

////////////////////////////////////////////        BASE USE LOGIC        ////////////////////////////////////////////

/// Uses the item, reducing its count and applying effects.
pub fn use_item(
    state: &mut State,
    audio: &mut Audio,
    dt: f32,
    user_vid: Option<VID>,
    item: &mut Item,
) -> bool {
    if user_vid.is_some() && !item.usable {
        return false;
    }

    if item.use_cooldown_countdown > 0.0 {
        return false; // Item is still on cooldown
    }

    if item.count == 0 {
        return false; // No items left to use
    }

    // try to use it
    use_item_internal_lookup(state, audio, dt, user_vid, item);

    // decrement the item count if it is consumable
    if item.consume_on_use && item.count > 0 && item.is_stackable() {
        item.count -= 1;
        if item.count == 0 {
            item.marked_for_destruction = true; // Mark the item for destruction if count reaches 0
        }
    }

    // reset the cooldown
    item.step_cooldown(dt);

    true
}
//////////////////////////////////////////// ITEM LOGIC LUT ////////////////////////////////////////////

pub fn use_item_internal_lookup(
    state: &mut State,
    audio: &mut Audio,
    dt: f32,
    user_vid: Option<VID>,
    item: &mut Item,
) {
    match item.type_ {
        ItemType::Wall => use_wall(state, audio, dt, user_vid, item),
        ItemType::Medkit => use_medkit(state, audio, dt, user_vid, item),
    }
}

//////////////////////////////////////////// SPECIFIC ITEM LOGIC ////////////////////////////////////////////
pub fn use_wall(
    state: &mut State,
    audio: &mut Audio,
    dt: f32,
    user_vid: Option<VID>,
    item: &mut Item,
) {
    // Logic for using a wall item
    if let Some(vid) = user_vid {
        // Place the wall at the user's position or target position
        // This is a placeholder; actual implementation will depend on game logic
        println!("Using wall item by user with VID: {:?}", vid);
    }
}
pub fn use_medkit(
    state: &mut State,
    audio: &mut Audio,
    dt: f32,
    user_vid: Option<VID>,
    item: &mut Item,
) {
    // Logic for using a medkit item
    if let Some(vid) = user_vid {
        // Heal the user entity
        if let Some(entity) = state.entity_manager.get_entity_mut(vid) {
            entity.health += 20; // Heal by 20 health points
            if entity.health > 100 {
                entity.health = 100; // Cap health at 100
            }
            println!(
                "User with VID {:?} healed to {} health.",
                vid, entity.health
            );
        }
    }
}
