mod audio;
mod config;

use std::vec;

use config::Config;
use raylib::{consts::KeyboardKey, prelude::*};

fn main() {
    // Load configuration from a TOML file
    let config = Config::load("config.toml").expect("Failed to load configuration");

    let (mut rl, thread) = raylib::init()
        .size(config.window.width, config.window.height)
        .title(&config.window.title)
        .build();

    // Initialize the game state
    let mut game = Game {
        world: World {
            width: config.window.width / config.game.tile_size,
            height: config.window.height / config.game.tile_size,
            tiles: vec![(0..config.window.width / config.game.tile_size)
                .flat_map(|x| {
                    (0..config.window.height / config.game.tile_size).map(move |y| Tile {
                        x,
                        y,
                        walkable: true,
                        tile_type: TileType::Grass,
                    })
                })
                .collect::<Vec<_>>()],
        },
        player: Player {
            x: config.window.width / config.game.tile_size / 2,
            y: config.window.height / config.game.tile_size / 2,
            health: 100,
            inventory: vec![],
        },
        entities: vec![],
    };

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        // Handle player input
        let mut move_player = vec![0, 0];
        if d.is_key_pressed(KeyboardKey::KEY_W) {
            move_player[1] -= 1;
        }
        if d.is_key_pressed(KeyboardKey::KEY_S) {
            move_player[1] += 1;
        }
        if d.is_key_pressed(KeyboardKey::KEY_A) {
            move_player[0] -= 1;
        }
        if d.is_key_pressed(KeyboardKey::KEY_D) {
            move_player[0] += 1;
        }

        // Update player position
        if move_player[0] != 0 || move_player[1] != 0 {
            let tile = game.world.tile_at(
                game.player.x + move_player[0],
                game.player.y + move_player[1],
            );
            if let Some(tile) = tile {
                if tile.walkable {
                    game.player.x = game.player.x + move_player[0];
                    game.player.y = game.player.y + move_player[1];
                }
            }
        }

        // Draw the game world
        d.clear_background(Color::new(30, 20, 30, 255));
        for row in &game.world.tiles {
            for tile in row {
                let color = match tile.tile_type {
                    TileType::Grass => Color::GREEN,
                    TileType::Water => Color::BLUE,
                    TileType::Mountain => Color::GRAY,
                    TileType::Wall => Color::BLACK,
                };
                d.draw_rectangle(
                    tile.x * config.game.tile_size,
                    tile.y * config.game.tile_size,
                    config.game.tile_size,
                    config.game.tile_size,
                    color,
                );
            }
        }

        // Draw player
        d.draw_rectangle(
            game.player.x * config.game.tile_size,
            game.player.y * config.game.tile_size,
            config.game.tile_size,
            config.game.tile_size,
            Color::WHITE,
        );

        // Draw entities
        for entity in &game.entities {
            let color = match entity.entity_type {
                EntityType::Monster => Color::RED,
                EntityType::Item => Color::YELLOW,
            };
            d.draw_rectangle(
                entity.position.0 * config.game.tile_size,
                entity.position.1 * config.game.tile_size,
                config.game.tile_size,
                config.game.tile_size,
                color,
            );
        }

        d.draw_text(
            &format!("Player Health: {}", game.player.health),
            10,
            10,
            20,
            Color::WHITE,
        );
        d.draw_text(
            &format!("Player Inventory: {}", game.player.inventory.len()),
            10,
            40,
            20,
            Color::WHITE,
        );
    }
}

struct Game {
    world: World,
    player: Player,
    entities: Vec<Entity>,
}

struct World {
    width: i32,
    height: i32,
    tiles: Vec<Vec<Tile>>,
}

impl World {
    fn tile_at(&self, x: i32, y: i32) -> Option<&Tile> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            None
        } else {
            self.tiles
                .get(y as usize)
                .and_then(|row| row.get(x as usize))
        }
    }
}

struct Tile {
    x: i32,
    y: i32,
    walkable: bool,
    tile_type: TileType,
}
enum TileType {
    Grass,
    Wall,
    Water,
    Mountain,
}

struct Player {
    x: i32,
    y: i32,
    health: i32,
    inventory: Vec<Item>,
}

struct Item {
    name: String,
    quantity: i32,
}

struct Entity {
    id: u32,
    position: (i32, i32),
    entity_type: EntityType,
}

enum EntityType {
    Monster,
    Item,
}
