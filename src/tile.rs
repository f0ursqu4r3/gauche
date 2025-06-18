#[derive(Debug, Clone, Copy)]
pub enum Tile {
    None,
    Grass,
    Wall,
    Water,
}

pub fn walkable(tile: Tile) -> bool {
    matches!(tile, Tile::None | Tile::Grass | Tile::Water)
}
