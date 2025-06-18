use glam::IVec2;

use crate::{entity::EntityType, sprite::Sprite, state::State, tile::Tile};

pub enum StageType {
    TestArena,
}

pub struct Stage {
    pub stage_type: StageType,
    pub tiles: Vec<Vec<Tile>>,
}

impl Stage {
    pub fn new(stage_type: StageType, width: usize, height: usize) -> Self {
        let tiles = vec![vec![Tile::None; height]; width];
        Self { stage_type, tiles }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < self.tiles.len() && y < self.tiles[0].len() {
            Some(&self.tiles[x][y])
        } else {
            None
        }
    }
    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if x < self.tiles.len() && y < self.tiles[0].len() {
            self.tiles[x][y] = tile;
        }
    }
    pub fn clear(&mut self) {
        for row in &mut self.tiles {
            for tile in row {
                *tile = Tile::None;
            }
        }
    }

    pub fn get_center_position(&self) -> IVec2 {
        let width = self.tiles.len() as f32;
        let height = self.tiles[0].len() as f32;
        IVec2::new((width / 2.0) as i32, (height / 2.0) as i32)
    }

    pub fn get_height(&self) -> usize {
        if self.tiles.is_empty() {
            0
        } else {
            self.tiles[0].len()
        }
    }
    pub fn get_width(&self) -> usize {
        self.tiles.len()
    }

    pub fn get_dims(&self) -> IVec2 {
        IVec2::new(self.get_width() as i32, self.get_height() as i32)
    }
}

pub fn init_playing_state(state: &mut State) {
    // setup state
    state.stage_frame = 0;
    state.mode = crate::state::Mode::Playing;
    state.stage = Stage::new(StageType::TestArena, 64, 64);
    state.game_over = false;
    state.pause = false;
    state.win = false;
    state.points = 0;
    state.deaths = 0;
    state.frame_pause = 0;
    state.time_since_last_update = 0.0;
    state.entity_manager.clear_all_entities();

    // make player
    let player_vid = match state.entity_manager.new_entity() {
        Some(vid) => vid,
        None => {
            println!("Failed to create player entity, entity budget exceeded.");
            return;
        }
    };
    state.player_vid = Some(player_vid);

    let player = match state.entity_manager.get_entity_mut(player_vid) {
        Some(entity) => entity,
        None => {
            println!("Failed to retrieve player entity.");
            return;
        }
    };
    player.active = true;
    player.type_ = EntityType::Player;
    player.sprite = Sprite::Player;

    let center = state.stage.get_center_position();
    player.pos = IVec2::new(center.x, center.y).as_vec2();
}
