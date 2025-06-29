use glam::{IVec2, Vec2};
use noise::{NoiseFn, Perlin};
use rand::{random, random_range};

use crate::{
    entity::{self, EntityType, Mood},
    entity_templates::{init_as_chicken, init_as_player, init_as_zombie},
    graphics::Graphics,
    item::{Item, ItemType},
    sprite::Sprite,
    state::State,
    step::FRAMES_PER_SECOND,
    tile::{get_tile_variants, is_tile_walkable, Tile},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StageType {
    TestArena,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileData {
    pub tile: Tile,
    pub hp: u8,
    pub max_hp: u8,
    pub breakable: bool,
    pub variant: u8,
    pub flip_speed: u16,
    pub rot: f32,
    pub shake: f32,
}

impl Default for TileData {
    fn default() -> Self {
        TileData {
            tile: Tile::None,
            hp: 0,
            max_hp: 0,
            breakable: false,
            variant: 0,
            flip_speed: 0,
            rot: 0.0,
            shake: 0.0,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Stage {
    pub stage_type: StageType,
    pub tiles: Vec<Vec<TileData>>,
}

impl Stage {
    pub fn new(stage_type: StageType, width: usize, height: usize) -> Stage {
        let tiles = vec![vec![TileData::default(); height]; width];

        Stage { stage_type, tiles }
    }

    pub fn get_tile_type(&self, x: usize, y: usize) -> Option<Tile> {
        if x < self.tiles.len() && y < self.tiles[0].len() {
            Some(self.tiles[x][y].tile)
        } else {
            None
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<TileData> {
        if x < self.tiles.len() && y < self.tiles[0].len() {
            Some(self.tiles[x][y])
        } else {
            None
        }
    }

    pub fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut TileData> {
        if x < self.tiles.len() && y < self.tiles[0].len() {
            Some(&mut self.tiles[x][y])
        } else {
            None
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile_data: TileData) {
        if x < self.tiles.len() && y < self.tiles[0].len() {
            self.tiles[x][y] = tile_data;
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.tiles {
            for tile in row {
                *tile = TileData::default()
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

    // function to see if coords are in bounds
    pub fn in_bounds(&self, pos: IVec2) -> bool {
        pos.x >= 0
            && pos.x < self.get_width() as i32
            && pos.y >= 0
            && pos.y < self.get_height() as i32
    }
}

pub fn init_playing_state(state: &mut State, _graphics: &mut Graphics) {
    state.mode = crate::state::Mode::Playing;
    state.stage = Stage::new(StageType::TestArena, 64, 64);
    // ... other state init ...
    state.game_over = false;
    state.pause = false;
    state.win = false;
    state.points = 0;
    state.deaths = 0;
    state.frame_pause = 0;
    state.time_since_last_update = 0.0;
    state.entity_manager.clear_all_entities();

    let width = state.stage.get_width();
    let height = state.stage.get_height();
    state.spatial_grid = vec![vec![Vec::new(); height]; width];

    // --- NEW: Perlin Noise World Generation ---
    let perlin = Perlin::new(random::<u32>());
    let scale = 0.08; // You can tweak this! Lower value = larger features.

    for x in 0..width {
        for y in 0..height {
            let nx = x as f64 * scale;
            let ny = y as f64 * scale;

            // Get a noise value between -1.0 and 1.0
            let noise_value = perlin.get([nx, ny]);

            // Set tile based on noise value thresholds.
            // Values around 0 will be void.
            if noise_value > 0.4 {
                let mut tile = TileData::default();
                tile.tile = Tile::Grass;
                state.stage.set_tile(x, y, tile);
            } else if noise_value < -0.8 {
                let mut tile = TileData::default();
                tile.tile = Tile::Water;
                tile.variant = if random::<bool>() {
                    0 // Variant 0 for dirt
                } else {
                    1 // Variant 1 for dirt
                };
                tile.flip_speed = FRAMES_PER_SECOND as u16; // Flip every 2 seconds
                state.stage.set_tile(x, y, tile);
            } else {
                let tile = TileData::default();
                state.stage.set_tile(x, y, tile);
            }
        }
    }
    // --- End of new generation logic ---

    // --- Make Player ---
    let player_vid = state.entity_manager.new_entity().unwrap();
    state.player_vid = Some(player_vid);
    let player_grid_pos;
    {
        let player = state.entity_manager.get_entity_mut(player_vid).unwrap();
        init_as_player(player);

        // Try to spawn player on a walkable tile near the center
        loop {
            let center = state.stage.get_center_position();
            let x = random_range(center.x - 5..center.x + 5);
            let y = random_range(center.y - 5..center.y + 5);
            if is_tile_walkable(state, IVec2::new(x as i32, y as i32)) {
                let player = state.entity_manager.get_entity_mut(player_vid).unwrap();
                player.pos = IVec2::new(x as i32, y as i32).as_vec2() + Vec2::splat(0.5);
                player_grid_pos = player.pos.as_ivec2();
                break;
            }
        }
    }
    state.add_entity_to_grid(player_vid, player_grid_pos);

    // --- Spawn Zombies ---
    // let num_zombies = 0;
    let num_zombies = 32;
    for _ in 0..num_zombies {
        if let Some(vid) = state.entity_manager.new_entity() {
            let zombie_grid_pos;
            {
                let zombie = state.entity_manager.get_entity_mut(vid).unwrap();
                init_as_zombie(zombie);

                // place zombie
                loop {
                    let x = random_range(0..width);
                    let y = random_range(0..height);
                    if is_tile_walkable(state, IVec2::new(x as i32, y as i32)) {
                        let zombie = state.entity_manager.get_entity_mut(vid).unwrap();
                        zombie.pos = IVec2::new(x as i32, y as i32).as_vec2() + Vec2::splat(0.5);
                        zombie_grid_pos = zombie.pos.as_ivec2();
                        break;
                    }
                }
            }
            state.add_entity_to_grid(vid, zombie_grid_pos);
        }
    }

    // spawn a bunch of chickens
    // let num_chickens = 0;
    let num_chickens = 32;
    for _ in 0..num_chickens {
        if let Some(vid) = state.entity_manager.new_entity() {
            let chicken_grid_pos;
            {
                let chicken = state.entity_manager.get_entity_mut(vid).unwrap();
                init_as_chicken(chicken);

                // place chicken
                loop {
                    let x = random_range(0..width);
                    let y = random_range(0..height);
                    if is_tile_walkable(state, IVec2::new(x as i32, y as i32)) {
                        let chicken = state.entity_manager.get_entity_mut(vid).unwrap();
                        chicken.pos = IVec2::new(x as i32, y as i32).as_vec2() + Vec2::splat(0.5);
                        chicken_grid_pos = chicken.pos.as_ivec2();
                        break;
                    }
                }

                state.add_entity_to_grid(vid, chicken_grid_pos);
            }
        }
    }
}

/// check tile data flip speed % state.frame to see if it should flip
pub fn flip_stage_tiles(state: &mut State) {
    for x in 0..state.stage.get_width() {
        for y in 0..state.stage.get_height() {
            if let Some(tile_data) = state.stage.get_tile_mut(x, y) {
                if tile_data.flip_speed > 0 && state.frame % tile_data.flip_speed as u32 == 0 {
                    let new_variant =
                        (tile_data.variant + 1) % get_tile_variants(tile_data).len() as u8;
                    tile_data.variant = new_variant;
                }
            }
        }
    }
}
