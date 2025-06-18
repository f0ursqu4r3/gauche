use glam::Vec2;

pub enum Mode {
    Title,
    Settings,
    VideoSettings,
    Playing,
    GameOver,
    Win,
}

pub struct State {
    pub mode: Mode,
    pub world: World,
    pub player: Player,
    pub entities: Vec<Entity>,
}

impl State {
    pub fn new() -> Self {
        Self {
            mode: Mode::Title,
            world: World::new(64, 64),
            player: Player {
                pos: Vec2::new(50.0, 50.0),
                health: 100,
                inventory: vec![],
            },
            entities: vec![],
        }
    }
}

pub struct World {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Vec<Tile>>,
}

// constructor for world, 64 by 64 tiles
impl World {
    pub fn new(width: i32, height: i32) -> Self {
        let tiles = vec![
            vec![
                Tile {
                    pos: Vec2::ZERO,
                    walkable: true,
                    tile_type: TileType::Grass,
                };
                width as usize
            ];
            height as usize
        ];

        Self {
            width,
            height,
            tiles,
        }
    }
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
