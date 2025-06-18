mod audio;
mod config;
mod state;

use std::vec;

use config::Config;
use glam::Vec2;
use raylib::{consts::KeyboardKey, prelude::*};
use state::{EntityType, Player, State, Tile, TileType, World};

fn main() {
    // Load configuration from a TOML file
    let config = Config::load("config.toml").expect("Failed to load configuration");

    let (mut rl, thread) = raylib::init()
        .size(config.window.width, config.window.height)
        .title(&config.window.title)
        .build();

    // Initialize the game state
    let mut game = State {
        world: World {
            width: config.window.width / config.game.tile_size,
            height: config.window.height / config.game.tile_size,
            tiles: vec![(0..config.window.width / config.game.tile_size)
                .flat_map(|x| {
                    (0..config.window.height / config.game.tile_size).map(move |y| Tile {
                        pos: Vec2::new(x as f32, y as f32),
                        walkable: true,
                        tile_type: TileType::Grass,
                    })
                })
                .collect::<Vec<_>>()],
        },
        player: Player {
            pos: Vec2::new(
                (config.window.width / config.game.tile_size / 2) as f32,
                (config.window.height / config.game.tile_size / 2) as f32,
            ),
            health: 100,
            inventory: vec![],
        },
        entities: vec![],
    };

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        // Handle player input
        let mut move_player = Vec2::new(0.0, 0.0);
        if d.is_key_pressed(KeyboardKey::KEY_W) {
            move_player.y -= 1.0;
        }
        if d.is_key_pressed(KeyboardKey::KEY_S) {
            move_player.y += 1.0;
        }
        if d.is_key_pressed(KeyboardKey::KEY_A) {
            move_player.x -= 1.0;
        }
        if d.is_key_pressed(KeyboardKey::KEY_D) {
            move_player.x += 1.0;
        }

        // Update player position
        if move_player.x != 0.0 || move_player.y != 0.0 {
            let new_pos = game.player.pos + move_player;
            let tile = game.world.tile_at(new_pos.x as i32, new_pos.y as i32);
            if let Some(tile) = tile {
                if tile.walkable {
                    game.player.pos += move_player;
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
                    tile.pos.x as i32 * config.game.tile_size,
                    tile.pos.y as i32 * config.game.tile_size,
                    config.game.tile_size,
                    config.game.tile_size,
                    color,
                );
            }
        }

        // Draw player
        d.draw_rectangle(
            game.player.pos.x as i32 * config.game.tile_size,
            game.player.pos.y as i32 * config.game.tile_size,
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
                entity.pos.x as i32 * config.game.tile_size,
                entity.pos.y as i32 * config.game.tile_size,
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
