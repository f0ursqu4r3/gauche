use glam::Vec2;

pub struct State {
    pub world: World,
    pub player: Player,
    pub entities: Vec<Entity>,
}

pub struct World {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Vec<Tile>>,
}

impl World {
    pub fn tile_at(&self, x: i32, y: i32) -> Option<&Tile> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            None
        } else {
            self.tiles
                .get(x as usize)
                .and_then(|row| row.get(y as usize))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub pos: Vec2,
    pub walkable: bool,
    pub tile_type: TileType,
}

#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Grass,
    Wall,
    Water,
    Mountain,
}

pub struct Player {
    pub pos: Vec2,
    pub health: i32,
    pub inventory: Vec<Item>,
}

pub struct Item {
    pub name: String,
    pub quantity: i32,
}

pub struct Entity {
    pub id: u32,
    pub pos: Vec2,
    pub entity_type: EntityType,
}

pub enum EntityType {
    Monster,
    Item,
}
