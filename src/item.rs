use crate::{item, item_use::use_item, sprite::Sprite, tile::Tile};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemType {
    Wall,
    Medkit,
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

    pub range: f32, // in tiles

    // --- Associated Game Objects ---
    pub tile: Option<Tile>,
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
                range: 1.0, // Walls can be placed on adjacent tiles
                tile: Some(Tile::Wall),
                sprite: None,
            },
            ItemType::Medkit => Item {
                type_: ItemType::Medkit,
                name: "Medkit",
                description: "first aid, second aid, cool-aid",
                marked_for_destruction: false,
                can_be_placed: false,
                usable: true,
                can_be_dropped: true,
                consume_on_use: true,
                max_count: 10,
                count: 1, // Will be set below
                use_cooldown: 5.0,
                use_cooldown_countdown: 0.0,
                range: 0.0, // Medkits are used on the player, not on tiles
                tile: None,
                sprite: None,
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
                use_cooldown: 0.5,
                use_cooldown_countdown: 0.0,
                range: 1.0, // Fists can hit adjacent tiles
                tile: None,
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
