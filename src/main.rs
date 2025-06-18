mod audio;
mod config;

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
        if d.is_key_pressed(KeyboardKey::KEY_W) {
            game.player.y -= 1;
            game.player.y = game.player.y.clamp(0, game.world.height - 1);
        }
        if d.is_key_pressed(KeyboardKey::KEY_S) {
            game.player.y += 1;
            game.player.y = game.player.y.clamp(0, game.world.height - 1);
        }
        if d.is_key_pressed(KeyboardKey::KEY_A) {
            game.player.x -= 1;
            game.player.x = game.player.x.clamp(0, game.world.width - 1);
        }
        if d.is_key_pressed(KeyboardKey::KEY_D) {
            game.player.x += 1;
            game.player.x = game.player.x.clamp(0, game.world.width - 1);
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

struct Tile {
    x: i32,
    y: i32,
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
