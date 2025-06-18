use glam::{IVec2, Vec2};
use noise::{NoiseFn, Perlin, Seedable}; // <-- Add noise crate imports
use rand::{random, random_range}; // <-- Import `random` for seeding

use crate::{
    entity::EntityType,
    graphics::Graphics,
    sprite::Sprite,
    state::State,
    tile::{walkable, Tile},
};

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
        self.tiles.get(x).and_then(|row| row.get(y))
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

pub fn init_playing_state(state: &mut State, _graphics: &mut Graphics) {
    state.stage_frame = 0;
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
                state.stage.set_tile(x, y, Tile::Grass);
            } else if noise_value < -0.3 {
                state.stage.set_tile(x, y, Tile::Water);
            } else {
                state.stage.set_tile(x, y, Tile::None);
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
        player.active = true;
        player.type_ = EntityType::Player;
        player.sprite = Sprite::Player;
        player.impassable = true;

        // Try to spawn player on a walkable tile near the center
        loop {
            let center = state.stage.get_center_position();
            let x = random_range(center.x - 5..center.x + 5);
            let y = random_range(center.y - 5..center.y + 5);
            if let Some(tile) = state.stage.get_tile(x as usize, y as usize) {
                if walkable(*tile) {
                    player.pos = IVec2::new(x, y).as_vec2() + Vec2::splat(0.5);
                    player_grid_pos = player.pos.as_ivec2();
                    break;
                }
            }
        }
    }
    state.add_entity_to_grid(player_vid, player_grid_pos);

    // --- Spawn Zombies ---
    let num_zombies = 512;
    for _ in 0..num_zombies {
        if let Some(vid) = state.entity_manager.new_entity() {
            let zombie_grid_pos;
            {
                let zombie = state.entity_manager.get_entity_mut(vid).unwrap();
                zombie.active = true;
                zombie.type_ = EntityType::Zombie;
                zombie.sprite = Sprite::Zombie;
                zombie.impassable = true;

                loop {
                    let x = random_range(0..width);
                    let y = random_range(0..height);
                    if let Some(tile) = state.stage.get_tile(x, y) {
                        if walkable(*tile) {
                            zombie.pos =
                                IVec2::new(x as i32, y as i32).as_vec2() + Vec2::splat(0.5);
                            zombie_grid_pos = zombie.pos.as_ivec2();
                            break;
                        }
                    }
                }
            }
            state.add_entity_to_grid(vid, zombie_grid_pos);
        }
    }
}
