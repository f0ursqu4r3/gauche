use crate::{item, item_use::use_item, sprite::Sprite, tile::Tile};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemType {
    Wall,
    Medkit,
    Bandage,
    Bandaid,
    Fist,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemAttributes {
    Strong,
    Agile,
    Durable,
    Fragile,
    Heavy, // makes you slow
    Big,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Item {
    pub type_: ItemType,
    pub name: &'static str,
    pub description: &'static str,
    pub marked_for_destruction: bool, // whether this item should be destroyed

    pub can_be_placed: bool,
    pub usable: bool,
    pub can_be_dropped: bool,

    pub max_count: u32,              // maximum count in this stack
    pub count: u32,                  // current count in this stack
    pub consume_on_use: bool,        // whether to consume the item on use
    pub use_cooldown: f32,           // cooldown in seconds after using this item
    pub use_cooldown_countdown: f32, // countdown for the cooldown

    pub min_range: f32, // in tiles, minimum range for use
    pub range: f32,     // in tiles

    // --- Associated Game Objects ---
    pub sprite: Option<Sprite>,
    // pub attributes: Vec<ItemAttributes>,
}

impl Item {
    /// Creates a new item stack of a given type and count.
    pub fn new(kind: ItemType) -> Self {
        match kind {
            ItemType::Wall => Item {
                type_: ItemType::Wall,
                name: "Wall",
                description: "...a wall",
                marked_for_destruction: false,
                can_be_placed: true,
                usable: false,
                can_be_dropped: true,
                max_count: 99,
                count: 1, // Will be set below
                consume_on_use: true,
                use_cooldown: 0.1,
                use_cooldown_countdown: 0.0,
                min_range: 1.0, // Walls can be placed on the same tile
                range: 2.0,     // Walls can be placed on adjacent tiles
                sprite: Some(Sprite::Wall),
            },
            // the big heal
            ItemType::Medkit => Item {
                type_: ItemType::Medkit,
                name: "Medkit",
                description: "first aid, second aid, cool-aid",
                marked_for_destruction: false,
                can_be_placed: false,
                usable: true,
                can_be_dropped: true,
                consume_on_use: true,
                max_count: 10, // Medkits are not stackable
                count: 1,
                use_cooldown: 5.0,
                use_cooldown_countdown: 0.0,
                min_range: 0.0, // Medkits are used on the player, not on tiles
                range: 0.0,     // Medkits are used on the player, not on tiles
                sprite: Some(Sprite::Medkit),
            },
            // the medium heal: bandage
            ItemType::Bandage => Item {
                type_: ItemType::Bandage, // Using Medkit type for now
                name: "Bandage",
                description: "a bandage to stop the bleeding",
                marked_for_destruction: false,
                can_be_placed: false,
                usable: true,
                can_be_dropped: true,
                consume_on_use: true,
                max_count: 10,
                count: 1,
                use_cooldown: 2.0,
                use_cooldown_countdown: 0.0,
                min_range: 0.0, // Bandages are used on the player, not on tiles
                range: 0.0,     // Bandages are used on the player, not on tiles
                sprite: Some(Sprite::Bandage),
            },
            // the mini heal: bandaid
            ItemType::Bandaid => Item {
                type_: ItemType::Bandaid, // Using Medkit type for now
                name: "Bandaid",
                description: "a bandaid to stop the bleeding",
                marked_for_destruction: false,
                can_be_placed: false,
                usable: true,
                can_be_dropped: true,
                consume_on_use: true,
                max_count: 20,
                count: 1,
                use_cooldown: 0.2,
                use_cooldown_countdown: 0.0,
                min_range: 0.0, // Bandaids are used on the player, not on tiles
                range: 0.0,     // Bandaids are used on the player, not on tiles
                sprite: Some(Sprite::Bandaid),
            },

            ItemType::Fist => Item {
                type_: ItemType::Fist,
                name: "Fist",
                description: "your fist",
                marked_for_destruction: false,
                can_be_placed: false,
                usable: true,
                can_be_dropped: true,
                consume_on_use: false,
                max_count: 1, // Fists are not stackable
                count: 1,     // Always 1 for fists
                use_cooldown: 0.2,
                use_cooldown_countdown: 0.0,
                min_range: 1.0, // Fists can hit the same tile
                range: 1.0,     // Fists can hit adjacent tiles
                sprite: Some(Sprite::Fist),
            },
        }
    }

    /// Returns whether the item stack is stackable (i.e., its max_count > 1).
    pub fn is_stackable(&self) -> bool {
        self.max_count > 1
    }

    pub fn step_cooldown(&mut self, dt: f32) {
        // Decrease the cooldown countdown by the elapsed time
        if self.use_cooldown_countdown > 0.0 {
            self.use_cooldown_countdown -= dt;
            if self.use_cooldown_countdown < 0.0 {
                self.use_cooldown_countdown = 0.0; // Ensure it doesn't go negative
            }
        }
    }
}

pub fn can_use_item(item: &Item) -> bool {
    // Check if the item is usable and not on cooldown
    item.usable && item.use_cooldown_countdown <= 0.0
}
