use strum::{EnumCount, EnumIter, IntoStaticStr}; // Add IntoStaticStr here

/// Enum representing all static sprites in the game.
/// For example, `PlayerIdle` will automatically become "player_idle" when converted to a string.
#[derive(Copy, Clone, Debug, EnumIter, EnumCount, PartialEq, Eq, Hash, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Sprite {
    Reticle,
    Cursor,
    SelectedArrow,

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

    // Chicken Sprites
    Chick,
    Hen,
    Rooster,

    // Zombie Sprites
    Zombie,
    ZombieAngry,
    ZombieScratch1,
    ZombieDead,
    ZombieGib1,
    ZombieFootprint,

    BloodSmall,
    BloodMedium,

    Cloud1,
    Cloud2,
    Cloud3,

    // Item Sprites
    Fist,
    Medkit,
    Bandage,
    Bandaid,
    ConductorHat,

    // train
    TrainHead,
    TrainCarA,
    TrainCarB,
    Caboose,
    Rail,
    RailCrossing,
    TrainBlinkensign,
    TrainCarBlockPole,
}
