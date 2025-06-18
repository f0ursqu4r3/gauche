use glam::Vec2;

use crate::{
    inputs::{MenuInputDebounceTimers, MenuInputs, PlayingInputs},
    tile::{Tile, TileType},
};

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

    pub menu_inputs: MenuInputs,
    pub menu_input_debounce_timers: MenuInputDebounceTimers,
    pub playing_inputs: PlayingInputs,

    pub running: bool,
    pub now: f64,
    pub time_since_last_update: f32,
    pub scene_frame: u32,
    pub frame: u32,
    pub stage_frame: u32,

    pub game_over: bool,
    pub pause: bool,
    pub win: bool,

    pub points: u32,
    pub deaths: u32,
    pub frame_pause: u32,

    pub entity_manager: EntityManager,
    // pub special_effects: Vec<Box<dyn SpecialEffect>>,
    pub stage: Stage,

    pub world: World,
    pub player: Player,
    pub entities: Vec<Entity>,

    pub rebuild_render_texture: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            mode: Mode::Title,

            menu_inputs: MenuInputs::new(),
            menu_input_debounce_timers: MenuInputDebounceTimers::new(),
            playing_inputs: PlayingInputs::new(),

            running: true,
            now: 0.0,
            time_since_last_update: 0.0,
            scene_frame: 0,
            frame: 0,
            stage_frame: 0,
            frame_pause: 0,

            world: World::new(64, 64),
            player: Player {
                // center
                pos: Vec2::new(32.0, 32.0),
                health: 100,
                inventory: vec![],
            },
            entities: vec![],

            rebuild_render_texture: false,
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
