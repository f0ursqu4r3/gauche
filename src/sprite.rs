use strum::{EnumCount, EnumIter, IntoStaticStr}; // Add IntoStaticStr here

/// Enum representing all static sprites in the game.
/// For example, `PlayerIdle` will automatically become "player_idle" when converted to a string.
#[derive(Copy, Clone, Debug, EnumIter, EnumCount, PartialEq, Eq, Hash, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Sprite {
    NoSprite,
    Reticle,
    Arrow,

    // Player Sprites
    Player,
    PlayerDead,
    PlayerFootprint,

    // Tile Sprites
    Grass,
    Wall,
    Ruin,
    Water1,
    Water2,
    Water3,
    Water4,

    // Enemy Sprites
    Zombie,
    ZombieAngry,
    ZombieScratch1,
    ZombieGib1,
    ZombieFootprint,
}
