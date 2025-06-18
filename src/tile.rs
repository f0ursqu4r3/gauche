use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub pos: Vec2,
    pub walkable: bool,
    pub tile_type: TileType,
}

impl Tile {
    pub const SIZE: i32 = 16;
}

#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Grass,
    Wall,
    Water,
    Mountain,
}
